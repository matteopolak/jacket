//! The Win32 threading API on the baton scheduler: what `Thread` (and `Basic`'s sleep) call when
//! `OS == .WINDOWS`, plus the close relatives a program is likely to use directly.
//!
//! - `CreateThread` starts a scheduler thread, like `pthread_create`; its handle is one of the
//!   scheduler's own objects, as are semaphores and events. Waiting on such a handle blocks in the
//!   scheduler (`Block::Object`); a handle the scheduler did not make (a process, a file) goes to
//!   the real call.
//! - Critical sections and SRW locks share the POSIX mutex emulation (recursive, keyed by
//!   address). Shared SRW acquisition is treated as exclusive: correct for every program that
//!   does not rely on two readers holding the lock at once.
//! - Condition variables share the POSIX ones; a timed-out `SleepConditionVariable*` returns 0 and
//!   sets the thread's last error to `ERROR_TIMEOUT`, as Windows does.
//! - Thread ids and TLS need nothing: each scheduler thread is a real OS thread, so
//!   `GetCurrentThreadId` and `TlsGetValue` already answer per thread.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

const INFINITE: u32 = u32::MAX;
const WAIT_OBJECT_0: u64 = 0;
const WAIT_TIMEOUT: u64 = 258;
const WAIT_FAILED: u64 = u32::MAX as u64;
const STILL_ACTIVE: u32 = 259;
const ERROR_TIMEOUT: u32 = 1460;
const ERROR_INVALID_PARAMETER: u32 = 87;
const ERROR_TOO_MANY_POSTS: u32 = 298;

/// `WaitForMultipleObjects` takes at most this many handles.
const MAXIMUM_WAIT_OBJECTS: u64 = 64;

/// Handles the scheduler hands out start here (4-aligned like real handles, but far from the
/// small values Windows uses), so a handle is recognized by lookup alone.
const HANDLE_BASE: u64 = 0x7a1c_0000_0000;

pub(super) enum Object {
    Thread(usize),
    Semaphore {
        count: u32,
        max: u32,
    },
    /// `real`: the kernel event behind it (0: none). A program may hand its event to a real call
    /// as an `OVERLAPPED`'s `hEvent`, which only a kernel handle satisfies, and the kernel signals
    /// it behind the scheduler's back, so the flag below is refreshed from it when waiting.
    Event {
        manual: bool,
        set: bool,
        real: u64,
    },
}

impl Object {
    fn signaled(&mut self, threads: &[GThread]) -> bool {
        match *self {
            Object::Thread(id) => threads[id].finished,
            Object::Semaphore {
                count, ..
            } => count > 0,
            Object::Event {
                manual,
                ref mut set,
                real,
            } => {
                if *set || real == 0 || !real_event_signaled(real) {
                    return *set;
                }
                // An auto-reset kernel event is consumed by this poll, so the flag keeps it. A
                // manual-reset one stays signaled in the kernel, which clears it itself when the
                // next overlapped operation starts, so no flag is kept.
                *set = !manual;
                true
            }
        }
    }

