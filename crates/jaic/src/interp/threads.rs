//! Cooperative threads for interpreted programs.
//!
//! `pthread_create` starts an OS thread for each Jai thread, but only one of them runs at a
//! time: the one that holds the baton. The baton changes hands only where a thread blocks
//! (join, mutex, condition variable, sleep), yields, or has run for a while (`preempt`), so
//! the interpreter state (`Interp`) is never touched by two threads at once and a run is
//! deterministic apart from sleep timing. Mutexes and condition variables are implemented
//! here instead of calling libc, because a thread that blocked inside libc would keep the
//! baton and stop the others.
//!
//! The per-thread interpreter state (value stack, stack pointer, call depth, current line)
//! is swapped in and out of `Interp` when the baton moves.
#![allow(unsafe_code)]

use super::*;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// Interpreter value stack of a thread other than the first.
const THREAD_STACK: usize = 8 << 20;
/// Native stack reserved for each OS thread that runs interpreted code.
const NATIVE_STACK: usize = 256 << 20;
/// Block transitions between preemption checks.
const PREEMPT_TICKS: u64 = 20_000;

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
}

struct Saved {
    stack: Box<[u64]>,
    sp: u64,
    depth: usize,
    loc: Option<(u32, u32, u32)>,
    trace_loc: Option<Option<(u32, u32, u32)>>,
}

struct GThread {
    finished: bool,
    block: Block,
    /// A timed condition wait ended because its deadline passed.
    timed_out: bool,
    /// The scheduler found every thread blocked and woke this one to report it.
    deadlocked: bool,
    result: u64,
    saved: Saved,
}

#[derive(Default)]
struct MutexState {
    owner: Option<usize>,
    count: u32,
}

pub(super) struct Sched {
    threads: Vec<GThread>,
    current: usize,
    baton: Arc<(Mutex<usize>, Condvar)>,
    mutexes: HashMap<u64, MutexState>,
    cond_waiters: HashMap<u64, VecDeque<usize>>,
    ticks: u64,
}

impl Sched {
    fn new() -> Self {
        Sched {
            threads: vec![GThread {
                finished: false,
                block: Block::None,
                timed_out: false,
                deadlocked: false,
                result: 0,
                saved: Saved {
                    stack: Box::default(),
                    sp: 0,
                    depth: 0,
                    loc: None,
                    trace_loc: None,
                },
            }],
            current: 0,
            baton: Arc::new((Mutex::new(0), Condvar::new())),
            mutexes: HashMap::new(),
            cond_waiters: HashMap::new(),
            ticks: 0,
        }
    }
}

impl Interp {
    fn sched(&mut self) -> &mut Sched {
        self.sched.get_or_insert_with(|| Box::new(Sched::new()))
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
            "pthread_join" => self.thread_join(program, arg(0), arg(1)),
            "pthread_detach" => Ok(0),
            "pthread_self" => Ok(self.sched().current as u64 + 1),
            "pthread_mutex_init" => {
                self.sched().mutexes.insert(arg(0), MutexState::default());
                Ok(0)
            }
            "pthread_mutex_destroy" => {
                self.sched().mutexes.remove(&arg(0));
                Ok(0)
            }
            "pthread_mutex_lock" => self.lock_mutex(program, arg(0), 1).map(|_| 0),
            "pthread_mutex_trylock" => {
                let me = self.sched().current;
                let state = self.sched().mutexes.entry(arg(0)).or_default();
                if state.owner.is_none() || state.owner == Some(me) {
                    state.owner = Some(me);
                    state.count += 1;
                    Ok(0)
                } else {
                    Ok(EBUSY)
                }
            }
            "pthread_mutex_unlock" => Ok(self.unlock_mutex(arg(0)).err().unwrap_or(0)),
            "pthread_cond_init" | "pthread_cond_destroy" => {
                self.sched().cond_waiters.remove(&arg(0));
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
            "pthread_cond_wait" => self.cond_wait(program, arg(0), arg(1), None),
            "pthread_cond_timedwait" => {
                let deadline = absolute_deadline(arg(2));
                self.cond_wait(program, arg(0), arg(1), Some(deadline))
            }
            // Sleeping and yielding only matter once another thread exists.
            "nanosleep" if started => {
                let (secs, nanos) = (self.read_u64(arg(0)), self.read_u64(arg(0) + 8));
                self.sleep_for(program, Duration::new(secs, nanos as u32))
            }
            "usleep" if started => {
                self.sleep_for(program, Duration::from_micros(arg(0) as u32 as u64))
            }
            "sleep" if started => {
                self.sleep_for(program, Duration::from_secs(arg(0) as u32 as u64))
            }
            "sched_yield" | "pthread_yield_np" if started => self.yield_now(program).map(|_| 0),
            _ => return None,
        };
        Some(result.map(|v| vec![v]))
    }

