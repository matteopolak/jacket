//! Turning an interpreter `Trap` into a diagnostic: which check failed, the line of the
//! program's own code that led there, and the call stack, with standard-library frames kept
//! out of the way. See `docs/compiler/diagnostics.md`.
use super::Compiler;
use crate::interp::{Trap, TrapFrame};
use crate::source::{Diagnostic, FileId, Span};
use std::path::{Path, PathBuf};

/// The runtime procedure that prints a failed `assert` (message and stack trace) before it
/// stops the program with `debug_break`.
const ASSERTION_REPORTER: &str = "runtime_support_assertion_failed";

/// Call stack lines shown before the rest are summarized.
const MAX_STACK_LINES: usize = 12;

/// The interpreter's message for `ir::TRAP_MISSING_RETURN`.
const MISSING_RETURN: &str = "reached the end of a procedure that must return a value";

impl Compiler {
    /// The diagnostic for `trap`. `prefix` says what was running ("runtime error" or "error
    /// during compile-time execution"); `site` is the directive that started compile-time code.
    ///
    /// The primary location is the innermost frame in the program's own code: when a check
    /// fails inside the standard library, the user's call into it is what needs fixing.
    pub(crate) fn trap_diagnostic(&self, trap: &Trap, prefix: &str, site: Option<Span>) -> Diagnostic {
        let frames: Vec<&TrapFrame> = trap.frames.iter().collect();
        let library_roots = self.library_roots();
        let is_library = |loc: Option<(u32, u32, u32)>| {
            loc.is_some_and(|(file, ..)| self.is_library_file(FileId(file), &library_roots))
        };
        // A failed `assert` already printed its message and stack trace; what is left is to
        // say where, not that `debug_break` ran inside the runtime.
        let assertion = trap.message == "debug_break() was called"
            && frames.iter().any(|f| f.name == ASSERTION_REPORTER);
        let message = if assertion {
            "assertion failed (message and stack trace above)".to_string()
        } else if trap.message == MISSING_RETURN
            && let Some(frame) = frames.first()
            && !is_internal_name(&frame.name)
        {
            format!(
                "{} reached the end of its body without returning a value",
                display_name(&frame.name)
            )
        } else {
            trap.message.clone()
        };
        // Innermost user frame with a location; else the innermost located frame.
        let primary = frames
            .iter()
            .position(|f| f.loc.is_some() && !is_library(f.loc))
            .or_else(|| frames.iter().position(|f| f.loc.is_some()));
        let primary_loc = primary.and_then(|i| frames[i].loc).or(trap.loc);
        let span = primary_loc
            .map(|loc| self.statement_span(loc))
            .or(site)
            .unwrap_or(Span::NONE);
        let mut d = Diagnostic::error(span, format!("{prefix}: {message}"));
        // The failure was inside library code the user's line called: name the procedure the
        // user called and where inside it things went wrong.
        if let Some(i) = primary.filter(|&i| i > 0)
            && !assertion
        {
            let innermost = frames[0];
            let called = frames[..i]
                .iter()
                .rev()
                .find(|f| !is_internal_name(&f.name))
                .copied()
                .unwrap_or(innermost);
            let within = if std::ptr::eq(called, innermost) || is_internal_name(&innermost.name) {
                String::new()
            } else {
                format!(" in {}", display_name(&innermost.name))
            };
            d = d.with_note(
                Span::NONE,
                format!(
                    "this call failed inside {}{}{within}",
                    display_name(&called.name),
                    self.loc_suffix(innermost.loc),
                ),
            );
        }
        if let Some(site) = site
            && site != span
        {
            d = d.with_note(site, "while running compile-time code started here");
        }
        // The runtime's own entry procedures and other compiler-made code are left out.
        let visible: Vec<&TrapFrame> = frames
            .iter()
            .copied()
            .filter(|f| !is_internal_name(&f.name))
            .collect();
        if !assertion && visible.len() > 1 {
            d = d.with_note(Span::NONE, self.call_stack(&visible, trap.omitted_frames));
        }
        if let Some(help) = help_for(&trap.message) {
            d = d.with_help(help);
        }
        d
    }

