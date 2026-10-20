//! `unpaired_lock`: `lock(*mutex);` and a `return` that leaves it locked.
//!
//! A lock that one path forgets to release blocks every other thread that wants it.
//!
//! ```jai
//! lock(*state.mutex);
//! if state.closed return false;      // still locked
//! state.count += 1;
//! unlock(*state.mutex);
//! ```
//!
//! Procedures declare what they do to a lock with notes: `@Locks` on a procedure that takes
//! the lock its first argument names, `@Unlocks` on the one that gives it back. Fires on a
//! statement-level call of a `@Locks` procedure at the top level of a procedure body when some
//! path from it reaches a `return`, or the end of the body, without a call of an `@Unlocks`
//! procedure on the same first argument (compared as written, ignoring whitespace) or a
//! `defer` of one. A `defer` earlier in the body counts too.
//!
//! Only paths made of statements in sequence, `if`/`else` and blocks are followed. Anything
//! else stays quiet: a loop or `switch` that mentions the unlock or returns without it (a loop
//! holding a `return` and no unlock is reported at that `return`), `break` and `continue`, a
//! branch that unlocks while its sibling does not, `#insert`, `using` and macro calls. A
//! procedure that is itself `@Locks` is a wrapper and is not checked, and so is a lock taken
//! inside a nested block.
use super::frees::opaque;
use super::{calls_noted, finding, has_note};
use crate::Finding;
use crate::syntax::{Cx, Node, walk_node};
use jaic::ast::{Expr, ExprKind as E, Stmt, StmtKind as S};
use jaic::source::Span;

/// What happens to a held lock along the statements that follow the call.
enum Flow {
    /// Still held when the statements run out.
    Locked,
    /// Released on the way, and the path carries on.
    Unlocked,
    /// Released, and the path ends in a `return`, `break` or `continue` of its own.
    Exited,
    /// A `return` at this span leaves it held.
    Leak(Span),
    /// The scan cannot tell.
    Unsure,
}

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        let wrapper = cx
            .facts
            .procs_by_header
            .get(&p.header.span)
            .is_some_and(|ids| ids.iter().any(|&id| has_note(cx, id, "Locks")));
        if wrapper {
            continue;
        }
        let list = &p.body.stmts;
        for (k, s) in list.iter().enumerate() {
            let Some((arg, call)) = lock_call(cx, s) else {
                continue;
            };
            if list[..k]
                .iter()
                .any(|s| matches!(&s.kind, S::Defer { body, .. } if releases(cx, body, arg)))
            {
                continue;
            }
            let shown = cx.whole_src(arg.span).trim();
            match flow_list(cx, arg, &list[k + 1..]) {
                Flow::Locked => out.push(finding(
                    call,
                    format!(
                        "`{shown}` is locked here but the procedure can end without unlocking it"
                    ),
                    Some(
                        "unlock it on every path, or `defer` the unlock right after locking".into(),
                    ),
                )),
                Flow::Leak(at) => out.push(finding(
                    at,
                    format!("this `return` leaves `{shown}` locked"),
                    Some(
                        "unlock it before returning, or `defer` the unlock right after locking"
                            .into(),
                    ),
                )),
                Flow::Unlocked | Flow::Exited | Flow::Unsure => {}
            }
        }
    }
}

/// The statement is only a call of a `@Locks` procedure with an argument: the first argument
/// and the call's span.
fn lock_call<'a>(cx: &Cx, s: &'a Stmt) -> Option<(&'a Expr, Span)> {
    let S::Expr(e) = &s.kind else {
        return None;
    };
    let E::Call {
        callee,
        args,
        ..
    } = &e.kind
    else {
        return None;
    };
    let first = args.first()?;
    (calls_noted(cx, callee, "Locks") && !first.spread).then_some((&first.value, e.span))
}

