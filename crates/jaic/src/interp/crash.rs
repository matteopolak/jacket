//! What jaic says when native code it called for an interpreted program crashes.
//!
//! `#run` code and `jaic -interp` call foreign procedures directly, so a bad pointer passed to C
//! faults inside jaic's own process. Without a handler the process dies with a bare signal
//! and no hint of which call did it. While a foreign call is in progress (`ForeignCall`), a
//! fault handler — `sigaction` on an alternate stack for SIGSEGV, SIGBUS, SIGILL and SIGFPE,
//! a vectored exception handler on Windows — prints the foreign procedure, the Jai line that
//! called it and the interpreter's call stack, then exits with `CRASH_STATUS`.
//!
//! Several interpreted threads can be inside foreign calls at once (other threads run while
//! one is in C), so the call in progress is kept per OS thread: these faults are delivered
//! to the thread that caused them, which reports its own call and call stack.
//!
//! Faults outside a foreign call are not ours: the handler puts the previous action back and
//! returns, so the fault repeats under it (Rust's stack overflow report, or the default).
//! The report is built in a fixed buffer, without allocating, since the crash may have
//! happened inside `malloc`.
#![allow(unsafe_code)]

use super::ExecState;
use crate::ir::Program;
use std::cell::Cell;

/// Exit status after a crash in native code (see docs/compiler/diagnostics.md).
pub const CRASH_STATUS: i32 = 121;

/// Frames of the interpreter's call stack shown at most, innermost first.
const MAX_SHOWN_FRAMES: usize = 24;

/// The foreign call in progress, read by the fault handler: the symbol's bytes, the calling
/// thread's interpreter state (its call stack and line) and the program (null when no call is
/// in progress).
#[derive(Clone, Copy)]
struct Current {
    symbol: (*const u8, usize),
    state: *const ExecState,
    program: *const Program,
}

const NONE: Current = Current {
    symbol: (std::ptr::null(), 0),
    state: std::ptr::null(),
    program: std::ptr::null(),
};

thread_local! {
    // Constant-initialized and without a destructor: plain thread-local storage, which the
    // fault handler may read.
    static CURRENT: Cell<Current> = const { Cell::new(NONE) };
}

fn current() -> Current {
    CURRENT.with(Cell::get)
}

fn publish(c: Current) {
    CURRENT.with(|cell| cell.set(c));
}

/// Marks a foreign call in progress on this thread for as long as it lives; nested calls (C
/// calling back into interpreted code that calls C again) restore the outer one when they end.
/// Entering is a few stores: it happens on every foreign call.
pub(super) struct ForeignCall {
    previous: Current,
}

impl ForeignCall {
    /// `state` is the calling thread's interpreter state, set aside for the call.
    pub(super) fn enter(symbol: &str, state: &ExecState, program: &Program) -> Self {
        install();
        let previous = current();
        publish(Current {
            symbol: (symbol.as_ptr(), symbol.len()),
            state,
            program,
        });
        Self {
            previous,
        }
    }
}

/// The foreign call in progress on one thread, carried to another thread that makes the call
/// for it (macOS forwards some calls to the main thread; see `native::main_thread`).
#[cfg(target_os = "macos")]
pub(super) struct Attribution(Current);

#[cfg(target_os = "macos")]
impl Attribution {
    pub(super) fn of_this_thread() -> Self {
        Self(current())
    }

    /// Attribute faults on this thread to the carried call until the result is dropped. The
    /// caller must keep its own `ForeignCall` alive until then (it waits for the result).
    pub(super) fn enter(self) -> ForeignCall {
        let previous = current();
        publish(self.0);
        ForeignCall {
            previous,
        }
    }
}

impl Drop for ForeignCall {
    fn drop(&mut self) {
        publish(self.previous);
    }
}

/// A fixed-size message buffer: the report must not allocate.
struct Report<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> Report<N> {
    const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    fn text(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).unwrap_or("a fault")
    }
}

/// Bytes for the name of the fault (`describe`), which the handler keeps on its stack.
const WHAT_BYTES: usize = 160;

