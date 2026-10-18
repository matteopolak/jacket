//! Worker thread entry points and deadlines.
use super::sched::{CANCELLED_EXIT_CODE, Shared, WorkerCancelled};
use super::win32;
use crate::interp::*;
use std::time::{Duration, Instant, SystemTime};

/// The instant a POSIX absolute `timespec` (seconds and nanoseconds since the epoch) names.
pub(super) fn absolute_deadline(secs: u64, nanos: u64) -> Instant {
    let target = Duration::new(secs, nanos as u32);
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    Instant::now() + target.saturating_sub(now)
}

/// Body of the OS thread that runs Jai thread `id`.
fn thread_main(shared: &Shared, program: usize, id: usize, func: FuncId, argument: u64) -> u64 {
    let (interp, state) = {
        let mut s = shared.lock();
        s.threads[id].native_id = win32::native_thread_id(id);
        shared.cv.notify_all();
        let mut s = shared.acquire(s, id);
        (s.interp, s.threads[id].start.take().unwrap_or_default())
    };
    // SAFETY: only the thread holding the baton touches the interpreter, and the program
    // stays alive until Shared::close has cancelled/joined every managed OS entry.
    let interp = unsafe { &mut *(interp as *mut Interp) };
    // SAFETY: as above, the program outlives every managed OS entry.
    let program = unsafe { &*(program as *const Program) };
    interp.put_exec_state(state);
    let result = interp.exec(program, func, &[argument]);
    let value = match result {
        Ok(values) if values.len() == 1 => values[0],
        Ok(_) => {
            eprintln!("error: interpreter thread entry returned an invalid result shape");
            std::process::exit(1);
        }
        Err(trap) => {
            if let Some(TrapKind::ThreadExit(code)) = trap.kind {
                interp.finish_thread(id, code as u64);
                return code as u64;
            }
            let at = trap
                .loc
                .map(|(_, line, col)| format!(" (line {line}, column {col})"))
                .unwrap_or_default();
            eprintln!("error: runtime error in a thread: {}{at}", trap.message);
            std::process::exit(1);
        }
    };
    interp.finish_thread(id, value);
    value
}

/// Catch cancellation before returning across an OS/CRT thread-entry boundary.
pub(super) fn run_worker(
    shared: &Shared,
    program: usize,
    id: usize,
    func: FuncId,
    argument: u64,
) -> u64 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        thread_main(shared, program, id, func, argument)
    })) {
        Ok(value) => value,
        Err(cancelled) if cancelled.is::<WorkerCancelled>() => {
            let mut s = shared.lock();
            let thread = &mut s.threads[id];
            thread.start.take();
            thread.finished = true;
            thread.result = CANCELLED_EXIT_CODE as u64;
            shared.cv.notify_all();
            CANCELLED_EXIT_CODE as u64
        }
        Err(other) => std::panic::resume_unwind(other),
    }
}
