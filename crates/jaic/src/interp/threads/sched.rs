//! Scheduler state: thread slots, blocking reasons and the shared baton.
use super::{pthread, win32};
use crate::interp::thread_sync::{CondQueues, Mutexes};
use crate::interp::{ExecState, HashMap};
use std::collections::BTreeSet;
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

pub(super) use crate::interp::errno::{EBUSY, EINVAL, ETIMEDOUT_HOST as ETIMEDOUT};

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
    /// Counts the owners the slot has had; part of every handle to it (`Sched::handle_of`), so
    /// a handle kept past its thread's end can never name the thread that takes the slot next.
    pub(super) generation: u32,
    /// The public handle (`pthread_t`, or a Win32 thread handle) is gone: joined, detached or
    /// closed. The slot is free once its thread has also finished.
    pub(super) released: bool,
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
            generation: 0,
            released: false,
            block: Block::None,
            timed_out: false,
            deadlocked: false,
            result: 0,
            adopted,
            start,
        }
    }
}

pub(in crate::interp) struct Sched {
    /// Every slot in use or free; a slot number is the thread id, so slots never move. Add
    /// through `add` and end through `finish`, which keep `live` in step.
    pub(super) threads: Vec<GThread>,
    /// The slots of threads that have not finished: the only ones a scan visits.
    pub(super) live: BTreeSet<usize>,
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
    pub(super) mutexes: Mutexes,
    pub(super) cond_waiters: CondQueues<usize>,
    /// Win32 handles the scheduler made (threads, semaphores, events).
    pub(super) objects: HashMap<u64, win32::Object>,
    pub(super) next_object: u64,
    /// Free slots: nothing can refer to them any more, and `add` fills them first.
    pub(super) spare: Vec<usize>,
    /// The interpreter (`*mut Interp`), for threads that start running it.
    pub(super) interp: usize,
    /// The interpreter is gone; callbacks into it fail.
    pub(super) dead: bool,
}

impl Sched {
    /// Put a thread in a free slot, or in a new one, and return its number.
    pub(super) fn add(&mut self, mut thread: GThread) -> usize {
        let id = if let Some(id) = self.spare.pop() {
            thread.generation = self.threads[id].generation;
            self.threads[id] = thread;
            id
        } else {
            self.threads.push(thread);
            self.threads.len() - 1
        };
        if !self.threads[id].finished {
            self.live.insert(id);
        }
        id
    }

    /// Mark thread `t` finished and stop scanning its slot. The slot is free at once when
    /// nothing can ask for its result: an adopted thread's (nobody joins one), or one whose
    /// handle is already gone.
    pub(super) fn finish(&mut self, t: usize) {
        self.threads[t].finished = true;
        self.live.remove(&t);
        if self.threads[t].adopted || self.threads[t].released {
            // Whoever runs this is the worker itself or a teardown that joins it: dropping the
            // handle only lets go of it.
            drop(self.recycle(t));
        }
    }

    /// The handle of thread `t` is gone (joined, detached or closed). Frees the slot when
    /// the thread has finished, and returns its OS worker to join; otherwise the slot is
    /// freed by `finish`.
    pub(super) fn release_handle(&mut self, t: usize) -> Option<Worker> {
        if std::mem::replace(&mut self.threads[t].released, true) || !self.threads[t].finished {
            return None;
        }
        self.recycle(t)
    }

    /// Free the finished slot `t` for `add`, and end every handle to it.
    fn recycle(&mut self, t: usize) -> Option<Worker> {
        let thread = &mut self.threads[t];
        debug_assert!(thread.finished && !self.spare.contains(&t));
        thread.generation = thread.generation.wrapping_add(1);
        thread.released = true;
        thread.start = None;
        let worker = thread.worker.take();
        // The thread's locks stay held, as they would for a thread that exited holding them,
        // but no thread that takes the slot can own them.
        self.mutexes.orphan(t);
        self.spare.push(t);
        worker
    }

    /// The number Jai code sees for thread `t` (`pthread_t`): the slot plus one, with the
    /// generation in the high half. The first owner of a slot has just the slot plus one.
    pub(super) fn handle_of(&self, t: usize) -> u64 {
        (u64::from(self.threads[t].generation) << 32) | (t as u64 + 1)
    }

    /// The slot a handle names, if its thread (or its result) is still there.
    pub(super) fn resolve(&self, handle: u64) -> Option<usize> {
        let slot = usize::try_from((handle & 0xffff_ffff).checked_sub(1)?).ok()?;
        let thread = self.threads.get(slot)?;
        (u64::from(thread.generation) == handle >> 32).then_some(slot)
    }