/// The report itself lives in a static buffer, not on the stack: the handler runs on the
/// thread's alternate signal stack, which Rust makes `SIGSTKSZ` bytes (8 KiB on x86-64
/// Linux), and overflowing it there kills the process before anything is printed.
struct ReportBuffer(std::cell::UnsafeCell<Report<8192>>);

// SAFETY: only the thread that sets `REPORTING` first touches the buffer.
unsafe impl Sync for ReportBuffer {
}

static REPORT: ReportBuffer = ReportBuffer(std::cell::UnsafeCell::new(Report::new()));

/// Set by the first thread to report a crash; another thread faulting meanwhile waits for
/// the process to exit instead of writing into the same buffer.
static REPORTING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl<const N: usize> std::fmt::Write for Report<N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let room = self.bytes.len() - self.len;
        let n = s.len().min(room);
        self.bytes[self.len..self.len + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n;
        Ok(())
    }
}

/// `path` relative to the directory jaic was started in, when it is inside it.
fn shown_path(path: &str) -> &str {
    let Some(base) = crate::display_base().and_then(|b| b.to_str()) else {
        return path;
    };
    match path.strip_prefix(base) {
        Some(rest) if rest.starts_with(['/', '\\']) && rest.len() > 1 => &rest[1..],
        _ => path,
    }
}

/// The report for a fault while `current` was in progress: `what` names the fault.
fn write_report(current: Current, what: &str, out: &mut Report<8192>) {
    use std::fmt::Write;
    // SAFETY: `current` is only published while its `ForeignCall` (and so the symbol, the
    // thread's state and the program it points at) is alive, and the fault interrupted that
    // call on this thread.
    let (symbol, state, program) = unsafe {
        let bytes = std::slice::from_raw_parts(current.symbol.0, current.symbol.1);
        (
            std::str::from_utf8(bytes).unwrap_or("?"),
            &*current.state,
            &*current.program,
        )
    };
    let at = |loc: Option<(u32, u32, u32)>| {
        loc.and_then(|(file, line, col)| {
            let path = program.file_paths.get(file as usize)?;
            Some((shown_path(path), line, col))
        })
    };
    if let Some((path, line, col)) = at(state.loc) {
        let _ = write!(out, "{path}:{line}:{col}: ");
    }
    let _ = writeln!(
        out,
        "error: native code crashed ({what}) while calling foreign procedure `{symbol}`"
    );
    // Frame k runs `calls[k].0`; it is at the line it called frame k + 1 from, and the
    // innermost frame is at the foreign call.
    let calls = &state.calls;
    let mut shown = 0;
    for k in (0..calls.len()).rev() {
        let name = program
            .func_names
            .get(calls[k].0.0 as usize)
            .map_or("", String::as_str);
        if name.is_empty() || name.starts_with("__") {
            continue;
        }
        if shown == 0 {
            let _ = writeln!(out, "note: call stack (innermost first):");
        }
        if shown == MAX_SHOWN_FRAMES {
            let _ = writeln!(out, "    ...");
            break;
        }
        let loc = calls.get(k + 1).map_or(state.loc, |c| c.1);
        // A polymorphic instance `name#N` is `name` to the user; `#run` stays as it is.
        let name = match name.split_once('#') {
            Some((base, _)) if !base.is_empty() => base,
            _ => name,
        };
        match at(loc) {
            Some((path, line, _)) => {
                let _ = writeln!(out, "    `{name}` at {path}:{line}");
            }
            None => {
                let _ = writeln!(out, "    `{name}`");
            }
        }
        shown += 1;
    }
    let _ = writeln!(
        out,
        "help: check the arguments passed to `{symbol}` (pointers and sizes) and its `#foreign` declaration against the C signature; jaic cannot continue after a crash in native code"
    );
}

/// Report the fault if a foreign call is in progress; false when it is not ours.
fn report_and_exit(what: &str) -> bool {
    let current = current();
    if current.program.is_null() || current.state.is_null() {
        return false;
    }
    if REPORTING.swap(true, std::sync::atomic::Ordering::Acquire) {
        // Another thread is reporting and exits the process when done: sleep until then
        // instead of spinning on the flag.
        sys::wait_forever();
    }
    // SAFETY: this thread set `REPORTING` first, so no other thread uses the buffer, and it
    // exits without returning.
    let out = unsafe { &mut *REPORT.0.get() };
    write_report(current, what, out);
    sys::write_stderr(&out.bytes[..out.len]);
    sys::exit_now(CRASH_STATUS)
}

