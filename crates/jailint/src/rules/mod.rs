//! The rules. Each lives in its own module, documented with why it exists; [`RULES`] lists
//! them with their default levels.
use crate::syntax::{Cx, Node, squash};
use crate::{Edit, Finding, Fix, Level};
use jaic::ast::{BinOp, Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};
use jaic::sema::ProcId;
use jaic::source::Span;

mod absurd_comparison;
mod almost_swapped;
mod bitwise_precedence;
mod bool_comparison;
mod defer_in_loop;
mod duplicate_condition;
mod erasing_op;
mod float_equality;
mod format_arg_count;
mod identical_branches;
mod identical_operands;
mod identity_op;
mod index_only_loop;
mod infinite_loop;
mod integer_division_in_float;
mod lossy_xx;
mod manual_assign_op;
mod manual_index_counter;
mod min_max;
mod needless_bool;
mod no_effect;
mod range_past_count;
mod redundant_cast;
mod remove_in_for;
mod reversed_range;
mod self_assignment;
mod shadowed_it;
pub(crate) mod unused_import;
mod unused_parameter;
mod unused_result;
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
        name: "absurd_comparison",
        default: Level::Warn,
        summary: "a comparison an unsigned value always passes or fails (`u >= 0`)",
        check: absurd_comparison::check,
    },
    RuleInfo {
        name: "almost_swapped",
        default: Level::Warn,
        summary: "`a = b; b = a;`, a swap that sets both to `b`",
        check: almost_swapped::check,
    },
    RuleInfo {
        name: "bitwise_precedence",
        default: Level::Warn,
        summary: "a bitwise operator that binds differently than in C (`1 << n - 1`)",
        check: bitwise_precedence::check,
    },
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
        name: "duplicate_condition",
        default: Level::Warn,
        summary: "an `else if` or `case` that repeats an earlier one",
        check: duplicate_condition::check,
    },
    RuleInfo {
        name: "erasing_op",
        default: Level::Warn,
        summary: "an operation that is always `0` (`x * 0`, `x & 0`)",
        check: erasing_op::check,
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
        name: "identical_branches",
        default: Level::Warn,
        summary: "an `if`/`else` or `ifx` whose branches are the same",
        check: identical_branches::check,
    },
    RuleInfo {
        name: "identical_operands",
        default: Level::Warn,
        summary: "an operator with the same expression on both sides (`a == a`)",
        check: identical_operands::check,
    },
    RuleInfo {
        name: "identity_op",
        default: Level::Warn,
        summary: "an operation that changes nothing (`x + 0`, `x * 1`)",
        check: identity_op::check,
    },
    RuleInfo {
        name: "index_only_loop",
        default: Level::Warn,
        summary: "`for i: 0..xs.count-1` where `i` only indexes `xs`",
        check: index_only_loop::check,
    },
    RuleInfo {
        name: "infinite_loop",
        default: Level::Warn,
        summary: "a `while` whose condition nothing in the loop changes",
        check: infinite_loop::check,
    },
    RuleInfo {
        name: "integer_division_in_float",
        default: Level::Warn,
        summary: "an integer division whose result is used as a float",
        check: integer_division_in_float::check,
    },
    RuleInfo {
        name: "lossy_xx",
        default: Level::Allow,
        summary: "`xx` that narrows a number to a smaller type",
        check: lossy_xx::check,
    },
    RuleInfo {
        name: "manual_assign_op",
        default: Level::Warn,
        summary: "`a = a + b` instead of `a += b`",
        check: manual_assign_op::check,
    },
    RuleInfo {
        name: "manual_index_counter",
        default: Level::Warn,
        summary: "a counter incremented in a `for` loop that always equals `it_index`",
        check: manual_index_counter::check,
    },
    RuleInfo {
        name: "min_max",
        default: Level::Warn,
        summary: "`min(0, max(100, x))`, a clamp that is always one bound",
        check: min_max::check,
    },
    RuleInfo {
        name: "needless_bool",
        default: Level::Warn,
        summary: "`ifx c then true else false` and `if c return true; else return false;`",
        check: needless_bool::check,
    },
    RuleInfo {
        name: "no_effect",
        default: Level::Warn,
        summary: "a statement that computes a value and drops it (`x == 5;`)",
        check: no_effect::check,
    },
    RuleInfo {
        name: "range_past_count",
        default: Level::Warn,
        summary: "`for i: 0..xs.count` indexing `xs[i]`, one past the end",
        check: range_past_count::check,
    },
    RuleInfo {
        name: "redundant_cast",
        default: Level::Warn,
        summary: "a cast to the type the value already has",
        check: redundant_cast::check,
    },
    RuleInfo {
        name: "remove_in_for",
        default: Level::Warn,
        summary: "removing from an array while a `for` walks it forward",
        check: remove_in_for::check,
    },
    RuleInfo {
        name: "reversed_range",
        default: Level::Warn,
        summary: "a range whose start is above its end, which never runs",
        check: reversed_range::check,
    },
    RuleInfo {
        name: "self_assignment",
        default: Level::Warn,
        summary: "a value assigned to itself (`x = x;`)",
        check: self_assignment::check,
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
        name: "unused_result",
        default: Level::Warn,
        summary: "a call to a library procedure that only computes a value, as a statement",
        check: unused_result::check,
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

/// Evaluating `e` has no side effects and gives the same value twice in a row: names, member
/// paths, literals, and indexing, operators and casts over them. Calls never qualify.
pub(crate) fn pure(e: &Expr) -> bool {
    match &e.kind {
        E::Ident(_)
        | E::Int(_)
        | E::Float(_)
        | E::Bool(_)
        | E::Null
        | E::Str(_)
        | E::Char(_)
        | E::InferredMember(_) => true,
        E::Member(base, _) => pure(base),
        E::Index(a, b) | E::Binary(_, a, b) => pure(a) && pure(b),
        E::Unary(_, a) => pure(a),
        E::Cast {
            value, ..
        } => pure(value),
        _ => false,
    }
}

/// `a` and `b` are the same pure expression, written the same way.
pub(crate) fn same_pure(cx: &Cx, a: &Expr, b: &Expr) -> bool {
    pure(a) && pure(b) && squash(cx.whole_src(a.span)) == squash(cx.whole_src(b.span))
}

/// An integer literal, possibly negated: its value.
pub(crate) fn int_literal(e: &Expr) -> Option<i128> {
    match &e.kind {
        E::Int(v) => i128::try_from(*v).ok(),
        E::Unary(UnOp::Neg, inner) => int_literal(inner).map(|v| -v),
        E::Unary(UnOp::Plus, inner) => int_literal(inner),
        _ => None,
    }
}

/// The expression is written inside its own parentheses: `(a & b)`.
pub(crate) fn parenthesized(cx: &Cx, e: &Expr) -> bool {
    let (start, end) = cx.whole(e.span);
    start < e.span.start as usize || end > e.span.end as usize
}

/// The procedures a call's callee named, when it is a plain name or `Module.name`.
pub(crate) fn callee_procs<'c>(cx: &'c Cx, callee: &Expr) -> &'c [ProcId] {
    let span = match &callee.kind {
        E::Ident(_) => callee.span,
        E::Member(_, name) => name.span,
        _ => return &[],
    };
    cx.facts.procs_at(span)
}

