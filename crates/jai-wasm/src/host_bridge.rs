//! Host functions the embedding page provides (the playground's WebGPU and canvas, see
//! `js/webgpu_host.mjs`). A foreign procedure the sandbox does not implement is offered to the
//! page by name through the `jai_host` imports, with its arguments as 64-bit slots (pointers are
//! addresses in this module's memory). The page answers at once, or with a promise: then the
//! call suspends the whole module through JSPI (`WebAssembly.Suspending`) until it settles.
//!
//! Status codes of `call` and `wait`: 0 done, 1 not provided, 2 failed (`error` has the
//! message), 3 pending (call `wait`).
use jaic::interp::{Host, SharedHost};
use jaic::ir;

#[cfg(target_arch = "wasm32")]
mod imports {
    #[link(wasm_import_module = "jai_host")]
    unsafe extern "C" {
        pub fn call(
            name: *const u8,
            name_len: usize,
            args: *const u64,
            nargs: usize,
            results: *mut u64,
            nresults: usize,
        ) -> i32;
        pub fn wait(results: *mut u64, nresults: usize) -> i32;
        pub fn error(buffer: *mut u8, capacity: usize) -> usize;
        pub fn output(data: *const u8, len: usize, to_stderr: i32);
        pub fn now_ms() -> f64;
    }
}

/// The playground's interpreter host: the sandbox plus the page's host functions.
pub struct PlayHost {
    pub shared: SharedHost,
    budget: Option<u64>,
    waited: bool,
    last_ms: f64,
    sent: (usize, usize),
}

impl PlayHost {
    pub fn new(shared: SharedHost, budget: Option<u64>) -> PlayHost {
        PlayHost {
            shared,
            budget,
            waited: false,
            last_ms: now_ms(),
            sent: (0, 0),
        }
    }

    /// Hand output written since the last call to the page, so a long-running program shows it
    /// while it runs. The run's result still carries all of it.
    fn stream_output(&mut self) {
        let host = self.shared.0.borrow();
        let (out, err) = self.sent;
        send_output(&host.stdout[out.min(host.stdout.len())..], false);
        send_output(&host.stderr[err.min(host.stderr.len())..], true);
        self.sent = (host.stdout.len(), host.stderr.len());
    }

    fn after_wait(&mut self) {
        self.waited = true;
        self.stream_output();
        // Sleeping only moves the sandbox's virtual clock; a wait for the page took real time.
        let now = now_ms();
        if now > self.last_ms {
            self.shared
                .advance_clock(((now - self.last_ms) * 1_000_000.0) as u64);
        }
        self.last_ms = now;
    }
}

impl Host for PlayHost {
    fn write(&mut self, bytes: &[u8], to_stderr: bool) {
        self.shared.write(bytes, to_stderr);
    }

    fn foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>> {
        if let Some(result) = self.shared.foreign(symbol, args, sig) {
            return Some(result);
        }
        let mut results = vec![0u64; sig.returns.len()];
        match host_call(symbol, args, &mut results)? {
            Ok(waited) => {
                if waited {
                    self.after_wait();
                }
                Some(Ok(results))
            }
            Err(message) => Some(Err(format!("{symbol}: {message}"))),
        }
    }

    fn native_linking(&self) -> bool {
        false
    }

    fn cooperative_threads(&self) -> bool {
        true
    }

    fn advance_clock(&mut self, nanoseconds: u64) {
        self.shared.advance_clock(nanoseconds);
    }

    fn virtual_now_ns(&mut self) -> Option<u64> {
        self.shared.virtual_now_ns()
    }

    fn refill_budget(&mut self) -> Option<u64> {
        if std::mem::take(&mut self.waited) {
            self.budget
        } else {
            None
        }
    }
}

/// `None`: the page does not provide `symbol`. `Ok(true)`: it answered after a wait.
#[cfg(target_arch = "wasm32")]
#[allow(unsafe_code)]
fn host_call(symbol: &str, args: &[u64], results: &mut [u64]) -> Option<Result<bool, String>> {
    // SAFETY: the imports read `args` and write `results` within the lengths passed.
    let status = unsafe {
        imports::call(
            symbol.as_ptr(),
            symbol.len(),
            args.as_ptr(),
            args.len(),
            results.as_mut_ptr(),
            results.len(),
        )
    };
    let (status, waited) = match status {
        3 => (
            unsafe { imports::wait(results.as_mut_ptr(), results.len()) },
            true,
        ),
        s => (s, false),
    };
    match status {
        0 => Some(Ok(waited)),
        1 => None,
        _ => {
            let mut buffer = vec![0u8; 4096];
            // SAFETY: the import writes at most `capacity` bytes.
            let len = unsafe { imports::error(buffer.as_mut_ptr(), buffer.len()) };
            buffer.truncate(len.min(4096));
            Some(Err(String::from_utf8_lossy(&buffer).into_owned()))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn host_call(_symbol: &str, _args: &[u64], _results: &mut [u64]) -> Option<Result<bool, String>> {
    None
}

#[cfg(target_arch = "wasm32")]
#[allow(unsafe_code)]
fn send_output(bytes: &[u8], to_stderr: bool) {
    if !bytes.is_empty() {
        // SAFETY: the import only reads `bytes`.
        unsafe { imports::output(bytes.as_ptr(), bytes.len(), to_stderr as i32) };
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn send_output(_bytes: &[u8], _to_stderr: bool) {
}

#[cfg(target_arch = "wasm32")]
#[allow(unsafe_code)]
fn now_ms() -> f64 {
    // SAFETY: no arguments.
    unsafe { imports::now_ms() }
}

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> f64 {
    0.0
}

/// Memory the page fills for the program (mapped buffer ranges, strings it returns).
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn jai_host_alloc(size: usize) -> usize {
    match std::alloc::Layout::from_size_align(size.max(1), 16) {
        // SAFETY: the layout has a non-zero size.
        Ok(layout) => unsafe { std::alloc::alloc_zeroed(layout) as usize },
        Err(_) => 0,
    }
}

/// Free what `jai_host_alloc(size)` returned.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn jai_host_free(ptr: usize, size: usize) {
    if ptr == 0 {
        return;
    }
    if let Ok(layout) = std::alloc::Layout::from_size_align(size.max(1), 16) {
        // SAFETY: the page passes back a pointer and size from jai_host_alloc.
        unsafe { std::alloc::dealloc(ptr as *mut u8, layout) };
    }
}
