//! The rules. Each lives in its own module, documented with why it exists; [`RULES`] lists
//! them with their default levels.
use crate::syntax::{Cx, Node};
use crate::{Finding, Level};
use jaic::ast::{Expr, ExprKind as E, StmtKind as S, UnOp};

mod bool_comparison;
mod defer_in_loop;
mod float_equality;
mod format_arg_count;
mod index_only_loop;
mod lossy_xx;
mod manual_index_counter;
mod redundant_cast;
mod shadowed_it;
pub(crate) mod unused_import;
mod unused_parameter;
mod unused_variable;

pub struct RuleInfo {
    pub name: &'static str,
    pub default: Level,
    /// One line for `--list`.
    pub summary: &'static str,
    pub(crate) check: fn(&Cx, &mut Vec<Finding>),
}

pub static RULES: &[RuleInfo] = &[
    RuleInfo {
        name: "bool_comparison",
        default: Level::Warn,
        summary: "comparing a bool with `true` or `false`",
        check: bool_comparison::check,
    },
    RuleInfo {
        name: "defer_in_loop",
        default: Level::Warn,
        summary: "a `defer` in a loop body that cleans up something from outside the loop",
        check: defer_in_loop::check,
    },
    RuleInfo {
        name: "float_equality",
        default: Level::Allow,
        summary: "`==` or `!=` between two computed floats",
        check: float_equality::check,
    },
    RuleInfo {
        name: "format_arg_count",
        default: Level::Deny,
        summary: "a format string that uses more or fewer arguments than the call passes",
        check: format_arg_count::check,
    },
    RuleInfo {
        name: "index_only_loop",
        default: Level::Warn,
        summary: "`for i: 0..xs.count-1` where `i` only indexes `xs`",
        check: index_only_loop::check,
    },
    RuleInfo {
        name: "lossy_xx",
        default: Level::Allow,
        summary: "`xx` that narrows a number to a smaller type",
        check: lossy_xx::check,
    },
    RuleInfo {
        name: "manual_index_counter",
        default: Level::Warn,
        summary: "a counter incremented in a `for` loop that always equals `it_index`",
        check: manual_index_counter::check,
    },
    RuleInfo {
        name: "redundant_cast",
        default: Level::Warn,
        summary: "a cast to the type the value already has",
        check: redundant_cast::check,
    },
    RuleInfo {
        name: "shadowed_it",
        default: Level::Warn,
        summary: "a nested `for` hides an `it` the enclosing loop uses",
        check: shadowed_it::check,
    },
    RuleInfo {
        name: "unused_import",
        default: Level::Warn,
        summary: "an `#import` nothing in its scope uses",
        check: unused_import::check,
    },
    RuleInfo {
        name: "unused_parameter",
        default: Level::Warn,
        summary: "a parameter the procedure never uses",
        check: unused_parameter::check,
    },
    RuleInfo {
        name: "unused_variable",
        default: Level::Warn,
        summary: "a local variable that is never used",
        check: unused_variable::check,
    },
];

/// How an expression that denotes a place is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaceUse {
    Read,
    /// Assigned to, or a part of it is.
    Write,
    /// Its address (or the address of a part) is taken: it may be written through.
    Address,
}

/// How the expression `node` (whose ancestors are `chain`, outermost first) is used: the
/// access path it heads (`x`, `x.a`, `x[1].b`) decides.
pub(crate) fn place_use(node: &Expr, chain: &[Node<'_>]) -> PlaceUse {
    let mut cur = node;
    let mut i = chain.len();
    while i > 0 {
        i -= 1;
        match chain[i] {
            Node::Expr(p) => match &p.kind {
                E::Member(base, _) if base.span == cur.span => cur = p,
                E::Index(base, _) if base.span == cur.span => cur = p,
                E::Unary(UnOp::Star, inner) if inner.span == cur.span => return PlaceUse::Address,
                // `x.*`: the pointer's target; writes go elsewhere.
                _ => return PlaceUse::Read,
            },
            Node::Stmt(s) => {
                return match &s.kind {
                    S::Assign {
                        lhs, ..
                    } if lhs.iter().any(|l| l.span == cur.span) => PlaceUse::Write,
                    S::For(f) if f.by_pointer || f.pointer_if.is_some() => match &f.over {
                        jaic::ast::ForOver::Collection(c) if c.span == cur.span => {
                            PlaceUse::Address
                        }
                        _ => PlaceUse::Read,
                    },
                    S::Using {
                        value, ..
                    } if value.span == cur.span => PlaceUse::Address,
                    _ => PlaceUse::Read,
                };
            }
        }
    }
    PlaceUse::Read
}

/// `e` is a call argument (directly) in `chain`'s last node.
pub(crate) fn is_call_argument(e: &Expr, chain: &[Node<'_>]) -> bool {
    matches!(chain.last(), Some(Node::Expr(Expr { kind: E::Call { args, .. }, .. }))
        if args.iter().any(|a| a.value.span == e.span))
}

pub(crate) fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}