/// The callee is one of `names` from one of the library `modules`: every procedure it named
/// is declared in such a module of an import directory.
pub(crate) fn calls_library(cx: &Cx, callee: &Expr, modules: &[&str], names: &[&str]) -> bool {
    let name = match &callee.kind {
        E::Ident(n) => *n,
        E::Member(_, n) => n.name,
        _ => return false,
    };
    if !names.contains(&name.as_str()) {
        return false;
    }
    let procs = callee_procs(cx, callee);
    !procs.is_empty()
        && procs.iter().all(|&p| {
            let file = cx.compiler.procs[p.0 as usize].lit.header.span.file;
            library_module(cx, &cx.compiler.sources.get(file).path)
                .is_some_and(|m| modules.contains(&m))
        })
}

/// The module a file in an import directory belongs to: `Basic` for `<dir>/Basic/Array.jai`
/// or `<dir>/Basic.jai`.
fn library_module<'p>(cx: &Cx, path: &'p str) -> Option<&'p str> {
    cx.compiler.options.import_paths.iter().find_map(|dir| {
        let dir = dir.to_str()?;
        let rest = path.strip_prefix(dir)?.trim_start_matches(['/', '\\']);
        let first = rest.split(['/', '\\']).next()?;
        Some(first.strip_suffix(".jai").unwrap_or(first))
    })
}

/// Some statement inside `stmt` (itself included, nested procedures not entered) satisfies `f`.
pub(crate) fn any_stmt(stmt: &Stmt, f: &mut dyn FnMut(&Stmt) -> bool) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    crate::syntax::walk_node(Node::Stmt(stmt), &mut stack, &mut |n, _| {
        if !found
            && let Node::Stmt(s) = n
            && f(s)
        {
            found = true;
        }
        !found
    });
    found
}

/// Every statement list in a procedure body: the body, nested blocks, `case` bodies and `#if`
/// branches.
pub(crate) fn statement_lists<'a>(body: &'a [Stmt], f: &mut dyn FnMut(&'a [Stmt])) {
    f(body);
    crate::syntax::walk(body, &mut |n, _| {
        if let Some(s) = n.stmt() {
            match &s.kind {
                S::Block(b) => f(&b.stmts),
                S::Switch {
                    cases, ..
                }
                | S::StaticSwitch {
                    cases, ..
                } => {
                    for c in cases {
                        f(&c.body);
                    }
                }
                S::StaticIf {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    f(then_branch);
                    f(else_branch);
                }
                _ => {}
            }
        }
        true
    });
}

/// A finding without a fix.
pub(crate) fn finding(span: Span, message: String, help: Option<String>) -> Finding {
    Finding {
        start: span.start as usize,
        end: span.end as usize,
        message,
        label: None,
        help,
        fix: None,
    }
}

/// A finding whose fix replaces bytes `start..end` with `text`.
pub(crate) fn replacing(
    span: Span,
    message: String,
    help: String,
    (start, end): (usize, usize),
    text: String,
    machine_applicable: bool,
) -> Finding {
    Finding {
        start: span.start as usize,
        end: span.end as usize,
        message,
        label: None,
        help: Some(help.clone()),
        fix: Some(Fix {
            title: help,
            edits: vec![Edit {
                start,
                end,
                text,
            }],
            machine_applicable,
        }),
    }
}

/// How the operator is written.
pub(crate) fn op_text(op: BinOp) -> &'static str {
    match op {
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::Sub => "-",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::Add => "+",
        BinOp::Mul => "*",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::Rotl => "<<<",
        BinOp::Rotr => ">>>",
    }
}
