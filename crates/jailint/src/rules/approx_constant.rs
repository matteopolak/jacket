//! `approx_constant`: `3.14159`, `6.2832` or `1.4142` written out where `Math` has the constant.
//!
//! ```jai
//! area := 3.14159 * r * r;         // `PI * r * r`
//! ```
//!
//! A hand-typed approximation is less precise than the library's value, easy to mistype
//! (`3.14259`), and says less than the name.
//!
//! Fires on a float literal with at least four significant digits that matches one of these,
//! rounded or cut off where the literal stops: π (`PI`, `PI64`), 2π (`TAU`, `TAU64`), π/2
//! (`PI / 2`), π/4 (`PI / 4`), √2 (`SQRT2`) and √½ (`SQRTH`). Constant declarations are left
//! alone: they are where such names come from.
use super::suggesting;
use crate::syntax::{Cx, Node, walk};
use crate::{Edit, Finding};
use jaic::ast::{DeclKind, ExprKind as E, StmtKind as S};
use jaic::types::TypeKind;
use std::f64::consts;

/// The values, and how to write them with `Math` as `float32` and as `float64`.
const KNOWN: &[(f64, &str, &str)] = &[
    (consts::PI, "PI", "PI64"),
    (consts::TAU, "TAU", "TAU64"),
    (consts::FRAC_PI_2, "PI / 2", "PI64 / 2"),
    (consts::FRAC_PI_4, "PI / 4", "PI64 / 4"),
    (consts::SQRT_2, "SQRT2", "SQRT2"),
    (consts::FRAC_1_SQRT_2, "SQRTH", "SQRTH"),
];

/// Fewer digits than this are a rough value on purpose (`3.14`).
const MIN_DIGITS: usize = 4;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "approx_constant",
    category: crate::config::Category::Style,
    summary: "a hand-typed `3.14159` or `1.4142` where `Math` has the constant",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, _| match n {
            Node::Stmt(s) => !matches!(&s.kind, S::Decl(d) if d.kind == DeclKind::Const),
            Node::Expr(e) => {
                if let E::Float(v, ..) = e.kind {
                    let text = cx.src(e.span).trim();
                    if let Some((name, wide)) = known(text, v) {
                        let help = if name == wide {
                            format!("write `{name}` from `Math`")
                        } else {
                            format!("write `{name}` from `Math` (`{wide}` for `float64`)")
                        };
                        let types = &cx.compiler.ty.types;
                        let wide_type = cx.ty(e).is_some_and(|t| {
                            matches!(
                                types.kind(t),
                                TypeKind::Float {
                                    bits: 64
                                }
                            )
                        });
                        let written = if wide_type {
                            wide
                        } else {
                            name
                        };
                        // A quotient is parenthesized so the neighbouring operators keep their grouping.
                        let replacement = if written.contains('/') {
                            format!("({written})")
                        } else {
                            written.to_string()
                        };
                        out.push(suggesting(
                            e.span,
                            format!("`{text}` is an approximation of `{name}`"),
                            help,
                            format!("write `{written}` from `Math`"),
                            vec![Edit {
                                start: e.span.start as usize,
                                end: e.span.end as usize,
                                text: replacement,
                            }],
                        ));
                    }
                }
                true
            }
        });
    }
}

/// The constant a literal spells, to as many decimals as it has.
fn known(text: &str, value: f64) -> Option<(&'static str, &'static str)> {
    let digits: String = text.chars().filter(|c| *c != '_').collect();
    // `1.5e3` and suffixed or hex forms are not hand-typed constants.
    if !digits.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let (whole, fraction) = digits.split_once('.')?;
    let significant = whole.trim_start_matches('0').len()
        + if whole.trim_start_matches('0').is_empty() {
            fraction.trim_start_matches('0').len()
        } else {
            fraction.len()
        };
    if significant < MIN_DIGITS {
        return None;
    }
    // Rounded or cut off at the literal's last digit.
    let step = 10f64.powi(-(fraction.len() as i32));
    KNOWN
        .iter()
        .find(|(c, ..)| (value - c).abs() < step)
        .map(|(_, name, wide)| (*name, *wide))
}