    fn thread_create(
        &mut self,
        program: &Program,
        out: u64,
        start: u64,
        argument: u64,
    ) -> Res<u64> {
        if start & TAG_MASK != FUNC_TAG {
            return self.trap("pthread_create needs an interpreted thread procedure");
        }
        let func = FuncId((start & 0xFFFF_FFFF) as u32);
        let id = {
            let sched = self.sched();
            sched.threads.push(GThread {
                finished: false,
                block: Block::None,
                timed_out: false,
                deadlocked: false,
                result: 0,
                saved: Saved {
                    stack: vec![0u64; THREAD_STACK / 8].into_boxed_slice(),
                    sp: 0,
                    depth: 0,
                    loc: None,
                    trace_loc: None,
                },
            });
            sched.threads.len() - 1
        };
        self.multi = true;
        let baton = self.sched().baton.clone();
        let interp = self as *mut Interp as usize;
        let program = program as *const Program as usize;
        let spawned = std::thread::Builder::new()
            .stack_size(NATIVE_STACK)
            .spawn(move || thread_main(interp, program, baton, id, func, argument));
        if spawned.is_err() {
            let sched = self.sched();
            sched.threads[id].finished = true;
            return Ok(11); // EAGAIN
        }
        if out != 0 {
            self.write(out, &(id as u64 + 1).to_le_bytes());
        }
        Ok(0)
    }

    fn thread_join(&mut self, program: &Program, handle: u64, result_out: u64) -> Res<u64> {
        let me = self.sched().current;
        let id = handle.wrapping_sub(1) as usize;
        if id >= self.sched().threads.len() || id == me {
            return Ok(EINVAL);
        }
        while !self.sched().threads[id].finished {
            self.block(program, Block::Join(id))?;
        }
        if result_out != 0 {
            let value = self.sched().threads[id].result;
            self.write(result_out, &value.to_le_bytes());
        }
        Ok(0)
    }

    fn lock_mutex(&mut self, program: &Program, addr: u64, count: u32) -> Res<()> {
        loop {
            let me = self.sched().current;
            let state = self.sched().mutexes.entry(addr).or_default();
            if state.owner.is_none() || state.owner == Some(me) {
                state.owner = Some(me);
                state.count += count;
                return Ok(());
            }
            self.block(program, Block::Mutex(addr))?;
        }
    }

    /// Release one level of the mutex; `Err(EPERM)` when the caller does not hold it.
    fn unlock_mutex(&mut self, addr: u64) -> Result<(), u64> {
        let me = self.sched().current;
        let sched = self.sched();
        let Some(state) = sched.mutexes.get_mut(&addr) else {
            return Err(EPERM);
        };
        if state.owner != Some(me) {
            return Err(EPERM);
        }
        state.count -= 1;
        if state.count == 0 {
            state.owner = None;
            for thread in &mut sched.threads {
                if thread.block == Block::Mutex(addr) {
                    thread.block = Block::None;
                }
            }
        }
        Ok(())
    }

