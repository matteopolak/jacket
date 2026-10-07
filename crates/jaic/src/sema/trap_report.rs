//! Turning an interpreter `Trap` into a diagnostic: which check failed, the line of the
//! program's own code that led there, and the call stack, with standard-library frames kept
//! out of the way. See `docs/compiler/diagnostics.md`.
use super::Compiler;
use crate::interp::{Trap, TrapFrame, TrapKind};
use crate::ir;
use crate::source::{Diagnostic, DiagnosticKind, FileId, Span};
use std::path::{Path, PathBuf};

/// Call stack lines shown before the rest are summarized.
const MAX_STACK_LINES: usize = 12;

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
        // A failed `assert`: the frames inside the assertion machinery (`assert_helper`, the
        // context's handler, Runtime_Support's report) are left out, down to the frame running
        // the line the assertion names.
        let asserting = trap.assertion.as_deref().and_then(|(path, line, _)| {
            let file = self.file_by_path(path)?;
            trap.frames
                .iter()
                .position(|f| f.loc.is_some_and(|(id, l, _)| id == file.0 && l == *line))
        });
        let frames: Vec<&TrapFrame> = trap.frames.iter().skip(asserting.unwrap_or(0)).collect();
        let library_roots = self.library_roots();
        let is_library = |loc: Option<(u32, u32, u32)>| {
            loc.is_some_and(|(file, ..)| self.is_library_file(FileId(file), &library_roots))
        };
        // The asserted condition: the first argument of the `assert(...)` call.
        let condition = trap
            .assertion
            .as_deref()
            .and_then(|(path, line, col)| self.assert_condition(path, *line, *col));
        let bare_assertion = trap.kind == Some(TrapKind::BareAssertion);
        let message = if bare_assertion && let Some(condition) = &condition {
            format!("assertion failed: `{condition}` is false")
        } else if matches!(
            trap.kind,
            Some(TrapKind::Check {
                reason: ir::TRAP_MISSING_RETURN,
                ..
            })
        ) && let Some(frame) = frames.first()
            && !is_internal_name(&frame.name)
        {
            format!(
                "{} reached the end of its body without returning a value",
                display_name(frame)
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
        if let Some(condition) = condition.filter(|_| !bare_assertion) {
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
                format!(" in {}", display_name(innermost))
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
                    display_name(called),
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
        if let Some(help) = trap.kind.and_then(help_for) {
            d = d.with_help(help);
        }
        if trap.kind == Some(TrapKind::Unavailable) {
            d = d.with_kind(DiagnosticKind::Unavailable);
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
                display_name(frame),
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

    /// The condition of the `assert(...)` call at `path:line:col` (its `#caller_location`), as
    /// written, when it fits on one line.
    fn assert_condition(&self, path: &str, line: u32, col: u32) -> Option<String> {
        let file = self.file_by_path(path)?;
        let span = *self.first_arguments.get(&(file.0, line, col))?;
        let tokens = crate::lexer::lex(file, &self.sources.get(file).text).ok()?;
        let text = self
            .sources
            .snippet_or_empty(crate::lexer::balanced(&tokens, span))
            .trim();
        (!text.is_empty() && !text.contains('\n')).then(|| text.to_string())
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

    /// The directories of the standard library its `module.jai` loads from
    /// beside it, canonical through the compiler's file system (the browser's is virtual).
    fn library_roots(&self) -> Vec<PathBuf> {
        let Some(stdlib) = self.options.preload.as_deref().and_then(Path::parent) else {
            return Vec::new();
        };
        [stdlib.to_path_buf(), stdlib.join("prelude")]
            .iter()
            .map(|root| self.fs.canonical(root))
            .collect()
    }

    fn is_library_file(&self, file: FileId, roots: &[PathBuf]) -> bool {
        if file.0 as usize >= self.sources.len() {
            return true;
        }
        let path = self.fs.canonical(Path::new(&self.sources.get(file).path));
        roots.iter().any(|root| path.starts_with(root))
    }
}

/// Compiler-made procedures (`__jaic_*`, `__init_*`...) have no name the user wrote.
fn is_internal_name(name: &str) -> bool {
    name.is_empty() || name.starts_with("__")
}

fn display_name(frame: &TrapFrame) -> String {
    match frame.origin {
        ir::FuncOrigin::Run => return "the `#run` code".to_string(),
        ir::FuncOrigin::ConstInit => return "a constant's compile-time initializer".to_string(),
        ir::FuncOrigin::Procedure => {}
    }
    // A polymorphic instance is `name#N`; the user knows it as `name`.
    let name = match frame.name.rsplit_once('#') {
        Some((base, n)) if !base.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => base,
        _ => &frame.name,
    };
    if name.is_empty() {
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
fn help_for(kind: TrapKind) -> Option<String> {
    let fixed = match kind {
        TrapKind::Check {
            reason,
            b,
            ..
        } => match reason {
            ir::TRAP_BOUNDS => {
                "valid indices are 0 up to the array's count minus one; check the index or the array's length first"
            }
            ir::TRAP_MISSING_RETURN => {
                "every path through a procedure with results must end in `return`"
            }
            ir::TRAP_CAST_OVERFLOW => return cast_help(b),
            ir::TRAP_SWITCH_UNMATCHED => {
                "the value is not one of the enum's members (was it cast from an integer, or left uninitialized?); add a `case;` to handle other values"
            }
            ir::TRAP_DIVIDE_BY_ZERO => "check the divisor against zero first",
            _ => return None,
        },
        TrapKind::NullPointer => {
            "check the pointer against null before using it, or make sure it is set"
        }
        TrapKind::StackOverflow => "check that the recursion has a base case it reaches",
        TrapKind::BareAssertion | TrapKind::Unavailable | TrapKind::Abandoned => return None,
    };
    Some(fixed.to_string())
}

/// Help for "cast of V to `T` overflows" (`code` as in `ir::cast_check_target`): the range `T`
/// holds and the casts that do not check.
fn cast_help(code: u64) -> Option<String> {
    let target = ir::cast_check_target(code);
    let bits = (code & 0xff) * 8;
    let range = match (code & 0x100 != 0, bits) {
        (false, 1..=64) => format!("0 to {}", u64::MAX >> (64 - bits)),
        (true, 1..=64) => format!("{} to {}", i64::MIN >> (64 - bits), i64::MAX >> (64 - bits)),
        _ => return None,
    };
    Some(format!(
        "`{target}` holds {range}; write `cast,trunc({target})` (or `xx,trunc`) to keep the low bits, or `cast,no_check({target})` to skip the check"
    ))
}
