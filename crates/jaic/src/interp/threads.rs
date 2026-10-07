//! Threads for interpreted programs under native `jaic run`.
//!
//! Every Jai thread is an OS thread, but only the one holding the baton runs interpreted code,
//! so `Interp` is never touched by two threads at once. The baton moves when its holder blocks
//! (join, mutex, condition variable, semaphore, sleep), yields, has run for a while
//! (`preempt`), or stays inside one native call for longer than `NATIVE_SLICE`: a thread
//! calling C keeps the baton but lets others take it (`enter_native`), and waits for it again
//! when C returns (`leave_native`). Short C calls therefore cost no switch, and long ones
//! (`read`, `accept`, a sleep in C) no longer stop the other threads.
//!
//! Mutexes and condition variables are emulated here instead of calling libc, because a
//! thread blocked inside libc on a lock another Jai thread holds could never get it.
//!
//! A thread C started itself that calls a `#c_call` procedure (an audio render thread) is
//! adopted for the length of that call: it becomes a scheduler thread, takes the baton like any
//! other, and may block and be woken. A callback on a thread already inside a native call of
//! this interpreter (C -> Jai -> C -> Jai) runs as that thread.
//!
//! The scheduler (`Sched`) sits behind one mutex shared by all of these threads (`Shared`,
//! also the gate thunks enter through). Each thread carries its own interpreter state
//! (`ExecState`) and puts it into `Interp` while it holds the baton.
//!
//! The Win32 equivalents (`CreateThread`, critical sections, SRW locks, condition variables,
//! semaphores and events, waits on their handles) map onto the same scheduler; see `win32.rs`.
#![allow(unsafe_code)]

mod win32;

use super::*;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};

/// Interpreter value stack of a thread other than the first.
const THREAD_STACK: usize = 8 << 20;

/// Native stack reserved for each OS thread that runs interpreted code.
const NATIVE_STACK: usize = 256 << 20;

/// Block transitions between preemption checks.
const PREEMPT_TICKS: u64 = 20_000;

/// How long a thread may stay in one native call before a runnable thread takes the baton.
/// Runnable threads also check this often whether the holder has entered such a call.
const NATIVE_SLICE: Duration = Duration::from_millis(1);

/// Once C may hold thunks (`Sched::callbacks`), a thread C started may call one and wake the
/// blocked threads: every thread must stay blocked this long before that counts as a deadlock.
pub const DEADLOCK_GRACE: Duration = Duration::from_secs(1);

const EPERM: u64 = 1;
const EBUSY: u64 = 16;
const EINVAL: u64 = 22;

