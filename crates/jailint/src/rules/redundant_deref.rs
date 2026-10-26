//! `redundant_deref`: `p.*.field` where `p.field` means the same.
//!
//! Member access looks through a pointer to a struct, so `p.field` reads (and writes) the
//! field of what `p` points to, and the `.*` in `p.*.field` adds nothing.
//!
//! Fires when the member is taken from the dereference of an expression whose type is a
//! pointer to a struct or union (a pointer to a pointer, or to an array, is left alone: there
//! the `.*` matters). `--fix` removes the `.*` when it is written after the pointer; the
//! prefix form `(<<p).field` is reported without a fix.
use crate::syntax::{Cx, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{ExprKind as E, UnOp};
use jaic::types::TypeKind;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "redundant_deref",
    category: crate::config::Category::Complexity,
    summary: "`p.*.field` where `p.field` means the same",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let types = &cx.compiler.ty.types;
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Member(base, member) = &e.kind else {
                return true;
            };
            let E::Unary(UnOp::Deref, pointer) = &base.kind else {
                return true;
            };
            let Some(ty) = cx.ty(pointer) else {
                return true;
            };
            let TypeKind::Pointer(target) = types.kind(ty) else {
                return true;
            };
            if !matches!(types.kind(*target), TypeKind::Struct(_)) {
                return true;
            }
            let gap = jaic::source::Span::new(
                cx.file,
                pointer.span.end as usize,
                member.span.start as usize,
            );
            let compact: String = cx.src(gap).chars().filter(|c| !c.is_whitespace()).collect();
            let shown = cx.src(pointer.span).trim();
            let help = format!(
                "write `{shown}.{}`: member access already looks through the pointer",
                member.name.as_str()
            );
            let fix = (compact == ".*.").then(|| Fix {
                title: help.clone(),
                edits: vec![Edit {
                    start: gap.start as usize,
                    end: gap.end as usize,
                    text: ".".into(),
                }],
                machine_applicable: true,
            });
            // Report the `.*` itself, so a chain like `a.*.b.*.c` gives one finding per `.*`.
            let star = cx
                .src(gap)
                .find('*')
                .map_or(gap.end as usize, |i| gap.start as usize + i + 1);
            out.push(Finding {
                start: gap.start as usize,
                end: star,
                message: format!(
                    "`{shown}.*` is not needed to reach `{}`",
                    member.name.as_str()
                ),
                label: None,
                help: Some(help),
                fix,
            });
            true
        });
    }
}
