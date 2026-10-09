//! `index_only_loop`: `for i: 0..xs.count-1 { ... xs[i] ... }`.
//!
//! Counting through an array's indices to read its elements says more than the loop means
//! and leaves room for off-by-one mistakes in the bound. `for xs` visits the same elements as
//! `it` (and `for *xs` gives a pointer to each, for writing). When the index is needed too,
//! `it_index` holds it.
//!
//! Fires when the range starts at `0` and ends at `xs.count - 1` for an array `xs` (by name or
//! member path), and the loop variable is used to index `xs`. Kept quiet when the loop could
//! change `xs` itself (its address is taken, it is assigned, the body has `remove`), since
//! `for xs` re-reads the count each iteration while the range was fixed at the start. Also
//! quiet for loops that walk a grid (`for c: 0..xs[i].count - 1` inside), index other arrays
//! with the variable or compute with it inside an index (`ys[i]`, `xs[i + 1]`). Filling
//! elements from the index (`xs[i] = sin(i * step)`) is reported: `for *xs` with `it_index`
//! in the value. Writes to elements (`xs[i] = v`, `xs[i].f = v`, `*xs[i]`, passing `xs` to a
//! call) make the suggestion `for *xs`. Constant arrays (`xs :: T.[...]`) are reported too, except where the
//! suggestion would be `for *xs`.
use super::{PlaceUse, is_call_argument, place_use};
use crate::syntax::{Cx, Node, is_path, root_ident, squash, walk_node};
use crate::{Edit, Finding, Fix};
use jaic::ast::{BinOp, Expr, ExprKind as E, For, ForOver, Stmt, StmtKind as S};
use jaic::intern::Sym;
use jaic::types::TypeKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        crate::syntax::walk(&p.body.stmts, &mut |n, _| {
            if let Some(s) = n.stmt()
                && let S::For(f) = &s.kind
                && let Some(found) = examine(cx, s, f)
            {
                out.push(found);
            }
            true
        });
    }
}

/// The array of `0..arr.count-1`.
fn counted_array(f: &For) -> Option<(&Expr, &Expr, &Expr)> {
    let ForOver::Range(lo, hi) = &f.over else {
        return None;
    };
    if !matches!(lo.kind, E::Int(0)) {
        return None;
    }
    let E::Binary(BinOp::Sub, count, one) = &hi.kind else {
        return None;
    };
    if !matches!(one.kind, E::Int(1)) {
        return None;
    }
    let E::Member(arr, member) = &count.kind else {
        return None;
    };
    (member.name.as_str() == "count" && is_path(arr)).then_some((&**arr, lo, hi))
}

struct Uses {
    /// `arr[i]` reads/writes: the index expression and how it is used.
    elements: Vec<(jaic::source::Span, Replace)>,
    /// Other uses of the loop variable.
    others: Vec<jaic::source::Span>,
    /// Elements are (or may be) written: iterate by pointer.
    by_pointer: bool,
    /// An `it`/`it_index` already in the body would be captured by the new loop.
    captures_it: bool,
    captures_it_index: bool,
}

#[derive(Clone, Copy)]
enum Replace {
    /// Replace `node` with `it` (value) or `it.*` (pointer).
    Element,
    /// Replace `node` (an address-of `*arr[i]`) with `it`.
    Address,
    /// `arr[i].member`: `it` serves both ways.
    Member,
}

