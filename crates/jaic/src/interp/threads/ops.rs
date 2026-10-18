//! Thread, mutex and condition operations of the interpreter.
use super::pthread;
#[cfg(not(windows))]
use super::sched::NATIVE_STACK;
use super::sched::{
    Block, EBUSY, EINVAL, EPERM, ETIMEDOUT, GThread, MutexState, PREEMPT_TICKS, Sched, Shared,
    THREAD_STACK,
};
#[cfg(windows)]
use super::win32;
use super::worker::absolute_deadline;
#[cfg(not(windows))]
use super::worker::run_worker;
use crate::interp::threads_inline::ThreadFn;
use crate::interp::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

impl Interp {
    pub(in crate::interp) fn compiler_native_threads_idle(&self) -> bool {
        self.shared.as_ref().is_none_or(|shared| {
            shared
                .lock()
                .threads
                .iter()
                .skip(1)
                .all(|thread| thread.finished)
        })
    }

    /// The scheduler, made on first use (a thread, a lock, or a thunk C may call).
    pub(in crate::interp) fn shared(&mut self) -> Arc<Shared> {
        if let Some(shared) = &self.shared {
            return shared.clone();
        }
        let shared = Shared::new(self as *mut Interp as usize);
        self.shared = Some(shared.clone());
        // Another thread may want the baton from now on.
        self.multi = true;
        shared
    }

    pub(super) fn with_sched<R>(&mut self, f: impl FnOnce(&mut Sched) -> R) -> R {
        let shared = self.shared();
        let mut s = shared.lock();
        let out = f(&mut s);
        if std::mem::take(&mut s.woke) {
            shared.cv.notify_all();
        }
        out
    }

    pub(super) fn me(&mut self) -> usize {
        self.with_sched(|s| s.current.unwrap_or(0))
    }

    /// Foreign procedures the thread scheduler implements itself. `None`: not one of them.
    pub(in crate::interp) fn thread_foreign(
        &mut self,
        program: &Program,
        symbol: &str,
        args: &[u64],
    ) -> Option<Res<Vec<u64>>> {
        // A declaration may pass fewer words than the Win32 call has parameters; the rest are 0.
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        let started = self.multi;
        let Some(fun) = ThreadFn::from_name(symbol) else {
            return self.win32_foreign(program, symbol, args);
        };
        let result: Res<u64> = match fun {
            ThreadFn::Create => self.thread_create(program, arg(0), arg(2), arg(3)),
            ThreadFn::Join => self.thread_join(arg(0), arg(1)),
            ThreadFn::Detach => Ok(0),
            ThreadFn::SelfId => Ok(self.me() as u64 + 1),
            ThreadFn::SetName | ThreadFn::GetName => self.pthread_name(program, fun, args),
            ThreadFn::MutexInit => {
                self.with_sched(|s| s.mutexes.insert(arg(0), MutexState::default()));
                Ok(0)
            }
            ThreadFn::MutexDestroy => {
                self.with_sched(|s| s.mutexes.remove(&arg(0)));
                Ok(0)
            }
            ThreadFn::MutexLock => self.lock_mutex(arg(0), 1).map(|_| 0),
            ThreadFn::MutexTryLock => Ok(if self.try_lock_mutex(arg(0)) {
                0
            } else {
                EBUSY
            }),
            ThreadFn::MutexUnlock => Ok(self.unlock_mutex(arg(0)).err().unwrap_or(0)),
            ThreadFn::CondInit | ThreadFn::CondDestroy => {
                self.with_sched(|s| s.cond_waiters.remove(&arg(0)));
                Ok(0)
            }
            ThreadFn::CondSignal => {
                self.wake_cond(arg(0), false);
                Ok(0)
            }
            ThreadFn::CondBroadcast => {
                self.wake_cond(arg(0), true);
                Ok(0)
            }
            ThreadFn::CondWait => self.cond_wait(arg(0), arg(1), None),
            ThreadFn::CondTimedWait => match (self.fetch_u64(arg(2)), self.fetch_u64(arg(2) + 8)) {
                (Ok(secs), Ok(nanos)) => {
                    let deadline = absolute_deadline(secs, nanos);
                    self.cond_wait(arg(0), arg(1), Some(deadline))
                }
                (Err(trap), _) | (_, Err(trap)) => Err(trap),
            },
            // Sleeping and yielding only matter once another thread may exist.
            ThreadFn::Nanosleep if started => {
                match (self.fetch_u64(arg(0)), self.fetch_u64(arg(0) + 8)) {
                    (Ok(secs), Ok(nanos)) => self.sleep_for(Duration::new(secs, nanos as u32)),
                    (Err(trap), _) | (_, Err(trap)) => Err(trap),
                }
            }
            ThreadFn::Usleep if started => {
                self.sleep_for(Duration::from_micros(arg(0) as u32 as u64))
            }
            ThreadFn::Sleep if started => self.sleep_for(Duration::from_secs(arg(0) as u32 as u64)),
            ThreadFn::SchedYield | ThreadFn::YieldNp if started => self.yield_now().map(|_| 0),
            _ => return self.win32_foreign(program, symbol, args),
        };
        Some(result.map(|v| vec![v]))
    }

