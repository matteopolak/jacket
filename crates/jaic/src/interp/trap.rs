//! Runtime failures: the trap type, its kinds and the frames recorded as it unwinds.
use super::{Interp, Res};
use crate::ir::{self, FuncId};

/// A runtime failure inside interpreted code.
///
/// Boxed, so a `Res<T>` is a word or two wider than `T` instead of carrying a hundred bytes of
/// failure on every load, store and arithmetic op that merely might fail.
#[derive(Debug, Clone, Default)]
pub struct Trap(Box<TrapDetails>);

impl std::ops::Deref for Trap {
    type Target = TrapDetails;

    fn deref(&self) -> &TrapDetails {
        &self.0
    }
}

impl std::ops::DerefMut for Trap {
    fn deref_mut(&mut self) -> &mut TrapDetails {
        &mut self.0
    }
}

impl Trap {
    /// A failure with `message` and nothing else known yet.
    pub fn new(message: impl Into<String>) -> Self {
        Trap(Box::new(TrapDetails {
            message: message.into(),
            ..TrapDetails::default()
        }))
    }

    /// The message, consuming the trap.
    pub fn into_message(self) -> String {
        self.0.message
    }

    /// The failure at `loc`.
    #[must_use]
    pub fn at(mut self, loc: Option<(u32, u32, u32)>) -> Self {
        self.loc = loc;
        self
    }

    /// The failure as a `kind`.
    #[must_use]
    pub fn of_kind(mut self, kind: Option<TrapKind>) -> Self {
        self.kind = kind;
        self
    }

    /// The failure with its message replaced by `wrap(old message)`.
    #[must_use]
    pub fn map_message(mut self, wrap: impl FnOnce(&str) -> String) -> Self {
        self.message = wrap(&self.message);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct TrapDetails {
    pub message: String,
    /// (file, line, column) of the last executed statement.
    pub loc: Option<(u32, u32, u32)>,
    /// The interpreted procedures the failure unwound through, innermost first: each one's
    /// name and the statement it was executing (for the callee, the call). Filled by `exec` as
    /// the error propagates; capped at `MAX_TRAP_FRAMES`, the rest counted in `omitted_frames`.
    pub frames: Vec<TrapFrame>,
    pub omitted_frames: usize,
    /// A failed `assert`: the location it was given (path, line, column).
    pub assertion: Option<Box<(String, u32, u32)>>,
    /// An error a metaprogram reported itself (`compiler_report`, a failed workspace): the
    /// message is the program's own, shown without the compile-time-execution prefix, at the
    /// location it named (path, line, column; an empty path names none).
    pub reported: Option<Box<(String, u32, u32)>>,
    /// Which kind of failure this is, for the report's notes and help (the message is for
    /// people; nothing parses it).
    pub kind: Option<TrapKind>,
}

/// What failed, for failures the report treats specially.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapKind {
    /// A runtime check: an `ir::CheckReason` reason and its details (`ir::check_message`).
    Check {
        reason: Option<ir::CheckReason>,
        a: u64,
        b: u64,
    },
    /// A load, store, copy or call through a null (or nearly null) pointer.
    NullPointer,
    /// Recursion deeper than the interpreter allows.
    StackOverflow,
    /// A failed `assert` without a message of its own.
    BareAssertion,
    /// A foreign procedure neither the host nor a loaded library provides.
    Unavailable,
    /// Not a failure: the program called `exit` where the sandbox ends it itself instead of
    /// the process. Carries the exit code.
    Exit(i32),
    /// Unwind one managed native-host thread, preserving its Windows exit code.
    ThreadExit(u32),
    /// Not a failure: unwinds a thread of the sandbox's scheduler that blocks or yields, its
    /// frames saved so it can resume later (`threads_inline.rs`). Never reported.
    Suspended,
    /// A declared compile-time callee needs its body compiled by its owning compiler.
    CompileDependency(FuncId),
}

/// What a load or store at an address in the never-mapped first page did (`Interp::null_trap`).
pub(super) fn null_access(kind: &str, addr: u64) -> String {
    if addr == 0 {
        format!("{kind} through a null pointer")
    } else {
        format!("{kind} at address {addr:#x}, just past null (a member of a null struct pointer?)")
    }
}

/// One procedure on the interpreter's call stack when a `Trap` was raised.
#[derive(Debug, Clone)]
pub struct TrapFrame {
    pub name: String,
    pub origin: ir::FuncOrigin,
    /// (file, line, column) of the statement this procedure was executing.
    pub loc: Option<(u32, u32, u32)>,
}

/// How many frames a `Trap` keeps (deep recursion keeps the innermost ones).
const MAX_TRAP_FRAMES: usize = 64;

impl Trap {
    pub(super) fn suspends_frames(&self) -> bool {
        matches!(
            self.kind,
            Some(TrapKind::Suspended | TrapKind::CompileDependency(_))
        )
    }

    /// The trap as text, for a report made where the compiler's sources are out of reach (a
    /// procedure called from C): the message, where it happened, and the procedures it unwound
    /// through. `file_paths` names files by id (`Program::file_paths`); without them a
    /// location is its line and column.
    pub fn describe(&self, file_paths: &[String]) -> String {
        let at = |(file, line, col): (u32, u32, u32)| match file_paths.get(file as usize) {
            Some(path) => format!(
                "{}:{line}:{col}",
                crate::display_path(std::path::Path::new(path))
            ),
            None => format!("line {line}, column {col}"),
        };
        let mut out = self.message.clone();
        if let Some(loc) = self.loc {
            out += &format!(" (at {})", at(loc));
        }
        for frame in &self.frames {
            out += &format!("\n    in `{}`", frame.name);
            if let Some(loc) = frame.loc {
                out += &format!(" at {}", at(loc));
            }
        }
        if self.omitted_frames > 0 {
            out += &format!("\n    ... {} more", self.omitted_frames);
        }
        out
    }

    pub(super) fn push_frame(&mut self, func: &ir::Func, loc: Option<(u32, u32, u32)>) {
        if func.debug_hidden {
            return;
        }
        if self.frames.len() < MAX_TRAP_FRAMES {
            self.frames.push(TrapFrame {
                name: func.name.clone(),
                origin: func.origin,
                loc,
            });
        } else {
            self.omitted_frames += 1;
        }
    }
}

impl Interp {
    #[cold]
    pub(super) fn trap<T>(&self, message: impl Into<String>) -> Res<T> {
        Err(Trap::new(message).at(self.loc))
    }

    pub(super) fn trap_of<T>(&self, kind: TrapKind, message: impl Into<String>) -> Res<T> {
        Err(Trap::new(message).at(self.loc).of_kind(Some(kind)))
    }

    /// A failed runtime check (`ir::CheckReason`), worded by `ir::check_message`.
    pub(super) fn check_trap<T>(
        &self,
        reason: impl Into<Option<ir::CheckReason>>,
        a: u64,
        b: u64,
    ) -> Res<T> {
        let reason = reason.into();
        self.trap_of(
            TrapKind::Check {
                reason,
                a,
                b,
            },
            ir::check_message(reason, a, b),
        )
    }

    /// An access through a null pointer: `detail` says what was done.
    pub(super) fn null_trap<T>(&self, detail: impl std::fmt::Display) -> Res<T> {
        self.trap_of(
            TrapKind::NullPointer,
            format!("null pointer dereference: {detail}"),
        )
    }
}
