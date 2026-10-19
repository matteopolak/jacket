//! Scheduler state: thread slots, blocking reasons and the shared baton.
use super::{pthread, win32};
use crate::interp::*;
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

/// Interpreter value stack of a thread other than the first.
pub(super) const THREAD_STACK: usize = 8 << 20;

/// Native stack reserved for each OS thread that runs interpreted code.
pub(super) const NATIVE_STACK: usize = 256 << 20;

/// Block transitions between preemption checks.
pub(super) const PREEMPT_TICKS: u64 = 20_000;

/// How long a thread may stay in one native call before a runnable thread takes the baton.
/// Runnable threads also check this often whether the holder has entered such a call.
pub(super) const NATIVE_SLICE: Duration = Duration::from_millis(1);

/// Once C may hold thunks (`Sched::callbacks`), a thread C started may call one and wake the
/// blocked threads: every thread must stay blocked this long before that counts as a deadlock.
pub const DEADLOCK_GRACE: Duration = Duration::from_secs(1);

pub(super) use crate::interp::errno::{EBUSY, EINVAL, EPERM, ETIMEDOUT_HOST as ETIMEDOUT};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Block {
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
    pub(super) fn deadline(self) -> Option<Instant> {
        match self {
            Block::Sleep(d) | Block::Cond(_, Some(d)) | Block::Object(Some(d)) => Some(d),
            _ => None,
        }
    }
}

#[cfg(windows)]
pub(super) type Worker = std::os::windows::io::OwnedHandle;

#[cfg(not(windows))]
pub(super) type Worker = std::thread::JoinHandle<u64>;

/// Private unwind signal: never escapes a native callback or OS thread entry.
pub(super) struct WorkerCancelled;

pub(super) const CANCELLED_EXIT_CODE: u32 = 1223; // ERROR_CANCELLED; interpreter teardown, not Jai success.

pub(super) struct GThread {
    pub(super) finished: bool,
    pub(super) cancelled: bool,
    /// Private join ownership survives closing every public thread handle.
    pub(super) worker: Option<Worker>,
    /// Win32 suspension gates scheduling independently of waits/deadlines.
    pub(super) suspend_count: u32,
    /// Actual Windows OS id, recorded before the thread waits for its baton.
    pub(super) native_id: u32,
    /// Main/adopted OS pthread identity. Managed workers retain theirs in `worker`.
    pub(super) native_pthread: Option<u64>,
    /// Own the actual kernel thread handle until `CloseHandle` or interpreter teardown.
    /// Scheduler state remains alive independently when the user closes a running thread.
    #[cfg(windows)]
    pub(super) native_handle: Option<std::os::windows::io::OwnedHandle>,
    pub(super) block: Block,
    /// A timed condition wait ended because its deadline passed.
    pub(super) timed_out: bool,
    /// The scheduler found every thread blocked and woke this one to report it.
    pub(super) deadlocked: bool,
    pub(super) result: u64,
    /// A thread C started, running one callback; its slot is reused once that returns.
    pub(super) adopted: bool,
    /// The interpreter state a new Jai thread starts with.
    pub(super) start: Option<ExecState>,
}

impl GThread {
    pub(super) fn new(adopted: bool, start: Option<ExecState>) -> Self {
        GThread {
            finished: false,
            cancelled: false,
            worker: None,
            suspend_count: 0,
            native_id: 0,
            native_pthread: adopted.then(pthread::current).flatten(),
            #[cfg(windows)]
            native_handle: None,
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
pub(super) struct MutexState {
    pub(super) owner: Option<usize>,
    pub(super) count: u32,
}

pub(in crate::interp) struct Sched {
    pub(super) threads: Vec<GThread>,
    /// The thread holding the baton (`None`: free, nothing runnable yet).
    pub(super) current: Option<usize>,
    /// The last thread given the baton: the round robin starts after it.
    pub(super) last: usize,
    /// The holder is inside a native call, so the baton may be taken from it.
    pub(super) in_native: bool,
    /// Native calls entered by holders so far: tells a waiter whether the holder is still in
    /// the call it saw last time.
    pub(super) native_calls: u64,
    /// Callbacks waiting for the baton: a holder entering C wakes them at once.
    pub(super) urgent: usize,
    /// A foreign call ran after a thunk was made that does not start a Jai thread, so C may
    /// hold it and threads the scheduler does not know yet may call back
    /// (`Interp::hand_thunks_to_c`).
    pub(super) callbacks: bool,
    /// A blocked thread became runnable: its OS thread must be woken to wait for its turn
    /// (`with_sched`).
    pub(super) woke: bool,
    pub(super) mutexes: HashMap<u64, MutexState>,
    pub(super) cond_waiters: HashMap<u64, VecDeque<usize>>,
    /// Win32 handles the scheduler made (threads, semaphores, events).
    pub(super) objects: HashMap<u64, win32::Object>,
    pub(super) next_object: u64,
    /// Slots of adopted threads that have returned to C.
    pub(super) spare: Vec<usize>,
    /// The interpreter (`*mut Interp`), for threads that start running it.
    pub(super) interp: usize,
    /// The interpreter is gone; callbacks into it fail.
    pub(super) dead: bool,
}

impl Sched {
    pub(super) fn runnable(&self, t: usize) -> bool {
        !self.threads[t].finished
            && self.threads[t].suspend_count == 0
            && self.threads[t].block == Block::None
    }

    /// Make runnable every thread whose sleep or timed wait is over.
    pub(super) fn expire_waits(&mut self) {
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
    pub(super) fn wake(&mut self, reason: Block) {
        for thread in &mut self.threads {
            if thread.block == reason {
                thread.block = Block::None;
                self.woke = true;
            }
        }
    }

    /// Let every thread waiting on Win32 handles check them again.
    pub(super) fn wake_object_waiters(&mut self) {
        for thread in &mut self.threads {
            if matches!(thread.block, Block::Object(_)) {
                thread.block = Block::None;
                self.woke = true;
            }
        }
    }

    /// Give the free baton to the next runnable thread after the last holder.
    pub(super) fn pick(&mut self) -> bool {
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
    pub(super) fn stuck(&self) -> bool {
        self.threads.iter().all(|t| {
            t.finished
                || t.suspend_count != 0
                || (t.block != Block::None
                    && t.block != Block::Native
                    && t.block.deadline().is_none())
        })
    }

    /// The holder gives the baton up: hand it to the next runnable thread. When nothing can
    /// run again, wake `report` (or the first unfinished thread) to report a deadlock; with
    /// thunks in C's hands, the blocked threads do that after `DEADLOCK_GRACE` (`acquire`).
    pub(super) fn release(&mut self, report: Option<usize>) {
        self.current = None;
        self.in_native = false;
        if self.pick() || !self.stuck() || self.callbacks {
            return;
        }
        self.report_deadlock(report);
    }

    pub(super) fn report_deadlock(&mut self, report: Option<usize>) {
        let first = (0..self.threads.len())
            .find(|&t| !self.threads[t].finished && self.threads[t].suspend_count == 0);
        if let Some(t) = report.or(first) {
            self.threads[t].block = Block::None;
            self.threads[t].deadlocked = true;
            self.current = Some(t);
            self.last = t;
        }
    }

    /// A slot for a thread C started that is calling back.
    pub(super) fn adopt(&mut self) -> usize {
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
pub(in crate::interp) struct Shared {
    pub(super) sched: Mutex<Sched>,
    pub(super) cv: Condvar,
}
