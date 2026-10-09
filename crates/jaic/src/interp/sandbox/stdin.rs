//! Standard input for the sandbox: descriptor 0 and the C `stdin` stream.
//!
//! Bytes arrive from the embedder (`feed_stdin`, or a fixed input from `set_stdin`); the program
//! reads them with `read(0, ..)`, `fread`, `fgets`, `getline`/`getdelim` and `getc` on `stdin`.
//! The sandbox never waits itself. An embedder that can (the browser host, which suspends the
//! module until the user types) asks `stdin_wants_data` before a call and feeds what it needs;
//! a read that still finds nothing reports the end of input, as on a terminal after Ctrl+D.
use super::*;
use std::collections::VecDeque;

#[derive(Default)]
pub(super) struct Stdin {
    data: VecDeque<u8>,
    /// No more bytes will ever come: a fixed input that was fed in full, or a host with no input.
    closed: bool,
    /// The embedder reported an end of input; the next read that finds nothing consumes it.
    end_pending: bool,
    /// The last read stopped at the end (`feof`).
    at_end: bool,
    /// Address of the `FILE*` the program sees as `stdin`, once asked for.
    stream: u64,
}

impl Stdin {
    /// Up to `max` bytes, ending after the first `delimiter` when there is one.
    fn take(&mut self, max: usize, delimiter: Option<u8>) -> Vec<u8> {
        let mut len = self.data.len().min(max);
        if let Some(delimiter) = delimiter
            && let Some(at) = self.data.iter().take(len).position(|&b| b == delimiter)
        {
            len = at + 1;
        }
        let bytes: Vec<u8> = self.data.drain(..len).collect();
        self.at_end = bytes.is_empty();
        if bytes.is_empty() {
            self.end_pending = false;
        }
        bytes
    }

    fn holds(&self, bytes: usize, delimiter: Option<u8>) -> bool {
        self.data.len() >= bytes || delimiter.is_some_and(|d| self.data.iter().any(|&b| b == d))
    }
}

/// `-1`, as a C `int` or `ssize_t` result.
const FAILED: u64 = u64::MAX;

impl SandboxHost {
    /// Give the program `bytes` as its whole input; it sees the end after them.
    pub fn set_stdin(&mut self, bytes: &[u8]) {
        self.stdin.data.extend(bytes);
        self.stdin.closed = true;
    }

    /// More input for the program to read.
    pub fn feed_stdin(&mut self, bytes: &[u8]) {
        self.stdin.data.extend(bytes);
    }

    /// The input ended (Ctrl+D): the next read that finds nothing sees the end.
    pub fn end_stdin(&mut self) {
        self.stdin.end_pending = true;
    }

    /// No input will come at all (the embedder has no source): reads see the end.
    pub fn close_stdin(&mut self) {
        self.stdin.closed = true;
    }

    /// Whether `symbol(args)` reads standard input and would find too little buffered, so an
    /// embedder that can wait should feed more (or end the input) before the call.
    pub fn stdin_wants_data(&self, symbol: &str, args: &[u64]) -> bool {
        let stdin = &self.stdin;
        if stdin.closed || stdin.end_pending {
            return false;
        }
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        let on = |handle: u64| stdin.stream != 0 && handle == stdin.stream;
        match symbol {
            "read" if arg(0) as i32 == 0 => arg(2) > 0 && !stdin.holds(1, None),
            "fread" if on(arg(3)) => {
                let want = (arg(1) as usize).saturating_mul(arg(2) as usize);
                want > 0 && !stdin.holds(want, None)
            }
            "fgets" if on(arg(2)) => {
                let room = (arg(1) as i32).saturating_sub(1).max(0) as usize;
                room > 0 && !stdin.holds(room, Some(b'\n'))
            }
            "getline" if on(arg(2)) => !stdin.holds(usize::MAX, Some(b'\n')),
            "getdelim" if on(arg(3)) => !stdin.holds(usize::MAX, Some(arg(2) as u8)),
            "fgetc" | "getc" | "fgetc_unlocked" | "getc_unlocked" if on(arg(0)) => {
                !stdin.holds(1, None)
            }
            "getchar" | "getchar_unlocked" => !stdin.holds(1, None),
            _ => false,
        }
    }

