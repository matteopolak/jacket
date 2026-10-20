//! Compile-time check of print-style calls: a literal format string whose `%` directives use
//! a different number of arguments than the call passes is a warning.
use super::calls::{CallArg, Slot};
use super::{Compiler, Diagnostic, FileId, ProcId, Result, Span, Sym, TypeId, ast};
use crate::ast::ExprKind as E;
use crate::format_string;
use crate::lexer::{self, P, Tok};
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
            || self.proc_decl_notes.get(&p).is_some_and(|n| noted(n))
    }

    /// Where `proc` takes a format string: the `string` parameter right before a variadic
    /// `..Any`, by the parameters' resolved types rather than how they are spelled.
    pub(crate) fn format_string_param(&mut self, proc: ProcId, span: Span) -> Option<usize> {
        let sig = self.signature(proc, span).ok()?;
        sig.params.windows(2).position(|w| {
            w[0].ty == TypeId::STRING
                && w[1].variadic
                && matches!(
                    self.types.kind(w[1].ty),
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
        let warning = if given.len() < need {
            Diagnostic::warning(call, message)
                .with_label("no argument for the last `%`")
                .with_help("pass a value for every `%`, or write `\\%` for a percent sign")
        } else {
            Diagnostic::warning(call, message)
                .with_label("the extra arguments are not printed")
                .with_help("add a `%` for each argument, or remove the extra ones")
        };
        self.warn(warning);
        Ok(())
    }
}
