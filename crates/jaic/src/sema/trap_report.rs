//! Turning an interpreter `Trap` into a diagnostic: which check failed, the line of the
//! program's own code that led there, and the call stack, with standard-library frames kept
//! out of the way. See `docs/compiler/diagnostics.md`.
use super::Compiler;
use crate::interp::{Trap, TrapFrame};
use crate::source::{Diagnostic, FileId, Span};
use std::path::{Path, PathBuf};

/// The procedures between a failed `assert` and its report: left out of the call stack.
const ASSERTION_FRAMES: &[&str] = &[
    "runtime_support_report_assertion",
    "runtime_support_assertion_failed",
    "assert_helper",
];

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
    pub(crate) fn trap_diagnostic(
        &self,
        trap: &Trap,
        prefix: &str,
        site: Option<Span>,
    ) -> Diagnostic {
        let assertion = trap.assertion.is_some();
        let frames: Vec<&TrapFrame> = trap
            .frames
            .iter()
            .skip_while(|f| assertion && ASSERTION_FRAMES.contains(&f.name.as_str()))
            .collect();
        let library_roots = self.library_roots();
        let is_library = |loc: Option<(u32, u32, u32)>| {
            loc.is_some_and(|(file, ..)| self.is_library_file(FileId(file), &library_roots))
        };
        // The asserted condition, read from the `assert(...)` call.
        let condition = frames
            .first()
            .filter(|_| assertion)
            .and_then(|f| f.loc)
            .and_then(|loc| self.assert_condition(loc));
        let message = if trap.message == "assertion failed"
            && let Some(condition) = &condition
        {
            format!("assertion failed: `{condition}` is false")
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
        // A metaprogram's own report: its message, at the place it named when it named one;
        // the line that reported it becomes a note.
        if let Some((path, line, col)) = trap.reported.as_deref() {
            let named = (!path.is_empty())
                .then(|| self.file_by_path(path))
                .flatten()
                .map(|file| self.statement_span((file.0, *line, (*col).max(1))));
            // The line of the program's own code that made the report (not the stdlib's).
            let reporter = frames
                .iter()
                .find(|f| f.loc.is_some() && !is_library(f.loc))
                .and_then(|f| f.loc)
                .map(|loc| self.statement_span(loc))
                .or(site)
                .unwrap_or(span);
            let primary = named.unwrap_or(reporter);
            let mut d = Diagnostic::error(primary, message);
            if named.is_some() && reporter != primary && Some(reporter) != site {
                d = d.with_note(reporter, "reported by the metaprogram here");
            } else if named.is_none() && !path.is_empty() {
                d = d.with_note(Span::NONE, format!("reported for {path}:{line}:{col}"));
            }
            // (Not when the report is already inside the directive's own line.)
            if let Some(site) = site
                && !(site.file == primary.file
                    && site.start <= primary.start
                    && primary.end <= site.end)
            {
                d = d.with_note(site, "while running compile-time code started here");
            }
            return d;
        }
        let mut d = Diagnostic::error(span, format!("{prefix}: {message}"));
        if let Some(condition) = condition.filter(|_| trap.message != "assertion failed") {
            d = d.with_label(format!("`{condition}` is false"));
        }
        // The failure was inside library code the user's line called: name the procedure the
        // user called and where inside it things went wrong.
        if let Some(i) = primary.filter(|&i| i > 0) {
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
                    "{} inside {}{}{within}",
                    if assertion {
                        "an assertion failed"
                    } else {
                        "this call failed"
                    },
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
        if visible.len() > 1 {
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
                    if repeats == 1 {
                        ""
                    } else {
                        "s"
                    }
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

    /// The condition of the `assert(...)` call at `loc`, when that is what the line holds.
    fn assert_condition(&self, (file, line, col): (u32, u32, u32)) -> Option<String> {
        if file as usize >= self.sources.len() {
            return None;
        }
        let text = self.sources.get(FileId(file)).line_text(line);
        let call = text.get(col.saturating_sub(1) as usize..)?.trim_start();
        let args = call
            .strip_prefix("assert")?
            .trim_start()
            .strip_prefix('(')?;
        // Up to the first comma or closing parenthesis outside brackets and strings.
        let mut depth = 0i32;
        let mut quoted = false;
        let mut previous = ' ';
        for (i, c) in args.char_indices() {
            match c {
                '"' if previous != '\\' => quoted = !quoted,
                _ if quoted => {}
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' if depth > 0 => depth -= 1,
                ')' | ',' if depth == 0 => {
                    let condition = args[..i].trim();
                    return (!condition.is_empty()).then(|| condition.to_string());
                }
                _ => {}
            }
            previous = c;
        }
        None
    }

    /// ` at path:line` for a frame location, or nothing.
    fn loc_suffix(&self, loc: Option<(u32, u32, u32)>) -> String {
        match loc {
            Some((file, line, _)) if (file as usize) < self.sources.len() => {
                let path = Path::new(&self.sources.get(FileId(file)).path);
                format!(" at {}:{line}", crate::display_path(path))
            }
            _ => String::new(),
        }
    }

    /// The loaded file at `path`, compared as given and as an absolute path.
    fn file_by_path(&self, path: &str) -> Option<FileId> {
        let wanted = std::path::absolute(path).ok();
        (0..self.sources.len() as u32).map(FileId).find(|&id| {
            let have = &self.sources.get(id).path;
            have == path || wanted.as_deref().is_some_and(|w| Path::new(have) == w)
        })
    }

    /// The statement at `loc`: from its column to the end of that line (without a trailing
    /// `;` or whitespace), so the carets cover what ran.
    fn statement_span(&self, (file, line, col): (u32, u32, u32)) -> Span {
        if file as usize >= self.sources.len() {
            return Span::NONE;
        }
        let source = self.sources.get(FileId(file));
        let start = source.offset_of(line, col) as usize;
        let text = source.line_text(line);
        let rest = text.get(col.saturating_sub(1) as usize..).unwrap_or("");
        let len = rest
            .trim_end()
            .trim_end_matches(';')
            .trim_end()
            .len()
            .max(1);
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
    // A polymorphic instance is `name#N`; the user knows it as `name`.
    let name = match name.rsplit_once('#') {
        Some((base, n)) if !base.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => base,
        _ => name,
    };
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
fn help_for(message: &str) -> Option<String> {
    let fixed = if message.starts_with("array bounds check failed") {
        "valid indices are 0 up to the array's count minus one; check the index or the array's length first"
    } else if message.starts_with("null pointer dereference") {
        "check the pointer against null before using it, or make sure it is set"
    } else if message.starts_with("stack overflow") {
        "check that the recursion has a base case it reaches"
    } else if message == MISSING_RETURN {
        "every path through a procedure with results must end in `return`"
    } else if message.starts_with("cast of ") {
        return cast_help(message);
    } else if message.starts_with("no case of the `#complete` switch") {
        "the value is not one of the enum's members (was it cast from an integer, or left uninitialized?); add a `case;` to handle other values"
    } else if message == "integer division by zero" {
        "check the divisor against zero first"
    } else {
        return None;
    };
    Some(fixed.to_string())
}

/// Help for "cast of V to `T` overflows": the range `T` holds and the casts that do not check.
fn cast_help(message: &str) -> Option<String> {
    let target = message.split('`').nth(1)?;
    let bits: u32 = target.get(1..)?.parse().ok()?;
    let range = match (target.as_bytes().first()?, bits) {
        (b'u', 1..=64) => format!("0 to {}", u64::MAX >> (64 - bits)),
        (b's', 1..=64) => format!("{} to {}", i64::MIN >> (64 - bits), i64::MAX >> (64 - bits)),
        _ => return None,
    };
    Some(format!(
        "`{target}` holds {range}; write `cast,trunc({target})` (or `xx,trunc`) to keep the low bits, or `cast,no_check({target})` to skip the check"
    ))
}