    /// The side effect of a successful wait.
    fn consume(&mut self) {
        match self {
            Object::Thread(_) => {}
            Object::Semaphore {
                count, ..
            } => *count -= 1,
            Object::Event {
                manual,
                set,
                real,
            } => {
                *set &= *manual;
                if *real != 0 && !*set {
                    real_event_call(*real, false);
                }
            }
        }
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetLastError(code: u32);
    fn GetCurrentThreadId() -> u32;
}

pub(super) fn native_thread_id(scheduler_id: usize) -> u32 {
    #[cfg(windows)]
    {
        let _ = scheduler_id;
        // SAFETY: GetCurrentThreadId has no preconditions.
        unsafe { GetCurrentThreadId() }
    }
    #[cfg(not(windows))]
    {
        // Host-independent scheduler tests have no Windows OS thread identity.
        scheduler_id as u32 + 1
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateEventW(attributes: usize, manual: i32, initial: i32, name: usize) -> usize;
    fn SetEvent(handle: usize) -> i32;
    fn ResetEvent(handle: usize) -> i32;
    fn CloseHandle(handle: usize) -> i32;
    fn WaitForSingleObject(handle: usize, milliseconds: u32) -> u32;
}

/// How many kernel events the program holds open.
static REAL_EVENTS: AtomicUsize = AtomicUsize::new(0);

/// A kernel event, or 0 where there is none to make.
fn real_event_new(manual: bool, set: bool) -> u64 {
    #[cfg(windows)]
    {
        // SAFETY: CreateEventW with no attributes or name has no preconditions.
        let handle = unsafe { CreateEventW(0, manual as i32, set as i32, 0) as u64 };
        if handle != 0 {
            REAL_EVENTS.fetch_add(1, Ordering::Relaxed);
        }
        handle
    }
    #[cfg(not(windows))]
    {
        let _ = (manual, set);
        0
    }
}

/// Sets (`true`) or resets (`false`) the kernel event.
fn real_event_call(handle: u64, set: bool) {
    #[cfg(windows)]
    // SAFETY: the handle is an event this module made and has not closed.
    unsafe {
        if set {
            SetEvent(handle as usize);
        } else {
            ResetEvent(handle as usize);
        }
    };
    #[cfg(not(windows))]
    let _ = (handle, set);
}

fn real_event_close(handle: u64) {
    #[cfg(windows)]
    // SAFETY: the handle is an event this module made; the program has just closed it.
    unsafe {
        CloseHandle(handle as usize);
        REAL_EVENTS.fetch_sub(1, Ordering::Relaxed);
    };
    #[cfg(not(windows))]
    let _ = handle;
}

/// Whether the kernel event is signaled (an auto-reset one is consumed).
fn real_event_signaled(handle: u64) -> bool {
    #[cfg(windows)]
    // SAFETY: the handle is an event this module made and has not closed.
    return unsafe { WaitForSingleObject(handle as usize, 0) == 0 };
    #[cfg(not(windows))]
    {
        let _ = handle;
        false
    }
}

/// The calling thread's `GetLastError` value. Every scheduler thread is the OS thread the
/// program reads it on, so the real one is set.
fn set_last_error(code: u32) {
    #[cfg(windows)]
    // SAFETY: SetLastError only writes the calling thread's error slot.
    unsafe {
        SetLastError(code)
    };
    #[cfg(not(windows))]
    let _ = code;
}

impl Interp {
    /// The Win32 counterpart of `thread_foreign`. `None`: not one of them (or a handle the
    /// scheduler does not own), so the real procedure runs.
    pub(super) fn win32_foreign(
        &mut self,
        program: &Program,
        symbol: &str,
        args: &[u64],
    ) -> Option<Res<Vec<u64>>> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        let started = self.multi;
        let result: Res<u64> = match symbol {
            "CreateThread" | "_beginthreadex" => {
                self.create_thread(program, arg(2), arg(3), arg(4), arg(5))
            }
            "ExitThread" | "_endthreadex" => {
                let me = self.me();
                if !started || me == 0 || self.with_sched(|s| s.threads[me].adopted) {
                    return Some(self.trap(
                        "terminating the interpreter's entry thread through a native thread API is unsupported",
                    ));
                }
                return Some(Err(Trap::new("managed thread exited")
                    .of_kind(Some(TrapKind::ThreadExit(arg(0) as u32)))));
            }
            "GetThreadId" if self.owns(arg(0)) => {
                let id = self.with_sched(|s| match s.objects.get(&arg(0)) {
                    Some(&Object::Thread(id)) => Some(s.threads[id].native_id),
                    _ => None,
                });
                match id {
                    Some(id) => Ok(id as u64),
                    None => {
                        set_last_error(ERROR_INVALID_PARAMETER);
                        Ok(0)
                    }
                }
            }
            "ResumeThread" if self.owns(arg(0)) => {
                let previous = self.with_sched(|s| {
                    let &Object::Thread(id) = s.objects.get(&arg(0))? else {
                        return None;
                    };
                    let thread = &mut s.threads[id];
                    let previous = thread.suspend_count;
                    thread.suspend_count = previous.saturating_sub(1);
                    if previous == 1 {
                        s.woke = true;
                    }
                    Some(previous)
                });
                match previous {
                    Some(count) => Ok(count as u64),
                    None => {
                        set_last_error(ERROR_INVALID_PARAMETER);
                        Ok(u32::MAX as u64)
                    }
                }
            }
            "SuspendThread" if self.owns(arg(0)) => {
                let previous = self.with_sched(|s| {
                    let Some(&Object::Thread(id)) = s.objects.get(&arg(0)) else {
                        return Ok(None);
                    };
                    if s.threads[id].block == Block::Native {
                        return Err(());
                    }
                    let thread = &mut s.threads[id];
                    if thread.finished || thread.suspend_count == 127 {
                        return Ok(None);
                    }
                    let previous = thread.suspend_count;
                    thread.suspend_count += 1;
                    Ok(Some((previous, s.current == Some(id))))
                });
                match previous {
                    Ok(Some((count, current))) => {
                        if current {
                            // The call returns only after another thread resumes it.
                            if let Err(trap) = self.block(Block::None) {
                                return Some(Err(trap));
                            }
                        }
                        Ok(count as u64)
                    }
                    Ok(None) => {
                        set_last_error(ERROR_INVALID_PARAMETER);
                        Ok(u32::MAX as u64)
                    }
                    Err(()) => {
                        return Some(self.trap(
                            "SuspendThread cannot suspend an interpreted thread inside native code",
                        ));
                    }
                }
            }
            "WaitForSingleObject" | "WaitForSingleObjectEx" if self.owns(arg(0)) => {
                self.wait_objects(&[arg(0)], false, arg(1) as u32)
            }
            "WaitForMultipleObjects" | "WaitForMultipleObjectsEx" => {
                let count = arg(0) as u32 as u64;
                if count == 0 || count > MAXIMUM_WAIT_OBJECTS {
                    return None;
                }
                let handles: Res<Vec<u64>> =
                    (0..count).map(|i| self.fetch_u64(arg(1) + i * 8)).collect();
                let handles = match handles {
                    Ok(handles) => handles,
                    Err(trap) => return Some(Err(trap)),
                };
                if !handles.iter().all(|&h| self.owns(h)) {
                    return None;
                }
                self.wait_objects(&handles, arg(2) as u32 != 0, arg(3) as u32)
            }
            "GetExitCodeThread" if self.owns(arg(0)) => {
                let code = self.with_sched(|s| match s.objects.get(&arg(0)) {
                    Some(&Object::Thread(id)) if s.threads[id].finished => {
                        Some(s.threads[id].result as u32)
                    }
                    Some(Object::Thread(_)) => Some(STILL_ACTIVE),
                    _ => None,
                });
                let Some(code) = code else {
                    set_last_error(ERROR_INVALID_PARAMETER);
                    return Some(Ok(vec![0]));
                };
                if arg(1) != 0
                    && let Err(trap) = self.put(arg(1), &code.to_le_bytes())
                {
                    return Some(Err(trap));
                }
                Ok(1)
            }
            "CloseHandle" if self.owns(arg(0)) => {
                let removed = self.with_sched(|s| s.objects.remove(&arg(0)));
                if let Some(Object::Event {
                    real, ..
                }) = removed
                    && real != 0
                {
                    real_event_close(real);
                    return Some(Ok(vec![1]));
                }
                Ok(1)
            }

            // Critical sections and SRW locks. Initialization also runs the real procedure, so
            // the memory holds a valid lock should C code ever take it directly.
            "InitializeCriticalSection"
            | "InitializeCriticalSectionAndSpinCount"
            | "InitializeCriticalSectionEx"
            | "InitializeSRWLock"
            | "DeleteCriticalSection" => {
                self.with_sched(|s| s.mutexes.remove(&arg(0)));
                return None;
            }
            "EnterCriticalSection" | "AcquireSRWLockExclusive" | "AcquireSRWLockShared" => {
                self.lock_mutex(arg(0), 1).map(|_| 0)
            }
            "TryEnterCriticalSection"
            | "TryAcquireSRWLockExclusive"
            | "TryAcquireSRWLockShared" => Ok(self.try_lock_mutex(arg(0)) as u64),
            "LeaveCriticalSection" | "ReleaseSRWLockExclusive" | "ReleaseSRWLockShared" => {
                let _ = self.unlock_mutex(arg(0));
                Ok(0)
            }

            // Condition variables.
            "InitializeConditionVariable" => {
                self.with_sched(|s| s.cond_waiters.remove(&arg(0)));
                Ok(0)
            }
            "WakeConditionVariable" => {
                self.wake_cond(arg(0), false);
                Ok(0)
            }
            "WakeAllConditionVariable" => {
                self.wake_cond(arg(0), true);
                Ok(0)
            }
            "SleepConditionVariableCS" | "SleepConditionVariableSRW" => {
                let deadline = deadline_after(arg(2) as u32);
                match self.cond_wait(arg(0), arg(1), deadline) {
                    Ok(ETIMEDOUT) => {
                        set_last_error(ERROR_TIMEOUT);
                        Ok(0)
                    }
                    other => other.map(|_| 1),
                }
            }

            // Semaphores and events: (attributes, initial, maximum, name) and
            // (attributes, manual reset, initially set, name).
            "CreateSemaphoreA" | "CreateSemaphoreW" | "CreateSemaphoreExA"
            | "CreateSemaphoreExW" => {
                let (initial, max) = (arg(1) as u32 as i32, arg(2) as u32 as i32);
                if max <= 0 || initial < 0 || initial > max {
                    set_last_error(ERROR_INVALID_PARAMETER);
                    Ok(0)
                } else {
                    Ok(self.new_object(Object::Semaphore {
                        count: initial as u32,
                        max: max as u32,
                    }))
                }
            }
            "ReleaseSemaphore" if self.owns(arg(0)) => {
                let release = arg(1) as u32 as i32;
                let released = self.with_sched(|s| {
                    let Some(Object::Semaphore {
                        count,
                        max,
                    }) = s.objects.get_mut(&arg(0))
                    else {
                        return Err(ERROR_INVALID_PARAMETER);
                    };
                    let previous = *count;
                    if release <= 0 || (*max - previous) < release as u32 {
                        return Err(ERROR_TOO_MANY_POSTS);
                    }
                    *count += release as u32;
                    s.wake_object_waiters();
                    Ok(previous)
                });
                let previous = match released {
                    Ok(previous) => previous,
                    Err(code) => {
                        set_last_error(code);
                        return Some(Ok(vec![0]));
                    }
                };
                if arg(2) != 0
                    && let Err(trap) = self.put(arg(2), &previous.to_le_bytes())
                {
                    return Some(Err(trap));
                }
                Ok(1)
            }
            "CreateEventA" | "CreateEventW" => {
                let (manual, set) = (arg(1) as u32 != 0, arg(2) as u32 != 0);
                let real = real_event_new(manual, set);
                Ok(self.new_object_at(
                    real,
                    Object::Event {
                        manual,
                        set,
                        real,
                    },
                ))
            }
            "SetEvent" | "ResetEvent" if self.owns(arg(0)) => {
                let now_set = symbol == "SetEvent";
                let found = self.with_sched(|s| {
                    let Some(Object::Event {
                        set,
                        real,
                        ..
                    }) = s.objects.get_mut(&arg(0))
                    else {
                        return false;
                    };
                    *set = now_set;
                    if *real != 0 {
                        real_event_call(*real, now_set);
                    }
                    if now_set {
                        s.wake_object_waiters();
                    }
                    true
                });
                if !found {
                    set_last_error(ERROR_INVALID_PARAMETER);
                    return Some(Ok(vec![0]));
                }
                Ok(1)
            }

            // Sleeping and yielding only matter once another thread exists.
            "Sleep" | "SleepEx" if started => match arg(0) as u32 {
                0 => self.yield_now(),
                ms => self.sleep_for(Duration::from_millis(ms as u64)).map(|_| ()),
            }
            .map(|_| 0),
            "SwitchToThread" if started => self.yield_now().map(|_| 1),
            _ => return None,
        };
        Some(result.map(|v| vec![v]))
    }

    fn owns(&mut self, handle: u64) -> bool {
        // A handle below the scheduler's own range can only be a kernel event it made; programs
        // that make none (most of them) never touch the scheduler for a real handle.
        if handle < HANDLE_BASE && REAL_EVENTS.load(Ordering::Relaxed) == 0 {
            return false;
        }

        handle != 0 && self.with_sched(|s| s.objects.contains_key(&handle))
    }

    /// Like `new_object`, but the object is known by `handle` when that is not 0 (a kernel
    /// handle).
    fn new_object_at(&mut self, handle: u64, object: Object) -> u64 {
        if handle == 0 {
            return self.new_object(object);
        }
        self.with_sched(|s| s.objects.insert(handle, object));
        handle
    }

    fn new_object(&mut self, object: Object) -> u64 {
        self.with_sched(|s| {
            let handle = HANDLE_BASE + s.next_object * 4;
            s.next_object += 1;
            s.objects.insert(handle, object);
            handle
        })
    }

    /// `CreateThread(attributes, stack size, start, parameter, flags, id out)`: the handle, or
    /// null when no thread could be started.
    fn create_thread(
        &mut self,
        program: &Program,
        start: u64,
        parameter: u64,
        flags: u64,
        id_out: u64,
    ) -> Res<u64> {
        const CREATE_SUSPENDED: u64 = 4;
        let valid_entry = self
            .func_of(start)
            .and_then(|func| program.funcs[func.0 as usize].as_ref())
            .is_some_and(|function| {
                let signature = &function.sig;
                signature.conv == ir::Conv::C
                    && signature.params == [Ty::Ptr]
                    && signature.returns == [Ty::I32]
                    && !signature.c_varargs
                    && signature.c_abi.is_none()
            });
        if !valid_entry {
            return self.trap("Win32 thread entry ABI requires one pointer argument and one 32-bit result with C convention");
        }
        let Some(id) =
            self.spawn_gated_thread(program, start, parameter, flags & CREATE_SUSPENDED != 0)?
        else {
            return Ok(0);
        };
        let shared = self.shared();
        let native_id = {
            let mut s = shared.lock();
            while s.threads[id].native_id == 0 {
                s = shared.cv.wait(s).unwrap_or_else(|e| e.into_inner());
            }
            s.threads[id].native_id
        };
        if id_out != 0 {
            self.put(id_out, &native_id.to_le_bytes())?;
        }
        Ok(self.new_object(Object::Thread(id)))
    }

    /// `WaitForSingleObject` / `WaitForMultipleObjects` on handles the scheduler owns.
    fn wait_objects(&mut self, handles: &[u64], all: bool, milliseconds: u32) -> Res<u64> {
        let deadline = deadline_after(milliseconds);
        loop {
            let done = self.with_sched(|s| {
                let mut ready = Vec::new();
                for (i, h) in handles.iter().enumerate() {
                    let object = s.objects.get_mut(h)?;
                    if object.signaled(&s.threads) {
                        ready.push(i);
                    }
                }
                let done = if all {
                    ready.len() == handles.len()
                } else {
                    !ready.is_empty()
                };
                if !done {
                    return Some(None);
                }
                let taken = if all {
                    &ready[..]
                } else {
                    &ready[..1]
                };
                for &i in taken {
                    if let Some(object) = s.objects.get_mut(&handles[i]) {
                        object.consume();
                    }
                }
                Some(Some(if all {
                    WAIT_OBJECT_0
                } else {
                    WAIT_OBJECT_0 + ready[0] as u64
                }))
            });
            match done {
                None => {
                    set_last_error(ERROR_INVALID_PARAMETER);
                    return Ok(WAIT_FAILED);
                }
                Some(Some(result)) => return Ok(result),
                Some(None) => {}
            }
            if milliseconds == 0 || deadline.is_some_and(|d| d <= Instant::now()) {
                return Ok(WAIT_TIMEOUT);
            }
            // A kernel event is signaled without telling the scheduler, so look again soon.
            let poll = self.with_sched(|s| {
                handles.iter().any(|h| {
                    matches!(
                        s.objects.get(h),
                        Some(Object::Event { real, .. }) if *real != 0
                    )
                })
            });
            let wake = if poll {
                let soon = Instant::now() + Duration::from_millis(1);
                Some(deadline.map_or(soon, |d| d.min(soon)))
            } else {
                deadline
            };
            self.block(Block::Object(wake))?;
        }
    }
}

/// When a Win32 timeout in milliseconds ends (`INFINITE`: never).
fn deadline_after(milliseconds: u32) -> Option<Instant> {
    (milliseconds != INFINITE).then(|| Instant::now() + Duration::from_millis(milliseconds as u64))
}
