//! Compile-time check of print-style calls: a literal format string whose `%` directives use
//! a different number of arguments than the call passes is a warning.
use super::calls::{CallArg, Slot};
use super::{Compiler, Diagnostic, FileId, ProcId, Result, Span, Sym, TypeId, ast};
use crate::ast::ExprKind as E;
use crate::diagnostics::Code;
use crate::format_string;
use crate::lexer::{self, P, Tok};
use crate::source::{Applicability, Edit, Suggestion};
use crate::types::{ArrayKind, TypeKind};

/// Whether `body` passes the parameters `format` and `values` to the same call, which is what
/// makes a `(string, ..Any)` procedure a formatter rather than a coincidence of signature.
fn forwards_format(text: &str, format: Sym, values: Sym) -> bool {
    let Ok(tokens) = lexer::lex(FileId(0), text) else {
        return false;
    };
    // (saw the format, saw the values) per open parenthesis.
    let mut open: Vec<(bool, bool)> = Vec::new();
    for t in tokens {
        match t.tok {
            Tok::Punct(P::LParen) => open.push((false, false)),
            Tok::Punct(P::RParen) => {
                if let Some((f, v)) = open.pop() {
                    if f && v {
                        return true;
                    }
                    if let Some(parent) = open.last_mut() {
                        parent.0 |= f;
                        parent.1 |= v;
                    }
                }
            }
            Tok::Ident(n) => {
                if let Some(top) = open.last_mut() {
                    top.0 |= n == format;
                    top.1 |= n == values;
                }
            }
            _ => {}
        }
    }
    false
}

impl Compiler {
    /// Whether `p` is declared with the `@PrintLike` note: its `string` parameter before a
    /// variadic `..Any` takes a format string.
    pub fn is_print_like(&self, p: ProcId) -> bool {
        self.proc_has_note(p, format_string::PRINT_LIKE_NOTE)
    }

    /// Whether `p` is declared with the note `@name`, on its header or after its body.
    pub fn proc_has_note(&self, p: ProcId, name: &str) -> bool {
        let noted = |notes: &[ast::Note]| notes.iter().any(|n| n.is(name));
        noted(&self.proc(p).lit.header.notes)
            || self.pr.proc_decl_notes.get(&p).is_some_and(|n| noted(n))
    }

    /// Where `proc` takes a format string: the `string` parameter right before a variadic
    /// `..Any`, by the parameters' resolved types rather than how they are spelled.
    pub(crate) fn format_string_param(&mut self, proc: ProcId, span: Span) -> Option<usize> {
        let sig = self.signature(proc, span).ok()?;
        sig.params.windows(2).position(|w| {
            w[0].ty == TypeId::STRING
                && w[1].variadic
                && matches!(
                    self.ty.types.kind(w[1].ty),
                    TypeKind::Array {
                        elem: TypeId::ANY,
                        kind: ArrayKind::View,
                    }
                )
        })
    }

    /// Check a call to `chosen` with literal format string and argument count.
    pub(super) fn check_format_call(
        &mut self,
        call: Span,
        chosen: ProcId,
        slots: &[Slot],
        args: &[CallArg],
    ) -> Result<()> {
        let Some(fmt) = self.format_string_param(chosen, call) else {
            return Ok(());
        };
        let lit = &self.proc(chosen).lit;
        let params = &lit.header.params;
        let Some(Slot::Arg(format_arg)) = slots.get(fmt) else {
            return Ok(());
        };
        let format_arg = &args[*format_arg];
        let Some(ast::Expr {
            kind: E::Str(bytes),
            ..
        }) = &format_arg.expr
        else {
            return Ok(());
        };
        let given: &[usize] = match slots.get(fmt + 1) {
            Some(Slot::Variadic(list)) => list,
            Some(Slot::Default) | None => &[],
            Some(_) => return Ok(()),
        };
        if given
            .iter()
            .any(|&a| args[a].spread || args[a].name.is_some())
        {
            return Ok(());
        }
        // `@PrintLike` states the intent; without it a body that hands both parameters to one
        // call (a wrapper around `print`) is taken for a formatter.
        if !self.is_print_like(chosen) {
            let (Some(body), Some(f), Some(v)) =
                (&lit.body, params[fmt].name, params[fmt + 1].name)
            else {
                return Ok(());
            };
            if !forwards_format(self.src.sources.snippet(body.span), f.name, v.name) {
                return Ok(());
            }
        }
        let need = format_string::required(&format_string::specs_of_value(bytes));
        let name = self.proc(chosen).name;
        let plural = |n: usize| {
            format!(
                "{n} argument{}",
                if n == 1 {
                    ""
                } else {
                    "s"
                }
            )
        };
        if given.len() == need {
            return Ok(());
        }
        let message = format!(
            "incorrect number of arguments supplied to `{name}`: the format string requires {}, but {} {} given",
            plural(need),
            plural(given.len()),
            if given.len() == 1 {
                "is"
            } else {
                "are"
            }
        );
        let mut warning = if given.len() < need {
            Diagnostic::new(Code::FormatMissingArgument, call, message)
                .with_label("no argument for the last `%`")
                .with_help("pass a value for every `%`, or write `\\%` for a percent sign")
        } else {
            Diagnostic::new(Code::FormatExtraArgument, call, message)
                .with_label("the extra arguments are not printed")
                .with_help("add a `%` for each argument, or remove the extra ones")
        };
        // The literal's text in the source is not the string that was checked.
        let fix = if self.pr.interpolated.contains(&format_arg.span) {
            None
        } else if given.len() < need {
            self.escape_unmatched(format_arg.span, given.len())
        } else {
            self.drop_extra_arguments(format_arg.span, need, given.iter().map(|&a| args[a].span))
        };
        if let Some(fix) = fix {
            warning = warning.with_suggestion(fix);
        }
        self.warn(warning);
        Ok(())
    }

