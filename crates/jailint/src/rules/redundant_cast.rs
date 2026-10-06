//! `redundant_cast`: a cast to the type the value already has.
//!
//! `cast(s32) n` where `n: s32`, or `x: u8 = xx y` where `y: u8`. The cast does nothing and
//! makes the reader look for a conversion that is not there.
//!
//! Type aliases can hide a difference that only shows on another platform (`c_long` is `s64`
//! here and `s32` on Windows), so equal types are not enough: the value must be a variable
//! or parameter declared with the very same type spelling as the cast's (`int` and `s64`,
//! `float` and `float32` count as the same), or another cast to that spelling. Polymorphic
//! bodies, macros and cast modifiers (`,trunc`, `,no_check`, `,force`) are left alone.
use crate::facts::Recorded;
use crate::syntax::{Cx, Node, squash, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{Expr, ExprKind as E, StmtKind as S};
use jaic::fxhash::HashMap;
use jaic::source::Span;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let spelled = declared_types(cx);
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            match n {
                Node::Expr(e) => {
                    if let E::Cast {
                        ty: Some(t),
                        value,
                        flags,
                    } = &e.kind
                        && *flags == Default::default()
                        && let Some(c) = cx.compiler.cast_fact(e.span)
                        && !c.conflicting
                        && !c.from_constant
                        && c.target == c.from
                        && spelling(cx, value, &spelled).is_some_and(|s| same(&s, cx.src(t.span)))
                    {
                        // Keep what follows the `cast(T)`, parentheses included.
                        let after = cx.text[t.span.end as usize..e.span.end as usize]
                            .trim_start()
                            .trim_start_matches(')')
                            .trim_start();
                        report(cx, e, cx.src(t.span), after, out);
                    }
                }
                Node::Stmt(s) => {
                    // `x: T = xx v;`
                    if let S::Decl(d) = &s.kind
                        && let (Some(t), Some(v)) = (&d.ty, &d.value)
                        && let E::Cast {
                            ty: None,
                            value,
                            flags,
                        } = &v.kind
                        && *flags == Default::default()
                        && let Some(c) = cx.compiler.cast_fact(v.span)
                        && !c.conflicting
                        && !c.from_constant
                        && c.target == c.from
                        && spelling(cx, value, &spelled).is_some_and(|s| same(&s, cx.src(t.span)))
                    {
                        let after =
                            cx.text[v.span.start as usize + 2..v.span.end as usize].trim_start();
                        report(cx, v, cx.src(t.span), after, out);
                    }
                }
            }
            true
        });
    }
}

fn report(cx: &Cx, cast: &Expr, ty: &str, rest: &str, out: &mut Vec<Finding>) {
    let help = "remove the cast".to_string();
    out.push(Finding {
        start: cast.span.start as usize,
        end: cast.span.end as usize,
        message: format!(
            "`{}` is already `{}`",
            cx.src(value_of(cast).span).trim(),
            ty.trim()
        ),
        label: None,
        help: Some(help.clone()),
        fix: Some(Fix {
            title: help,
            edits: vec![Edit {
                start: cast.span.start as usize,
                end: cast.span.end as usize,
                text: rest.to_string(),
            }],
            machine_applicable: true,
        }),
    });
}

fn value_of(cast: &Expr) -> &Expr {
    match &cast.kind {
        E::Cast {
            value, ..
        } => value,
        _ => cast,
    }
}

/// Two type spellings name the same type on every platform.
fn same(a: &str, b: &str) -> bool {
    let norm = |t: &str| {
        let t = squash(t);
        match t.as_str() {
            "int" => "s64".to_string(),
            "float" => "float32".to_string(),
            _ => t,
        }
    };
    norm(a) == norm(b)
}

/// How the type of `value` is written where it is declared.
fn spelling(cx: &Cx, value: &Expr, spelled: &HashMap<Span, String>) -> Option<String> {
    match &value.kind {
        E::Ident(_) => {
            let ents = cx.entities(value.span);
            let first = *ents.first()?;
            let span = cx.compiler.entity(first).span;
            // Every entity (one per instance) must come from the same declaration.
            if ents.iter().any(|&e| cx.compiler.entity(e).span != span) {
                return None;
            }
            spelled.get(&span).cloned()
        }
        E::Cast {
            ty: Some(t),
            flags,
            ..
        } if *flags == Default::default() => Some(cx.src(t.span).to_string()),
        _ => None,
    }
}

/// Declared types of the file's locals (by name span) and parameters (by declaration span).
fn declared_types(cx: &Cx) -> HashMap<Span, String> {
    let mut out = HashMap::default();
    for p in &cx.procs {
        for param in &p.header.params {
            if let Some(t) = &param.ty
                && !param.variadic
            {
                out.insert(param.span, cx.src(t.span).to_string());
            }
        }
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(s) = n.stmt()
                && let S::Decl(d) = &s.kind
                && let Some(t) = &d.ty
            {
                for name in &d.names {
                    out.insert(name.span, cx.src(t.span).to_string());
                }
            }
            true
        });
    }
    out
}
