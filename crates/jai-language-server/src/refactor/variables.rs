//! Extract and inline variable.
use super::exprs::{atomic, changes, contains_range, has_call, hoistable, suggest_name, type_like};
use super::text::{fresh, indent_at, starts_line, stmt_extent, unparenthesized};
use super::tree::{in_list, plain_statement};
use super::{Cx, EXTRACT, INLINE};
use crate::CodeAction;
use jaic::ast::{BinOp, DeclKind, ExprKind as E, StmtKind as S, UnOp};
use jaic::source::{line_end, line_start};
use jailint::syntax::{Node, walk};

impl Cx<'_> {
    // Extract into variable

    /// `foo(a + b)` with `a + b` selected becomes `value := a + b;` above and `foo(value)`.
    pub(super) fn extract_variable(&self, out: &mut Vec<CodeAction>) {
        let (a, b) = self.sel;
        if a >= b {
            return;
        }
        let inner = unparenthesized(self.text, self.sel);
        let Some(((e, extent), parents)) = self.find(false, |n, _| {
            let e = n.expr()?;
            let x = self.ex(e);
            (x == (a, b) || inner == Some(x)).then_some((e, x))
        }) else {
            return;
        };
        if !hoistable(e) || type_like(e) {
            return;
        }
        // The statement the expression is evaluated in.
        let Some(at) = parents.iter().rposition(|n| n.stmt().is_some()) else {
            return;
        };
        let stmt = parents[at].stmt().unwrap();
        if !in_list(&parents[..at]) || !plain_statement(stmt) {
            return;
        }
        let (sa, _) = stmt_extent(self.text, stmt);
        if !starts_line(self.text, sa) {
            return;
        }
        // Moving the expression before the statement must not change when, or whether, it runs.
        for (k, node) in parents[at + 1..].iter().enumerate() {
            let Some(up) = node.expr() else {
                continue;
            };
            let below = parents[at + 1 + k + 1..]
                .first()
                .and_then(|n| n.expr())
                .unwrap_or(e);
            let below = self.ex(below);
            match &up.kind {
                E::Binary(BinOp::And | BinOp::Or, first, _) if self.ex(first) != below => return,
                E::Ifx {
                    cond, ..
                } if self.ex(cond) != below => return,
                E::Call {
                    callee, ..
                } if self.ex(callee) == below => return,
                E::Unary(UnOp::Star, _) => return,
                _ => {}
            }
        }
        if let Some(Node::Expr(up)) = parents.last()
            && matches!(&up.kind, E::Call { callee, .. } if self.ex(callee) == extent)
        {
            return;
        }
        match &stmt.kind {
            S::Decl(d) => {
                if d.kind != DeclKind::Var
                    || d.value.as_ref().is_none_or(|v| self.ex(v) == extent)
                    || d.ty
                        .as_ref()
                        .is_some_and(|t| contains_range(self.ex(t), extent))
                {
                    return;
                }
            }
            S::Expr(x) if self.ex(x) == extent => return,
            S::Assign {
                lhs, ..
            } if lhs.iter().any(|l| contains_range(self.ex(l), extent)) => {
                return;
            }
            S::Switch(switch)
                if switch
                    .cases
                    .iter()
                    .flat_map(|c| &c.values)
                    .any(|v| contains_range(self.ex(v), extent)) =>
            {
                return;
            }
            S::For(f) if f.pointer_if.is_some() || f.reverse_if.is_some() => return,
            _ => {}
        }
        let name = fresh(self.text, &suggest_name(e));
        let indent = indent_at(self.text, sa);
        let value = self.slice(extent);
        let edits = vec![
            self.edit(
                line_start(self.text, sa),
                line_start(self.text, sa),
                format!("{indent}{name} := {value};{}", self.nl),
            ),
            self.edit(a, b, name),
        ];
        out.extend(self.action("Extract into variable".into(), EXTRACT, edits));
    }

    // Inline variable

    /// `x := a + b;` with every use of `x` replaced by `a + b`, when `x` is never changed.
    pub(super) fn inline_variable(&self, out: &mut Vec<CodeAction>) {
        let (a, b) = self.sel;
        let word = |c: char| c.is_alphanumeric() || c == '_';
        let from = self.text[..a]
            .char_indices()
            .rev()
            .take_while(|(_, c)| word(*c))
            .last()
            .map_or(a, |(i, _)| i);
        let to = a + self.text[a..]
            .find(|c: char| !word(c))
            .unwrap_or(self.text.len() - a);
        if from >= to || b > to {
            return;
        }
        let Ok(doc) = self.session.document(self.uri) else {
            return;
        };
        let Ok(position) = doc.index.position(self.text, from) else {
            return;
        };
        let Ok(refs) = self.session.references(self.uri, position, true) else {
            return;
        };
        let mut spans: Vec<(usize, usize)> = Vec::new();
        for r in &refs {
            if r.uri != self.uri.as_str() {
                return;
            }
            let (Ok(s), Ok(e)) = (
                doc.index.byte(self.text, r.range.start),
                doc.index.byte(self.text, r.range.end),
            ) else {
                return;
            };
            spans.push((s, e));
        }
        if !spans.contains(&(from, to)) {
            return;
        }
        // The declaration among the references.
        let Some(((decl, stmt_at), parents)) = self.find(false, |n, _| {
            let s = n.stmt()?;
            let S::Decl(d) = &s.kind else {
                return None;
            };
            let name = d.names.first()?;
            let at = name.span.start as usize;
            (spans.contains(&(at, name.span.end as usize))).then_some((d.clone(), s))
        }) else {
            return;
        };
        let d = &*decl;
        if d.names.len() != 1
            || d.kind != DeclKind::Var
            || d.ty.is_some()
            || !d.extra_values().is_empty()
            || d.using
            || d.backtick
            || d.as_
            || d.align().is_some()
            || !d.flags().is_empty()
            || !d.notes.is_empty()
            || !d.existing().is_empty()
        {
            return;
        }
        let Some(value) = &d.value else {
            return;
        };
        if matches!(value.kind, E::Uninit | E::Proc(_) | E::Block(_)) {
            return;
        }
        let name = d.names[0];
        if !in_list(&parents) {
            return;
        }
        let (sa, sb) = stmt_extent(self.text, stmt_at);
        let (la, lb) = (line_start(self.text, sa), line_end(self.text, sb));
        if !starts_line(self.text, sa) || !self.text[sb..lb].trim().is_empty() {
            return;
        }
        let uses: Vec<(usize, usize)> = spans
            .iter()
            .copied()
            .filter(|&s| s != (name.span.start as usize, name.span.end as usize))
            .collect();
        if uses.is_empty() {
            return;
        }
        let value_extent = self.ex(value);
        let calls = has_call(value);
        if calls && uses.len() > 1 {
            return;
        }
        // Find each use in the tree of the body holding the declaration.
        let Some(body) = self.roots.iter().filter(|r| r.in_proc).find(|r| {
            r.stmts
                .iter()
                .any(|s| contains_range(stmt_extent(self.text, s), (sa, sb)))
        }) else {
            return;
        };
        let mut texts: Vec<((usize, usize), String)> = Vec::new();
        let mut bad = false;
        let value_text = self.slice(value_extent);
        let atom = atomic(value);
        walk(body.stmts, &mut |n, parents| {
            let Some(x) = n.expr() else {
                return true;
            };
            let at = self.ex(x);
            if !matches!(x.kind, E::Ident(n) if n == name.name) || !uses.contains(&at) {
                return true;
            }
            if changes(parents, x) {
                bad = true;
            }
            let wrap = !atom
                && match parents.last() {
                    Some(Node::Stmt(_)) => false,
                    Some(Node::Expr(up)) => !matches!(&up.kind, E::Call { args, .. }
                        if args.iter().any(|a| self.ex(&a.value) == at)),
                    None => false,
                };
            texts.push((
                at,
                if wrap {
                    format!("({value_text})")
                } else {
                    value_text.to_string()
                },
            ));
            true
        });
        // A use the walk did not reach (inside a lambda, say) cannot be checked.
        if bad || texts.len() != uses.len() {
            return;
        }
        let mut edits =
            vec![self.edit(la, (lb + self.nl.len()).min(self.text.len()), String::new())];
        edits.extend(texts.into_iter().map(|((s, e), t)| self.edit(s, e, t)));
        out.extend(self.action(
            format!("Inline variable `{}`", name.name.as_str()),
            INLINE,
            edits,
        ));
    }
}
