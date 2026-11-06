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
//! Checks top-level acquisition statements along sequences, `if`/`else` and blocks.
//! Native-backed checked wrappers can supply exact recursive depth effects; explicit
//! `@Locks` and `@Unlocks` contracts cover other protocols. Arguments are compared as
//! written, ignoring whitespace. Every acquired recursive level needs one release;
//! a proven deferred release supplies one credit, including a defer earlier in the body.
//!
//! `@Locks` also declares intentional wrapper behavior: a procedure carrying it is
//! exempt. Inferring that a procedure returns held does not grant that exemption.
//! Unknown calls receiving the same argument keep the result uncertain. Loops, switches,
//! `#insert`, `using`, macro calls and mismatched branch depths retain the rule's limited
//! syntax coverage; this is not a general concurrency or lock-alias analysis.
use super::frees::opaque;
use super::{finding, statement_range, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::{Node, walk_node};
use jaic::ast::{Expr, ExprKind as E, Stmt, StmtKind as S};
use jaic::source::Span;

type Projection = Vec<super::lifetime_summary::Step>;

#[derive(Clone, PartialEq, Eq)]
struct Balance {
    formal: usize,
    required: u8,
    delta: i16,
    projection: Projection,
    native: bool,
    declared: bool,
}

struct Subject<'a> {
    expression: &'a Expr,
    projection: Projection,
    native: bool,
    allow_fix: bool,
}

/// What happens to a held lock along the statements that follow the call.
enum Flow {
    /// Still held when the statements run out.
    Locked(i16),
    /// Released on the way, and the path carries on.
    Unlocked,
    /// Released, and the path ends in a `return`, `break` or `continue` of its own.
    Exited,
    /// A `return` at this span leaves it held.
    Leak(Span),
    /// The scan cannot tell.
    Unsure,
}

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unpaired_lock",
    category: crate::config::Category::Suspicious,
    summary: "a lock acquisition with a path that leaves recursive levels held",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        let instances = cx.procs_with_header(p.header.span);
        let wrapper = !instances.is_empty()
            && instances
                .iter()
                .all(|&id| super::lifetime_summary::intentional_lock_wrapper(cx.compiler, id));
        if wrapper {
            continue;
        }
        for at in super::lifetime_summary::native_locks::findings(cx, &instances) {
            out.push(finding(
                at,
                "this native lock acquisition can leave recursive levels held".into(),
                Some("unlock it on every path, or defer the matching release".into()),
            ));
        }
        let list = &p.body.stmts;
        for (k, s) in list.iter().enumerate() {
            let Some((arg, call, depth)) = lock_call(cx, s) else {
                continue;
            };
            if arg.native {
                continue;
            }
            let credits = list[..k].iter().try_fold(0i16, |credit, stmt| {
                if let S::Defer {
                    body, ..
                } = &stmt.kind
                {
                    credit.checked_sub(deferred_delta(cx, body, &arg)?)
                } else {
                    Some(credit)
                }
            });
            let Some(credits) = credits else {
                continue;
            };
            let shown = cx.whole_src(arg.expression).trim();
            // Deferring the unlock is offered when nothing below unlocks by hand (a second
            // unlock would then run twice) and the call is spelled `lock(...)`.
            let defer = (arg.allow_fix
                && depth == 1
                && !list[k + 1..].iter().any(|t| releases(cx, t, &arg)))
            .then(|| deferred_unlock(cx, s))
            .flatten();
            let (message, help, at) = match flow_list(cx, &arg, &list[k + 1..], depth - credits) {
                Flow::Locked(_) => (
                    format!(
                        "`{shown}` is locked here but the procedure can end without unlocking it"
                    ),
                    "unlock it on every path, or `defer` the unlock right after locking",
                    call,
                ),
                Flow::Leak(at) => (
                    format!("this `return` leaves `{shown}` locked"),
                    "unlock it before returning, or `defer` the unlock right after locking",
                    at,
                ),
                Flow::Unlocked | Flow::Exited | Flow::Unsure => continue,
            };
            out.push(match defer {
                Some(text) => {
                    let end = statement_range(cx, s.span).1;
                    let (_, indent) = cx.line_start(s.span.start as usize);
                    suggesting(
                        at,
                        message,
                        help.into(),
                        "unlock when the procedure ends",
                        vec![Edit {
                            start: end,
                            end,
                            text: format!("\n{indent}{text}"),
                        }],
                    )
                }
                None => finding(at, message, Some(help.into())),
            });
        }
    }
}