    fn cond_wait(
        &mut self,
        program: &Program,
        cond: u64,
        mutex: u64,
        deadline: Option<Instant>,
    ) -> Res<u64> {
        let me = self.sched().current;
        // Give the mutex up completely while waiting, and take it back as it was.
        let held = match self.sched().mutexes.get(&mutex) {
            Some(state) if state.owner == Some(me) => state.count,
            _ => 0,
        };
        for _ in 0..held {
            let _ = self.unlock_mutex(mutex);
        }
        {
            let sched = self.sched();
            sched.cond_waiters.entry(cond).or_default().push_back(me);
            sched.threads[me].timed_out = false;
        }
        let waited = self.block(program, Block::Cond(cond, deadline));
        if waited.is_err() {
            return waited.map(|_| 0);
        }
        let timed_out = std::mem::take(&mut self.sched().threads[me].timed_out);
        if held > 0 {
            self.lock_mutex(program, mutex, held)?;
        }
        Ok(if timed_out {
            ETIMEDOUT
        } else {
            0
        })
    }

    fn wake_cond(&mut self, cond: u64, all: bool) {
        let sched = self.sched();
        let Some(waiters) = sched.cond_waiters.get_mut(&cond) else {
            return;
        };
        while let Some(id) = waiters.pop_front() {
            if matches!(sched.threads[id].block, Block::Cond(c, _) if c == cond) {
                sched.threads[id].block = Block::None;
            }
            if !all {
                break;
            }
        }
    }

    fn sleep_for(&mut self, program: &Program, duration: Duration) -> Res<u64> {
        self.block(program, Block::Sleep(Instant::now() + duration))?;
        Ok(0)
    }

    /// Let another runnable thread go first.
    pub(super) fn yield_now(&mut self, program: &Program) -> Res<()> {
        let me = self.sched().current;
        self.switch(program, me)
    }

    /// Called between basic blocks while several threads exist: after enough of them, offer the
    /// baton to the other threads so that busy-waiting on an atomic makes progress.
    pub(super) fn preempt(&mut self, program: &Program) -> Res<()> {
        let sched = self.sched();
        sched.ticks += 1;
        if sched.ticks % PREEMPT_TICKS == 0 {
            self.yield_now(program)?;
        }
        Ok(())
    }

    fn block(&mut self, program: &Program, reason: Block) -> Res<()> {
        let me = self.sched().current;
        self.sched().threads[me].block = reason;
        let switched = self.switch(program, me);
        let sched = self.sched();
        sched.threads[me].block = Block::None;
        if std::mem::take(&mut sched.threads[me].deadlocked) {
            return self.trap("deadlock: every thread is blocked");
        }
        switched
    }

    /// Hand the baton to the next runnable thread (`me` itself when it is the only one).
    #[inline(never)]
    fn switch(&mut self, program: &Program, me: usize) -> Res<()> {
        let _ = program;
        let next = loop {
            let sched = self.sched();
            let now = Instant::now();
            for t in 0..sched.threads.len() {
                match sched.threads[t].block {
                    Block::Sleep(d) if d <= now => sched.threads[t].block = Block::None,
                    Block::Cond(c, Some(d)) if d <= now => {
                        sched.threads[t].block = Block::None;
                        sched.threads[t].timed_out = true;
                        if let Some(waiters) = sched.cond_waiters.get_mut(&c) {
                            waiters.retain(|&w| w != t);
                        }
                    }
                    _ => {}
                }
            }
            let n = sched.threads.len();
            let runnable = (1..=n)
                .map(|k| (me + k) % n)
                .find(|&t| !sched.threads[t].finished && sched.threads[t].block == Block::None);
            if let Some(t) = runnable {
                break t;
            }
            let earliest = sched
                .threads
                .iter()
                .filter(|t| !t.finished)
                .filter_map(|t| match t.block {
                    Block::Sleep(d) | Block::Cond(_, Some(d)) => Some(d),
                    _ => None,
                })
                .min();
            match earliest {
                Some(d) => {
                    let now = Instant::now();
                    if d > now {
                        std::thread::sleep(d - now);
                    }
                }
                None if sched.threads[me].finished => {
                    // Nothing can run again: report it to the first thread.
                    sched.threads[0].block = Block::None;
                    sched.threads[0].deadlocked = true;
                    break 0;
                }
                None => return self.trap("deadlock: every thread is blocked"),
            }
        };
        if next == me {
            return Ok(());
        }
        self.hand_over(me, next);
        Ok(())
    }

