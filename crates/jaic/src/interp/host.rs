//! The interface an embedder supplies to the interpreter, and the host that runs on the real machine.
use super::globals::ZeroedBlock;
use crate::ir::{self};

/// Compiler-host operations for metaprograms and explicit private runtime transports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    PublicCompiler(usize),
    UnsupportedCompiler(ir::FuncId),
    /// `write_string(s: string, to_standard_error: bool)`
    WriteString,
    /// `write_strings(strings: ..string, to_standard_error: bool)`
    WriteStrings,
    DebugBreak,
    /// `() -> s64`: nanoseconds on the compiler host's monotonic clock. Compile-time code runs on
    /// the host whatever target is selected, so the clock identifiers of the target's C library
    /// (`clock_gettime`'s differ between Linux and macOS) cannot be passed to the host's.
    HostMonotonicNanos,
    /// A runtime-opted-in assertion callback: a failed
    /// `assert`, reported by the compiler as a runtime error (`Trap::assertion`) rather than
    /// printed by the runtime. `context` says whether it takes a context pointer.
    AssertionFailed {
        context: bool,
        location: LocationLayout,
    },
    /// A retained private syntax metadata query; the flag says whether the
    /// procedure takes a context pointer.
    Meta(crate::build::MetaOp, bool),
}

impl Hook {
    pub(super) fn active(self, compile_time: bool) -> bool {
        compile_time
            || !matches!(
                self,
                Self::PublicCompiler(_) | Self::UnsupportedCompiler(_) | Self::Meta(..)
            )
    }

    /// The hook behind the `#compiler` procedure `name`, if the interpreter provides one.
    /// `takes_context` says whether the procedure takes the implicit context pointer.
    #[must_use]
    pub fn for_compiler_proc(name: &str, takes_context: bool) -> Option<Self> {
        match name {
            "write_string" => Some(Self::WriteString),
            "write_strings" => Some(Self::WriteStrings),
            "compile_time_debug_break" => Some(Self::DebugBreak),
            "compile_time_host_monotonic_nanoseconds" => Some(Self::HostMonotonicNanos),
            "__jaic_syntax_int" => Some(Self::Meta(crate::build::MetaOp::SyntaxInt, takes_context)),
            "__jaic_syntax_string" => Some(Self::Meta(
                crate::build::MetaOp::SyntaxString,
                takes_context,
            )),
            "__jaic_syntax_child" => {
                Some(Self::Meta(crate::build::MetaOp::SyntaxChild, takes_context))
            }
            _ => None,
        }
    }
}

/// Byte offsets of a `Source_Code_Location`'s fields, from the Preload's declaration
/// (`Compiler::location_layout`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationLayout {
    pub path: u64,
    pub line: u64,
    pub column: u64,
}

/// Where program output and platform services go.
pub trait Host {
    fn write(&mut self, bytes: &[u8], to_stderr: bool);

    /// A provider for a fresh isolated compile-time interpreter. Embedders with
    /// a workspace factory use that instead. An unavailable provider is an
    /// explicit unsupported operation, never a fallback to native execution.
    fn isolated_host(&self) -> Option<Box<dyn Host>> {
        None
    }

    /// Implement a foreign procedure without native linking. `None` = not provided.
    fn foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>>;

    /// The address of a foreign variable (a C global such as `stdin`) the host provides. `None` =
    /// not provided.
    fn foreign_data(&mut self, _symbol: &str) -> Option<u64> {
        None
    }

    /// Whether foreign symbols may be resolved through the native dynamic linker.
    fn native_linking(&self) -> bool;

    /// Run threads on the interpreter's own stack, one after another (no OS threads); see
    /// `threads_inline.rs`. The sandbox says yes.
    fn cooperative_threads(&self) -> bool {
        false
    }

    /// Move the virtual clock forward (sleeping). Hosts with a real clock ignore it.
    fn advance_clock(&mut self, _nanoseconds: u64) {
    }

    /// The virtual clock's reading, nanoseconds since the epoch, if the host has one.
    fn virtual_now_ns(&mut self) -> Option<u64> {
        None
    }

    /// Asked when `Interp::block_budget` runs out: a new budget to go on with, or `None` to stop
    /// with "execution budget exhausted". The browser host refills it after the program waited
    /// for the page (an animation frame), so the budget bounds the work between two waits.
    fn refill_budget(&mut self) -> Option<u64> {
        None
    }

    /// The directory relative paths start from, when the host keeps its own (`chdir` in a
    /// sandbox); `None` is the process's.
    fn working_directory(&self) -> Option<String> {
        None
    }
}

/// Writes to the process's stdout/stderr and links natively.
pub struct NativeHost;

/// A real allocation registered by a compile-time global-data API call. The
/// compiler adopts its storage after the active interpreter call returns.
pub struct PendingDataGlobal {
    pub(super) storage: ZeroedBlock,
    pub size: u64,
    pub align: u64,
    pub read_only: bool,
    pub segment_tag: u16,
}

impl PendingDataGlobal {
    pub fn address(&self) -> u64 {
        self.storage.addr()
    }
}

impl Host for NativeHost {
    fn isolated_host(&self) -> Option<Box<dyn Host>> {
        Some(Box::new(NativeHost))
    }

    fn write(&mut self, bytes: &[u8], to_stderr: bool) {
        use std::io::Write;
        crate::progress::before_output();
        if to_stderr {
            let _ = std::io::stderr().write_all(bytes);
        } else {
            let mut out = std::io::stdout();
            let _ = out.write_all(bytes);
            let _ = out.flush();
        }
    }

    fn foreign(
        &mut self,
        _symbol: &str,
        _args: &[u64],
        _sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>> {
        None
    }

    fn native_linking(&self) -> bool {
        // Windows: `interp/native/windows.rs` (x64 and arm64).
        cfg!(any(
            unix,
            all(
                windows,
                any(target_arch = "x86_64", target_arch = "aarch64")
            )
        ))
    }
}