/// `defer unlock(...);` for a statement `lock(...);`: the same arguments, with the callee's
/// last word `lock` turned into `unlock`. `None` for any other spelling.
fn deferred_unlock(cx: &Cx, s: &Stmt) -> Option<String> {
    let S::Expr(e) = &s.kind else {
        return None;
    };
    let E::Call {
        callee, ..
    } = &e.kind
    else {
        return None;
    };
    let name = cx.whole_src(callee).trim();
    let stem = name.strip_suffix("lock")?;
    if stem.ends_with("un") || stem.ends_with(|c: char| c.is_alphanumeric()) {
        return None;
    }
    let args = cx.text.get(cx.whole(callee).1..e.span.end as usize)?.trim();
    Some(format!("defer {stem}unlock{args};"))
}

/// The statement is only a call of a `@Locks` procedure with an argument: the first argument
/// and the call's span.
fn lock_call<'a>(cx: &Cx, s: &'a Stmt) -> Option<(Subject<'a>, Span, i16)> {
    let S::Expr(e) = &s.kind else {
        return None;
    };
    let E::Call {
        callee, ..
    } = &e.kind
    else {
        return None;
    };
    let balance = call_balance(cx, callee)?;
    let actual = super::call_bindings::argument(cx, e, balance.formal)?;
    (balance.required == 0 && balance.delta > 0).then_some((
        Subject {
            expression: actual,
            projection: balance.projection,
            native: balance.native,
            allow_fix: balance.declared && balance.formal == 0,
        },
        e.span,
        balance.delta,
    ))
}

/// The expression is a call of an `@Unlocks` procedure on `arg`.
fn unlocks(cx: &Cx, e: &Expr, arg: &Subject<'_>) -> bool {
    expression_balance(cx, e, arg).is_some_and(|(_, delta)| delta < 0)
}

/// Some statement in `s` (nested blocks included) unlocks `arg`.
fn releases(cx: &Cx, s: &Stmt, arg: &Subject<'_>) -> bool {
    super::any_stmt(
        s,
        &mut |t| matches!(&t.kind, S::Expr(e) if unlocks(cx, e, arg)),
    )
}

fn flow_list(cx: &Cx, arg: &Subject<'_>, list: &[Stmt], mut depth: i16) -> Flow {
    for s in list {
        match flow_stmt(cx, arg, s, depth) {
            Flow::Locked(next) => depth = next,
            Flow::Unlocked => depth = 0,
            other => return other,
        }
    }
    if depth > 0 {
        Flow::Locked(depth)
    } else {
        Flow::Unlocked
    }
}

fn flow_stmt(cx: &Cx, arg: &Subject<'_>, s: &Stmt, depth: i16) -> Flow {
    match &s.kind {
        S::Expr(e) if expression_balance(cx, e, arg).is_some() => {
            let (required, delta) = expression_balance(cx, e, arg).unwrap();
            if depth < i16::from(required) {
                return Flow::Unsure;
            }
            match depth.checked_add(delta) {
                Some(0) => Flow::Unlocked,
                Some(next) => Flow::Locked(next),
                None => Flow::Unsure,
            }
        }
        S::Defer {
            body, ..
        } => match deferred_delta(cx, body, arg).and_then(|delta| depth.checked_add(delta)) {
            Some(next) => Flow::Locked(next),
            None => Flow::Unsure,
        },
        S::Return {
            ..
        } if opaque(cx, s) => Flow::Unsure,
        S::Return {
            ..
        } => {
            if depth > 0 {
                Flow::Leak(s.span)
            } else {
                Flow::Exited
            }
        }
        S::Break(_) | S::Continue(_) | S::Remove(_) => Flow::Unsure,
        S::Block(b) => flow_list(cx, arg, &b.stmts, depth),
        S::If {
            then_branch,
            else_branch,
            ..
        } => {
            let then = flow_stmt(cx, arg, then_branch, depth);
            let other = else_branch
                .as_ref()
                .map_or(Flow::Locked(depth), |e| flow_stmt(cx, arg, e, depth));
            combine(then, other)
        }
        S::Expr(_)
        | S::Decl(_)
        | S::Assign {
            ..
        }
        | S::Empty => {
            if opaque(cx, s) || matches!(&s.kind, S::Expr(e) if unknown_call_on_arg(cx, e, arg)) {
                Flow::Unsure
            } else {
                Flow::Locked(depth)
            }
        }
        _ => {
            if opaque(cx, s) || releases(cx, s, arg) || defers_release(cx, s, arg) {
                return Flow::Unsure;
            }
            match first_return(s) {
                Some(at) if depth > 0 => Flow::Leak(at),
                _ => Flow::Locked(depth),
            }
        }
    }
}

