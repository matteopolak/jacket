//! `range_past_count`: `for i: 0..xs.count { xs[i] }`.
//!
//! Jai ranges include their end, so `0..xs.count` visits `xs.count + 1` indices and the last
//! `xs[i]` reads one past the end: a bounds-check failure, or a read of whatever follows the
//! array when checks are off. Loops written with C's `i < count` in mind make this slip.
//!
//! Fires when a `for` range ends at exactly `xs.count` for an array or string `xs`, and the
//! body indexes `xs` with the loop variable directly (`xs[i]`). Loops that mention `xs.count`
//! in their body (they may guard the last index) are left alone. The suggested fix ends the
//! range at `xs.count - 1`; it changes what the loop does, so it is not applied by `--fix`.
use super::finding;
use crate::syntax::{Cx, Node, is_path, squash, walk, walk_node};
use crate::{Edit, Finding, Fix};
use jaic::ast::{ExprKind as E, ForOver, StmtKind as S};
use jaic::intern::Sym;
use jaic::types::TypeKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            let S::For(f) = &s.kind else {
                return true;
            };
            let ForOver::Range(_, hi) = &f.over else {
                return true;
            };
            let E::Member(arr, member) = &hi.kind else {
                return true;
            };
            if member.name.as_str() != "count" || !is_path(arr) || f.iterator.is_some() {
                return true;
            }
            let Some(ty) = cx.ty(arr) else {
                return true;
            };
            if !matches!(
                cx.compiler.types.kind(ty),
                TypeKind::Array { .. } | TypeKind::String
            ) {
                return true;
            }
            let arr_text = squash(cx.whole_src(arr.span));
            let count_text = format!("{arr_text}.count");
            let var = f.it.map_or_else(|| Sym::intern("it"), |i| i.name);
            let mut indexed = None;
            let mut guarded = false;
            let mut stack = Vec::new();
            walk_node(Node::Stmt(&f.body), &mut stack, &mut |n, _| {
                if let Some(e) = n.expr() {
                    match &e.kind {
                        E::Index(base, index)
                            if matches!(index.kind, E::Ident(v) if v == var)
                                && squash(cx.whole_src(base.span)) == arr_text =>
                        {
                            indexed.get_or_insert(e.span);
                        }
                        E::Member(..) if squash(cx.whole_src(e.span)) == count_text => {
                            guarded = true;
                        }
                        _ => {}
                    }
                }
                true
            });
            let Some(_) = indexed else {
                return true;
            };
            if guarded {
                return true;
            }
            let (_, end) = cx.whole(hi.span);
            let mut found = finding(
                hi.span,
                format!(
                    "`{}` is one past the last index of `{}`",
                    cx.src(hi.span),
                    cx.src(arr.span)
                ),
                Some(format!(
                    "ranges include their end: write `{}.count - 1`",
                    cx.src(arr.span)
                )),
            );
            found.fix = Some(Fix {
                title: format!("end the range at `{}.count - 1`", cx.src(arr.span)),
                edits: vec![Edit {
                    start: end,
                    end,
                    text: " - 1".into(),
                }],
                machine_applicable: false,
            });
            out.push(found);
            true
        });
    }
}