fn install() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(sys::install);
}

#[cfg(all(unix, any(target_os = "macos", target_os = "linux")))]
mod sys {
    use std::ffi::{c_int, c_void};

    const SIGILL: c_int = 4;
    const SIGFPE: c_int = 8;
    const SIGSEGV: c_int = 11;

    #[cfg(target_os = "macos")]
    pub(super) const SIGBUS: c_int = 10;

    #[cfg(target_os = "linux")]
    pub(super) const SIGBUS: c_int = 7;

    #[cfg(target_os = "macos")]
    const SA_SIGINFO: c_int = 0x40;

    #[cfg(target_os = "macos")]
    const SA_ONSTACK: c_int = 0x1;

    #[cfg(target_os = "linux")]
    const SA_SIGINFO: c_int = 4;

    #[cfg(target_os = "linux")]
    const SA_ONSTACK: c_int = 0x0800_0000;

    /// Offset of `si_addr` in `siginfo_t`.
    #[cfg(target_os = "macos")]
    const SI_ADDR: usize = 24;

    #[cfg(target_os = "linux")]
    const SI_ADDR: usize = 16;

    #[cfg(target_os = "macos")]
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SigAction {
        handler: usize,
        mask: u32,
        flags: c_int,
    }

    #[cfg(target_os = "linux")]
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SigAction {
        handler: usize,
        mask: [u64; 16],
        flags: c_int,
        restorer: usize,
    }

    unsafe extern "C" {
        fn sigaction(sig: c_int, act: *const SigAction, old: *mut SigAction) -> c_int;
        fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
        fn _exit(status: c_int) -> !;
        fn pause() -> c_int;
    }

    const SIGNALS: [c_int; 4] = [SIGSEGV, SIGBUS, SIGILL, SIGFPE];

    /// The actions in place before ours, by index in `SIGNALS`.
    static mut PREVIOUS: [Option<SigAction>; 4] = [None; 4];

    pub fn install() {
        let handler: extern "C" fn(c_int, *mut c_void, *mut c_void) = on_signal;
        for (i, &sig) in SIGNALS.iter().enumerate() {
            // SAFETY: plain C calls on properly laid out structs; `PREVIOUS` is written once,
            // before the handler that reads it is in place.
            unsafe {
                let mut action: SigAction = std::mem::zeroed();
                action.handler = handler as usize;
                // The fault may be a native stack overflow: run on the alternate stack Rust
                // sets up for its threads.
                action.flags = SA_SIGINFO | SA_ONSTACK;
                let mut old: SigAction = std::mem::zeroed();
                if sigaction(sig, std::ptr::null(), &raw mut old) == 0 {
                    (*std::ptr::addr_of_mut!(PREVIOUS))[i] = Some(old);
                    sigaction(sig, &raw const action, std::ptr::null_mut());
                }
            }
        }
    }

    extern "C" fn on_signal(sig: c_int, info: *mut c_void, _context: *mut c_void) {
        let address = if info.is_null() {
            0
        } else {
            // SAFETY: the kernel passes a `siginfo_t`; `si_addr` is at `SI_ADDR`.
            unsafe { std::ptr::read_unaligned(info.cast::<u8>().add(SI_ADDR).cast::<usize>()) }
        };

        let mut text = super::Report::<{ super::WHAT_BYTES }>::new();

        let what = describe(sig, address, &mut text);

        if super::report_and_exit(what) {
            return;
        }

        // Not during a foreign call: put the previous action back and let the fault repeat
        // under it.
        if let Some(i) = SIGNALS.iter().position(|&s| s == sig) {
            // SAFETY: as in `install`; `PREVIOUS` is no longer written.
            unsafe {
                if let Some(old) = (*std::ptr::addr_of!(PREVIOUS))[i] {
                    sigaction(sig, &raw const old, std::ptr::null_mut())
                } else {
                    let default: SigAction = std::mem::zeroed();
                    sigaction(sig, &raw const default, std::ptr::null_mut())
                };
            }
        }
    }

