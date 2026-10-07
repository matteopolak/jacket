//! Threads without OS threads, for the sandbox (the browser has none).
//!
//! All threads run on the one host thread, one at a time, each on a value stack of its own.
//! The interpreter is recursive Rust, so a thread cannot simply be paused where it is: one that
//! blocks (a contended mutex, a condition wait, `pthread_join`, `sleep`) or yields (`sched_yield`,
//! or after `POLLS_BEFORE_SWITCH` polls, so a busy wait progresses) *suspends*. It returns a
//! `TrapKind::Suspended` trap, and every interpreted frame it unwinds through saves its state
//! (`SuspFrame`: block, op, value registers; the frame's memory stays on the thread's value
//! stack). The trap reaches the scheduler loop (`inline_schedule`) in the outermost
//! `Interp::call`, which runs another thread; resuming one rebuilds its frames
//! (`resume_frames`) and runs the foreign call it blocked in again, which then finds its wait
//! over. Blocking calls whose progress cannot be read off the scheduler again keep a `Resume`
//! record (a condition wait that has already released its mutex, a sleep's deadline).
//!
//! * A thread runs when its wait is satisfied; the others are tried round-robin from the one
//!   that just stopped, so runs are deterministic.
//! * When nothing can run, the virtual clock jumps to the earliest deadline (timed waits,
//!   sleeps). Threads that only yielded (polling for something) come after that, so a thread
//!   spinning on a flag another thread sets after a sleep lets the sleep end.
//! * With nothing runnable, no deadline and nobody yielding, it is a deadlock error.
//!
//! The native interpreter uses real OS threads instead (`threads.rs`).
#![allow(unsafe_code)]

use super::*;
use std::collections::{HashSet, VecDeque};

const EPERM: u64 = 1;
const EBUSY: u64 = 16;
const EINVAL: u64 = 22;
const ETIMEDOUT: u64 = 110;

/// Polls (see `inline_poll`) after which a running thread lets the others run.
const POLLS_BEFORE_SWITCH: u64 = 1_000;

/// Value stack of a thread other than the one `Interp::call` started with.
const THREAD_STACK: usize = 8 << 20;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// Created, not started yet.
    Pending,
    Running,
    /// Blocked or yielded; `ThreadRec::frames` holds where it stopped.
    Suspended,
    Finished,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wait {
    /// Yielded: may run again whenever the others let it.
    None,
    Join(usize),
    Mutex(u64),
    Cond(u64),
    /// Only a deadline ends it (sleep).
    Time,
    /// Let the others run before it waits for the embedding page (`jai_sched_yield_for_wait`):
    /// unlike a yield, it may run again as soon as they stop, before the clock jumps to a
    /// sleeper's deadline, since its wait takes real time that moves the clock anyway.
    Host,
}

/// What a blocking call that suspended had done already, for when it runs again.
#[derive(Clone, Copy)]
enum Resume {
    /// `pthread_cond_wait` released `held` levels of its mutex and waits for `token`.
    Cond {
        token: u64,
        held: u32,
    },
    /// `pthread_cond_wait` was woken (or timed out) and waits to take its mutex back.
    Relock {
        held: u32,
        timed_out: bool,
    },
    Sleep {
        until: u64,
    },
    Yield,
}

/// An interpreted frame of a suspended thread.
pub(super) struct SuspFrame {
    exit: FrameExit,
    frame: Rc<Frame>,
    stack_base: u64,
    vals: Vec<u64>,
    /// The block and op it stopped at: the innermost frame runs that op again (the blocking
    /// call), or starts the block there (a preemption); the others take the results of the
    /// call at that op and go on after it.
    at: (usize, usize),
}

struct ThreadRec {
    func: FuncId,
    argument: u64,
    state: State,
    result: u64,
    /// While suspended: what it waits for, and until when at most.
    wait: Wait,
    deadline: Option<u64>,
    /// Picked because its deadline came: the wait it runs again times out.
    expired: bool,
    resume: Option<Resume>,
    /// While suspended: its frames, innermost first, and its value stack and call state.
    frames: Vec<SuspFrame>,
    exec: Option<ExecState>,
}