/// The expression is a call of an `@Unlocks` procedure on `arg`.
fn unlocks(cx: &Cx, e: &Expr, arg: &Expr) -> bool {
    let E::Call {
        callee,
        args,
        ..
    } = &e.kind
    else {
        return false;
    };
    calls_noted(cx, callee, "Unlocks")
        && args
            .first()
            .is_some_and(|a| !a.spread && cx.same_tokens(a.value.span, arg.span))
}

/// Some statement in `s` (nested blocks included) unlocks `arg`.
fn releases(cx: &Cx, s: &Stmt, arg: &Expr) -> bool {
    super::any_stmt(
        s,
        &mut |t| matches!(&t.kind, S::Expr(e) if unlocks(cx, e, arg)),
    )
}

fn flow_list(cx: &Cx, arg: &Expr, list: &[Stmt]) -> Flow {
    for (k, s) in list.iter().enumerate() {
        match flow_stmt(cx, arg, s) {
            Flow::Locked => {}
            Flow::Unlocked => {
                let leaves = list[k + 1..]
                    .iter()
                    .any(|t| matches!(t.kind, S::Return { .. } | S::Break(_) | S::Continue(_)));
                return if leaves {
                    Flow::Exited
                } else {
                    Flow::Unlocked
                };
            }
            other => return other,
        }
    }
    Flow::Locked
}

fn flow_stmt(cx: &Cx, arg: &Expr, s: &Stmt) -> Flow {
    match &s.kind {
        S::Expr(e) if unlocks(cx, e, arg) => Flow::Unlocked,
        S::Defer {
            body, ..
        } => {
            if releases(cx, body, arg) {
                Flow::Unlocked
            } else {
                Flow::Locked
            }
        }
        S::Return {
            ..
        } if opaque(cx, s) => Flow::Unsure,
        S::Return {
            ..
        } => Flow::Leak(s.span),
        S::Break(_) | S::Continue(_) | S::Remove(_) => Flow::Unsure,
        S::Block(b) => flow_list(cx, arg, &b.stmts),
        S::If {
            then_branch,
            else_branch,
            ..
        } => {
            let then = flow_stmt(cx, arg, then_branch);
            let other = else_branch
                .as_ref()
                .map_or(Flow::Locked, |e| flow_stmt(cx, arg, e));
            combine(then, other)
        }
        S::Expr(_)
        | S::Decl(_)
        | S::Assign {
            ..
        }
        | S::Empty => {
            if opaque(cx, s) {
                Flow::Unsure
            } else {
                Flow::Locked
            }
        }
        // Loops, `switch`, `#if` and the rest: followed only when they neither unlock nor
        // do anything unseen.
        _ => {
            if opaque(cx, s) || releases(cx, s, arg) || defers_release(cx, s, arg) {
                return Flow::Unsure;
            }
            match first_return(s) {
                Some(at) => Flow::Leak(at),
                None => Flow::Locked,
            }
        }
    }
}

/// Some `defer` inside `s` releases `arg`.
fn defers_release(cx: &Cx, s: &Stmt, arg: &Expr) -> bool {
    super::any_stmt(
        s,
        &mut |t| matches!(&t.kind, S::Defer { body, .. } if releases(cx, body, arg)),
    )
}

/// The first `return` inside `s` (nested procedures not entered).
fn first_return(s: &Stmt) -> Option<Span> {
    let mut found = None;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(s), &mut stack, &mut |n, _| {
        if let Node::Stmt(t) = n
            && found.is_none()
            && matches!(t.kind, S::Return { .. })
        {
            found = Some(t.span);
        }
        found.is_none()
    });
    found
}

fn combine(a: Flow, b: Flow) -> Flow {
    match (a, b) {
        (Flow::Leak(at), _) | (_, Flow::Leak(at)) => Flow::Leak(at),
        (Flow::Unsure, _) | (_, Flow::Unsure) => Flow::Unsure,
        (Flow::Exited, other) | (other, Flow::Exited) => other,
        (Flow::Locked, Flow::Locked) => Flow::Locked,
        (Flow::Unlocked, Flow::Unlocked) => Flow::Unlocked,
        _ => Flow::Unsure,
    }
}
