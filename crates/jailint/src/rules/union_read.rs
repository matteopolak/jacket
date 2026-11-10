//! `union_read`: a read of a union member with nothing nearby saying it is the live one.
//!
//! A union has no tag: its members share the same bytes, and nothing records which was
//! written last. Reading the wrong one gives garbage, not an error, and the compiler cannot
//! check it either (it keeps a union's words as written when it freezes a value at compile
//! time). A struct that wraps a union usually carries the tag beside it (`kind: Kind;` next to
//! `union { ... }`); the read is safe only where code has looked at that tag.
//!
//! Fires on a read of a member of an untagged union, a named `union` or an anonymous one
//! inside a struct, as `v.member`, unless one of these holds for the same `v` (written the
//! same way, without calls):
//!
//! - the read is inside an `if`, `while`, `ifx` or `switch` whose condition reads another
//!   member of `v` (its tag) or passes `v` to a call (a predicate such as `is_table(v)`), or
//!   after an `&&` or `||` whose left side does;
//! - an earlier statement of an enclosing block is an `if` or `assert(...)` on such a member
//!   (an early exit such as `if v.kind != .TEXT return "";`);
//! - earlier in the procedure, `v` or a place inside one of its union members is assigned,
//!   `v` is declared with a value, or `*v` is taken (a call may fill it): the live member is
//!   decided in view (a pun such as `b.f = x; return b.u;` is deliberate);
//! - the member is part of the union's common initial sequence: every struct member begins
//!   with the same fields, as C event unions do (`event.type`, `event.xany.window`). Such a
//!   member also serves as the tag.
//!
//! Writes, `*v.member` and compound assignments are not reads. Only bodies the compiler fully
//! checked, outside polymorphs and macros, are looked at. The rule only sees the shape of the
//! code: a check of the wrong tag value still counts as a check. Where the caller checks the
//! tag, an `assert` on it at the top of the helper states the requirement and satisfies the
//! rule.
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk};
use jaic::ast::{BinOp, Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};
use jaic::intern::Sym;
use jaic::types::{ArrayKind, StructId, TypeId, TypeKind, Types};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "union_read",
    category: crate::config::Category::Pedantic,
    summary: "a union member read without checking which one is live",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        let body = &p.body.stmts;
        walk(body, &mut |n, chain| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Member(base, member) = &e.kind else {
                return true;
            };
            if super::place_use(e, chain) != super::PlaceUse::Read || !super::pure(base) {
                return true;
            }
            let Some(read) = union_member(cx, base, member.name) else {
                return true;
            };
            let then = match chain.last() {
                Some(Node::Expr(Expr {
                    kind: E::Member(of, field),
                    ..
                })) if of.span == e.span => Some(field.name),
                _ => None,
            };
            if in_common_sequence(&cx.compiler.ty.types, &read, member.name, then) {
                return true;
            }
            if guarded(cx, &read, e, chain, body) || written_before(cx, &read, e, body) {
                return true;
            }
            let shown = cx.src(e.span).trim();
            let help = match tag_of(cx, &read) {
                Some(tag) => format!(
                    "check `{}.{tag}` first, or write `{shown}` here before reading it",
                    cx.src(base.span).trim()
                ),
                None => format!(
                    "keep the union in a struct beside a tag and check it, or write `{shown}` here before reading it"
                ),
            };
            out.push(super::finding(
                e.span,
                format!("`{shown}` reads a union member, but nothing here says it is the live one"),
                Some(help),
            ));
            true
        });
    }
}

/// A read `base.name` of a union member.
struct Read<'a> {
    base: &'a Expr,
    name: Sym,
    /// The struct `base` is (through a pointer).
    holder: StructId,
    /// The union the member belongs to.
    union: StructId,
    /// When `base` is itself a whole union reached as `outer.field`: `outer`, whose other
    /// members may hold the tag.
    outer: Option<(&'a Expr, Sym)>,
}

