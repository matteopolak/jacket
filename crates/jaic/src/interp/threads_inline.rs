//! Threads without OS threads, for the sandbox (the browser has none).
//!
//! `pthread_create` only records the thread. A recorded thread runs, to completion, on top of the
//! interpreter stack of whichever thread blocks first: at `pthread_join`, a contended mutex, a
//! condition wait, `sleep`/`nanosleep`/`sched_yield`, and now and then while a thread keeps
//! running (so busy-waiting on an atomic makes progress). Because a thread that has started can
//! only continue after everything stacked above it has returned, the schedule is a stack:
//!
//! * A wait that nothing runnable can satisfy but that a thread lower on the stack could end
//!   *abandons* the threads above it: they are unwound (their mutexes are released) and count as
//!   finished. This is what lets a worker that waits for more work give control back to the
//!   thread that is waiting for the worker's results. Work handed to an abandoned worker later
//!   is never done, and the program then ends with a deadlock error.
//! * A wait nothing can satisfy at all is a deadlock error.
//! * Timed waits that cannot be satisfied time out at once and advance the virtual clock.
//!
//! Runs are deterministic. The native interpreter uses real OS threads instead (`threads.rs`).
#![allow(unsafe_code)]

use super::*;
use std::collections::{HashSet, VecDeque};

const EPERM: u64 = 1;
const EBUSY: u64 = 16;
const EINVAL: u64 = 22;
const ETIMEDOUT: u64 = 110;

/// Block transitions between preemption checks.
const PREEMPT_TICKS: u64 = 20_000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Pending,
    Running,
    Finished,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wait {
    /// Running, or lent its stack to pending threads (sleep, yield).
    None,
    Join(usize),
    Mutex(u64),
    Cond(u64),
}

struct ThreadRec {
    func: FuncId,
    argument: u64,
    state: State,
    result: u64,
}

struct Level {
    thread: usize,
    wait: Wait,
}

#[derive(Default)]
struct MutexState {
    owner: Option<usize>,
    count: u32,
}

pub(super) struct InlineSched {
    threads: Vec<ThreadRec>,
    /// Threads that have started and not returned, lowest first. Level 0 is the main thread.
    levels: Vec<Level>,
    mutexes: HashMap<u64, MutexState>,
    cond_waiters: HashMap<u64, VecDeque<u64>>,
    woken: HashSet<u64>,
    next_token: u64,
    ticks: u64,
    /// While unwinding abandoned threads: the level that continues.
    unwind_to: Option<usize>,
}

impl InlineSched {
    fn new() -> Self {
        InlineSched {
            threads: vec![ThreadRec {
                func: FuncId(0),
                argument: 0,
                state: State::Running,
                result: 0,
            }],
            levels: vec![Level {
                thread: 0,
                wait: Wait::None,
            }],
            mutexes: HashMap::default(),
            cond_waiters: HashMap::default(),
            woken: HashSet::default(),
            next_token: 1,
            ticks: 0,
            unwind_to: None,
        }
    }

    fn current(&self) -> usize {
        self.levels.last().map_or(0, |l| l.thread)
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
        }
    }

    fn has_pending(&self) -> bool {
        self.threads.iter().any(|t| t.state == State::Pending)
    }
}

impl Interp {
    fn isched(&mut self) -> &mut InlineSched {
        self.isched
            .get_or_insert_with(|| Box::new(InlineSched::new()))
    }