const ETIMEDOUT: u64 = if cfg!(target_os = "macos") {
    60
} else {
    110
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Block {
    /// Runnable (or running).
    None,
    Join(usize),
    Mutex(u64),
    Cond(u64, Option<Instant>),
    Sleep(Instant),
    /// A Win32 wait on handles, rechecked whenever one of them may have become signaled.
    Object(Option<Instant>),
    /// Inside a native call, without the baton: not runnable, but it will come back.
    Native,
}

impl Block {
    fn deadline(self) -> Option<Instant> {
        match self {
            Block::Sleep(d) | Block::Cond(_, Some(d)) | Block::Object(Some(d)) => Some(d),
            _ => None,
        }
    }
}

struct GThread {
    finished: bool,
    block: Block,
    /// A timed condition wait ended because its deadline passed.
    timed_out: bool,
    /// The scheduler found every thread blocked and woke this one to report it.
    deadlocked: bool,
    result: u64,
    /// A thread C started, running one callback; its slot is reused once that returns.
    adopted: bool,
    /// The interpreter state a new Jai thread starts with.
    start: Option<ExecState>,
}

impl GThread {
    fn new(adopted: bool, start: Option<ExecState>) -> Self {
        GThread {
            finished: false,
            block: Block::None,
            timed_out: false,
            deadlocked: false,
            result: 0,
            adopted,
            start,
        }
    }
}

#[derive(Default)]
struct MutexState {
    owner: Option<usize>,
    count: u32,
}

pub(super) struct Sched {
    threads: Vec<GThread>,
    /// The thread holding the baton (`None`: free, nothing runnable yet).
    current: Option<usize>,
    /// The last thread given the baton: the round robin starts after it.
    last: usize,
    /// The holder is inside a native call, so the baton may be taken from it.
    in_native: bool,
    /// Native calls entered by holders so far: tells a waiter whether the holder is still in
    /// the call it saw last time.
    native_calls: u64,
    /// Callbacks waiting for the baton: a holder entering C wakes them at once.
    urgent: usize,
    /// A foreign call ran after a thunk was made that does not start a Jai thread, so C may
    /// hold it and threads the scheduler does not know yet may call back
    /// (`Interp::hand_thunks_to_c`).
    callbacks: bool,
    /// A blocked thread became runnable: its OS thread must be woken to wait for its turn
    /// (`with_sched`).
    woke: bool,
    mutexes: HashMap<u64, MutexState>,
    cond_waiters: HashMap<u64, VecDeque<usize>>,
    /// Win32 handles the scheduler made (threads, semaphores, events).
    objects: HashMap<u64, win32::Object>,
    next_object: u64,
    /// Slots of adopted threads that have returned to C.
    spare: Vec<usize>,
    /// The interpreter (`*mut Interp`), for threads that start running it.
    interp: usize,
    /// The interpreter is gone; callbacks into it fail.
    dead: bool,
}

impl Sched {
    fn runnable(&self, t: usize) -> bool {
        !self.threads[t].finished && self.threads[t].block == Block::None
    }

    /// Make runnable every thread whose sleep or timed wait is over.
    fn expire_waits(&mut self) {
        let now = Instant::now();
        for t in 0..self.threads.len() {
            match self.threads[t].block {
                Block::Sleep(d) | Block::Object(Some(d)) if d <= now => {
                    self.threads[t].block = Block::None;
                }
                Block::Cond(c, Some(d)) if d <= now => {
                    self.threads[t].block = Block::None;
                    self.threads[t].timed_out = true;
                    if let Some(waiters) = self.cond_waiters.get_mut(&c) {
                        waiters.retain(|&w| w != t);
                    }
                }
                _ => {}
            }
        }
    }

    /// Make runnable every thread blocked for `reason`.
    fn wake(&mut self, reason: Block) {
        for thread in &mut self.threads {
            if thread.block == reason {
                thread.block = Block::None;
                self.woke = true;
            }
        }
    }

    /// Let every thread waiting on Win32 handles check them again.
    fn wake_object_waiters(&mut self) {
        for thread in &mut self.threads {
            if matches!(thread.block, Block::Object(_)) {
                thread.block = Block::None;
                self.woke = true;
            }
        }
    }

    /// Give the free baton to the next runnable thread after the last holder.
    fn pick(&mut self) -> bool {
        self.expire_waits();
        let n = self.threads.len();
        let next = (1..=n)
            .map(|k| (self.last + k) % n)
            .find(|&t| self.runnable(t));
        if let Some(t) = next {
            self.current = Some(t);
            self.last = t;
            self.in_native = false;
        }
        next.is_some()
    }

    /// No thread can ever run again: none is runnable, waiting for a deadline or in native code
    /// (which may come back and wake the others).
    fn stuck(&self) -> bool {
        self.threads.iter().all(|t| {
            t.finished
                || (t.block != Block::None
                    && t.block != Block::Native
                    && t.block.deadline().is_none())
        })
    }

    /// The holder gives the baton up: hand it to the next runnable thread. When nothing can
    /// run again, wake `report` (or the first unfinished thread) to report a deadlock; with
    /// thunks in C's hands, the blocked threads do that after `DEADLOCK_GRACE` (`acquire`).
    fn release(&mut self, report: Option<usize>) {
        self.current = None;
        self.in_native = false;
        if self.pick() || !self.stuck() || self.callbacks {
            return;
        }
        self.report_deadlock(report);
    }

    fn report_deadlock(&mut self, report: Option<usize>) {
        let first = (0..self.threads.len()).find(|&t| !self.threads[t].finished);
        if let Some(t) = report.or(first) {
            self.threads[t].block = Block::None;
            self.threads[t].deadlocked = true;
            self.current = Some(t);
            self.last = t;
        }
    }

    /// A slot for a thread C started that is calling back.
    fn adopt(&mut self) -> usize {
        if let Some(t) = self.spare.pop() {
            self.threads[t] = GThread::new(true, None);
            return t;
        }
        self.threads.push(GThread::new(true, None));
        self.threads.len() - 1
    }
}

/// The scheduler shared by every thread running the interpreter, and the gate thunks enter
/// it through.
pub(super) struct Shared {
    sched: Mutex<Sched>,
    cv: Condvar,
}

impl Shared {
    /// A scheduler whose only thread (0, the caller) holds the baton.
    fn new(interp: usize) -> Arc<Shared> {
        Arc::new(Shared {
            sched: Mutex::new(Sched {
                threads: vec![GThread::new(false, None)],
                current: Some(0),
                last: 0,
                in_native: false,
                native_calls: 0,
                urgent: 0,
                callbacks: false,
                woke: false,
                mutexes: HashMap::default(),
                cond_waiters: HashMap::default(),
                objects: HashMap::default(),
                next_object: 0,
                spare: Vec::new(),
                interp,
                dead: false,
            }),
            cv: Condvar::new(),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Sched> {
        self.sched.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The identity C callbacks compare with `native::caller`.
    pub(super) fn key(&self) -> usize {
        self as *const Shared as usize
    }

    /// Wait until `me` (runnable, or blocked waiting to be woken) holds the baton.
    fn acquire<'a>(&'a self, s: MutexGuard<'a, Sched>, me: usize) -> MutexGuard<'a, Sched> {
        let Some(s) = self.wait_turn(s, me, false) else {
            unreachable!("only callbacks stop waiting when the interpreter is gone")
        };
        s
    }

    /// `acquire` for a callback from C, which takes the baton at once from a holder inside a
    /// native call. `None`: the interpreter is gone.
    fn acquire_callback<'a>(
        &'a self,
        s: MutexGuard<'a, Sched>,
        me: usize,
    ) -> Option<MutexGuard<'a, Sched>> {
        self.wait_turn(s, me, true)
    }

    /// Any waiter that finds the baton free hands it on. A runnable waiter takes it from a
    /// holder that has been in one native call for `NATIVE_SLICE` (`urgent`: at once).
    fn wait_turn<'a>(
        &'a self,
        mut s: MutexGuard<'a, Sched>,
        me: usize,
        urgent: bool,
    ) -> Option<MutexGuard<'a, Sched>> {
        if urgent {
            s.urgent += 1;
        }
        // The holder's native call this thread saw, and when it first saw it.
        let mut watched: Option<(u64, Instant)> = None;
        // When this thread found every thread blocked for good.
        let mut stuck_since: Option<Instant> = None;
        while s.current != Some(me) {
            if s.dead {
                // The interpreter is gone (the program ended): never touch it again.
                if urgent {
                    s.urgent -= 1;
                    return None;
                }
                s = self.cv.wait(s).unwrap_or_else(|e| e.into_inner());
                continue;
            }
            if s.threads[me]
                .block
                .deadline()
                .is_some_and(|d| d <= Instant::now())
            {
                // Runnable now: wait for a turn like any runnable thread.
                s.expire_waits();
            }
            let mut timeout = None;
            match s.current {
                None => {
                    if s.pick() {
                        self.cv.notify_all();
                        continue;
                    }
                    if s.stuck() {
                        let since = *stuck_since.get_or_insert_with(Instant::now);
                        let left = DEADLOCK_GRACE.saturating_sub(since.elapsed());
                        if left.is_zero() {
                            s.report_deadlock(None);
                            self.cv.notify_all();
                            continue;
                        }
                        timeout = Some(left);
                    } else {
                        stuck_since = None;
                    }
                }
                Some(holder) if s.in_native => {
                    s.expire_waits();
                    if s.runnable(me) {
                        let call = s.native_calls;
                        let now = Instant::now();
                        let long = match watched {
                            Some((c, since)) if c == call => now - since >= NATIVE_SLICE,
                            _ => {
                                watched = Some((call, now));
                                false
                            }
                        };
                        if urgent || long {
                            s.threads[holder].block = Block::Native;
                            s.current = None;
                            s.pick();
                            self.cv.notify_all();
                            continue;
                        }
                    }
                }
                Some(_) => watched = None,
            }
            let timeout = if s.runnable(me) {
                Some(NATIVE_SLICE)
            } else {
                let deadline = s.threads[me]
                    .block
                    .deadline()
                    .map(|d| d.saturating_duration_since(Instant::now()));
                deadline.into_iter().chain(timeout).min()
            };
            s = match timeout {
                Some(t) => {
                    self.cv
                        .wait_timeout(s, t)
                        .unwrap_or_else(|e| e.into_inner())
                        .0
                }
                None => self.cv.wait(s).unwrap_or_else(|e| e.into_inner()),
            };
        }
        if urgent {
            s.urgent -= 1;
        }
        s.in_native = false;
        Some(s)
    }

    /// The holder is about to call C: it keeps the baton, but another thread may take it.
    /// Returns the holder's id.
    pub(super) fn enter_native(&self, interp: usize) -> usize {
        let mut s = self.lock();
        s.in_native = true;
        s.native_calls += 1;
        s.interp = interp;
        if s.urgent > 0 {
            self.cv.notify_all();
        }
        s.current.unwrap_or(0)
    }

    /// Thread `me` is back from C: take the baton back, waiting for it if it was taken.
    pub(super) fn leave_native(&self, me: usize) {
        let mut s = self.lock();
        if s.current == Some(me) {
            s.in_native = false;
            return;
        }
        s.threads[me].block = Block::None;
        drop(self.acquire(s, me));
    }

    /// C may have been given a thunk.
    pub(super) fn callbacks_possible(&self) {
        self.lock().callbacks = true;
    }

    /// The interpreter is being dropped.
    pub(super) fn close(&self) {
        self.lock().dead = true;
        self.cv.notify_all();
    }
}

impl native::Gate for Shared {
    fn run(&self, program: u64, f: &mut dyn FnMut(&mut native::Reenter<'_>)) -> Result<(), String> {
        const GONE: &str = "C called a procedure of an interpreter that has finished";
        let (gate, caller) = native::caller();
        let mut s = self.lock();
        if s.dead {
            return Err(GONE.into());
        }
        // On a thread inside a native call of this interpreter, the callback runs as the
        // thread that made the call; on any other thread C started, as an adopted thread.
        let nested = (gate == self.key()).then_some(caller);
        let me = match nested {
            Some(me) if s.current == Some(me) => {
                s.in_native = false;
                me
            }
            Some(me) => {
                s.threads[me].block = Block::None;
                s = self.acquire_callback(s, me).ok_or(GONE)?;
                me
            }
            None => {
                let me = s.adopt();
                s = self.acquire_callback(s, me).ok_or(GONE)?;
                me
            }
        };
        let interp = s.interp as *mut Interp;
        drop(s);
        // SAFETY: this thread holds the baton, so nothing else touches the interpreter, and
        // `program` is the program the thunk was made for, alive while its interpreter is.
        let (interp, program) = unsafe { (&mut *interp, &*(program as *const Program)) };
        f(&mut |func, args: &[u64]| interp.run_callback(program, func, args));
        let mut s = self.lock();
        if nested.is_some() {
            // Back to C, inside the native call the callback came from.
            s.in_native = true;
            s.native_calls += 1;
            if s.urgent > 0 {
                self.cv.notify_all();
            }
        } else {
            s.threads[me].finished = true;
            s.spare.push(me);
            s.release(None);
            self.cv.notify_all();
        }
        Ok(())
    }
}

impl Interp {
    /// The scheduler, made on first use (a thread, a lock, or a thunk C may call).
    pub(super) fn shared(&mut self) -> Arc<Shared> {
        if let Some(shared) = &self.shared {
            return shared.clone();
        }
        let shared = Shared::new(self as *mut Interp as usize);
        self.shared = Some(shared.clone());
        // Another thread may want the baton from now on.
        self.multi = true;
        shared
    }

    fn with_sched<R>(&mut self, f: impl FnOnce(&mut Sched) -> R) -> R {
        let shared = self.shared();
        let mut s = shared.lock();
        let out = f(&mut s);
        if std::mem::take(&mut s.woke) {
            shared.cv.notify_all();
        }
        out
    }

    fn me(&mut self) -> usize {
        self.with_sched(|s| s.current.unwrap_or(0))
    }

    /// Foreign procedures the thread scheduler implements itself. `None`: not one of them.
    pub(super) fn thread_foreign(
        &mut self,
        program: &Program,
        symbol: &str,
        args: &[u64],
    ) -> Option<Res<Vec<u64>>> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        let started = self.multi;
        let result: Res<u64> = match symbol {
            "pthread_create" => self.thread_create(program, arg(0), arg(2), arg(3)),
            "pthread_join" => self.thread_join(arg(0), arg(1)),
            "pthread_detach" => Ok(0),
            "pthread_self" => Ok(self.me() as u64 + 1),
            "pthread_mutex_init" => {
                self.with_sched(|s| s.mutexes.insert(arg(0), MutexState::default()));
                Ok(0)
            }
            "pthread_mutex_destroy" => {
                self.with_sched(|s| s.mutexes.remove(&arg(0)));
                Ok(0)
            }
            "pthread_mutex_lock" => self.lock_mutex(arg(0), 1).map(|_| 0),
            "pthread_mutex_trylock" => Ok(if self.try_lock_mutex(arg(0)) {
                0
            } else {
                EBUSY
            }),
            "pthread_mutex_unlock" => Ok(self.unlock_mutex(arg(0)).err().unwrap_or(0)),
            "pthread_cond_init" | "pthread_cond_destroy" => {
                self.with_sched(|s| s.cond_waiters.remove(&arg(0)));
                Ok(0)
            }
            "pthread_cond_signal" => {
                self.wake_cond(arg(0), false);
                Ok(0)
            }
            "pthread_cond_broadcast" => {
                self.wake_cond(arg(0), true);
                Ok(0)
            }
            "pthread_cond_wait" => self.cond_wait(arg(0), arg(1), None),
            "pthread_cond_timedwait" => {
                let deadline = absolute_deadline(arg(2));
                self.cond_wait(arg(0), arg(1), Some(deadline))
            }
            // Sleeping and yielding only matter once another thread may exist.
            "nanosleep" if started => {
                let (secs, nanos) = (self.read_u64(arg(0)), self.read_u64(arg(0) + 8));
                self.sleep_for(Duration::new(secs, nanos as u32))
            }
            "usleep" if started => self.sleep_for(Duration::from_micros(arg(0) as u32 as u64)),
            "sleep" if started => self.sleep_for(Duration::from_secs(arg(0) as u32 as u64)),
            "sched_yield" | "pthread_yield_np" if started => self.yield_now().map(|_| 0),
            _ => return self.win32_foreign(program, symbol, args),
        };
        Some(result.map(|v| vec![v]))
    }

    /// Start a Jai thread running `start(argument)`: its id, or `None` when the OS refused.
    fn spawn_thread(&mut self, program: &Program, start: u64, argument: u64) -> Res<Option<usize>> {
        let Some(func) = self.func_of(start) else {
            return self.trap("pthread_create needs an interpreted thread procedure");
        };
        // The scheduler runs this thunk, not C: it gives no thread of C's a way back in.
        self.unseen_thunks.retain(|&t| t != start);
        let shared = self.shared();
        let id = {
            let mut s = shared.lock();
            let state = ExecState {
                stack: vec![0u64; THREAD_STACK / 8].into_boxed_slice(),
                ..ExecState::default()
            };
            s.threads.push(GThread::new(false, Some(state)));
            s.threads.len() - 1
        };
        let program = program as *const Program as usize;
        let thread_shared = shared.clone();
        let spawned = std::thread::Builder::new()
            .stack_size(NATIVE_STACK)
            .spawn(move || thread_main(&thread_shared, program, id, func, argument));
        if spawned.is_err() {
            shared.lock().threads[id].finished = true;
            return Ok(None);
        }
        Ok(Some(id))
    }

    fn thread_create(
        &mut self,
        program: &Program,
        out: u64,
        start: u64,
        argument: u64,
    ) -> Res<u64> {
        let Some(id) = self.spawn_thread(program, start, argument)? else {
            return Ok(11); // EAGAIN
        };
        if out != 0 {
            self.write(out, &(id as u64 + 1).to_le_bytes());
        }
        Ok(0)
    }

    fn thread_join(&mut self, handle: u64, result_out: u64) -> Res<u64> {
        let id = handle.wrapping_sub(1) as usize;
        let me = self.me();
        let valid = self.with_sched(|s| id < s.threads.len() && !s.threads[id].adopted);
        if !valid || id == me {
            return Ok(EINVAL);
        }
        while !self.with_sched(|s| s.threads[id].finished) {
            self.block(Block::Join(id))?;
        }
        if result_out != 0 {
            let value = self.with_sched(|s| s.threads[id].result);
            self.write(result_out, &value.to_le_bytes());
        }
        Ok(0)
    }

    fn try_lock_mutex(&mut self, addr: u64) -> bool {
        self.with_sched(|s| {
            let me = s.current.unwrap_or(0);
            let state = s.mutexes.entry(addr).or_default();
            if state.owner.is_none() || state.owner == Some(me) {
                state.owner = Some(me);
                state.count += 1;
                true
            } else {
                false
            }
        })
    }

    fn lock_mutex(&mut self, addr: u64, count: u32) -> Res<()> {
        loop {
            let taken = self.with_sched(|s| {
                let me = s.current.unwrap_or(0);
                let state = s.mutexes.entry(addr).or_default();
                if state.owner.is_none() || state.owner == Some(me) {
                    state.owner = Some(me);
                    state.count += count;
                    true
                } else {
                    false
                }
            });
            if taken {
                return Ok(());
            }
            self.block(Block::Mutex(addr))?;
        }
    }

    /// Release one level of the mutex; `Err(EPERM)` when the caller does not hold it.
    fn unlock_mutex(&mut self, addr: u64) -> Result<(), u64> {
        self.with_sched(|s| {
            let me = s.current.unwrap_or(0);
            let Some(state) = s.mutexes.get_mut(&addr) else {
                return Err(EPERM);
            };
            if state.owner != Some(me) {
                return Err(EPERM);
            }
            state.count -= 1;
            if state.count == 0 {
                state.owner = None;
                s.wake(Block::Mutex(addr));
            }
            Ok(())
        })
    }

    fn cond_wait(&mut self, cond: u64, mutex: u64, deadline: Option<Instant>) -> Res<u64> {
        // Give the mutex up completely while waiting, and take it back as it was.
        let (me, held) = self.with_sched(|s| {
            let me = s.current.unwrap_or(0);
            let held = match s.mutexes.get(&mutex) {
                Some(state) if state.owner == Some(me) => state.count,
                _ => 0,
            };
            (me, held)
        });
        for _ in 0..held {
            let _ = self.unlock_mutex(mutex);
        }
        self.with_sched(|s| {
            s.cond_waiters.entry(cond).or_default().push_back(me);
            s.threads[me].timed_out = false;
        });
        self.block(Block::Cond(cond, deadline))?;
        let timed_out = self.with_sched(|s| std::mem::take(&mut s.threads[me].timed_out));
        if held > 0 {
            self.lock_mutex(mutex, held)?;
        }
        Ok(if timed_out {
            ETIMEDOUT
        } else {
            0
        })
    }

    fn wake_cond(&mut self, cond: u64, all: bool) {
        self.with_sched(|s| {
            let Some(waiters) = s.cond_waiters.get_mut(&cond) else {
                return;
            };
            while let Some(id) = waiters.pop_front() {
                if matches!(s.threads[id].block, Block::Cond(c, _) if c == cond) {
                    s.threads[id].block = Block::None;
                    s.woke = true;
                }
                if !all {
                    break;
                }
            }
        });
    }

    fn sleep_for(&mut self, duration: Duration) -> Res<u64> {
        self.block(Block::Sleep(Instant::now() + duration))?;
        Ok(0)
    }

    /// Let another runnable thread go first.
    pub(super) fn yield_now(&mut self) -> Res<()> {
        let shared = self.shared();
        let mut s = shared.lock();
        s.expire_waits();
        let Some(me) = s.current else {
            return Ok(());
        };
        if !(0..s.threads.len()).any(|t| t != me && s.runnable(t)) {
            return Ok(());
        }
        let state = self.take_exec_state();
        s.interp = self as *mut Interp as usize;
        s.current = None;
        s.last = me;
        s.pick();
        shared.cv.notify_all();
        drop(shared.acquire(s, me));
        self.put_exec_state(state);
        Ok(())
    }

    /// Called between basic blocks while several threads may exist: after enough of them,
    /// offer the baton to the other threads so that busy-waiting on an atomic makes progress.
    pub(super) fn preempt(&mut self) -> Res<()> {
        self.ticks += 1;
        if self.ticks.is_multiple_of(PREEMPT_TICKS) {
            self.yield_now()?;
        }
        Ok(())
    }

    /// Give the baton up until `reason` no longer holds (whoever ends it wakes this thread).
    fn block(&mut self, reason: Block) -> Res<()> {
        let shared = self.shared();
        let state = self.take_exec_state();
        let deadlocked = {
            let mut s = shared.lock();
            let me = s.current.unwrap_or(0);
            s.threads[me].block = reason;
            s.interp = self as *mut Interp as usize;
            s.release(Some(me));
            shared.cv.notify_all();
            let mut s = shared.acquire(s, me);
            s.threads[me].block = Block::None;
            std::mem::take(&mut s.threads[me].deadlocked)
        };
        self.put_exec_state(state);
        if deadlocked {
            return self.trap("deadlock: every thread is blocked");
        }
        Ok(())
    }

    /// The Jai thread `me` has ended: wake its joiners and pass the baton on for good.
    fn finish_thread(&mut self, me: usize, result: u64) {
        drop(self.take_exec_state());
        let shared = self.shared();
        let mut s = shared.lock();
        s.threads[me].finished = true;
        s.threads[me].result = result;
        s.wake(Block::Join(me));
        // Its Win32 handle is now signaled.
        s.wake_object_waiters();
        s.release(None);
        shared.cv.notify_all();
    }
}

/// The instant a POSIX absolute `timespec` (seconds and nanoseconds since the epoch) names.
fn absolute_deadline(timespec: u64) -> Instant {
    let (secs, nanos) = unsafe {
        (
            std::ptr::read_unaligned(timespec as *const u64),
            std::ptr::read_unaligned((timespec + 8) as *const u64),
        )
    };
    let target = Duration::new(secs, nanos as u32);
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    Instant::now() + target.saturating_sub(now)
}

/// Body of the OS thread that runs Jai thread `id`.
fn thread_main(shared: &Shared, program: usize, id: usize, func: FuncId, argument: u64) {
    let (interp, state) = {
        let s = shared.lock();
        let mut s = shared.acquire(s, id);
        (s.interp, s.threads[id].start.take().unwrap_or_default())
    };
    // SAFETY: only the thread holding the baton touches the interpreter, and the program
    // outlives every thread (the process ends when the program's `main` returns).
    let interp = unsafe { &mut *(interp as *mut Interp) };
    let program = unsafe { &*(program as *const Program) };
    interp.put_exec_state(state);
    let result = interp.exec(program, func, &[argument]);
    let value = match result {
        Ok(values) => values.first().copied().unwrap_or(0),
        Err(trap) => {
            let at = trap
                .loc
                .map(|(_, line, col)| format!(" (line {line}, column {col})"))
                .unwrap_or_default();
            eprintln!("error: runtime error in a thread: {}{at}", trap.message);
            std::process::exit(1);
        }
    };
    interp.finish_thread(id, value);
}