    /// Give `next` the baton and wait until somebody hands it back to `me`.
    #[inline(never)]
    fn hand_over(&mut self, me: usize, next: usize) {
        let saved = Saved {
            stack: std::mem::take(&mut self.stack),
            sp: self.sp,
            depth: self.depth,
            loc: self.loc,
            trace_loc: self.trace_loc,
        };
        let baton = {
            let sched = self.sched();
            sched.threads[me].saved = saved;
            sched.current = next;
            sched.baton.clone()
        };
        pass_baton(&baton, next);
        wait_baton(&baton, me);
        self.restore(me);
    }

    fn restore(&mut self, me: usize) {
        let saved = {
            let sched = self.sched();
            sched.current = me;
            std::mem::replace(
                &mut sched.threads[me].saved,
                Saved {
                    stack: Box::default(),
                    sp: 0,
                    depth: 0,
                    loc: None,
                    trace_loc: None,
                },
            )
        };
        self.stack = saved.stack;
        self.sp = saved.sp;
        self.depth = saved.depth;
        self.loc = saved.loc;
        self.trace_loc = saved.trace_loc;
    }

    /// The thread `me` has ended: wake its joiners and pass the baton on for good.
    fn finish_thread(&mut self, program: &Program, me: usize, result: u64) {
        self.stack = Box::default();
        {
            let sched = self.sched();
            sched.threads[me].finished = true;
            sched.threads[me].result = result;
            for thread in &mut sched.threads {
                if thread.block == Block::Join(me) {
                    thread.block = Block::None;
                }
            }
        }
        // `switch` never returns to a finished thread: it either finds another thread or
        // reports a deadlock to the first one.
        let next = self.pick_after_exit(program, me);
        let baton = {
            let sched = self.sched();
            sched.current = next;
            sched.baton.clone()
        };
        pass_baton(&baton, next);
    }

    fn pick_after_exit(&mut self, program: &Program, me: usize) -> usize {
        // Reuse the scheduler's search; a finished `me` is never picked.
        let _ = program;
        loop {
            let sched = self.sched();
            let now = Instant::now();
            for t in 0..sched.threads.len() {
                match sched.threads[t].block {
                    Block::Sleep(d) if d <= now => sched.threads[t].block = Block::None,
                    Block::Cond(c, Some(d)) if d <= now => {
                        sched.threads[t].block = Block::None;
                        sched.threads[t].timed_out = true;
                        if let Some(waiters) = sched.cond_waiters.get_mut(&c) {
                            waiters.retain(|&w| w != t);
                        }
                    }
                    _ => {}
                }
            }
            let n = sched.threads.len();
            if let Some(t) = (1..=n)
                .map(|k| (me + k) % n)
                .find(|&t| !sched.threads[t].finished && sched.threads[t].block == Block::None)
            {
                return t;
            }
            let earliest = sched
                .threads
                .iter()
                .filter(|t| !t.finished)
                .filter_map(|t| match t.block {
                    Block::Sleep(d) | Block::Cond(_, Some(d)) => Some(d),
                    _ => None,
                })
                .min();
            match earliest {
                Some(d) => {
                    let now = Instant::now();
                    if d > now {
                        std::thread::sleep(d - now);
                    }
                }
                None => {
                    sched.threads[0].block = Block::None;
                    sched.threads[0].deadlocked = true;
                    return 0;
                }
            }
        }
    }
}

fn pass_baton(baton: &Arc<(Mutex<usize>, Condvar)>, to: usize) {
    *baton.0.lock().unwrap() = to;
    baton.1.notify_all();
}

fn wait_baton(baton: &Arc<(Mutex<usize>, Condvar)>, me: usize) {
    let mut holder = baton.0.lock().unwrap();
    while *holder != me {
        holder = baton.1.wait(holder).unwrap();
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
fn thread_main(
    interp: usize,
    program: usize,
    baton: Arc<(Mutex<usize>, Condvar)>,
    id: usize,
    func: FuncId,
    argument: u64,
) {
    wait_baton(&baton, id);
    // SAFETY: only the thread holding the baton touches the interpreter, and the program
    // outlives every thread (the process ends when the program's `main` returns).
    let interp = unsafe { &mut *(interp as *mut Interp) };
    let program = unsafe { &*(program as *const Program) };
    interp.restore(id);
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
    interp.finish_thread(program, id, value);
}
