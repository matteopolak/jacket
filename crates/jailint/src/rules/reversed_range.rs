//! `reversed_range`: `for i: 10..0`, `for i: xs.count-1..0`.
//!
//! A range always counts up, so one whose start is above its end runs zero times. Counting
//! down is written `for < i: 0..10`: the range stays low-to-high and `<` walks it backwards.
//!
//! Fires on a forward `for` over a range whose start and end are integer literals with the
//! start greater, and on one that ends at the literal `0` and starts at a `.count` expression
//! (`xs.count - 1..0`), which is empty for every non-empty `xs`.
use super::int_literal;
use crate::syntax::{Cx, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{ExprKind as E, ForOver, StmtKind as S};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            let S::For(f) = &s.kind else {
                return true;
            };
            let ForOver::Range(lo, hi) = &f.over else {
                return true;
            };
            if f.reverse || f.reverse_if.is_some() || f.iterator.is_some() {
                return true;
            }
            let reversed = match (int_literal(lo), int_literal(hi)) {
                (Some(a), Some(b)) => a > b,
                (None, Some(0)) => counts(lo),
                _ => false,
            };
            if !reversed {
                return true;
            }
            let (lo_start, lo_end) = cx.whole(lo.span);
            let (hi_start, hi_end) = cx.whole(hi.span);
            let lo_text = &cx.text[lo_start..lo_end];
            let hi_text = &cx.text[hi_start..hi_end];
            // `for i: a..b` → `for < i: b..a`: the `<` goes right after `for`.
            let after_for = s.span.start as usize + "for".len();
            out.push(Finding {
                start: lo.span.start as usize,
                end: hi.span.end as usize,
                message: format!("the range `{lo_text}..{hi_text}` is empty: ranges count up"),
                label: None,
                help: Some(format!(
                    "to count down, write `for < ...: {hi_text}..{lo_text}`"
                )),
                fix: Some(Fix {
                    title: format!("count down over `{hi_text}..{lo_text}`"),
                    edits: vec![
                        Edit {
                            start: after_for,
                            end: after_for,
                            text: " <".into(),
                        },
                        Edit {
                            start: lo_start,
                            end: hi_end,
                            text: format!("{hi_text}..{lo_text}"),
                        },
                    ],
                    machine_applicable: false,
                }),
            });
            true
        });
    }
}

/// `xs.count`, `xs.count - 1`.
fn counts(e: &jaic::ast::Expr) -> bool {
    match &e.kind {
        E::Member(_, m) => m.name.as_str() == "count",
        E::Binary(jaic::ast::BinOp::Sub, a, b) => counts(a) && int_literal(b).is_some(),
        _ => false,
    }
}