    /// The `stdin` variable of the C library (`__stdinp` on macOS): the address of a cell
    /// holding the stream.
    pub(super) fn stdin_variable(&mut self, symbol: &str) -> Option<u64> {
        if symbol != "stdin" && symbol != "__stdinp" {
            return None;
        }
        if self.stdin.stream == 0 {
            let stream = self.alloc(256);
            self.streams.insert(stream, 0);
            self.stdin.stream = stream;
        }
        let cell = self.alloc(8);
        write_u64(cell, self.stdin.stream);
        Some(cell)
    }

    /// The reads of standard input; `None` for any other call.
    pub(super) fn stdin_foreign(&mut self, symbol: &str, args: &[u64]) -> Option<u64> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        let stream = self.stdin.stream;
        let on = |handle: u64| stream != 0 && handle == stream;
        let copy = |to: u64, bytes: &[u8]| unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), to as *mut u8, bytes.len());
        };
        Some(match symbol {
            "read" if arg(0) as i32 == 0 => {
                let bytes = self.stdin.take(arg(2) as usize, None);
                copy(arg(1), &bytes);
                bytes.len() as u64
            }
            "fread" if on(arg(3)) => {
                let size = arg(1) as usize;
                let want = size.saturating_mul(arg(2) as usize);
                if size == 0 || want == 0 {
                    return Some(0);
                }
                let bytes = self.stdin.take(want, None);
                copy(arg(0), &bytes);
                self.stdin.at_end = bytes.len() < want;
                (bytes.len() / size) as u64
            }
            "fgets" if on(arg(2)) => {
                let room = (arg(1) as i32).saturating_sub(1).max(0) as usize;
                if room == 0 {
                    return Some(0);
                }
                let mut bytes = self.stdin.take(room, Some(b'\n'));
                if bytes.is_empty() {
                    return Some(0);
                }
                bytes.push(0);
                copy(arg(0), &bytes);
                arg(0)
            }
            "getline" | "getdelim" => {
                let (handle, delimiter) = if symbol == "getline" {
                    (arg(2), b'\n')
                } else {
                    (arg(3), arg(2) as u8)
                };
                if !on(handle) {
                    return None;
                }
                let mut bytes = self.stdin.take(usize::MAX, Some(delimiter));
                if bytes.is_empty() {
                    return Some(FAILED);
                }
                let length = bytes.len() as u64;
                bytes.push(0);
                let (line, size) = (arg(0), arg(1));
                let (old, capacity) = unsafe {
                    (
                        std::ptr::read_unaligned(line as *const u64),
                        std::ptr::read_unaligned(size as *const u64),
                    )
                };
                let target = if old != 0 && capacity >= bytes.len() as u64 {
                    old
                } else {
                    let new = self.alloc(bytes.len().max(120));
                    if new == 0 {
                        return Some(FAILED);
                    }
                    self.allocations.remove(&old);
                    write_u64(line, new);
                    write_u64(size, bytes.len().max(120) as u64);
                    new
                };
                copy(target, &bytes);
                length
            }
            "fgetc" | "getc" | "fgetc_unlocked" | "getc_unlocked" | "getchar"
            | "getchar_unlocked"
                if symbol.starts_with("getchar") || on(arg(0)) =>
            {
                self.stdin
                    .take(1, None)
                    .first()
                    .map_or(FAILED, |&b| u64::from(b))
            }
            "ungetc" if on(arg(1)) => {
                let byte = arg(0) as i32;
                if byte < 0 {
                    return Some(FAILED);
                }
                self.stdin.data.push_front(byte as u8);
                self.stdin.at_end = false;
                byte as u64
            }
            "feof" if on(arg(0)) => u64::from(self.stdin.at_end),
            _ => return None,
        })
    }
}