fn examine(cx: &Cx, stmt: &Stmt, f: &For) -> Option<Finding> {
    if f.by_pointer
        || f.pointer_if.is_some()
        || f.reverse_if.is_some()
        || f.iterator.is_some()
        || f.backtick_names
        || f.index.is_some()
        || !f.flags.is_empty()
    {
        return None;
    }
    let (arr, lo, hi) = counted_array(f)?;
    // Arrays only: a string or a struct with `for_expansion` iterates differently.
    let arr_ty = cx.ty(arr)?;
    if !matches!(cx.compiler.types.kind(arr_ty), TypeKind::Array { .. }) {
        return None;
    }
    // A constant array (`xs :: int.[1, 2]`) iterates like any other, but has no elements to
    // point at: `for *xs` is not an option for one.
    let constant = cx.constant(arr);
    let var = f.it.map_or_else(|| Sym::intern("it"), |i| i.name);
    let loop_vars: Vec<_> = cx
        .facts
        .locals
        .get(&stmt.span)
        .map(|v| {
            v.iter()
                .copied()
                .filter(|&e| cx.compiler.entity(e).name == var)
                .collect()
        })
        .unwrap_or_default();
    if loop_vars.is_empty() {
        return None;
    }
    let arr_text = squash(cx.src(arr.span));
    let (root_name, root_span) = root_ident(arr)?;
    let root_entities = cx.entities(root_span).to_vec();
    let mut uses = Uses {
        elements: Vec::new(),
        others: Vec::new(),
        by_pointer: false,
        captures_it: false,
        captures_it_index: false,
    };
    let it = Sym::intern("it");
    let it_index = Sym::intern("it_index");
    let mut bail = false;
    // Spans of nested loops that rebind `it` / `it_index`.
    let mut rebinds_it: Vec<jaic::source::Span> = Vec::new();
    let mut rebinds_index: Vec<jaic::source::Span> = Vec::new();
    let mut stack = Vec::new();
    walk_node(Node::Stmt(&f.body), &mut stack, &mut |n, chain| {
        if bail {
            return false;
        }
        match n {
            Node::Stmt(s) => {
                match &s.kind {
                    S::Remove(_) => bail = true,
                    S::For(inner) => {
                        if inner.it.is_none_or(|i| i.name == it) {
                            rebinds_it.push(s.span);
                        }
                        if inner.index.is_none_or(|i| i.name == it_index) {
                            rebinds_index.push(s.span);
                        }
                    }
                    _ => {}
                }
                true
            }
            Node::Expr(e) => {
                let E::Ident(name) = &e.kind else {
                    return true;
                };
                let inside = |list: &[jaic::source::Span]| {
                    list.iter()
                        .any(|s| s.start <= e.span.start && e.span.end <= s.end)
                };
                let ents = cx.entities(e.span);
                if *name == var {
                    if ents.is_empty() {
                        // Not checked (an `#if` left it out): cannot tell what it names.
                        bail = true;
                        return false;
                    }
                    if !ents.iter().any(|x| loop_vars.contains(x)) {
                        // Another variable of the same name.
                        if *name == it && !inside(&rebinds_it) {
                            uses.captures_it = true;
                        }
                        return true;
                    }
                    if place_use(e, chain) != PlaceUse::Read {
                        bail = true;
                        return false;
                    }
                    match element_access(cx, e, chain, &arr_text) {
                        Some((span, how, written)) => {
                            // `for c: 0..grid[r].count - 1`: walking a grid by row and
                            // column reads best with both indices.
                            let in_inner_range = chain.iter().any(|a| {
                                matches!(a, Node::Stmt(Stmt { kind: S::For(g), .. })
                                    if matches!(&g.over, ForOver::Range(lo, hi)
                                        if lo.span.start <= e.span.start && e.span.end <= hi.span.end))
                            });
                            if in_inner_range {
                                bail = true;
                                return false;
                            }
                            uses.by_pointer |= written;
                            if inside(&rebinds_it) {
                                uses.captures_it = true;
                            }
                            uses.elements.push((span, how));
                        }
                        None => {
                            // Indexing another array with it (`xs[i] + ys[i]`), or index
                            // arithmetic (`xs[i + 1]`): a counting loop says that best.
                            let in_index = chain.iter().any(|a| {
                                matches!(a, Node::Expr(Expr { kind: E::Index(_, idx), .. })
                                    if idx.span.start <= e.span.start && e.span.end <= idx.span.end)
                            });
                            if in_index {
                                bail = true;
                                return false;
                            }
                            if inside(&rebinds_index) {
                                uses.captures_it_index = true;
                            }
                            uses.others.push(e.span);
                        }
                    }
                    return true;
                }
                if *name == it && !inside(&rebinds_it) {
                    uses.captures_it = true;
                }
                if *name == it_index && !inside(&rebinds_index) {
                    uses.captures_it_index = true;
                }
                if *name == root_name {
                    let same = ents.is_empty()
                        || root_entities.is_empty()
                        || ents.iter().any(|x| root_entities.contains(x));
                    if same && !part_of_element(cx, e, chain, &arr_text, var) {
                        match array_use(e, chain, arr) {
                            ArrayUse::Read => {}
                            ArrayUse::MayWriteElements => uses.by_pointer = true,
                            ArrayUse::Changes => bail = true,
                        }
                    }
                }
                true
            }
        }
    });
    if bail || uses.elements.is_empty() || (constant && uses.by_pointer) {
        return None;
    }
    let arr_src = cx.src(arr.span).trim();
    let header_start = f.it.map_or(lo.span.start, |i| i.span.start) as usize;
    let header = format!(
        "{}{arr_src}",
        if uses.by_pointer {
            "*"
        } else {
            ""
        }
    );
    let index_name = if var == it {
        "it"
    } else {
        var.as_str()
    };
    let only = uses.others.is_empty();
    let (message, help) = if only {
        (
            format!("`{index_name}` is only used to index `{arr_src}`"),
            if uses.by_pointer {
                format!("loop over pointers to the elements: `for {header}` and `it`")
            } else {
                format!("loop over the elements: `for {header}` and `it`")
            },
        )
    } else {
        (
            format!("`{index_name}` counts through `{arr_src}` to index it"),
            format!("loop over the elements: `for {header}`, with `it` and `it_index`"),
        )
    };
    let fixable = !uses.captures_it && (only || !uses.captures_it_index);
    let fix = fixable.then(|| {
        let mut edits = vec![Edit {
            start: header_start,
            end: hi.span.end as usize,
            text: header.clone(),
        }];
        for (span, how) in &uses.elements {
            let text = match (how, uses.by_pointer) {
                (Replace::Element, true) => "it.*",
                _ => "it",
            };
            edits.push(Edit {
                start: span.start as usize,
                end: span.end as usize,
                text: text.into(),
            });
        }
        for span in &uses.others {
            edits.push(Edit {
                start: span.start as usize,
                end: span.end as usize,
                text: "it_index".into(),
            });
        }
        Fix {
            title: help.clone(),
            edits,
            machine_applicable: true,
        }
    });
    Some(Finding {
        start: stmt.span.start as usize,
        end: hi.span.end as usize,
        message,
        label: None,
        help: Some(help),
        fix,
    })
}