    /// The slots of unfinished threads, in order.
    pub(super) fn live_slots(&self) -> impl Iterator<Item = usize> + '_ {
        self.live.iter().copied()
    }

    /// No thread but the first is still running.
    pub(super) fn only_first_live(&self) -> bool {
        self.live.iter().all(|&t| t == 0)
    }

    /// An adopted thread is still inside its callback.
    pub(super) fn adopted_live(&self) -> bool {
        self.live_slots().any(|t| self.threads[t].adopted)
    }

    pub(super) fn runnable(&self, t: usize) -> bool {
        !self.threads[t].finished
            && self.threads[t].suspend_count == 0
            && self.threads[t].block == Block::None
    }

    /// Make runnable every thread whose sleep or timed wait is over.
    pub(super) fn expire_waits(&mut self) {
        let now = Instant::now();
        for &t in &self.live {
            match self.threads[t].block {
                Block::Sleep(d) | Block::Object(Some(d)) if d <= now => {
                    self.threads[t].block = Block::None;
                }
                Block::Cond(c, Some(d)) if d <= now => {
                    self.threads[t].block = Block::None;
                    self.threads[t].timed_out = true;
                    self.cond_waiters.remove(c, t);
                }
                _ => {}
            }
        }
    }

    /// Make runnable every thread blocked for `reason`.
    pub(super) fn wake(&mut self, reason: Block) {
        for &t in &self.live {
            let thread = &mut self.threads[t];
            if thread.block == reason {
                thread.block = Block::None;
                self.woke = true;
            }
        }
    }

    /// Let every thread waiting on Win32 handles check them again.
    pub(super) fn wake_object_waiters(&mut self) {
        for &t in &self.live {
            let thread = &mut self.threads[t];
            if matches!(thread.block, Block::Object(_)) {
                thread.block = Block::None;
                self.woke = true;
            }
        }
    }

    /// Give the free baton to the next runnable thread after the last holder.
    pub(super) fn pick(&mut self) -> bool {
        self.expire_waits();
        let next = self
            .live
            .range(self.last + 1..)
            .chain(self.live.range(..=self.last))
            .copied()
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
        self.live_slots().map(|t| &self.threads[t]).all(|t| {
            t.suspend_count != 0
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
        let first = self
            .live_slots()
            .find(|&t| self.threads[t].suspend_count == 0);
        if let Some(t) = report.or(first) {
            self.threads[t].block = Block::None;
            self.threads[t].deadlocked = true;
            self.current = Some(t);
            self.last = t;
        }
    }

    /// A slot for a thread C started that is calling back.
    pub(super) fn adopt(&mut self) -> usize {
        self.add(GThread::new(true, None))
    }
}

/// The scheduler shared by every thread running the interpreter, and the gate thunks enter
/// it through.
pub(in crate::interp) struct Shared {
    pub(super) sched: Mutex<Sched>,
    pub(super) cv: Condvar,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn sched_with(count: usize) -> Arc<Shared> {
        let shared = Shared::new(0);
        for _ in 1..count {
            shared.lock().add(GThread::new(false, None));
        }
        shared
    }

    #[test]
    fn finished_slots_leave_the_scan_and_the_round_robin_skips_them() {
        let shared = sched_with(4);
        let mut s = shared.lock();
        s.finish(2);
        assert_eq!(s.live_slots().collect::<Vec<_>>(), [0, 1, 3]);
        s.last = 1;
        assert!(s.pick());
        assert_eq!(s.current, Some(3));
        assert!(s.pick());
        assert_eq!(s.current, Some(0), "the scan wraps past the finished slot");
        assert_eq!(
            s.threads.len(),
            4,
            "slot numbers are thread ids and stay put"
        );
    }

    #[test]
    fn idle_and_adopted_queries_look_only_at_unfinished_slots() {
        let shared = sched_with(3);
        let mut s = shared.lock();
        assert!(!s.only_first_live());
        s.finish(1);
        s.finish(2);
        assert!(s.only_first_live());
        assert!(!s.adopted_live());
        let slot = s.adopt();
        assert!(!s.only_first_live());
        assert!(s.adopted_live());
        s.finish(slot);
        assert!(s.only_first_live());
        assert!(!s.adopted_live());
    }

    #[test]
    fn a_reused_adopted_slot_is_scanned_again() {
        let shared = sched_with(1);
        let mut s = shared.lock();
        let slot = s.adopt();
        s.finish(slot);
        assert_eq!(s.live_slots().collect::<Vec<_>>(), [0]);
        assert_eq!(s.adopt(), slot);
        assert_eq!(s.live_slots().collect::<Vec<_>>(), [0, slot]);
    }

    #[test]
    fn a_thread_added_already_finished_is_not_scanned() {
        let shared = sched_with(1);
        let mut s = shared.lock();
        let mut done = GThread::new(false, None);
        done.finished = true;
        assert_eq!(s.add(done), 1);
        assert_eq!(s.live_slots().collect::<Vec<_>>(), [0]);
    }

    #[test]
    fn a_finished_thread_keeps_its_slot_until_its_handle_is_released() {
        let shared = sched_with(1);
        let mut s = shared.lock();
        let slot = s.add(GThread::new(false, None));
        let handle = s.handle_of(slot);
        s.finish(slot);
        assert_eq!(s.resolve(handle), Some(slot), "a joiner can still read it");
        assert!(s.spare.is_empty());
        s.release_handle(slot);
        assert_eq!(s.resolve(handle), None, "the old handle is dead");
        assert_eq!(s.spare, [slot]);
    }

    #[test]
    fn releasing_a_handle_before_the_thread_ends_frees_the_slot_at_the_end() {
        let shared = sched_with(1);
        let mut s = shared.lock();
        let slot = s.add(GThread::new(false, None));
        let handle = s.handle_of(slot);
        s.release_handle(slot);
        assert!(s.spare.is_empty(), "still running");
        assert_eq!(s.resolve(handle), Some(slot));
        s.finish(slot);
        assert_eq!(s.spare, [slot]);
        assert_eq!(s.resolve(handle), None);
    }

    #[test]
    fn releasing_twice_frees_the_slot_once() {
        let shared = sched_with(1);
        let mut s = shared.lock();
        let slot = s.add(GThread::new(false, None));
        s.finish(slot);
        s.release_handle(slot);
        s.release_handle(slot);
        assert_eq!(s.spare, [slot]);
    }

    #[test]
    fn a_reused_slot_gets_a_new_handle_and_the_old_one_never_aliases_it() {
        let shared = sched_with(1);
        let mut s = shared.lock();
        let first = s.add(GThread::new(false, None));
        let old = s.handle_of(first);
        s.finish(first);
        s.release_handle(first);
        let second = s.add(GThread::new(false, None));
        assert_eq!(second, first, "the slot is reused");
        let new = s.handle_of(second);
        assert_ne!(old, new);
        assert_eq!(s.resolve(old), None);
        assert_eq!(s.resolve(new), Some(second));
        assert_eq!(s.threads.len(), 2);
        assert_eq!(s.live_slots().collect::<Vec<_>>(), [0, second]);
    }

    #[test]
    fn the_first_generation_keeps_the_plain_slot_plus_one_handle() {
        let shared = sched_with(3);
        let s = shared.lock();
        assert_eq!(s.handle_of(2), 3);
        assert_eq!(s.resolve(3), Some(2));
        assert_eq!(s.resolve(0), None);
        assert_eq!(s.resolve(4), None, "no such slot");
    }

    #[test]
    fn adopted_and_started_threads_share_the_free_slots() {
        let shared = sched_with(1);
        let mut s = shared.lock();
        let started = s.add(GThread::new(false, None));
        s.finish(started);
        s.release_handle(started);
        let adopted = s.adopt();
        assert_eq!(adopted, started);
        let handle = s.handle_of(adopted);
        s.finish(adopted);
        assert_eq!(
            s.resolve(handle),
            None,
            "an adopted thread's id dies with its call"
        );
        let again = s.add(GThread::new(false, None));
        assert_eq!(again, started);
        assert!(!s.threads[again].adopted);
        assert_eq!(s.threads.len(), 2);
    }

    #[test]
    fn a_mutex_held_by_a_finished_thread_stays_locked_for_a_thread_in_its_slot() {
        let shared = sched_with(1);
        let mut s = shared.lock();
        let first = s.add(GThread::new(false, None));
        assert!(s.mutexes.try_lock(0x10, first, 1));
        s.finish(first);
        s.release_handle(first);
        let second = s.add(GThread::new(false, None));
        assert_eq!(second, first);
        assert!(!s.mutexes.try_lock(0x10, second, 1));
        assert!(s.mutexes.unlock(0x10, second).is_err());
    }
}
