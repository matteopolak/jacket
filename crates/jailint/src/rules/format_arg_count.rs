//! `format_arg_count`: a format string whose `%` directives use more arguments than the call
//! passes, or fewer.
//!
//! `print("% and %\n", a)` prints an error marker in place of the missing value at run time;
//! `print("total\n", n)` drops `n` silently. Both are almost always mistakes.
//!
//! A procedure the call resolved to is print-like when a `string` parameter is followed by a
//! variadic `..Any` one and its body hands both on to one call (`print`, `sprint`, `tprint`,
//! `log`, `assert`'s message, user wrappers): a `string` and `..Any` that are used apart
//! (`greet :: (name: string, extras: ..Any)`) are not a format. Only literal format strings
//! are read, and calls that
//! spread an array (`..args`) are skipped. Directives are read as `Basic` reads them (see
//! `format_string`).
use super::plural;
use crate::Finding;
use crate::format_string::{required, specs};
use crate::syntax::{Cx, Node, walk};
use jaic::ast::{Expr, ExprKind as E};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let Some(ide) = cx.compiler.ide.as_ref() else {
        return;
    };
    // Calls written in macro bodies are checked per expansion with substituted arguments.
    let macro_bodies: Vec<_> = cx
        .procs
        .iter()
        .filter(|p| p.is_macro)
        .map(|p| p.body.span)
        .collect();
    for call in ide.calls.iter().filter(|c| c.span.file == cx.file) {
        if macro_bodies
            .iter()
            .any(|b| b.start <= call.span.start && call.span.end <= b.end)
        {
            continue;
        }
        let header = &cx.compiler.proc(call.chosen).lit.header;
        let named = |e: &Option<Expr>, want: &str| matches!(e, Some(Expr { kind: E::Ident(n), .. }) if n.as_str() == want);
        let Some(fmt) = header
            .params
            .windows(2)
            .position(|w| named(&w[0].ty, "string") && w[1].variadic && named(&w[1].ty, "Any"))
        else {
            continue;
        };
        if !forwards_format(cx.compiler.proc(call.chosen).lit.as_ref(), fmt) {
            continue;
        }
        let strings: Vec<_> = call
            .args
            .iter()
            .filter(|a| a.param == fmt && !a.variadic)
            .collect();
        let [string] = strings.as_slice() else {
            continue;
        };
        let lit = cx.src(string.span);
        if string.spread || !lit.starts_with('"') || !lit.ends_with('"') || lit.len() < 2 {
            continue;
        }
        let rest: Vec<_> = call.args.iter().filter(|a| a.param == fmt + 1).collect();
        if rest.iter().any(|a| a.spread || a.named) {
            continue;
        }
        let found = specs(
            cx.text,
            string.span.start as usize,
            string.span.end as usize,
        );
        let need = required(&found);
        let given = rest.len();
        let callee = cx.compiler.proc(call.chosen).name;
        if given < need {
            let first_missing = found
                .iter()
                .find(|s| s.index.is_some_and(|i| i >= given))
                .copied();
            let (start, end) = first_missing.map_or(
                (string.span.start as usize, string.span.end as usize),
                |s| (s.start, s.end),
            );
            out.push(Finding {
                start,
                end,
                message: format!(
                    "the format string uses {} but `{callee}` is given {}",
                    plural(need, "argument"),
                    plural(given, "argument"),
                ),
                label: Some("no argument for this".into()),
                help: Some("pass a value for every `%`, or write `\\%` for a percent sign".into()),
                fix: None,
            });
        } else if given > need {
            let extra = rest[need];
            out.push(Finding {
                start: extra.span.start as usize,
                end: rest.last().map_or(extra.span.end, |a| a.span.end) as usize,
                message: format!(
                    "`{callee}` is given {} but the format string uses {}",
                    plural(given, "argument"),
                    plural(need, "argument"),
                ),
                label: Some("not printed".into()),
                help: Some("add a `%` for each argument, or remove the extra ones".into()),
                fix: None,
            });
        }
    }
}

/// The body of `lit` passes its parameters `fmt` (the format) and `fmt + 1` (the values) to
/// the same call: it formats with them.
fn forwards_format(lit: &jaic::ast::ProcLit, fmt: usize) -> bool {
    let (Some(body), Some(f), Some(v)) = (
        &lit.body,
        lit.header.params.get(fmt).and_then(|p| p.name),
        lit.header.params.get(fmt + 1).and_then(|p| p.name),
    ) else {
        return false;
    };
    let names = |e: &Expr, n: jaic::intern::Sym| matches!(&e.kind, E::Ident(x) if *x == n);
    let mut found = false;
    walk(&body.stmts, &mut |node, _| {
        if let Node::Expr(Expr {
            kind: E::Call {
                args, ..
            },
            ..
        }) = node
            && args.iter().any(|a| names(&a.value, f.name))
            && args.iter().any(|a| names(&a.value, v.name))
        {
            found = true;
        }
        !found
    });
    found
}