    /// The source text of `span`, when it is in a file with text (macro code is not).
    fn source_of(&self, span: Span) -> Option<&str> {
        ((span.file.0 as usize) < self.src.sources.len()).then(|| self.src.sources.snippet(span))
    }

    /// For a format string that asks for more arguments than the call passes (`given`): write
    /// each `%` that has none as `\%`, a literal percent sign. Only offered when every such
    /// directive is a bare `%` and the string then asks for exactly the arguments passed.
    fn escape_unmatched(&self, format: Span, given: usize) -> Option<Suggestion> {
        let literal = self.source_of(format)?;
        if !literal.starts_with('"') {
            return None;
        }
        let specs = format_string::specs(literal, 0, literal.len());
        let mut unmatched: Vec<_> = specs
            .iter()
            .filter(|s| s.index.is_some_and(|i| i >= given))
            .collect();
        if unmatched.is_empty() || unmatched.iter().any(|s| s.end - s.start != 1) {
            return None;
        }
        unmatched.sort_by_key(|s| s.start);
        let mut fixed = literal.to_string();
        for spec in unmatched.iter().rev() {
            fixed.insert(spec.start, '\\');
        }
        // The escaped string must use exactly the arguments the call passes.
        let after = format_string::specs(&fixed, 0, fixed.len());
        if format_string::required(&after) != given {
            return None;
        }
        let title = if unmatched.len() == 1 {
            "write the last `%` as `\\%` to print a percent sign"
        } else {
            "write each `%` without an argument as `\\%` to print a percent sign"
        };
        let edits = unmatched
            .iter()
            .map(|s| {
                let at = format.start as usize + s.start;
                Edit::Replace {
                    span: Span::new(format.file, at, at),
                    text: "\\".into(),
                }
            })
            .collect();
        Some(
            Suggestion::with_edits(title, edits, Applicability::MaybeIncorrect)
                .with_action("Escape the `%`"),
        )
    }

    /// For a call that passes more arguments than the format string uses (`need`): remove the
    /// ones after them. Only offered when the call ends right after the last argument and
    /// the removed text has balanced brackets, so it is exactly the arguments.
    fn drop_extra_arguments(
        &self,
        format: Span,
        need: usize,
        given: impl Iterator<Item = Span>,
    ) -> Option<Suggestion> {
        let given: Vec<Span> = given.collect();
        let last = *given.last()?;
        let keep = if need == 0 {
            format
        } else {
            *given.get(need - 1)?
        };
        if keep.file != last.file || keep.file != format.file || keep.end > last.start {
            return None;
        }
        let removed = Span::new(keep.file, keep.end as usize, last.end as usize);
        let removed_text = self.source_of(removed)?;
        let text_len = self.src.sources.get(keep.file).text.len();
        let after = Span::new(keep.file, last.end as usize, text_len);
        let rest = self.source_of(after)?.trim_start();
        let rest = rest.strip_prefix(',').unwrap_or(rest).trim_start();
        if !rest.starts_with(')') || !balanced(removed_text) {
            return None;
        }
        let (title, action) = if given.len() - need == 1 {
            ("remove the extra argument", "Remove the extra argument")
        } else {
            ("remove the extra arguments", "Remove the extra arguments")
        };
        Some(
            Suggestion::replace(title, removed, "", Applicability::MaybeIncorrect)
                .with_action(action),
        )
    }
}

/// Whether the brackets in `text` pair up and no string literal is left open.
fn balanced(text: &str) -> bool {
    let mut depth = 0i32;
    let mut quote = false;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' if quote => {
                chars.next();
            }
            '"' => quote = !quote,
            '(' | '[' | '{' if !quote => depth += 1,
            ')' | ']' | '}' if !quote => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0 && !quote
}