    /// Foreign procedures the inline scheduler implements. `None`: not one of them.
    pub(super) fn inline_thread_foreign(
        &mut self,
        program: &Program,
        symbol: &str,
        args: &[u64],
    ) -> Option<Res<Vec<u64>>> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        let started = self.multi;
        let result: Res<u64> = match symbol {
            "pthread_create" => self.inline_create(arg(0), arg(2), arg(3)),
            "pthread_join" => self.inline_join(program, arg(0), arg(1)),
            "pthread_tryjoin_np" => {
                let id = arg(0).wrapping_sub(1) as usize;
                match self.isched().threads.get(id) {
                    Some(t) if t.state == State::Finished => {
                        let value = t.result;
                        if arg(1) != 0 {
                            self.write(arg(1), &value.to_le_bytes());
                        }
                        Ok(0)
                    }
                    Some(_) => Ok(EBUSY),
                    None => Ok(EINVAL),
                }
            }
            "pthread_detach" => Ok(0),
            "pthread_self" => Ok(self.isched().current() as u64 + 1),
            "pthread_mutex_init" => {
                self.isched().mutexes.insert(arg(0), MutexState::default());
                Ok(0)
            }
            "pthread_mutex_destroy" => {
                self.isched().mutexes.remove(&arg(0));
                Ok(0)
            }
            "pthread_mutex_lock" => self.inline_lock(program, arg(0), 1).map(|_| 0),
            "pthread_mutex_trylock" => {
                let me = self.isched().current();
                let state = self.isched().mutexes.entry(arg(0)).or_default();
                if state.owner.is_none() || state.owner == Some(me) {
                    state.owner = Some(me);
                    state.count += 1;
                    Ok(0)
                } else {
                    Ok(EBUSY)
                }
            }
            "pthread_mutex_unlock" => Ok(self.inline_unlock(arg(0)).err().unwrap_or(0)),
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
            "pthread_cond_wait" => self.inline_cond_wait(program, arg(0), arg(1), None),
            "pthread_cond_timedwait" => {
                let deadline = unsafe {
                    std::ptr::read_unaligned(arg(2) as *const u64)
                        .saturating_mul(1_000_000_000)
                        .saturating_add(std::ptr::read_unaligned((arg(2) + 8) as *const u64))
                };
                self.inline_cond_wait(program, arg(0), arg(1), Some(deadline))
            }
            // Sleeping and yielding only matter once another thread exists.
            "nanosleep" if started => {
                let (secs, nanos) = (self.read_u64(arg(0)), self.read_u64(arg(0) + 8));
                self.inline_sleep(program, secs.saturating_mul(1_000_000_000) + nanos)
            }
            "usleep" if started => self.inline_sleep(program, (arg(0) as u32 as u64) * 1_000),
            "sleep" if started => {
                self.inline_sleep(program, (arg(0) as u32 as u64) * 1_000_000_000)
            }
            "sched_yield" | "pthread_yield_np" if started => self.inline_yield(program).map(|_| 0),
            _ => return None,
        };
        Some(result.map(|v| vec![v]))
    }

    fn inline_create(&mut self, out: u64, start: u64, argument: u64) -> Res<u64> {
        let Some(func) = self.func_of(start) else {
            return self.trap("pthread_create needs an interpreted thread procedure");
        };
        let sched = self.isched();
        sched.threads.push(ThreadRec {
            func,
            argument,
            state: State::Pending,
            result: 0,
        });
        let id = sched.threads.len() - 1;
        self.multi = true;
        if out != 0 {
            self.write(out, &(id as u64 + 1).to_le_bytes());
        }
        Ok(0)
    }

    fn inline_join(&mut self, program: &Program, handle: u64, result_out: u64) -> Res<u64> {
        let me = self.isched().current();
        let id = handle.wrapping_sub(1) as usize;
        if id >= self.isched().threads.len() || id == me {
            return Ok(EINVAL);
        }
        self.wait_until(program, Wait::Join(id), None)?;
        if result_out != 0 {
            let value = self.isched().threads[id].result;
            self.write(result_out, &value.to_le_bytes());
        }
        Ok(0)
    }

    fn inline_lock(&mut self, program: &Program, addr: u64, count: u32) -> Res<()> {
        loop {
            let me = self.isched().current();
            let state = self.isched().mutexes.entry(addr).or_default();
            if state.owner.is_none() || state.owner == Some(me) {
                state.owner = Some(me);
                state.count += count;
                return Ok(());
            }
            self.wait_until(program, Wait::Mutex(addr), None)?;
        }
    }

    /// Release one level of the mutex; `Err(EPERM)` when the caller does not hold it.
    fn inline_unlock(&mut self, addr: u64) -> Result<(), u64> {
        let me = self.isched().current();
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

    fn inline_cond_wait(
        &mut self,
        program: &Program,
        cond: u64,
        mutex: u64,
        deadline: Option<u64>,
    ) -> Res<u64> {
        let me = self.isched().current();
        let held = match self.isched().mutexes.get(&mutex) {
            Some(state) if state.owner == Some(me) => state.count,
            _ => 0,
        };
        for _ in 0..held {
            let _ = self.inline_unlock(mutex);
        }
        let token = {
            let sched = self.isched();
            let token = sched.next_token;
            sched.next_token += 1;
            sched.cond_waiters.entry(cond).or_default().push_back(token);
            token
        };
        let outcome = self.wait_until(program, Wait::Cond(token), deadline);
        {
            let sched = self.isched();
            sched.woken.remove(&token);
            if let Some(waiters) = sched.cond_waiters.get_mut(&cond) {
                waiters.retain(|&t| t != token);
            }
        }
        let timed_out = outcome?;
        if held > 0 {
            self.inline_lock(program, mutex, held)?;
        }
        Ok(if timed_out {
            ETIMEDOUT
        } else {
            0
        })
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

    /// Sleeping lets every pending thread run, then moves the virtual clock.
    fn inline_sleep(&mut self, program: &Program, nanoseconds: u64) -> Res<u64> {
        while self.run_pending_one(program)? {}
        self.host.advance_clock(nanoseconds);
        Ok(0)
    }

    pub(super) fn inline_yield(&mut self, program: &Program) -> Res<()> {
        self.run_pending_one(program)?;
        Ok(())
    }

    /// Called between basic blocks while threads exist: now and then, give pending threads a turn.
    pub(super) fn inline_preempt(&mut self, program: &Program) -> Res<()> {
        let sched = self.isched();
        sched.ticks += 1;
        if sched.ticks.is_multiple_of(PREEMPT_TICKS) && sched.has_pending() {
            while self.run_pending_one(program)? {}
        }
        Ok(())
    }

    /// Block the current thread until `wait` holds. `deadline` (absolute virtual nanoseconds)
    /// makes it return `true` instead when nothing else can run. `Err` when it can never hold,
    /// or when this thread is being abandoned.
    fn wait_until(&mut self, program: &Program, wait: Wait, deadline: Option<u64>) -> Res<bool> {
        let top = self.isched().levels.len() - 1;
        self.isched().levels[top].wait = wait;
        let outcome = loop {
            let sched = self.isched();
            let me = sched.current();
            if sched.satisfied(me, wait) {
                break Ok(false);
            }
            match self.run_pending_one(program) {
                Ok(true) => continue,
                Ok(false) => {}
                Err(e) => break Err(e),
            }
            if let Some(at) = deadline {
                if let Some(now) = self.host.virtual_now_ns() {
                    self.host.advance_clock(at.saturating_sub(now));
                }
                break Ok(true);
            }
            // Nothing can run: a thread lower on the stack whose wait is over may continue if
            // everything above it is abandoned.
            let loc = self.loc;
            let sched = self.isched();
            let resume = (0..top)
                .rev()
                .find(|&k| sched.satisfied(sched.levels[k].thread, sched.levels[k].wait));
            break match resume {
                Some(k) => {
                    sched.unwind_to = Some(k);
                    Err(Trap {
                        message: "thread abandoned".into(),
                        loc,
                        kind: Some(TrapKind::Abandoned),
                        ..Trap::default()
                    })
                }
                None => self.trap("deadlock: every thread is blocked"),
            };
        };
        if let Some(level) = self.isched().levels.get_mut(top) {
            level.wait = Wait::None;
        }
        outcome
    }

    /// Run the first pending thread to completion on top of the current stack.
    fn run_pending_one(&mut self, program: &Program) -> Res<bool> {
        let Some(id) = self
            .isched()
            .threads
            .iter()
            .position(|t| t.state == State::Pending)
        else {
            return Ok(false);
        };
        let (func, argument) = {
            let sched = self.isched();
            sched.threads[id].state = State::Running;
            sched.levels.push(Level {
                thread: id,
                wait: Wait::None,
            });
            (sched.threads[id].func, sched.threads[id].argument)
        };
        let result = self.exec(program, func, &[argument]);
        let sched = self.isched();
        sched.levels.pop();
        sched.threads[id].state = State::Finished;
        match result {
            Ok(values) => {
                sched.threads[id].result = values.first().copied().unwrap_or(0);
                Ok(true)
            }
            Err(trap) if trap.kind == Some(TrapKind::Abandoned) => {
                // Free what the abandoned thread held and forget its waits.
                for state in sched.mutexes.values_mut() {
                    if state.owner == Some(id) {
                        state.owner = None;
                        state.count = 0;
                    }
                }
                let top = sched.levels.len() - 1;
                match sched.unwind_to {
                    Some(k) if k < top => Err(trap),
                    _ => {
                        sched.unwind_to = None;
                        Ok(true)
                    }
                }
            }
            Err(trap) => {
                let at = trap
                    .loc
                    .map(|(_, line, col)| format!(" (line {line}, column {col})"))
                    .unwrap_or_default();
                Err(Trap {
                    message: format!("runtime error in a thread: {}{at}", trap.message),
                    ..trap
                })
            }
        }
    }
}