impl Read<'_> {
    /// `name` is one of the union's members, as seen from the holder.
    fn in_union(&self, types: &Types, name: Sym) -> bool {
        find(types, self.holder, name, None, 0) == Some(Some(self.union))
    }
}

fn union_member<'a>(cx: &Cx, base: &'a Expr, name: Sym) -> Option<Read<'a>> {
    let types = &cx.compiler.ty.types;
    let holder = struct_of(types, cx.ty(base)?)?;
    let union = find(types, holder, name, None, 0)??;
    let outer = match &base.kind {
        E::Member(outer, field) if types.struct_info(holder).is_union => {
            Some((&**outer, field.name))
        }
        _ => None,
    };
    Some(Read {
        base,
        name,
        holder,
        union,
        outer,
    })
}

/// The likely tag beside the union: the holder's first enum or integer member outside it.
fn tag_of(cx: &Cx, read: &Read) -> Option<Sym> {
    let types = &cx.compiler.ty.types;
    let info = types.struct_info(read.holder);
    if info.is_union {
        return None;
    }
    info.fields
        .iter()
        .filter(|f| !f.overlay)
        .filter_map(|f| f.name.map(|n| (n, f.ty)))
        .find(|&(n, ty)| {
            !read.in_union(types, n)
                && matches!(
                    types.kind(types.repr_struct(ty)),
                    jaic::types::TypeKind::Enum(_) | jaic::types::TypeKind::Int { .. }
                )
        })
        .map(|(n, _)| n)
}

/// The struct a value of type `ty` is, through one pointer and distinct types.
fn struct_of(types: &Types, ty: TypeId) -> Option<StructId> {
    let ty = types.pointee(ty).unwrap_or(ty);
    types.as_struct(types.repr_struct(ty))
}

/// Where member `name` of struct `s` is: `Some(Some(u))` in union `u`, `Some(None)` in no
/// union, `None` not a member. Anonymous and `using` members are looked through.
fn find(
    types: &Types,
    s: StructId,
    name: Sym,
    inside: Option<StructId>,
    depth: u32,
) -> Option<Option<StructId>> {
    let info = types.struct_info(s);
    // A tagged union's struct is not `is_union`: the tag says which member is live.
    let here = if info.is_union {
        Some(s)
    } else {
        inside
    };
    if info.fields.iter().any(|f| f.name == Some(name)) {
        return Some(here);
    }
    if depth > 8 {
        return None;
    }
    info.fields
        .iter()
        .filter(|f| f.name.is_none() || f.using)
        .filter_map(|f| types.as_struct(types.repr_struct(f.ty)))
        .find_map(|t| find(types, t, name, here, depth + 1))
}

/// The read is inside, or after, a check of a tag beside the union.
fn guarded(cx: &Cx, read: &Read, e: &Expr, chain: &[Node], body: &[Stmt]) -> bool {
    let mut child = Node::Expr(e);
    for &parent in chain.iter().rev() {
        let start = child_start(child);
        let checked = match parent {
            Node::Expr(p) => match &p.kind {
                E::Ifx {
                    cond, ..
                } => !within(cond, start) && mentions_tag(cx, read, cond),
                E::Binary(BinOp::And | BinOp::Or, left, _) => {
                    !within(left, start) && mentions_tag(cx, read, left)
                }
                _ => false,
            },
            Node::Stmt(s) => {
                (match &s.kind {
                    S::If {
                        cond, ..
                    } => !within(cond, start) && mentions_tag(cx, read, cond),
                    S::While(w) => !within(&w.cond, start) && mentions_tag(cx, read, &w.cond),
                    S::Switch(sw) => !within(&sw.value, start) && mentions_tag(cx, read, &sw.value),
                    S::Block(b) => checked_before(cx, read, &b.stmts, start),
                    _ => false,
                }) || case_checked_before(cx, read, s, start)
            }
        };
        if checked {
            return true;
        }
        child = parent;
    }
    checked_before(cx, read, body, child_start(child))
}