/// When the loop variable `e` is the index of `arr[e]`: the span to replace, how, and
/// whether the element is written.
fn element_access(
    cx: &Cx,
    e: &Expr,
    chain: &[Node<'_>],
    arr_text: &str,
) -> Option<(jaic::source::Span, Replace, bool)> {
    let index = chain.last()?.expr()?;
    let E::Index(base, idx) = &index.kind else {
        return None;
    };
    if idx.span != e.span || squash(cx.src(base.span)) != arr_text {
        return None;
    }
    let up = &chain[..chain.len() - 1];
    // Passing the element by value cannot change it; only writes and addresses do.
    let written = place_use(index, up) != PlaceUse::Read;
    Some(match up.last() {
        Some(Node::Expr(Expr {
            kind: E::Unary(jaic::ast::UnOp::Star, _),
            span,
        })) => (*span, Replace::Address, true),
        Some(Node::Expr(Expr {
            kind: E::Member(b, _),
            ..
        })) if b.span == index.span => (index.span, Replace::Member, written),
        _ => (index.span, Replace::Element, written),
    })
}

/// The array root `e` sits inside the base of an `arr[var]` access (rewritten anyway).
fn part_of_element(cx: &Cx, e: &Expr, chain: &[Node<'_>], arr_text: &str, var: Sym) -> bool {
    chain.iter().rev().any(|n| {
        matches!(n, Node::Expr(Expr { kind: E::Index(base, idx), .. })
            if base.span.start <= e.span.start && e.span.end <= base.span.end
                && matches!(idx.kind, E::Ident(v) if v == var)
                && squash(cx.src(base.span)) == arr_text)
    })
}

enum ArrayUse {
    Read,
    MayWriteElements,
    /// The array itself may change (count, data): `for arr` would not be equivalent.
    Changes,
}

/// How a mention of the array's root name (other than in `arr[var]`) uses the array.
fn array_use(e: &Expr, chain: &[Node<'_>], arr: &Expr) -> ArrayUse {
    // Climb to the largest access path that is still a prefix of `arr` (or `arr` itself).
    let mut cur = e;
    let mut i = chain.len();
    while i > 0 && cur.span.end < arr.span.end {
        match chain[i - 1] {
            Node::Expr(
                p @ Expr {
                    kind: E::Member(base, _),
                    ..
                },
            ) if base.span == cur.span => {
                cur = p;
                i -= 1;
            }
            _ => break,
        }
    }
    let up = &chain[..i];
    if cur.span.end - cur.span.start != arr.span.end - arr.span.start {
        // A prefix of the path (`a` of `a.items`): a write to it may replace the array.
        return match place_use(cur, up) {
            PlaceUse::Read if is_call_argument(cur, up) => ArrayUse::MayWriteElements,
            PlaceUse::Read => ArrayUse::Read,
            _ => ArrayUse::Changes,
        };
    }
    // `arr` itself: what is done with it?
    if let Some(Node::Expr(p)) = up.last() {
        match &p.kind {
            E::Member(base, m) if base.span == cur.span => {
                let up2 = &up[..up.len() - 1];
                return match (m.name.as_str(), place_use(p, up2)) {
                    (_, PlaceUse::Read) => ArrayUse::Read,
                    _ => ArrayUse::Changes,
                };
            }
            E::Index(base, _) if base.span == cur.span => {
                let up2 = &up[..up.len() - 1];
                return match place_use(p, up2) {
                    PlaceUse::Read if is_call_argument(p, up2) => ArrayUse::MayWriteElements,
                    PlaceUse::Read => ArrayUse::Read,
                    _ => ArrayUse::MayWriteElements,
                };
            }
            _ => {}
        }
    }
    match place_use(cur, up) {
        PlaceUse::Read if is_call_argument(cur, up) => ArrayUse::MayWriteElements,
        PlaceUse::Read => ArrayUse::Read,
        _ => ArrayUse::Changes,
    }
}
