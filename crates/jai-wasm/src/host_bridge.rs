//! Host functions the embedding page provides (the playground's WebGPU and canvas, see
//! `js/webgpu_host.mjs`). A foreign procedure the sandbox does not implement is offered to the
//! page by name through the `jai_host` imports, with its arguments as 64-bit slots (pointers are
//! addresses in this module's memory). The page answers at once, or with a promise: then the
//! call suspends the whole module through JSPI (`WebAssembly.Suspending`) until it settles.
//!
//! Status codes of `call` and `wait`: 0 done, 1 not provided, 2 failed (`error` has the
//! message), 3 pending (call `wait`).
//!
//! No unsafe blocks: the imports are declared `safe` and exchange plain addresses (`usize`), so
//! the only unsafe syntax is the `unsafe extern` block itself, which wasm imports need. They are
//! safe to call because the page only reads and writes the ranges passed, in this module's
//! memory (engine.mjs checks each range against the memory's size), and Rust never holds a
//! reference into a range while the page writes it.
use jaic::interp::{Host, SharedHost};
use jaic::ir;
use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
#[allow(unsafe_code)]
mod imports {
    #[link(wasm_import_module = "jai_host")]
    unsafe extern "C" {
        /// Call the page's host function named by the UTF-8 bytes at `name`: it reads `nargs`
        /// u64 slots at `args` and writes up to `nresults` at `results`.
        pub safe fn call(
            name: usize,
            name_len: usize,
            args: usize,
            nargs: usize,
            results: usize,
            nresults: usize,
        ) -> i32;

        /// Suspend until the promise of the last `call` settles; writes its results.
        pub safe fn wait(results: usize, nresults: usize) -> i32;

        /// Write the last failure's message (at most `capacity` bytes) to `buffer`.
        pub safe fn error(buffer: usize, capacity: usize) -> usize;

        /// Hand the page `len` bytes of program output.
        pub safe fn output(data: usize, len: usize, to_stderr: i32);

        pub safe fn now_ms() -> f64;
    }
}

/// Bytes asked of the page per standard input read.
const STDIN_CHUNK: usize = 4096;

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

    /// Wait for the page to supply standard input: it fills a buffer of ours and answers with
    /// the byte count (a promise that settles when the user typed a line), 0 for the end of
    /// input. A page with no `jai_stdin_read` closes the input. Returns whether bytes came.
    fn fill_stdin(&mut self) -> bool {
        // Whatever the program printed first (a prompt) is on screen before it waits.
        self.stream_output();
        let mut buffer = vec![0u8; STDIN_CHUNK];
        let args = [buffer.as_mut_ptr() as usize as u64, buffer.len() as u64];
        let mut results = [0u64];
        let outcome = host_call("jai_stdin_read", &args, &mut results);
        let mut shared = self.shared.0.borrow_mut();
        match outcome {
            Some(Ok(waited)) => {
                drop(shared);
                if waited {
                    self.after_wait();
                }
                let count = (results[0] as usize).min(buffer.len());
                let mut shared = self.shared.0.borrow_mut();
                if count == 0 {
                    shared.end_stdin();
                    return false;
                }
                shared.feed_stdin(&buffer[..count]);
                true
            }
            Some(Err(message)) => {
                shared.write(format!("stdin: {message}\n").as_bytes(), true);
                shared.close_stdin();
                false
            }
            None => {
                shared.close_stdin();
                false
            }
        }
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
        while self.shared.0.borrow().stdin_wants_data(symbol, args) {
            if !self.fill_stdin() {
                break;
            }
        }
        // Asked of the page first: a page without the procedure (or without any host) answers
        // that nothing is provided, which the sandbox says too.
        let ask_page_first = symbol == "jai_host_provides";
        if !ask_page_first && let Some(result) = self.shared.foreign(symbol, args, sig) {
            return Some(result);
        }
        let mut results = vec![0u64; sig.returns.len()];
        match host_call(symbol, args, &mut results) {
            Some(Ok(waited)) => {
                if waited {
                    self.after_wait();
                }
                Some(Ok(results))
            }
            Some(Err(message)) => Some(Err(format!("{symbol}: {message}"))),
            None if ask_page_first => self.shared.foreign(symbol, args, sig),
            None => None,
        }
    }

    fn foreign_data(&mut self, symbol: &str) -> Option<u64> {
        self.shared.foreign_data(symbol)
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
fn host_call(symbol: &str, args: &[u64], results: &mut [u64]) -> Option<Result<bool, String>> {
    let status = imports::call(
        symbol.as_ptr() as usize,
        symbol.len(),
        args.as_ptr() as usize,
        args.len(),
        results.as_mut_ptr() as usize,
        results.len(),
    );
    let (status, waited) = match status {
        3 => (
            imports::wait(results.as_mut_ptr() as usize, results.len()),
            true,
        ),
        s => (s, false),
    };
    match status {
        0 => Some(Ok(waited)),
        1 => None,
        _ => {
            let mut buffer = vec![0u8; 4096];
            let len = imports::error(buffer.as_mut_ptr() as usize, buffer.len());
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
fn send_output(bytes: &[u8], to_stderr: bool) {
    if !bytes.is_empty() {
        imports::output(bytes.as_ptr() as usize, bytes.len(), to_stderr as i32);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn send_output(_bytes: &[u8], _to_stderr: bool) {
}

#[cfg(target_arch = "wasm32")]
fn now_ms() -> f64 {
    imports::now_ms()
}

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> f64 {
    0.0
}

thread_local! {
    /// Blocks the page asked for (`jai_host_alloc`), by address. 16-byte units keep them aligned
    /// for any value the program reads from them.
    static HOST_BLOCKS: RefCell<HashMap<usize, Box<[u128]>>> = RefCell::new(HashMap::new());
}

/// Zeroed memory the page fills for the program (mapped buffer ranges, strings it returns),
/// 16-byte aligned; 0 when `size` is too large.
pub fn host_alloc(size: usize) -> usize {
    let Some(units) = size.max(1).checked_add(15).map(|n| n / 16) else {
        return 0;
    };
    let mut block = Vec::new();
    if block.try_reserve_exact(units).is_err() {
        return 0;
    }
    block.resize(units, 0u128);
    let block = block.into_boxed_slice();
    let address = block.as_ptr() as usize;
    HOST_BLOCKS.with(|blocks| blocks.borrow_mut().insert(address, block));
    address
}

/// Release a block `host_alloc` returned; other addresses are ignored.
pub fn host_free(address: usize) {
    HOST_BLOCKS.with(|blocks| blocks.borrow_mut().remove(&address));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_blocks_are_aligned_zeroed_and_freed() {
        let a = host_alloc(3);
        let b = host_alloc(100);
        assert!(a != 0 && b != 0 && a != b);
        assert_eq!(a % 16, 0);
        assert_eq!(b % 16, 0);
        HOST_BLOCKS.with(|blocks| {
            let blocks = blocks.borrow();
            assert_eq!(blocks[&b].len(), 7);
            assert!(blocks[&b].iter().all(|&unit| unit == 0));
        });
        host_free(a);
        host_free(a);
        host_free(12345);
        HOST_BLOCKS.with(|blocks| assert!(!blocks.borrow().contains_key(&a)));
        host_free(b);
        assert_eq!(host_alloc(usize::MAX), 0);
    }
}