fn child_start(n: Node) -> u32 {
    match n {
        Node::Expr(e) => e.span.start,
        Node::Stmt(s) => s.span.start,
    }
}

fn within(e: &Expr, at: u32) -> bool {
    e.span.start <= at && at < e.span.end
}

/// The case of switch `s` holding offset `at` checks the tag in an earlier statement.
fn case_checked_before(cx: &Cx, read: &Read, s: &Stmt, at: u32) -> bool {
    let S::Switch(sw) = &s.kind else {
        return false;
    };
    sw.cases
        .iter()
        .find(|c| c.span.start <= at && at < c.span.end)
        .is_some_and(|c| checked_before(cx, read, &c.body, at))
}

/// A statement of `list` before offset `at` is an `if` or an `assert(...)` (or `M.assert`) on
/// the tag.
fn checked_before(cx: &Cx, read: &Read, list: &[Stmt], at: u32) -> bool {
    list.iter()
        .take_while(|s| s.span.end <= at)
        .any(|s| match &s.kind {
            S::If {
                cond, ..
            } => mentions_tag(cx, read, cond),
            S::Expr(Expr {
                kind:
                    E::Call {
                        callee,
                        args,
                        ..
                    },
                ..
            }) => {
                let name = match &callee.kind {
                    E::Ident(n) => Some(*n),
                    E::Member(_, n) => Some(n.name),
                    _ => None,
                };
                name.is_some_and(|n| n.as_str() == "assert")
                    && args
                        .first()
                        .is_some_and(|a| mentions_tag(cx, read, &a.value))
            }
            _ => false,
        })
}

/// `e` reads a member of the read's value other than the union's own, or (for a whole union
/// reached as `outer.field`) another member of `outer`, or hands the value to a call, a
/// predicate that looks at the tag for it (`if is_table(v)`).
fn mentions_tag(cx: &Cx, read: &Read, e: &Expr) -> bool {
    let types = &cx.compiler.ty.types;
    let mut found = false;
    jaic::ast::walk::walk_node(Node::Expr(e), &mut Vec::new(), &mut |n, _| {
        if found {
            return false;
        }
        if let Some(Expr {
            kind: E::Call {
                args, ..
            },
            ..
        }) = n.expr()
        {
            found = args.iter().any(|a| {
                let value = match &a.value.kind {
                    E::Unary(UnOp::Star, inner) => inner,
                    _ => &a.value,
                };
                cx.same_tokens(value, read.base)
            });
        }
        if let Some(Expr {
            kind: E::Member(q, f),
            ..
        }) = n.expr()
            && super::pure(q)
            && !found
        {
            let tag = |name: Sym, then: Option<Sym>| {
                name != read.name
                    && (!read.in_union(types, name) || in_common_sequence(types, read, name, then))
            };
            found = (cx.same_tokens(q, read.base) && tag(f.name, None))
                || matches!(&q.kind, E::Member(inner, m)
                    if cx.same_tokens(inner, read.base) && tag(m.name, Some(f.name)))
                || read
                    .outer
                    .is_some_and(|(outer, field)| f.name != field && cx.same_tokens(q, outer));
        }
        !found
    });
    found
}

/// Before the read, the procedure assigns to the read's value or to a place inside one of its
/// union members, declares it with a value, or takes the address of either (a call it is
/// passed to may fill it: `XNextEvent(display, *event)`).
fn written_before(cx: &Cx, read: &Read, e: &Expr, body: &[Stmt]) -> bool {
    let at = e.span.start;
    let mut found = false;
    walk(body, &mut |n, _| {
        if found {
            return false;
        }
        match n {
            Node::Stmt(s) if s.span.start >= at => return false,
            Node::Stmt(s) if s.span.end <= at => match &s.kind {
                S::Assign {
                    lhs, ..
                } => found = lhs.iter().any(|l| writes_into(cx, read, l)),
                S::Decl(d) if d.value.is_some() => {
                    found = matches!(&read.base.kind, E::Ident(name)
                        if d.names.iter().any(|n| n.name == *name));
                }
                _ => {}
            },
            Node::Expr(x) if x.span.end <= at => {
                if let E::Unary(UnOp::Star, inner) = &x.kind {
                    found = writes_into(cx, read, inner);
                }
            }
            _ => {}
        }
        !found
    });
    found
}

