//! `large_stack_array`: a local fixed array bigger than `[lint] max_stack_array_bytes`.
//!
//! ```jai
//! work :: () {
//!     scratch: [1 << 20] u8;
//! }
//! ```
//!
//! Locals live on the stack, which is small (a thread's is often 1 MiB or less, and deep
//! recursion shares it). A megabyte-sized local can overflow it on the first call, with no
//! message beyond a crash. Allocate large buffers with `NewArray`, `alloc` or a resizable array
//! instead, or keep them in a global.
//!
//! Fires on a local variable (not a parameter) whose type is a fixed array larger than the
//! limit, 64 KiB by default; an array of arrays or structs counts by its total size. Locals of
//! polymorphic procedures and macros, and generated files, are skipped. No fix. Off by default
//! (`allow`): programs that run on a large stack may be fine with it.
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{DeclKind, StmtKind as S};
use jaic::sema::scope::EntityKind;
use jaic::types::{ArrayKind, TypeKind};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    let types = &cx.compiler.ty.types;
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            let S::Decl(d) = &s.kind else {
                return true;
            };
            if d.kind != DeclKind::Var {
                return true;
            }
            for name in &d.names {
                let Some(entities) = cx.facts.locals.get(&name.span) else {
                    continue;
                };
                let mut found = None;
                for &e in entities {
                    let EntityKind::Local {
                        ty, ..
                    } = cx.compiler.entity(e).kind
                    else {
                        continue;
                    };
                    found = match found {
                        None => Some(ty),
                        Some(other) if other == ty => Some(ty),
                        Some(_) => None,
                    };
                }
                let Some(ty) = found else {
                    continue;
                };
                if !matches!(
                    types.kind(ty),
                    TypeKind::Array {
                        kind: ArrayKind::Fixed(_),
                        ..
                    }
                ) {
                    continue;
                }
                let bytes = types.size_of(ty);
                if bytes as usize <= cx.max_stack_array_bytes {
                    continue;
                }
                out.push(super::finding(
                    name.span,
                    format!(
                        "`{}` is {} on the stack, past the limit of {}",
                        name.name.as_str(),
                        size(bytes),
                        size(cx.max_stack_array_bytes as u64)
                    ),
                    Some(
                        "allocate it (`NewArray`, `alloc`), use a resizable array, or keep it in a global"
                            .into(),
                    ),
                ));
            }
            true
        });
    }
}

/// `65536` as `64 KiB`.
fn size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * KIB;
    if bytes >= MIB && bytes.is_multiple_of(MIB) {
        format!("{} MiB", bytes / MIB)
    } else if bytes >= KIB && bytes.is_multiple_of(KIB) {
        format!("{} KiB", bytes / KIB)
    } else {
        format!("{bytes} bytes")
    }
}
