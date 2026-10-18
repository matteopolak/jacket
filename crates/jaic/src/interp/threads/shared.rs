//! Taking the baton, waiting for a turn and shutting the scheduler down.
use super::pthread;
use super::sched::{Block, DEADLOCK_GRACE, GThread, NATIVE_SLICE, Sched, Shared, WorkerCancelled};
#[cfg(windows)]
use super::win32;
use crate::interp::*;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Instant;

impl Shared {
    /// A scheduler whose only thread (0, the caller) holds the baton.
    pub(super) fn new(interp: usize) -> Arc<Shared> {
        let mut main = GThread::new(false, None);
        main.native_pthread = pthread::current();
        Arc::new(Shared {
            sched: Mutex::new(Sched {
                threads: vec![main],
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

    pub(super) fn lock(&self) -> MutexGuard<'_, Sched> {
        self.sched
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The identity C callbacks compare with `native::caller`.
    pub(in crate::interp) fn key(&self) -> usize {
        std::ptr::from_ref::<Shared>(self) as usize
    }

    /// Wait until `me` (runnable, or blocked waiting to be woken) holds the baton.
    pub(super) fn acquire<'a>(
        &'a self,
        s: MutexGuard<'a, Sched>,
        me: usize,
    ) -> MutexGuard<'a, Sched> {
        self.wait_turn(s, me, false).unwrap_or_else(|| {
            // Bypass the panic hook: this is cooperative cancellation, not a Rust error.
            // Unwind Rust frames without touching the interpreter after losing the baton.
            std::panic::resume_unwind(Box::new(WorkerCancelled))
        })
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
        loop {
            if s.dead || s.threads[me].cancelled {
                if urgent {
                    s.urgent -= 1;
                }
                return None;
            }
            if s.current == Some(me) {
                break;
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
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .0
                }
                None => self
                    .cv
                    .wait(s)
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
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
    pub(in crate::interp) fn enter_native(&self, interp: usize) -> usize {
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
    pub(in crate::interp) fn leave_native(&self, me: usize) {
        let mut s = self.lock();
        if s.current == Some(me) {
            s.in_native = false;
            return;
        }
        s.threads[me].block = Block::None;
        drop(self.acquire(s, me));
    }

    /// C may have been given a thunk.
    pub(in crate::interp) fn callbacks_possible(&self) {
        self.lock().callbacks = true;
    }

    /// The interpreter is being dropped.
    pub(in crate::interp) fn close(&self) {
        let workers = {
            let mut s = self.lock();
            s.dead = true;
            let workers: Vec<_> = s
                .threads
                .iter_mut()
                .filter_map(|thread| thread.worker.take())
                .collect();
            self.cv.notify_all();
            workers
        };
        // The interpreter/program remain alive until every managed OS entry returns.
        // Never forcibly terminate foreign code or leave parked Rust stacks behind.
        for worker in workers {
            #[cfg(windows)]
            win32::join_worker(worker);
            #[cfg(not(windows))]
            if worker.join().is_err() {
                eprintln!("error: interpreter worker panicked during teardown");
                std::process::abort();
            }
        }
        let mut s = self.lock();
        while s
            .threads
            .iter()
            .any(|thread| thread.adopted && !thread.finished)
        {
            s = self
                .cv
                .wait(s)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        #[cfg(windows)]
        for thread in &mut s.threads {
            if let Some(handle) = thread.native_handle.take() {
                win32::release_thread_handle(handle);
            }
        }
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
                let Some(acquired) = self.acquire_callback(s, me) else {
                    // A callback lease that never entered must also be released at shutdown.
                    let mut cancelled = self.lock();
                    cancelled.threads[me].finished = true;
                    cancelled.spare.push(me);
                    self.cv.notify_all();
                    return Err(GONE.into());
                };
                s = acquired;
                me
            }
        };
        let interp = s.interp as *mut Interp;
        drop(s);
        // SAFETY: this thread holds the baton, so nothing else touches the interpreter, and
        // `program` is the program the thunk was made for, alive while its interpreter is.
        let (interp, program) = unsafe { (&mut *interp, &*(program as *const Program)) };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f(&mut |func, args: &[u64]| interp.run_callback(program, func, args));
        }));
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
        drop(s);
        match result {
            Ok(()) => Ok(()),
            Err(cancelled) if cancelled.is::<WorkerCancelled>() => Err(GONE.into()),
            Err(other) => std::panic::resume_unwind(other),
        }
    }

    fn describe(&self, program: u64, trap: &Trap) -> String {
        // SAFETY: `program` is the program the thunk was made for, alive while its
        // interpreter is, and only read here.
        let program = unsafe { &*(program as *const Program) };
        trap.describe(&program.file_paths)
    }
}