/// The place `l` is the read's value, or lies inside one of its union members.
fn writes_into(cx: &Cx, read: &Read, mut l: &Expr) -> bool {
    let types = &cx.compiler.ty.types;
    loop {
        if cx.same_tokens(l, read.base) {
            return true;
        }
        match &l.kind {
            E::Member(q, f) => {
                if cx.same_tokens(q, read.base) {
                    return read.in_union(types, f.name);
                }
                l = q;
            }
            E::Index(q, _) => l = q,
            _ => return false,
        }
    }
}

/// How many leading fields of union member `name` every other member of `union` begins with
/// too: C's common initial sequence, which reads the same whichever member was written (the
/// `type` and `xany` of an X11 event). Fixed arrays are padding views and are not compared. A
/// plain member covers only a struct's first field, and shares nothing with another plain
/// member, so `radius: float; side: float` share nothing.
fn common_prefix(types: &Types, union: StructId, name: Sym) -> usize {
    let info = types.struct_info(union);
    let Some(member) = info.fields.iter().find(|f| f.name == Some(name)) else {
        return 0;
    };
    let Some((lead, is_struct)) = leading_fields(types, member.ty) else {
        return 0;
    };
    let mut shared = lead.len();
    for other in info.fields.iter().filter(|f| f.name != Some(name)) {
        let Some((other_lead, other_struct)) = leading_fields(types, other.ty) else {
            continue;
        };
        if !is_struct && !other_struct {
            return 0;
        }
        // A plain member covers only the first field's bytes.
        if !other_struct {
            if lead.first() != other_lead.first() {
                return 0;
            }
            continue;
        }
        let same = lead
            .iter()
            .zip(&other_lead)
            .take_while(|(a, b)| a == b)
            .count();
        shared = shared.min(same);
    }
    shared
}

/// The (offset, type) of each field of a struct type, or of a plain type as one field; `None`
/// for a fixed array. The flag says whether it was a struct.
fn leading_fields(types: &Types, ty: TypeId) -> Option<(Vec<(u64, TypeId)>, bool)> {
    let ty = types.repr_struct(ty);
    match types.kind(ty) {
        TypeKind::Array {
            kind: ArrayKind::Fixed(_),
            ..
        } => None,
        TypeKind::Struct(s) if !types.struct_info(*s).is_union => Some((
            types
                .struct_info(*s)
                .fields
                .iter()
                .filter(|f| !f.overlay)
                .map(|f| (f.offset, f.ty))
                .collect(),
            true,
        )),
        _ => Some((vec![(0, ty)], false)),
    }
}

/// `member` of the read's union, or field `then` of it, lies in the common initial sequence.
fn in_common_sequence(types: &Types, read: &Read, member: Sym, then: Option<Sym>) -> bool {
    let info = types.struct_info(read.union);
    let Some(field) = info.fields.iter().find(|f| f.name == Some(member)) else {
        return false;
    };
    let shared = common_prefix(types, read.union, member);
    let Some((lead, _)) = leading_fields(types, field.ty) else {
        return false;
    };
    match then {
        None => shared > 0 && shared == lead.len(),
        Some(then) => {
            let Some(s) = types.as_struct(types.repr_struct(field.ty)) else {
                return false;
            };
            types
                .struct_info(s)
                .fields
                .iter()
                .filter(|f| !f.overlay)
                .position(|f| f.name == Some(then))
                .is_some_and(|i| i < shared)
        }
    }
}
