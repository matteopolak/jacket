//! `autocast_used`: `xx`, the automatic cast, outside a call's arguments.
//!
//! ```jai
//! small: u8 = xx big;
//! return xx count;
//! ```
//!
//! `xx` picks the target type from the context, so the line does not say what the value is
//! converted to, and changing the type on the left silently changes the conversion on the
//! right. `cast(u8) big` says it where it happens. A project that wants every conversion
//! spelled out turns this on together with `autocast_in_arg`, which reports `xx` written as an
//! argument.
//!
//! Fires on every `xx` that is not directly an argument of a call. When the compiler resolved
//! the cast, the message names both types and the editor offers `cast(T)` for plain `xx`; it is
//! not applied by `--fix`. Off by default (`allow`): `xx` is idiomatic Jai.
use crate::syntax::Cx;
use crate::syntax::Recorded;
use crate::{Edit, Finding};
use jaic::ast::walk::{Node, walk};
use jaic::ast::{CastFlags, Expr, ExprKind as E};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "autocast_used",
    category: crate::config::Category::Restriction,
    summary: "`xx`, the automatic cast, outside a call's arguments",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    each_autocast(cx, &mut |e, as_argument| {
        if !as_argument {
            out.push(report(cx, e, "leaves the target type to the context"));
        }
    });
}

/// Call `f` with every `xx` of the file and whether it is directly a call argument.
pub(super) fn each_autocast(cx: &Cx, f: &mut dyn FnMut(&Expr, bool)) {
    let mut visit = |stmts: &[jaic::ast::Stmt]| {
        walk(stmts, &mut |n, chain| {
            if let Node::Expr(e) = n
                && let E::Cast {
                    ty: None, ..
                } = &e.kind
                && cx.src(e.span).starts_with("xx")
            {
                let as_argument = matches!(
                    chain.last(),
                    Some(Node::Expr(Expr { kind: E::Call { args, .. }, .. }))
                        if args.iter().any(|a| a.value.span == e.span)
                );
                f(e, as_argument);
            }
            true
        });
    };
    visit(cx.stmts);
    for p in &cx.procs {
        visit(&p.body.stmts);
    }
}

/// A finding for the `xx` at `e`.
pub(super) fn report(cx: &Cx, e: &Expr, what: &str) -> Finding {
    let types = &cx.compiler.ty.types;
    let known = cx
        .compiler
        .cast_fact(e.span)
        .filter(|c| !c.conflicting && !c.from_constant);
    let Some(c) = known else {
        return super::finding(
            jaic::source::Span::new(cx.file, e.span.start as usize, e.span.start as usize + 2),
            format!("`xx` {what}"),
            Some("write `cast(T)` with the type you mean".into()),
        );
    };
    let target = types.name(c.target);
    let message = format!("`xx` {what} (`{}` to `{target}`)", types.name(c.from));
    let help = format!("write `cast({target})` to say so");
    let plain = matches!(&e.kind, E::Cast { flags, .. } if *flags == CastFlags::default())
        && !target.contains(' ');
    let span = jaic::source::Span::new(cx.file, e.span.start as usize, e.span.start as usize + 2);
    if plain {
        super::suggesting(
            span,
            message,
            help.clone(),
            help,
            vec![Edit {
                start: e.span.start as usize,
                end: e.span.start as usize + 2,
                text: format!("cast({target})"),
            }],
        )
    } else {
        super::finding(span, message, Some(help))
    }
}