impl ThreadRec {
    fn new(func: FuncId, argument: u64, state: State) -> Self {
        ThreadRec {
            func,
            argument,
            state,
            result: 0,
            wait: Wait::None,
            deadline: None,
            expired: false,
            resume: None,
            frames: Vec::new(),
            exec: None,
        }
    }
}

#[derive(Default)]
struct MutexState {
    owner: Option<usize>,
    count: u32,
}

pub(super) struct InlineSched {
    threads: Vec<ThreadRec>,
    /// The thread running now. Thread 0 is the one `Interp::call` started with.
    current: usize,
    mutexes: HashMap<u64, MutexState>,
    cond_waiters: HashMap<u64, VecDeque<u64>>,
    woken: HashSet<u64>,
    next_token: u64,
    /// Polls since the running thread last let the others run.
    polls: u64,
    /// Value stacks of finished threads, for new ones.
    spare_stacks: Vec<Box<[u64]>>,
}

impl InlineSched {
    fn new() -> Self {
        InlineSched {
            threads: vec![ThreadRec::new(FuncId(0), 0, State::Running)],
            current: 0,
            mutexes: HashMap::default(),
            cond_waiters: HashMap::default(),
            woken: HashSet::default(),
            next_token: 1,
            polls: 0,
            spare_stacks: Vec::new(),
        }
    }

    fn satisfied(&self, thread: usize, wait: Wait) -> bool {
        match wait {
            Wait::None => true,
            Wait::Join(t) => self.threads[t].state == State::Finished,
            Wait::Mutex(addr) => self
                .mutexes
                .get(&addr)
                .is_none_or(|m| m.owner.is_none() || m.owner == Some(thread)),
            Wait::Cond(token) => self.woken.contains(&token),
            Wait::Time => false,
            Wait::Host => true,
        }
    }

    /// The other threads, round-robin after the current one, which comes last.
    fn order(&self) -> impl Iterator<Item = usize> + use<> {
        let (n, current) = (self.threads.len(), self.current);
        (1..=n).map(move |k| (current + k) % n)
    }

    /// May run now without waiting for anything: not started yet, or its wait is over.
    fn ready(&self, t: usize) -> bool {
        let rec = &self.threads[t];
        match rec.state {
            State::Pending => true,
            // `expired`: its deadline passed while another thread waited for the page.
            State::Suspended => {
                rec.expired || (rec.wait != Wait::None && self.satisfied(t, rec.wait))
            }
            State::Running | State::Finished => false,
        }
    }

    fn yielded(&self, t: usize) -> bool {
        let rec = &self.threads[t];
        rec.state == State::Suspended && rec.wait == Wait::None
    }

    /// The suspended thread (other than `except`) with the earliest deadline.
    fn earliest_deadline(&self, except: Option<usize>) -> Option<(u64, usize)> {
        self.order()
            .filter(|&t| Some(t) != except && self.threads[t].state == State::Suspended)
            .filter_map(|t| self.threads[t].deadline.map(|at| (at, t)))
            .min_by_key(|&(at, _)| at)
    }

    /// Whether any other thread could run, now or once the clock moves.
    fn others_can_go_on(&self) -> bool {
        let me = self.current;
        self.order().any(|t| {
            t != me && (self.ready(t) || self.yielded(t) || self.threads[t].deadline.is_some())
        })
    }
}

impl Interp {
    fn isched(&mut self) -> &mut InlineSched {
        self.isched
            .get_or_insert_with(|| Box::new(InlineSched::new()))
    }

    /// Whether the running thread may suspend: only in the outermost `Interp::call`, where every
    /// Rust frame between it and the scheduler loop is an interpreted frame that can be saved.
    fn can_suspend(&self) -> bool {
        self.call_nesting == 1 && self.isched.is_some()
    }

    fn suspend<T>(&self) -> Res<T> {
        Err(Trap {
            message: "thread suspended".into(),
            loc: self.loc,
            kind: Some(TrapKind::Suspended),
            ..Trap::default()
        })
    }