    /// Naming is an OS operation; scheduler ids must never cross the native ABI.
    fn pthread_name(&mut self, program: &Program, fun: ThreadFn, args: &[u64]) -> Res<u64> {
        if program.cross_target {
            return self
                .trap("pthread naming is unsupported for a cross-target interpreter program");
        }
        let Some(set_arity) = pthread::set_arity() else {
            return self.trap("pthread naming is unsupported on this interpreter host");
        };
        let setting = fun == ThreadFn::SetName;
        if args.len()
            != if setting {
                set_arity
            } else {
                3
            }
        {
            return self.trap("pthread naming signature does not match this interpreter host");
        }
        let shared = self.shared();
        // Retain the scheduler lock through the short native operation: teardown and
        // adopted-slot recycling cannot invalidate the OS identity while it is used.
        let sched = shared.lock();
        let id = if setting && set_arity == 1 {
            sched.current.unwrap_or(0)
        } else {
            let Some(id) = args[0]
                .checked_sub(1)
                .and_then(|id| usize::try_from(id).ok())
            else {
                return self.trap("pthread naming requires a live scheduler thread identity");
            };
            id
        };
        let Some(thread) = sched
            .threads
            .get(id)
            .filter(|thread| !thread.finished && !thread.cancelled)
        else {
            return self.trap("pthread naming requires a live scheduler thread identity");
        };
        let Some(handle) = thread
            .worker
            .as_ref()
            .and_then(pthread::worker_identity)
            .or(thread.native_pthread)
        else {
            return self
                .trap("pthread naming has no retained native identity for this scheduler thread");
        };
        if setting {
            let pointer = args[set_arity - 1];
            // Copy only a bounded C string through checked interpreter memory. An
            // overlong name is terminated just beyond the OS limit so the actual
            // naming API returns its own length error without an unbounded read.
            let mut name = Vec::new();
            for offset in 0..pthread::name_capacity() {
                let address = pointer
                    .checked_add(offset as u64)
                    .ok_or_else(|| Trap::new("pthread name pointer overflow").at(self.loc))?;
                let byte = self.fetch(address, 1)?[0];
                name.push(byte);
                if byte == 0 {
                    break;
                }
            }
            if name.last() != Some(&0) {
                name.push(0);
            }
            Ok(pthread::set_name(handle, &name))
        } else {
            let (status, name) = pthread::get_name(handle, args[2]);
            if status == 0 {
                self.put(args[1], &name)?;
            }
            Ok(status)
        }
    }

    /// Native callback ABI, independent of its nominal pointer types or module.
    pub(super) fn thread_entry_abi(&self, program: &Program, start: u64, result: Ty) -> bool {
        self.func_of(start)
            .and_then(|func| program.func_sig(func))
            .is_some_and(|signature| {
                signature.conv == ir::Conv::C
                    && signature.params == [Ty::Ptr]
                    && signature.returns == [result]
                    && !signature.c_varargs
                    && signature.c_abi.is_none()
            })
    }

    /// Start a Jai thread running `start(argument)`: its id, or `None` when the OS refused.
    fn spawn_thread(&mut self, program: &Program, start: u64, argument: u64) -> Res<Option<usize>> {
        self.spawn_gated_thread(program, start, argument, false)
    }