/// Deferred release credit must hold on every path, rather than merely occur
/// somewhere inside the deferred statement. Keep unsupported control flow unknown.
fn deferred_delta(cx: &Cx, stmt: &Stmt, arg: &Subject<'_>) -> Option<i16> {
    match &stmt.kind {
        S::Expr(e) => match expression_balance(cx, e, arg) {
            Some((_, delta)) if delta <= 0 => Some(delta),
            Some(_) => None,
            None if releases(cx, stmt, arg)
                || opaque(cx, stmt)
                || unknown_call_on_arg(cx, e, arg) =>
            {
                None
            }
            None => Some(0),
        },
        S::Block(block) => block.stmts.iter().try_fold(0i16, |prior, stmt| {
            prior.checked_add(deferred_delta(cx, stmt, arg)?)
        }),
        S::If {
            then_branch,
            else_branch,
            ..
        } => {
            let a = deferred_delta(cx, then_branch, arg)?;
            let b = else_branch
                .as_ref()
                .map_or(Some(0), |s| deferred_delta(cx, s, arg))?;
            (a == b).then_some(a)
        }
        S::Empty => Some(0),
        _ if releases(cx, stmt, arg) || opaque(cx, stmt) => None,
        _ => Some(0),
    }
}

/// Classify effects independently of the annotation that grants wrapper intent.
fn call_balance(cx: &Cx, callee: &Expr) -> Option<Balance> {
    let effect = |id| {
        let declared = super::lifetime_summary::lock_contract(cx.compiler, id);
        if let Some((formal, required, delta, projection)) =
            super::lifetime_summary::lock_balance(cx, id)
            && declared.is_none_or(|d| d == (formal, required, delta))
        {
            return Some(Balance {
                formal,
                required,
                delta,
                projection,
                native: true,
                declared: declared.is_some(),
            });
        }
        declared.map(|(formal, required, delta)| Balance {
            formal,
            required,
            delta,
            projection: Vec::new(),
            native: false,
            declared: true,
        })
    };
    let procs = super::callee_procs(cx, callee);
    let span = match &callee.kind {
        E::Member(_, name) => name.span,
        _ => callee.span,
    };
    if let Some(chosen) = cx.compiler.ide_resolved_overload(span, &procs) {
        return effect(chosen);
    }
    let mut selected = None;
    for &id in &*procs {
        let current = effect(id)?;
        if selected.as_ref().is_some_and(|prior| prior != &current) {
            return None;
        }
        selected = Some(current);
    }
    selected
}

fn unknown_call_on_arg(cx: &Cx, e: &Expr, arg: &Subject<'_>) -> bool {
    let E::Call {
        callee,
        args,
        ..
    } = &e.kind
    else {
        return false;
    };
    if !args
        .iter()
        .any(|a| !a.spread && cx.same_tokens(&a.value, arg.expression))
    {
        return false;
    }
    let Some(balance) = call_balance(cx, callee) else {
        return true;
    };
    if same_projection(cx, arg, &balance) || (balance.required == 0 && balance.delta == 0) {
        return false;
    }
    if !arg.native || !balance.native {
        return true;
    }
    let Some(ty) = cx.ty(arg.expression) else {
        return true;
    };
    let ty = cx.compiler.ty.types.pointee(ty).unwrap_or(ty);
    match (
        lock_offset(cx, ty, &arg.projection),
        lock_offset(cx, ty, &balance.projection),
    ) {
        (Some(a), Some(b)) => a == b,
        _ => true,
    }
}