    fn take_resume(&mut self) -> Option<Resume> {
        let sched = self.isched();
        let me = sched.current;
        sched.threads[me].resume.take()
    }

    fn set_resume(&mut self, resume: Resume) {
        let sched = self.isched();
        let me = sched.current;
        sched.threads[me].resume = Some(resume);
    }

    /// Foreign procedures the inline scheduler implements. `None`: not one of them.
    pub(super) fn inline_thread_foreign(
        &mut self,
        _program: &Program,
        symbol: &str,
        args: &[u64],
    ) -> Option<Res<Vec<u64>>> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        let started = self.multi;
        let result: Res<u64> = match symbol {
            "pthread_create" => self.inline_create(arg(0), arg(2), arg(3)),
            "pthread_join" => self.inline_join(arg(0), arg(1)),
            "pthread_tryjoin_np" => {
                let id = arg(0).wrapping_sub(1) as usize;
                match self.isched().threads.get(id) {
                    Some(t) if t.state == State::Finished => {
                        let value = t.result;
                        if arg(1) != 0
                            && let Err(trap) = self.put(arg(1), &value.to_le_bytes())
                        {
                            return Some(Err(trap));
                        }
                        Ok(0)
                    }
                    Some(_) => {
                        self.inline_poll();
                        Ok(EBUSY)
                    }
                    None => Ok(EINVAL),
                }
            }
            "pthread_detach" => Ok(0),
            "pthread_self" => Ok(self.isched().current as u64 + 1),
            "pthread_mutex_init" => {
                self.isched().mutexes.insert(arg(0), MutexState::default());
                Ok(0)
            }
            "pthread_mutex_destroy" => {
                self.isched().mutexes.remove(&arg(0));
                Ok(0)
            }
            "pthread_mutex_lock" => self.inline_lock(arg(0), 1).map(|_| 0),
            "pthread_mutex_trylock" => {
                let me = self.isched().current;
                let state = self.isched().mutexes.entry(arg(0)).or_default();
                if state.owner.is_none() || state.owner == Some(me) {
                    state.owner = Some(me);
                    state.count += 1;
                    Ok(0)
                } else {
                    self.inline_poll();
                    Ok(EBUSY)
                }
            }
            "pthread_mutex_unlock" => Ok(match self.inline_unlock(arg(0)) {
                Ok(()) => 0,
                Err(code) => code,
            }),
            "pthread_cond_init" | "pthread_cond_destroy" => {
                self.isched().cond_waiters.remove(&arg(0));
                Ok(0)
            }
            "pthread_cond_signal" => {
                self.inline_wake(arg(0), false);
                Ok(0)
            }
            "pthread_cond_broadcast" => {
                self.inline_wake(arg(0), true);
                Ok(0)
            }
            "pthread_cond_wait" => self.inline_cond_wait(arg(0), arg(1), None),
            "pthread_cond_timedwait" => {
                match (self.fetch_u64(arg(2)), self.fetch_u64(arg(2) + 8)) {
                    (Ok(secs), Ok(nanos)) => {
                        let deadline = secs.saturating_mul(1_000_000_000).saturating_add(nanos);
                        self.inline_cond_wait(arg(0), arg(1), Some(deadline))
                    }
                    (Err(trap), _) | (_, Err(trap)) => Err(trap),
                }
            }
            // Sleeping and yielding only matter once another thread exists.
            "nanosleep" if started => match (self.fetch_u64(arg(0)), self.fetch_u64(arg(0) + 8)) {
                (Ok(secs), Ok(nanos)) => {
                    self.inline_sleep(secs.saturating_mul(1_000_000_000) + nanos)
                }
                (Err(trap), _) | (_, Err(trap)) => Err(trap),
            },
            "usleep" if started => self.inline_sleep((arg(0) as u32 as u64) * 1_000),
            "sleep" if started => self.inline_sleep((arg(0) as u32 as u64) * 1_000_000_000),
            "sched_yield" | "pthread_yield_np" if started => self.inline_yield().map(|_| 0),
            "jai_sched_yield_for_wait" => self.inline_yield_for_wait(),
            _ => return None,
        };
        Some(result.map(|v| vec![v]))
    }

    fn inline_create(&mut self, out: u64, start: u64, argument: u64) -> Res<u64> {
        let Some(func) = self.func_of(start) else {
            return self.trap("pthread_create needs an interpreted thread procedure");
        };
        let sched = self.isched();
        sched
            .threads
            .push(ThreadRec::new(func, argument, State::Pending));
        let id = sched.threads.len() - 1;
        self.multi = true;
        if out != 0 {
            self.put(out, &(id as u64 + 1).to_le_bytes())?;
        }
        Ok(0)
    }

    fn inline_join(&mut self, handle: u64, result_out: u64) -> Res<u64> {
        let me = self.isched().current;
        let id = handle.wrapping_sub(1) as usize;
        if id >= self.isched().threads.len() || id == me {
            return Ok(EINVAL);
        }
        self.wait_until(Wait::Join(id), None)?;
        if result_out != 0 {
            let value = self.isched().threads[id].result;
            self.put(result_out, &value.to_le_bytes())?;
        }
        Ok(0)
    }

    fn inline_lock(&mut self, addr: u64, count: u32) -> Res<()> {
        loop {
            let me = self.isched().current;
            let state = self.isched().mutexes.entry(addr).or_default();
            if state.owner.is_none() || state.owner == Some(me) {
                state.owner = Some(me);
                state.count += count;
                return Ok(());
            }
            self.wait_until(Wait::Mutex(addr), None)?;
        }
    }

    /// Release one level of the mutex; `Err(EPERM)` when the caller does not hold it.
    fn inline_unlock(&mut self, addr: u64) -> Result<(), u64> {
        let me = self.isched().current;
        let Some(state) = self.isched().mutexes.get_mut(&addr) else {
            return Err(EPERM);
        };
        if state.owner != Some(me) {
            return Err(EPERM);
        }
        state.count -= 1;
        if state.count == 0 {
            state.owner = None;
        }
        Ok(())
    }

    fn inline_cond_wait(&mut self, cond: u64, mutex: u64, deadline: Option<u64>) -> Res<u64> {
        let (held, timed_out) = match self.take_resume() {
            Some(Resume::Relock {
                held,
                timed_out,
            }) => (held, timed_out),
            resume => {
                let (token, held) = match resume {
                    Some(Resume::Cond {
                        token,
                        held,
                    }) => (token, held),
                    _ => self.inline_cond_enter(cond, mutex),
                };
                let outcome = self.wait_until(Wait::Cond(token), deadline);
                if suspended(&outcome) {
                    self.set_resume(Resume::Cond {
                        token,
                        held,
                    });
                    return outcome.map(|_| 0);
                }
                let sched = self.isched();
                sched.woken.remove(&token);
                if let Some(waiters) = sched.cond_waiters.get_mut(&cond) {
                    waiters.retain(|&t| t != token);
                }
                (held, outcome?)
            }
        };
        if held > 0 {
            let relocked = self.inline_lock(mutex, held);
            if suspended(&relocked) {
                self.set_resume(Resume::Relock {
                    held,
                    timed_out,
                });
            }
            relocked?;
        }
        Ok(if timed_out {
            ETIMEDOUT
        } else {
            0
        })
    }

    /// Release the mutex completely and queue a wait on `cond`: its token and the levels held.
    fn inline_cond_enter(&mut self, cond: u64, mutex: u64) -> (u64, u32) {
        let me = self.isched().current;
        let held = match self.isched().mutexes.get(&mutex) {
            Some(state) if state.owner == Some(me) => state.count,
            _ => 0,
        };
        for _ in 0..held {
            let _ = self.inline_unlock(mutex);
        }
        let sched = self.isched();
        let token = sched.next_token;
        sched.next_token += 1;
        sched.cond_waiters.entry(cond).or_default().push_back(token);
        (token, held)
    }

    fn inline_wake(&mut self, cond: u64, all: bool) {
        let sched = self.isched();
        let Some(waiters) = sched.cond_waiters.get_mut(&cond) else {
            return;
        };
        while let Some(token) = waiters.pop_front() {
            sched.woken.insert(token);
            if !all {
                break;
            }
        }
    }

    /// Sleeping waits until the virtual clock reaches the deadline, which happens once no
    /// thread can run without it moving (see the module comment).
    fn inline_sleep(&mut self, nanoseconds: u64) -> Res<u64> {
        let until = match self.take_resume() {
            Some(Resume::Sleep {
                until,
            }) => until,
            Some(Resume::Yield) => return Ok(0),
            _ => match self.host.virtual_now_ns() {
                Some(now) if nanoseconds > 0 => now.saturating_add(nanoseconds),
                _ => return self.inline_yield().map(|_| 0),
            },
        };
        let outcome = self.wait_until(Wait::Time, Some(until));
        if suspended(&outcome) {
            self.set_resume(Resume::Sleep {
                until,
            });
        }
        outcome.map(|_| 0)
    }

    /// Let the other threads run, if any can.
    fn inline_yield(&mut self) -> Res<()> {
        if let Some(Resume::Yield) = self.take_resume() {
            return Ok(());
        }
        if !self.can_suspend() || !self.isched().others_can_go_on() {
            return Ok(());
        }
        self.set_resume(Resume::Yield);
        self.suspend_as(Wait::None, None)
    }

    /// `jai_sched_yield_for_wait` (stdlib/WebGPU/wasm.jai): the running thread is about to wait
    /// for the embedding page, which stops every thread. First the threads that can run now run
    /// (this one suspends with `Wait::Host`); then the result says whether another thread may
    /// still run later (a sleeper, a yielded thread), so the caller waits in short slices and
    /// asks again. Sleepers whose deadline the page's real time has passed count as ready.
    fn inline_yield_for_wait(&mut self) -> Res<u64> {
        if !self.multi || self.isched.is_none() {
            return Ok(0);
        }
        let resumed = matches!(self.take_resume(), Some(Resume::Yield));
        if let Some(now) = self.host.virtual_now_ns() {
            for rec in &mut self.isched().threads {
                if rec.state == State::Suspended && rec.deadline.is_some_and(|at| at <= now) {
                    rec.expired = true;
                }
            }
        }
        let can_suspend = self.can_suspend();
        let sched = self.isched();
        let me = sched.current;
        if !resumed && can_suspend && sched.order().any(|t| t != me && sched.ready(t)) {
            self.set_resume(Resume::Yield);
            return self.suspend_as(Wait::Host, None);
        }
        Ok(u64::from(self.isched().others_can_go_on()))
    }

    /// A sign that the running thread waits for another one: an atomic compare-and-swap that
    /// fails or writes the value already there (`atomic_read`, a spin on a taken lock), a
    /// `pause`, or a `trylock`/`tryjoin` that finds the mutex or thread busy.
    pub(super) fn inline_poll(&mut self) {
        self.isched().polls += 1;
    }

    /// Called between basic blocks while threads exist: once the running thread has polled
    /// for a while, it yields, so a busy wait progresses. Only polling switches, which keeps
    /// runs deterministic and switches rare.
    pub(super) fn inline_preempt(&mut self, _program: &Program) -> Res<()> {
        let sched = self.isched();
        if sched.polls < POLLS_BEFORE_SWITCH {
            return Ok(());
        }
        sched.polls = 0;
        if !self.can_suspend() || !self.isched().others_can_go_on() {
            return Ok(());
        }
        self.suspend_as(Wait::None, None)
    }

    /// Record what the running thread waits for and unwind it (`TrapKind::Suspended`).
    fn suspend_as<T>(&mut self, wait: Wait, deadline: Option<u64>) -> Res<T> {
        let sched = self.isched();
        let me = sched.current;
        let rec = &mut sched.threads[me];
        rec.wait = wait;
        rec.deadline = deadline;
        self.suspend()
    }

    /// Block the running thread until `wait` holds. `deadline` (absolute virtual nanoseconds)
    /// makes it return `true` instead once it passes. When other threads must run first, this
    /// suspends the thread; the call that blocked runs again when it resumes and comes back
    /// here. `Err` also for a wait nothing can ever satisfy (a deadlock).
    fn wait_until(&mut self, wait: Wait, deadline: Option<u64>) -> Res<bool> {
        let sched = self.isched();
        let me = sched.current;
        let expired = std::mem::take(&mut sched.threads[me].expired);
        if sched.satisfied(me, wait) {
            return Ok(false);
        }
        if expired {
            return Ok(true);
        }
        let now = self.host.virtual_now_ns();
        if let (Some(at), Some(now)) = (deadline, now)
            && now >= at
        {
            return Ok(true);
        }
        let can_suspend = self.can_suspend();
        let sched = self.isched();
        let others_ready = sched.order().any(|t| t != me && sched.ready(t));
        if !others_ready || !can_suspend {
            // Nothing else runs before the clock moves: when this deadline comes first, it is
            // over now.
            let other = sched.earliest_deadline(Some(me)).map(|(at, _)| at);
            if let Some(at) = deadline
                && (other.is_none_or(|o| at <= o) || !can_suspend)
            {
                if let Some(now) = now {
                    self.host.advance_clock(at.saturating_sub(now));
                }
                return Ok(true);
            }
            if !can_suspend || !self.isched().others_can_go_on() {
                return self.trap("deadlock: every thread is blocked");
            }
        }
        self.suspend_as(wait, deadline)
    }

    /// The scheduler loop: `result` is the outcome of the thread `Interp::call` started with,
    /// which has suspended. Runs threads until that one returns.
    pub(super) fn inline_schedule(
        &mut self,
        program: &Program,
        mut result: Res<Rets>,
    ) -> Res<Rets> {
        let main = self.isched().current;
        loop {
            let current = self.isched().current;
            match result {
                Err(trap) if trap.kind == Some(TrapKind::Suspended) => {
                    let frames = std::mem::take(&mut self.captured);
                    let exec = self.take_exec_state();
                    let rec = &mut self.isched().threads[current];
                    rec.frames = frames;
                    rec.exec = Some(exec);
                    rec.state = State::Suspended;
                }
                Ok(rets) if current == main => return Ok(rets),
                Ok(rets) => {
                    let stack = self.take_exec_state().stack;
                    let sched = self.isched();
                    sched.threads[current].state = State::Finished;
                    sched.threads[current].result = rets.first().copied().unwrap_or(0);
                    sched.spare_stacks.push(stack);
                }
                Err(trap) => {
                    self.inline_back_to(main);
                    if current == main {
                        return Err(trap);
                    }
                    let at = trap
                        .loc
                        .map(|(_, line, col)| format!(" (line {line}, column {col})"))
                        .unwrap_or_default();
                    return Err(Trap {
                        message: format!("runtime error in a thread: {}{at}", trap.message),
                        ..trap
                    });
                }
            }
            let Some(next) = self.inline_pick() else {
                // Report where the last thread to stop waits, or else where `main` does.
                let sched = self.isched();
                let loc_of = |t: usize| sched.threads[t].exec.as_ref().and_then(|e| e.loc);
                let loc = loc_of(current).or_else(|| loc_of(main));
                self.inline_back_to(main);
                self.loc = loc;
                return self.trap("deadlock: every thread is blocked");
            };
            let sched = self.isched();
            sched.current = next;
            sched.polls = 0;
            let rec = &mut sched.threads[next];
            let was = rec.state;
            rec.state = State::Running;
            rec.wait = Wait::None;
            rec.deadline = None;
            let (func, argument) = (rec.func, rec.argument);
            let frames = std::mem::take(&mut rec.frames);
            let exec = rec.exec.take();
            result = if was == State::Pending {
                let stack = sched
                    .spare_stacks
                    .pop()
                    .unwrap_or_else(|| vec![0u64; THREAD_STACK / 8].into_boxed_slice());
                self.put_exec_state(ExecState {
                    stack,
                    ..ExecState::default()
                });
                self.exec(program, func, &[argument])
            } else {
                self.put_exec_state(exec.unwrap_or_default());
                self.resume_frames(program, frames)
            };
        }
    }

    /// The thread to run next (see the module comment); moves the clock to a deadline if
    /// that is what lets one run. `None`: a deadlock.
    fn inline_pick(&mut self) -> Option<usize> {
        let sched = self.isched();
        if let Some(t) = sched.order().find(|&t| sched.ready(t)) {
            return Some(t);
        }
        if let Some((at, t)) = sched.earliest_deadline(None) {
            sched.threads[t].expired = true;
            if let Some(now) = self.host.virtual_now_ns() {
                self.host.advance_clock(at.saturating_sub(now));
            }
            return Some(t);
        }
        let sched = self.isched();
        sched.order().find(|&t| sched.yielded(t))
    }

    /// After an error: put back the value stack and call state of thread `main`, so the
    /// interpreter can run more code.
    fn inline_back_to(&mut self, main: usize) {
        let sched = self.isched();
        let current = sched.current;
        sched.current = main;
        let live = self.take_exec_state();
        if current == main {
            self.put_exec_state(live);
        } else {
            let sched = self.isched();
            if sched.threads[current].state == State::Running {
                sched.threads[current].state = State::Finished;
            }
            if !live.stack.is_empty() {
                sched.spare_stacks.push(live.stack);
            }
        }
        let rec = &mut self.isched().threads[main];
        rec.state = State::Running;
        rec.frames.clear();
        if let Some(exec) = rec.exec.take() {
            self.put_exec_state(exec);
        }
    }

    /// `exec` of a suspending thread: save the frame it is unwinding.
    pub(super) fn capture_frame(&mut self, exit: FrameExit, frame: Rc<Frame>, stack_base: u64) {
        let at = self
            .suspend_at
            .take()
            .expect("a suspending frame records where");
        let vals = self
            .suspend_vals
            .take()
            .expect("a suspending frame hands over its values");
        self.captured.push(SuspFrame {
            exit,
            frame,
            stack_base,
            vals,
            at,
        });
    }

    /// Continue a suspended thread: run its innermost frame from where it stopped, then each
    /// caller with the results of the frame above it.
    fn resume_frames(&mut self, program: &Program, frames: Vec<SuspFrame>) -> Res<Rets> {
        let mut rets: Option<Rets> = None;
        let mut frames = frames.into_iter();
        while let Some(mut f) = frames.next() {
            let func = program.funcs[f.exit.id.0 as usize]
                .as_ref()
                .expect("a suspended frame's procedure has a body");
            let (block, mut op) = f.at;
            if let Some(rets) = rets.take() {
                let (ir_block, inst) = f.frame.code.ir_op(op).expect("a frame suspends in a call");
                let Inst::Call(call) = &func.blocks[ir_block as usize].insts[inst as usize] else {
                    unreachable!("a frame suspends in a call");
                };
                for (r, &v) in call.results.iter().zip(rets.iter()) {
                    f.vals[r.0 as usize] = v;
                }
                op += 1;
            }
            (self.frame_blocks, self.frame_insts) = (0, 0);
            let frame = f.frame.clone();
            let result = self.run_code(
                program,
                func,
                &frame,
                f.stack_base,
                &mut f.vals,
                Some((block, op)),
            );
            if suspended(&result) {
                f.at = self
                    .suspend_at
                    .take()
                    .expect("a suspending frame records where");
                self.captured.push(f);
                self.captured.extend(frames);
                return result;
            }
            self.val_pool.push(std::mem::take(&mut f.vals));
            match self.leave_frame(func, f.exit, result) {
                Ok(r) => rets = Some(r),
                Err(mut trap) => {
                    for mut f in frames {
                        let func = program.funcs[f.exit.id.0 as usize].as_ref().unwrap();
                        self.val_pool.push(std::mem::take(&mut f.vals));
                        trap = self.leave_frame(func, f.exit, Err(trap)).unwrap_err();
                    }
                    return Err(trap);
                }
            }
        }
        Ok(rets.unwrap_or_default())
    }
}
