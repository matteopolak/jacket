//! `magic_number`: an unexplained number in arithmetic, a comparison or an assignment.
//!
//! ```jai
//! if elapsed > 8640 retry();         // what is 8640?
//! angle := turn * 1.0472;
//! ```
//!
//! A named constant (`RETRY_AFTER :: 8640;`) says what the number means and keeps every use in
//! step when it changes.
//!
//! Only a literal written directly as an operand of `+ - * / %`, of a comparison, or as the
//! value of an assignment (`x = 37;`, `x += 37;`) is reported. Arguments, indexes, array and
//! struct literals, `return` values, `case` labels and declarations (`x := 37;` already names
//! the number) are left alone, and so is anything inside a constant declaration, a type, a
//! `#run` or an `#asm` block. Bitwise and shift operators are idioms with their own numbers and
//! are skipped too.
//!
//! These numbers are never reported:
//! - integers whose magnitude is below `magic_number_min` (default 3: `-2` to `2`);
//! - powers of two (`4`, `64`, `4096`), all-ones masks (`255`, `1023`) and integers written in
//!   hex, octal or binary;
//! - floats with an allowed whole value (`2.0`) or a short binary fraction (`0.5`, `0.25`,
//!   `1.5`);
//! - `10`, `100`, `1000` and anything listed in `magic_number_allow`.
//!
//! Files in a `tests` or `test` folder are skipped unless `magic_number_in_tests` is set: test
//! data is full of expected values. Generated files are skipped. Off by default.
use super::finding;
use crate::Finding;
use crate::syntax::{Cx, Node, walk};
use jaic::ast::{AssignOp, BinOp, ExprKind as E, StmtKind as S, UnOp};
use jaic::ast::{DeclKind, Expr};

/// Numbers allowed whatever the configuration says.
const ALWAYS: &[f64] = &[10.0, 100.0, 1000.0];

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() || (!cx.magic_numbers.in_tests && in_tests(cx)) {
        return;
    }
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, ancestors| {
            if skipped(n, ancestors.last().copied()) {
                return false;
            }
            let Some(e) = n.expr() else {
                return true;
            };
            let Some(value) = number(cx, e) else {
                return true;
            };
            let Some(place) = ancestors.last().and_then(|parent| place(*parent, e)) else {
                return false;
            };
            if !allowed(cx, value) {
                let help = match ascii(value) {
                    Some(c) if equality(ancestors.last().copied()) => {
                        format!("if this is a character, write `#char \"{c}\"`")
                    }
                    _ => "name it with a constant (`NAME :: value;`), or add it to `[lint] magic_number_allow`".into(),
                };
                out.push(finding(
                    e.span,
                    format!("magic number `{}` in {place}", cx.src(e.span)),
                    Some(help),
                ));
            }
            // A literal has nothing to look into; `-37` is reported whole.
            false
        });
    }
}

/// Nodes whose numbers are not magic: constant declarations, types, `#run` and `#asm`.
fn skipped(n: Node, parent: Option<Node>) -> bool {
    match n {
        Node::Stmt(s) => matches!(&s.kind, S::Decl(d) if d.kind == DeclKind::Const),
        Node::Expr(e) => {
            if matches!(
                e.kind,
                E::Run { .. } | E::Asm(_) | E::ArrayType { .. } | E::TypeDirective { .. }
            ) {
                return true;
            }
            // The type of a declaration or a cast.
            let is = |t: Option<&Expr>| t.is_some_and(|t| std::ptr::eq(t, e));
            match parent {
                Some(Node::Stmt(s)) => matches!(&s.kind, S::Decl(d) if is(d.ty.as_ref())),
                Some(Node::Expr(p)) => {
                    matches!(&p.kind, E::Cast { ty, .. } if is(ty.as_deref()))
                }
                None => false,
            }
        }
    }
}

/// Where a number written as `e` under `parent` counts as magic, said for the message.
fn place(parent: Node, e: &Expr) -> Option<&'static str> {
    match parent {
        Node::Expr(p) => match &p.kind {
            E::Binary(op, ..) => match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                    Some("arithmetic")
                }
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    Some("a comparison")
                }
                _ => None,
            },
            _ => None,
        },
        Node::Stmt(s) => match &s.kind {
            S::Assign {
                op,
                rhs,
                ..
            } => {
                let arithmetic = match op {
                    AssignOp::Assign => true,
                    AssignOp::Op(op) => matches!(
                        op,
                        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
                    ),
                };
                (arithmetic && rhs.iter().any(|r| std::ptr::eq(r, e))).then_some("an assignment")
            }
            _ => None,
        },
    }
}

/// The parent is `==` or `!=`, where a character code is likely.
fn equality(parent: Option<Node>) -> bool {
    matches!(
        parent.and_then(Node::expr).map(|p| &p.kind),
        Some(E::Binary(BinOp::Eq | BinOp::Ne, ..))
    )
}

/// The value of a decimal number literal, with any sign in front of it.
fn number(cx: &Cx, e: &Expr) -> Option<f64> {
    match &e.kind {
        E::Unary(UnOp::Neg, inner) => number(cx, inner).map(|v| -v),
        E::Unary(UnOp::Plus, inner) => number(cx, inner),
        E::Int(v) => {
            // `0xFF`, `0b1010`: written as a bit pattern on purpose.
            let text = cx.src(e.span);
            let radix = text.len() > 1
                && text.starts_with('0')
                && text[1..].starts_with(|c: char| c.is_ascii_alphabetic());
            (!radix).then_some(v.get() as f64)
        }
        E::Float(v) => Some(*v),
        _ => None,
    }
}

/// The number needs no name.
fn allowed(cx: &Cx, value: f64) -> bool {
    let settings = &cx.magic_numbers;
    if ALWAYS
        .iter()
        .chain(&settings.allow)
        .any(|a| a.abs().to_bits() == value.abs().to_bits())
    {
        return true;
    }
    let magnitude = value.abs();
    if magnitude.fract() == 0.0 {
        return magnitude < settings.min
            || power_of_two(magnitude)
            || power_of_two(magnitude + 1.0);
    }
    // `0.5`, `0.25`, `1.5`, `0.125`: halves, quarters and eighths.
    (magnitude * 8.0).fract() == 0.0
}

fn power_of_two(v: f64) -> bool {
    v >= 1.0 && v < 2f64.powi(64) && (v as u64).is_power_of_two()
}

/// The printable character with code `value`, other than a quote or backslash.
fn ascii(value: f64) -> Option<char> {
    let c =
        (value.fract() == 0.0 && (33.0..127.0).contains(&value)).then_some(value as u8 as char)?;
    (c != '"' && c != '\\').then_some(c)
}

/// The file sits in a `tests` or `test` folder.
fn in_tests(cx: &Cx) -> bool {
    let path = cx.compiler.src.sources.get(cx.file).path.to_string();
    path.split(['/', '\\'])
        .rev()
        .skip(1)
        .any(|part| part == "tests" || part == "test")
}
