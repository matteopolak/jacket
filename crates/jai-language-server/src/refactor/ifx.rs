//! Converting between `ifx` and `if`.
use super::exprs::only;
use super::text::{indent_at, starts_line, stmt_extent};
use super::tree::in_list;
use super::{Cx, REWRITE};
use crate::CodeAction;
use jaic::ast::{AssignOp, DeclKind, ExprKind as E, StmtKind as S};
use jaic::source::line_end;

impl Cx<'_> {
    // ifx <-> if / else

    /// `x = ifx c then a else b;` as an `if` that assigns (or returns) `a` or `b`.
    pub(super) fn ifx_to_if(&self, out: &mut Vec<CodeAction>) {
        let Some(((stmt, cond, then, otherwise), parents)) = self.innermost(false, |n| {
            let s = n.stmt()?;
            let ifx = match &s.kind {
                S::Decl(d) => d.value.as_ref()?,
                S::Assign {
                    rhs, ..
                } if rhs.len() == 1 => &rhs[0],
                S::Return {
                    values, ..
                } if values.len() == 1 => &values[0].value,
                _ => return None,
            };
            let E::Ifx {
                cond,
                then_value: Some(then),
                else_value: Some(otherwise),
                is_static: false,
            } = &ifx.kind
            else {
                return None;
            };
            let (a, b) = stmt_extent(self.text, s);
            // The cursor is on the first line, where `ifx` is.
            let (x, y) = (a, line_end(self.text, a).min(b));
            Some(((s, &**cond, &**then, &**otherwise), (x, y)))
        }) else {
            return;
        };
        let (sa, sb) = stmt_extent(self.text, stmt);
        if !in_list(&parents) || !starts_line(self.text, sa) {
            return;
        }
        let (c, t, o) = (
            self.slice(self.ex(cond)),
            self.slice(self.ex(then)),
            self.slice(self.ex(otherwise)),
        );
        let indent = indent_at(self.text, sa);
        let (nl, unit) = (self.nl, &self.unit);
        let (prefix, target) = match &stmt.kind {
            S::Assign {
                lhs,
                rhs,
                op,
                ..
            } if lhs.len() == 1 => {
                let operator = match op {
                    AssignOp::Assign => "=",
                    AssignOp::Op(_) => self.slice((self.ex(&lhs[0]).1, self.ex(&rhs[0]).0)).trim(),
                };
                (
                    String::new(),
                    format!("{} {operator}", self.slice(self.ex(&lhs[0]))),
                )
            }
            S::Return {
                ..
            } => (String::new(), "return".into()),
            S::Decl(d)
                if d.names.len() == 1
                    && d.kind == DeclKind::Var
                    && d.extra_values().is_empty()
                    && d.existing().is_empty()
                    && !d.using
                    && !d.backtick =>
            {
                let Some(ty) = self.type_of_decl(d, 0) else {
                    return;
                };
                let name = d.names[0].name.as_str();
                (format!("{name}: {ty};{nl}{indent}"), format!("{name} ="))
            }
            _ => return,
        };
        let replacement = format!(
            "{prefix}if {c} {{{nl}{indent}{unit}{target} {t};{nl}{indent}}} else {{{nl}{indent}{unit}{target} {o};{nl}{indent}}}"
        );
        out.extend(self.action(
            "Convert ifx to if/else".into(),
            REWRITE,
            vec![self.edit(sa, sb, replacement)],
        ));
    }

    /// `if c { x = a; } else { x = b; }` as `x = ifx c then a else b;`.
    pub(super) fn if_to_ifx(&self, out: &mut Vec<CodeAction>) {
        let Some(((stmt, cond, then, otherwise), parents)) = self.innermost(false, |n| {
            let s = n.stmt()?;
            let S::If {
                cond,
                then_branch,
                else_branch: Some(otherwise),
            } = &s.kind
            else {
                return None;
            };
            let (a, b) = stmt_extent(self.text, s);
            Some((
                (s, cond, &**then_branch, &**otherwise),
                (a, line_end(self.text, a).min(b)),
            ))
        }) else {
            return;
        };
        let (sa, sb) = stmt_extent(self.text, stmt);
        if !in_list(&parents) || !starts_line(self.text, sa) {
            return;
        }
        let (Some(t), Some(o)) = (only(then), only(otherwise)) else {
            return;
        };
        let c = self.slice(self.ex(cond));
        let line = match (&t.kind, &o.kind) {
            (
                S::Assign {
                    op: op1,
                    lhs: l1,
                    rhs: r1,
                },
                S::Assign {
                    op: op2,
                    lhs: l2,
                    rhs: r2,
                },
            ) if op1 == op2 && l1.len() == 1 && l2.len() == 1 && r1.len() == 1 && r2.len() == 1 => {
                let target = self.slice(self.ex(&l1[0]));
                if target != self.slice(self.ex(&l2[0])) {
                    return;
                }
                let operator = match op1 {
                    AssignOp::Assign => "=",
                    AssignOp::Op(_) => self.slice((self.ex(&l1[0]).1, self.ex(&r1[0]).0)).trim(),
                };
                format!(
                    "{target} {operator} ifx {c} then {} else {};",
                    self.slice(self.ex(&r1[0])),
                    self.slice(self.ex(&r2[0]))
                )
            }
            (
                S::Return {
                    values: v1,
                    backtick: false,
                },
                S::Return {
                    values: v2,
                    backtick: false,
                },
            ) if v1.len() == 1 && v2.len() == 1 && v1[0].name.is_none() && v2[0].name.is_none() => {
                format!(
                    "return ifx {c} then {} else {};",
                    self.slice(self.ex(&v1[0].value)),
                    self.slice(self.ex(&v2[0].value))
                )
            }
            _ => return,
        };
        out.extend(self.action(
            "Convert if/else to ifx".into(),
            REWRITE,
            vec![self.edit(sa, sb, line)],
        ));
    }
}