    pub(super) fn describe(
        sig: c_int,
        address: usize,
        out: &mut super::Report<{ super::WHAT_BYTES }>,
    ) -> &str {
        use std::fmt::Write;
        let _ = match sig {
            SIGSEGV => write!(
                out,
                "SIGSEGV, invalid memory access at address {address:#x}"
            ),
            SIGBUS => write!(out, "SIGBUS, bad memory access at address {address:#x}"),
            SIGILL => write!(out, "SIGILL, illegal instruction at {address:#x}"),
            SIGFPE => write!(
                out,
                "SIGFPE, arithmetic fault such as an integer division by zero"
            ),
            _ => write!(out, "signal {sig}"),
        };
        out.text()
    }

    pub fn write_stderr(mut bytes: &[u8]) {
        while !bytes.is_empty() {
            // SAFETY: writes the initialized bytes of a live slice.
            let n = unsafe { write(2, bytes.as_ptr().cast(), bytes.len()) };
            if n <= 0 {
                return;
            }
            bytes = &bytes[n as usize..];
        }
    }

    pub fn exit_now(status: i32) -> ! {
        // SAFETY: `_exit` is async-signal-safe and ends the process.
        unsafe { _exit(status) }
    }

    /// Block this thread until the process ends.
    pub fn wait_forever() -> ! {
        loop {
            // SAFETY: `pause` is async-signal-safe and only suspends the calling thread.
            unsafe {
                pause();
            }
        }
    }
}

#[cfg(windows)]
mod sys {
    use std::ffi::c_void;

    const EXCEPTION_ACCESS_VIOLATION: u32 = 0xC000_0005;
    const EXCEPTION_IN_PAGE_ERROR: u32 = 0xC000_0006;
    const EXCEPTION_ILLEGAL_INSTRUCTION: u32 = 0xC000_001D;
    const EXCEPTION_INT_DIVIDE_BY_ZERO: u32 = 0xC000_0094;
    const EXCEPTION_PRIV_INSTRUCTION: u32 = 0xC000_0096;
    const EXCEPTION_STACK_OVERFLOW: u32 = 0xC000_00FD;
    const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
    const STD_ERROR_HANDLE: u32 = 0xFFFF_FFF4;

    #[repr(C)]
    struct ExceptionRecord {
        code: u32,
        flags: u32,
        record: *const ExceptionRecord,
        address: *const c_void,
        parameter_count: u32,
        information: [usize; 15],
    }

    #[repr(C)]
    struct ExceptionPointers {
        record: *const ExceptionRecord,
        context: *mut c_void,
    }

    type Handler = unsafe extern "system" fn(*mut ExceptionPointers) -> i32;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn AddVectoredExceptionHandler(first: u32, handler: Handler) -> *mut c_void;
        fn GetStdHandle(which: u32) -> *mut c_void;

        fn WriteFile(
            file: *mut c_void,
            buffer: *const c_void,
            length: u32,
            written: *mut u32,
            overlapped: *mut c_void,
        ) -> i32;

        fn GetCurrentProcess() -> *mut c_void;
        fn TerminateProcess(process: *mut c_void, code: u32) -> i32;
        fn Sleep(milliseconds: u32);
    }

    pub fn install() {
        // SAFETY: registers a handler with the documented signature.
        unsafe {
            AddVectoredExceptionHandler(1, on_exception);
        }
    }

    unsafe extern "system" fn on_exception(pointers: *mut ExceptionPointers) -> i32 {
        // SAFETY: Windows passes valid exception pointers.
        let record = unsafe { &*(*pointers).record };

        let address = record.address as usize;

        let target = if record.parameter_count >= 2 {
            record.information[1]
        } else {
            0
        };

        let mut text = super::Report::<{ super::WHAT_BYTES }>::new();

        let what = {
            use std::fmt::Write;
            let _ = match record.code {
                EXCEPTION_ACCESS_VIOLATION | EXCEPTION_IN_PAGE_ERROR => {
                    write!(text, "access violation at address {target:#x}")
                }
                EXCEPTION_ILLEGAL_INSTRUCTION | EXCEPTION_PRIV_INSTRUCTION => {
                    write!(text, "illegal instruction at {address:#x}")
                }
                EXCEPTION_INT_DIVIDE_BY_ZERO => write!(text, "integer division by zero"),
                EXCEPTION_STACK_OVERFLOW => write!(text, "stack overflow"),
                _ => return EXCEPTION_CONTINUE_SEARCH,
            };
            text.text()
        };

        super::report_and_exit(what);
        EXCEPTION_CONTINUE_SEARCH
    }

    pub fn write_stderr(bytes: &[u8]) {
        let mut written = 0u32;
        // SAFETY: writes the initialized bytes of a live slice to the standard error handle.
        unsafe {
            WriteFile(
                GetStdHandle(STD_ERROR_HANDLE),
                bytes.as_ptr().cast(),
                bytes.len() as u32,
                &raw mut written,
                std::ptr::null_mut(),
            );
        }
    }

    pub fn exit_now(status: i32) -> ! {
        // SAFETY: ends this process.
        unsafe {
            TerminateProcess(GetCurrentProcess(), status as u32);
        }
        std::process::abort()
    }

    /// Block this thread until the process ends.
    pub fn wait_forever() -> ! {
        loop {
            // SAFETY: suspends the calling thread (0xFFFFFFFF is INFINITE).
            unsafe { Sleep(u32::MAX) }
        }
    }
}

