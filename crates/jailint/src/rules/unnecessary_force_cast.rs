//! `unnecessary_force_cast`: `cast,force(T) x` where a plain `cast(T) x` does exactly the same.
//!
//! `,force` exists for two jobs: reinterpreting the bits of a number as a float (or the other
//! way round) or of an aggregate as another of the same size, and skipping the range check a
//! plain cast makes when it narrows an integer. When the value already has the target type,
//! or the conversion widens (`s32` to `s64`, `u8` to `u32`, `float32` to `float64`), neither
//! applies: nothing is reinterpreted and the check has nothing to catch. The flag then only
//! tells the reader there is something dangerous going on.
//!
//! Fires on a `cast,force(T)` with no other modifier when the operand is not a constant and its
//! type is `T` or converts to `T` without losing anything. `--fix` removes the `,force`.
use crate::facts::Recorded;
use crate::syntax::Cx;
use crate::{Edit, Finding, Fix};
use jaic::ast::walk::walk;
use jaic::ast::{CastFlags, ExprKind as E};
use jaic::types::{TypeId, TypeKind, Types};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unnecessary_force_cast",
    category: crate::config::Category::Complexity,
    summary: "`cast,force` where the value already has the type or the conversion widens",
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
            let E::Cast {
                ty: Some(ty),
                flags,
                ..
            } = &e.kind
            else {
                return true;
            };
            if *flags
                != (CastFlags {
                    force: true,
                    ..CastFlags::default()
                })
            {
                return true;
            }
            let Some(c) = cx.compiler.cast_fact(e.span) else {
                return true;
            };
            if c.conflicting || c.from_constant || !lossless(types, c.from, c.target) {
                return true;
            }
            // The `,force` sits between `cast` and the `(` of the type.
            let head = &cx.text[e.span.start as usize..ty.span.start as usize];
            let Some(at) = head.find(",force") else {
                return true;
            };
            let start = e.span.start as usize + at;
            let help = "remove `,force`: the plain cast does the same".to_string();
            out.push(Finding {
                start: e.span.start as usize,
                end: ty.span.end as usize,
                message: format!(
                    "`cast,force` from `{}` to `{}` reinterprets nothing and has no range to check",
                    types.name(c.from),
                    types.name(c.target)
                ),
                label: None,
                help: Some(help.clone()),
                fix: Some(Fix {
                    title: help,
                    edits: vec![Edit {
                        start,
                        end: start + ",force".len(),
                        text: String::new(),
                    }],
                    machine_applicable: true,
                }),
            });
            true
        });
    }
}

/// Converting `from` to `to` keeps every value, so a plain cast would not check anything.
fn lossless(types: &Types, from: TypeId, to: TypeId) -> bool {
    if from == to {
        return true;
    }
    let (fr, tr) = (types.repr(from), types.repr(to));
    match (types.kind(fr), types.kind(tr)) {
        (
            TypeKind::Int {
                bits: fb,
                signed: fs,
            },
            TypeKind::Int {
                bits: tb,
                signed: ts,
            },
        ) => {
            if fs == ts {
                tb >= fb
            } else {
                // Unsigned into a wider signed type.
                !fs && *ts && tb > fb
            }
        }
        (
            TypeKind::Float {
                bits: fb,
            },
            TypeKind::Float {
                bits: tb,
            },
        ) => tb >= fb,
        _ => false,
    }
}
