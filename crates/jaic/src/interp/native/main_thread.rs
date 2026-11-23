//! `AppKit` only works on the process's main thread, but the interpreter runs on a worker
//! with a big stack. The driver parks the main thread in `main_thread::serve`. The
//! primary thread forwards calls according to their owning native image, including the
//! first call into an unfamiliar library. The worker waits for each forwarded call.
//! Thread-safe system-library calls retain their existing direct fast path.

use super::{Plan, Rets, Sig, call_with, callbacks};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, OnceLock};

type Job = Box<dyn FnOnce() + Send>;

struct Route {
    /// Calls for the main thread to make; `None` (from `serve`) means the worker is done.
    jobs: Mutex<Sender<Option<Job>>>,
    worker: std::thread::ThreadId,
}

static ROUTE: OnceLock<Route> = OnceLock::new();

static DISABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Stop forwarding for good (a forked child has no main thread serving jobs).
pub fn disable() {
    DISABLED.store(true, std::sync::atomic::Ordering::SeqCst);
}

thread_local! {
    static DIRECT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run `f` with foreign calls made on the calling thread. `fork` needs this: the child
/// only has the forking thread, so the call must come from the interpreter's own thread.
pub fn direct<T>(f: impl FnOnce() -> T) -> T {
    let before = DIRECT.with(|d| d.replace(true));
    let result = f();
    DIRECT.with(|d| d.set(before));
    result
}

/// Run `body` on a new thread (`stack` bytes) while this thread serves its foreign calls.
pub fn serve<T: Send + 'static>(
    stack: usize,
    body: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    let (tx, rx): (Sender<Option<Job>>, Receiver<Option<Job>>) = channel();
    // Ends the loop below once `body` returns or unwinds: `ROUTE` keeps its sender alive
    // for good, so the channel never disconnects on its own.
    struct Finished(Sender<Option<Job>>);

    impl Drop for Finished {
        fn drop(&mut self) {
            let _ = self.0.send(None);
        }
    }

    let finished = Finished(tx.clone());
    let worker = std::thread::Builder::new()
        .stack_size(stack)
        .spawn(move || {
            let _finished = finished;
            let _ = ROUTE.set(Route {
                jobs: Mutex::new(tx),
                worker: std::thread::current().id(),
            });
            body()
        });
    let handle = worker.ok()?;
    while let Ok(Some(job)) = rx.recv() {
        job();
    }
    handle.join().ok()
}

/// libSystem (libc, libm, pthreads, libdispatch...) and libc++ are thread-safe and never
/// touch `AppKit`: calling them directly avoids a thread handoff per call. So is the
/// allocator this executable carries (`malloc` and friends), which foreign code reaches
/// through the same names. Everything else (frameworks, and user libraries such as SDL or
/// GLFW that call into Cocoa) is forwarded.
fn needs_main_thread(addr: u64) -> bool {
    use crate::fxhash::HashMap;
    thread_local! {
        static CACHE: std::cell::RefCell<HashMap<u64, bool>> = std::cell::RefCell::new(HashMap::default());
    }
    if let Some(known) = CACHE.with(|c| c.borrow().get(&addr).copied()) {
        return known;
    }
    #[repr(C)]
    struct DlInfo {
        fname: *const std::ffi::c_char,
        fbase: *mut std::ffi::c_void,
        sname: *const std::ffi::c_char,
        saddr: *mut std::ffi::c_void,
    }
    unsafe extern "C" {
        fn dladdr(addr: *const std::ffi::c_void, info: *mut DlInfo) -> i32;
    }
    let mut info = DlInfo {
        fname: std::ptr::null(),
        fbase: std::ptr::null_mut(),
        sname: std::ptr::null(),
        saddr: std::ptr::null_mut(),
    };
    // SAFETY: dladdr only reads the loaded-image tables and fills `info`.
    let found = unsafe { dladdr(addr as *const std::ffi::c_void, &raw mut info) } != 0;
    let mut own = DlInfo {
        fname: std::ptr::null(),
        fbase: std::ptr::null_mut(),
        sname: std::ptr::null(),
        saddr: std::ptr::null_mut(),
    };
    let own_found =
        // SAFETY: as above, for an address in this executable.
        unsafe { dladdr(needs_main_thread as *const std::ffi::c_void, &raw mut own) } != 0;
    let main = if !found || info.fname.is_null() {
        true
    } else if own_found && info.fbase == own.fbase {
        false
    } else {
        // SAFETY: dladdr returns a NUL-terminated path owned by the loader.
        let path = unsafe { std::ffi::CStr::from_ptr(info.fname) }.to_string_lossy();
        !(path.starts_with("/usr/lib/system/")
            || path.starts_with("/usr/lib/libSystem")
            || path.starts_with("/usr/lib/libc++"))
    };
    CACHE.with(|c| c.borrow_mut().insert(addr, main));
    main
}

struct Carry<T>(T);

// SAFETY: the sending thread blocks until the job finishes, so the pointers are never
// used from two threads at once.
unsafe impl<T> Send for Carry<T> {
}

pub(super) fn forward(
    addr: u64,
    args: &[u64],
    sig: &Sig,
    plan: &Plan,
) -> Option<Result<Rets, String>> {
    forward_inner(addr, args, sig, plan, false)
}

pub(super) fn forward_loader(
    addr: u64,
    args: &[u64],
    sig: &Sig,
    plan: &Plan,
) -> Option<Result<Rets, String>> {
    forward_inner(addr, args, sig, plan, true)
}

fn forward_inner(
    addr: u64,
    args: &[u64],
    sig: &Sig,
    plan: &Plan,
    initializing_library: bool,
) -> Option<Result<Rets, String>> {
    let route = ROUTE.get()?;
    if DISABLED.load(std::sync::atomic::Ordering::SeqCst) || DIRECT.with(std::cell::Cell::get) {
        return None;
    }
    if std::thread::current().id() != route.worker {
        return None;
    }
    if !initializing_library {
        let main = plan.main_thread.get().unwrap_or_else(|| {
            let main = needs_main_thread(addr);
            plan.main_thread.set(Some(main));
            main
        });
        if !main {
            return None;
        }
    }
    let (done_tx, done_rx) = channel();
    // Callbacks made during the job belong to the waiting worker's call.
    let caller = callbacks::caller();
    let carried = Carry((
        std::ptr::from_ref::<[u64]>(args),
        std::ptr::from_ref::<Sig>(sig),
        std::ptr::from_ref::<Plan>(plan),
    ));
    let attribution = Carry(crate::interp::crash::Attribution::of_this_thread());
    let job: Job = Box::new(move || {
        let carried = carried;
        let (args, sig, plan) = carried.0;
        // SAFETY: see `Carry`; the caller's borrows outlive this job.
        let (args, sig, plan) = unsafe { (&*args, &*sig, &*plan) };
        // A crash here is in the worker's foreign call, made on this thread for it.
        let attribution = attribution;
        let _crash_report = attribution.0.enter();
        let result = callbacks::calling_out(caller, || call_with(addr, args, sig, plan));
        let _ = done_tx.send(Carry(result));
    });
    route.jobs.lock().ok()?.send(Some(job)).ok()?;
    done_rx.recv().ok().map(|c| c.0)
}