#[cfg(not(any(windows, all(unix, any(target_os = "macos", target_os = "linux")))))]
mod sys {
    pub fn install() {
    }

    pub fn write_stderr(_bytes: &[u8]) {
    }

    pub fn exit_now(status: i32) -> ! {
        std::process::exit(status)
    }

    pub fn wait_forever() -> ! {
        loop {
            std::thread::park();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::FuncId;

    fn program() -> Program {
        Program {
            file_paths: vec!["/work/main.jai".into(), "/elsewhere/lib.jai".into()],
            func_names: vec![
                "main".into(),
                "helper#3".into(),
                "__hidden".into(),
                String::new(),
                "#run".into(),
                "inner".into(),
            ],
            ..Default::default()
        }
    }

    fn report_for(state: &ExecState, program: &Program, symbol: &str) -> String {
        let current = Current {
            symbol: (symbol.as_ptr(), symbol.len()),
            state,
            program,
        };
        let mut out = Report::<8192>::new();
        write_report(current, "SIGSEGV, test", &mut out);
        out.text().to_string()
    }

    #[test]
    fn the_report_names_the_call_the_line_and_the_visible_frames() {
        let program = program();
        let state = ExecState {
            // Each frame is shown at the line it called the next one from; the innermost, `inner`,
            // at the foreign call. The hidden and unnamed procedures are left out.
            calls: vec![
                (FuncId(0), Some((0, 3, 1))),
                (FuncId(1), Some((0, 4, 2))),
                (FuncId(2), Some((0, 5, 1))),
                (FuncId(3), None),
                (FuncId(4), Some((1, 9, 1))),
                (FuncId(5), Some((1, 12, 3))),
            ],
            loc: Some((1, 12, 7)),
            ..Default::default()
        };
        let text = report_for(&state, &program, "strlen");
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[0].ends_with(
                ":12:7: error: native code crashed (SIGSEGV, test) while calling foreign procedure `strlen`"
            ),
            "{text}"
        );
        assert_eq!(lines[1], "note: call stack (innermost first):");
        assert_eq!(lines[2], "    `inner` at /elsewhere/lib.jai:12");
        assert_eq!(lines[3], "    `#run` at /elsewhere/lib.jai:12");
        assert_eq!(lines[4], "    `helper` at /work/main.jai:5");
        assert_eq!(lines[5], "    `main` at /work/main.jai:4");
        assert!(lines[6].starts_with("help: check the arguments passed to `strlen`"));
        assert_eq!(lines.len(), 7, "{text}");
    }

    #[test]
    fn a_frame_without_a_known_file_is_shown_by_name_only() {
        let mut program = program();
        program.file_paths.clear();
        let state = ExecState {
            calls: vec![(FuncId(0), Some((0, 1, 1)))],
            loc: Some((0, 1, 1)),
            ..Default::default()
        };
        let text = report_for(&state, &program, "puts");
        assert!(text.starts_with("error: native code crashed"), "{text}");
        assert!(text.contains("\n    `main`\n"), "{text}");
    }

    #[test]
    fn deep_call_stacks_are_cut_after_the_shown_limit() {
        let mut program = program();
        program.func_names = vec!["recur".into()];
        let state = ExecState {
            calls: vec![(FuncId(0), Some((0, 2, 1))); MAX_SHOWN_FRAMES + 5],
            loc: Some((0, 2, 1)),
            ..Default::default()
        };
        let text = report_for(&state, &program, "memcpy");
        assert_eq!(text.matches("    `recur` at").count(), MAX_SHOWN_FRAMES);
        assert!(text.contains("\n    ...\n"), "{text}");
    }

    #[test]
    fn a_stack_of_hidden_procedures_prints_no_call_stack_note() {
        let program = program();
        let state = ExecState {
            calls: vec![(FuncId(2), None), (FuncId(3), None)],
            ..Default::default()
        };
        let text = report_for(&state, &program, "abort");
        assert!(!text.contains("call stack"), "{text}");
    }

    #[test]
    fn reports_stop_at_the_buffer_end_instead_of_failing() {
        use std::fmt::Write;
        let mut small = Report::<4>::new();
        write!(small, "abcdef").unwrap();
        assert_eq!(small.text(), "abcd");
        write!(small, "more").unwrap();
        assert_eq!(small.text(), "abcd");
        let mut broken = Report::<2>::new();
        broken.bytes = [0xff, 0xfe];
        broken.len = 2;
        assert_eq!(broken.text(), "a fault");
    }

    #[test]
    fn paths_are_shown_relative_only_inside_the_display_base() {
        // Nothing sets the base in this process unless another test did: either way an
        // unrelated path is left alone.
        assert_eq!(shown_path("/nowhere/at/all.jai"), "/nowhere/at/all.jai");
        if let Some(base) = crate::display_base().and_then(|b| b.to_str()) {
            let inside = format!("{base}/inside.jai");
            assert_eq!(shown_path(&inside), "inside.jai");
            assert_eq!(shown_path(base), base);
        }
    }

    #[test]
    fn foreign_calls_nest_and_restore_the_outer_call() {
        let program = program();
        let state = ExecState::default();
        assert!(current().program.is_null());
        {
            let _outer = ForeignCall::enter("outer", &state, &program);
            assert_eq!(current().symbol.1, "outer".len());
            {
                let _inner = ForeignCall::enter("nested", &state, &program);
                assert_eq!(current().symbol.1, "nested".len());
            }
            assert_eq!(current().symbol.1, "outer".len());
        }
        assert!(current().program.is_null());
        // Outside a call a fault is not ours.
        assert!(!report_and_exit("SIGSEGV"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn a_call_carried_to_another_thread_is_attributed_there() {
        let program = program();
        let state = ExecState::default();
        let _call = ForeignCall::enter("carried", &state, &program);
        let carried = Attribution::of_this_thread();
        let program_address = &raw const program as usize;
        struct Moved(Attribution);
        // SAFETY: the carried call only names data that outlives the scope below.
        unsafe impl Send for Moved {
        }
        let moved = Moved(carried);
        std::thread::scope(|scope| {
            scope.spawn(move || {
                let moved = moved;
                assert!(current().program.is_null());
                let call = moved.0.enter();
                assert_eq!(current().symbol.1, "carried".len());
                assert_eq!(current().program as usize, program_address);
                drop(call);
                assert!(current().program.is_null());
            });
        });
    }

    #[cfg(all(unix, any(target_os = "macos", target_os = "linux")))]
    #[test]
    fn faults_are_named_by_signal() {
        let name = |sig: i32, address: usize| {
            let mut text = Report::<WHAT_BYTES>::new();
            sys::describe(sig, address, &mut text).to_string()
        };
        assert_eq!(
            name(11, 0x10),
            "SIGSEGV, invalid memory access at address 0x10"
        );
        assert_eq!(name(4, 0x20), "SIGILL, illegal instruction at 0x20");
        assert_eq!(
            name(8, 0),
            "SIGFPE, arithmetic fault such as an integer division by zero"
        );
        assert!(name(sys::SIGBUS, 0x30).starts_with("SIGBUS, bad memory access at address 0x30"));
        assert_eq!(name(99, 0), "signal 99");
    }
}