    pub(super) fn spawn_gated_thread(
        &mut self,
        program: &Program,
        start: u64,
        argument: u64,
        suspended: bool,
    ) -> Res<Option<usize>> {
        let Some(func) = self.func_of(start) else {
            return self.trap("pthread_create needs an interpreted thread procedure");
        };
        self.require_callable(program, func)?;
        // The scheduler runs this thunk, not C: it gives no thread of C's a way back in.
        self.unseen_thunks.retain(|&t| t != start);
        let shared = self.shared();
        let id = {
            let mut s = shared.lock();
            let state = ExecState {
                stack: ValueStack::heap(THREAD_STACK / 8),
                ..ExecState::default()
            };
            s.threads.push(GThread::new(false, Some(state)));
            s.threads.last_mut().unwrap().suspend_count = u32::from(suspended);
            s.threads.len() - 1
        };
        let program = program as *const Program as usize;
        let thread_shared = shared.clone();
        #[cfg(windows)]
        let spawned = win32::spawn_worker(thread_shared, program, id, func, argument);
        #[cfg(not(windows))]
        let spawned = std::thread::Builder::new()
            .stack_size(NATIVE_STACK)
            .spawn(move || run_worker(&thread_shared, program, id, func, argument));
        let worker = match spawned {
            Ok(worker) => worker,
            Err(error) => {
                shared.lock().threads[id].finished = true;
                shared.lock().threads[id].start.take();
                #[cfg(windows)]
                {
                    win32::creation_error(&error);
                    if error.raw_os_error().is_none() {
                        return self.trap(error.to_string());
                    }
                }
                #[cfg(not(windows))]
                let _ = error;
                return Ok(None);
            }
        };
        #[cfg(windows)]
        {
            let public = match worker.try_clone() {
                Ok(public) => public,
                Err(error) => {
                    shared.lock().threads[id].cancelled = true;
                    shared.cv.notify_all();
                    win32::join_worker(worker);
                    win32::creation_error(&error);
                    if error.raw_os_error().is_none() {
                        return self.trap(error.to_string());
                    }
                    return Ok(None);
                }
            };
            shared.lock().threads[id].native_handle = Some(public);
        }
        shared.lock().threads[id].worker = Some(worker);
        Ok(Some(id))
    }

    fn thread_create(
        &mut self,
        program: &Program,
        out: u64,
        start: u64,
        argument: u64,
    ) -> Res<u64> {
        if !self.thread_entry_abi(program, start, Ty::Ptr) {
            return self.trap("POSIX thread entry ABI requires one pointer argument and one pointer result with C convention");
        }
        let Some(id) = self.spawn_thread(program, start, argument)? else {
            return Ok(11); // EAGAIN
        };
        if out != 0 {
            self.put(out, &(id as u64 + 1).to_le_bytes())?;
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
            self.put(result_out, &value.to_le_bytes())?;
        }
        Ok(0)
    }

    pub(super) fn try_lock_mutex(&mut self, addr: u64) -> bool {
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

    pub(super) fn lock_mutex(&mut self, addr: u64, count: u32) -> Res<()> {
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
    pub(super) fn unlock_mutex(&mut self, addr: u64) -> Result<(), u64> {
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

    pub(super) fn cond_wait(
        &mut self,
        cond: u64,
        mutex: u64,
        deadline: Option<Instant>,
    ) -> Res<u64> {
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

    pub(super) fn wake_cond(&mut self, cond: u64, all: bool) {
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

    pub(super) fn sleep_for(&mut self, duration: Duration) -> Res<u64> {
        self.block(Block::Sleep(Instant::now() + duration))?;
        Ok(0)
    }

    /// Let another runnable thread go first.
    pub(in crate::interp) fn yield_now(&mut self) -> Res<()> {
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
    pub(in crate::interp) fn preempt(&mut self) -> Res<()> {
        self.ticks += 1;
        if self.ticks.is_multiple_of(PREEMPT_TICKS) {
            self.yield_now()?;
        }
        Ok(())
    }

    /// Give the baton up until `reason` no longer holds (whoever ends it wakes this thread).
    pub(super) fn block(&mut self, reason: Block) -> Res<()> {
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
    pub(super) fn finish_thread(&mut self, me: usize, result: u64) {
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