/// Fixed-array storage is inline in its containing field. Descriptor storage may
/// instead follow a shared backing pointer and cannot establish field disjointness.
fn lock_projection(
    cx: &Cx,
    mut ty: jaic::types::TypeId,
    steps: &[super::lifetime_summary::Step],
) -> Option<Vec<super::lifetime_places::Projection>> {
    use super::lifetime_places::Projection;
    use super::lifetime_summary::Step;
    let mut out = Vec::new();
    for step in steps {
        match step {
            Step::Field(name) => {
                let st = cx.compiler.ty.types.as_struct(ty)?;
                ty = cx
                    .compiler
                    .ty
                    .types
                    .struct_info(st)
                    .fields
                    .iter()
                    .find(|f| f.name == Some(*name))?
                    .ty;
                out.push(Projection::Field(*name));
            }
            Step::Deref => {
                ty = cx.compiler.ty.types.pointee(ty)?;
                out.push(Projection::Deref);
            }
            Step::Storage
                if matches!(
                    cx.compiler.ty.types.kind(ty),
                    jaic::types::TypeKind::Array {
                        kind: jaic::types::ArrayKind::Fixed(_),
                        ..
                    }
                ) => {}
            Step::Element(index) => {
                let jaic::types::TypeKind::Array {
                    elem, ..
                } = cx.compiler.ty.types.kind(ty)
                else {
                    return None;
                };
                ty = *elem;
                out.push(Projection::Element(*index));
            }
            Step::Storage => return None,
        }
    }
    Some(out)
}

fn lock_offset(
    cx: &Cx,
    mut ty: jaic::types::TypeId,
    steps: &[super::lifetime_summary::Step],
) -> Option<u64> {
    let path = lock_projection(cx, ty, steps)?;
    let mut offset = 0u64;
    for projection in path {
        match projection {
            super::lifetime_places::Projection::Field(name) => {
                let st = cx.compiler.ty.types.as_struct(ty)?;
                let field = cx
                    .compiler
                    .ty
                    .types
                    .struct_info(st)
                    .fields
                    .iter()
                    .find(|f| f.name == Some(name))?;
                offset = offset.checked_add(field.offset)?;
                ty = field.ty;
            }
            super::lifetime_places::Projection::Element(index) => {
                let jaic::types::TypeKind::Array {
                    kind: jaic::types::ArrayKind::Fixed(count),
                    elem,
                } = cx.compiler.ty.types.kind(ty)
                else {
                    return None;
                };
                let index = u64::try_from(index).ok()?;
                if index >= *count {
                    return None;
                }
                offset =
                    offset.checked_add(index.checked_mul(cx.compiler.ty.types.size_of(*elem))?)?;
                ty = *elem;
            }
            super::lifetime_places::Projection::Deref
            | super::lifetime_places::Projection::DynamicElement(_)
            | super::lifetime_places::Projection::Inline {
                ..
            } => return None,
        }
    }
    Some(offset)
}

fn same_projection(cx: &Cx, arg: &Subject<'_>, balance: &Balance) -> bool {
    if arg.projection == balance.projection {
        return true;
    }
    if !arg.native || !balance.native {
        return false;
    }
    let Some(ty) = cx.ty(arg.expression) else {
        return false;
    };
    let ty = cx.compiler.ty.types.pointee(ty).unwrap_or(ty);
    lock_offset(cx, ty, &arg.projection)
        .zip(lock_offset(cx, ty, &balance.projection))
        .is_some_and(|(a, b)| a == b)
}

fn expression_balance(cx: &Cx, e: &Expr, arg: &Subject<'_>) -> Option<(u8, i16)> {
    let E::Call {
        callee, ..
    } = &e.kind
    else {
        return None;
    };
    let balance = call_balance(cx, callee)?;
    let actual = super::call_bindings::argument(cx, e, balance.formal)?;
    if !cx.same_tokens(actual, arg.expression) {
        return None;
    }
    same_projection(cx, arg, &balance).then_some((balance.required, balance.delta))
}

/// Some `defer` inside `s` releases `arg`.
fn defers_release(cx: &Cx, s: &Stmt, arg: &Subject<'_>) -> bool {
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
        (Flow::Locked(a), Flow::Locked(b)) if a == b => Flow::Locked(a),
        (Flow::Unlocked, Flow::Unlocked) => Flow::Unlocked,
        _ => Flow::Unsure,
    }
}