    /// `call stack (innermost first):` with one line per frame; runs of one procedure calling
    /// itself from the same line are folded.
    fn call_stack(&self, frames: &[&TrapFrame], mut omitted: usize) -> String {
        let mut text = String::from("call stack (innermost first):");
        let mut i = 0;
        let mut shown = 0;
        while i < frames.len() {
            if shown == MAX_STACK_LINES {
                omitted += frames.len() - i;
                break;
            }
            let frame = frames[i];
            let repeats = frames[i + 1..]
                .iter()
                .take_while(|f| f.name == frame.name && f.loc == frame.loc)
                .count();
            text += &format!(
                "\n    {}{}",
                display_name(&frame.name),
                self.loc_suffix(frame.loc)
            );
            if repeats > 0 {
                text += &format!(
                    "\n    ... the same call {repeats} more time{}",
                    if repeats == 1 { "" } else { "s" }
                );
            }
            shown += 1;
            i += repeats + 1;
        }
        if omitted > 0 {
            text += &format!("\n    ... {omitted} more frames");
        }
        text
    }

    /// ` at path:line` for a frame location, or nothing.
    fn loc_suffix(&self, loc: Option<(u32, u32, u32)>) -> String {
        match loc {
            Some((file, line, _)) if (file as usize) < self.sources.len() => {
                format!(" at {}:{line}", self.sources.get(FileId(file)).path)
            }
            _ => String::new(),
        }
    }

    /// The statement at `loc`: from its column to the end of that line (without a trailing
    /// `;` or comment-free whitespace), so the carets cover what ran.
    fn statement_span(&self, (file, line, col): (u32, u32, u32)) -> Span {
        if file as usize >= self.sources.len() {
            return Span::NONE;
        }
        let source = self.sources.get(FileId(file));
        let start = source.offset_of(line, col) as usize;
        let text = source.line_text(line);
        let rest = text.get(col.saturating_sub(1) as usize..).unwrap_or("");
        let len = rest.trim_end().trim_end_matches(';').trim_end().len().max(1);
        Span::new(FileId(file), start, start + len)
    }

    /// The directories of the standard library it loads.
    fn library_roots(&self) -> Vec<PathBuf> {
        let Some(stdlib) = self.options.preload.as_deref().and_then(Path::parent) else {
            return Vec::new();
        };
        let mut roots = vec![stdlib.to_path_buf(), stdlib.join("prelude")];
        roots.extend(
            roots
                .clone()
                .iter()
                .filter_map(|r| std::fs::canonicalize(r).ok()),
        );
        roots
    }

    fn is_library_file(&self, file: FileId, roots: &[PathBuf]) -> bool {
        if file.0 as usize >= self.sources.len() {
            return true;
        }
        let path = Path::new(&self.sources.get(file).path);
        let canonical = std::fs::canonicalize(path).ok();
        roots.iter().any(|root| {
            path.starts_with(root) || canonical.as_ref().is_some_and(|c| c.starts_with(root))
        })
    }
}

/// Compiler-made procedures (`__jaic_*`, `__init_*`...) have no name the user wrote.
fn is_internal_name(name: &str) -> bool {
    name.is_empty() || name.starts_with("__")
}

fn display_name(name: &str) -> String {
    if name == "#run" {
        "the `#run` code".to_string()
    } else if name == "#const" {
        "a constant's compile-time initializer".to_string()
    } else if name.is_empty() {
        "an anonymous procedure".to_string()
    } else if let Some(ty) = name.strip_prefix("__init_") {
        format!("the initializer of `{ty}`")
    } else if name.starts_with("__") {
        "compiler-generated code".to_string()
    } else {
        format!("`{name}`")
    }
}

/// A `help:` line for failures whose fix is clear.
fn help_for(message: &str) -> Option<&'static str> {
    if message.starts_with("array bounds check failed") {
        Some("valid indices are 0 up to the array's count minus one; check the index or the array's length first")
    } else if message.starts_with("null pointer dereference") {
        Some("check the pointer against null before using it, or make sure it is set")
    } else if message.starts_with("stack overflow") {
        Some("check that the recursion has a base case it reaches")
    } else if message == MISSING_RETURN {
        Some("every path through a procedure with results must end in `return`")
    } else {
        None
    }
}
