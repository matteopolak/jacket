//! Compile-time check of print-style calls: a literal format string whose `%` directives use
//! a different number of arguments than the call passes is a warning.
use super::calls::{CallArg, Slot};
use super::*;
use crate::ast::ExprKind as E;
use crate::lexer::{self, P, Tok};

/// Arguments the directives of `format` use, read as `Basic`'s formatter reads them: `%` (or
/// `%0`) takes the next argument, `%N` takes argument N (1-based) and a following `%` continues
/// after it, `%00` takes none, and `%%` is two arguments in a row. A `\%` in source is byte 31
/// by now, so it is not a directive.
pub(super) fn arguments_used(format: &[u8]) -> usize {
    let mut at = 0;
    let mut next = 0usize;
    let mut used = 0usize;
    while at < format.len() {
        if format[at] != b'%' {
            at += 1;
            continue;
        }
        at += 1;
        if format.get(at) == Some(&b'0') && format.get(at + 1) == Some(&b'0') {
            at += 2;
            continue;
        }
        let index = if format.get(at).is_some_and(|c| (b'1'..=b'9').contains(c)) {
            let mut n = 0usize;
            while let Some(d) = format.get(at).filter(|c| c.is_ascii_digit()) {
                n = n.saturating_mul(10).saturating_add((d - b'0') as usize);
                at += 1;
            }
            n - 1
        } else {
            if format.get(at) == Some(&b'0') {
                at += 1;
            }
            next
        };
        next = index.saturating_add(1);
        used = used.max(next);
    }
    used
}

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
    /// Check a call to `chosen` with literal format string and argument count.
    pub(super) fn check_format_call(
        &mut self,
        call: Span,
        chosen: ProcId,
        slots: &[Slot],
        args: &[CallArg],
    ) -> Result<()> {
        let lit = &self.proc(chosen).lit;
        let named = |e: &Option<ast::Expr>, want: &str| matches!(e, Some(ast::Expr { kind: E::Ident(n), .. }) if n.as_str() == want);
        let params = &lit.header.params;
        let Some(fmt) = params
            .windows(2)
            .position(|w| named(&w[0].ty, "string") && w[1].variadic && named(&w[1].ty, "Any"))
        else {
            return Ok(());
        };
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
        let (Some(body), Some(f), Some(v)) = (&lit.body, params[fmt].name, params[fmt + 1].name)
        else {
            return Ok(());
        };
        if !forwards_format(self.sources.snippet(body.span), f.name, v.name) {
            return Ok(());
        }
        let need = arguments_used(bytes);
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

#[cfg(test)]
mod tests {
    use super::arguments_used;

    #[test]
    fn counts_directives_as_basic_reads_them() {
        assert_eq!(arguments_used(b"none"), 0);
        assert_eq!(arguments_used(b"% and %"), 2);
        assert_eq!(arguments_used(b"%2 %"), 3);
        assert_eq!(arguments_used(b"%2 %1"), 2);
        assert_eq!(arguments_used(b"%%"), 2);
        assert_eq!(arguments_used(b"%00 x"), 0);
        assert_eq!(arguments_used(b"%0"), 1);
        assert_eq!(arguments_used(b"100\x1f done"), 0);
    }
}
