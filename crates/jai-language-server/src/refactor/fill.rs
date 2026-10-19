//! Fill a switch's cases or a struct literal's members.
use super::text::{indent_at, starts_line, stmt_extent};
use super::{Cx, REWRITE};
use crate::CodeAction;
use jaic::ast::{ExprKind as E, StmtKind as S};
use jaic::fxhash::HashSet;
use jaic::source::line_start;

impl Cx<'_> {
    // Fill in missing cases and fields

    /// Add a `case .NAME;` for each member an `if x == {` on an enum does not mention.
    pub(super) fn fill_switch(&self, out: &mut Vec<CodeAction>) {
        let Some(((stmt, value, cases), _)) = self.innermost(false, |n| {
            let s = n.stmt()?;
            let S::Switch(switch) = &s.kind else {
                return None;
            };
            Some(((s, &switch.value, &switch.cases), stmt_extent(self.text, s)))
        }) else {
            return;
        };
        // Not while the cursor is in a case's code.
        if cases.iter().flat_map(|c| &c.body).any(|b| {
            let (x, y) = stmt_extent(self.text, b);
            x <= self.cursor && self.cursor <= y
        }) {
            return;
        }
        let (_, ve) = self.ex(value);
        let Some(members) = self.members(ve.saturating_sub(1), false) else {
            return;
        };
        if !members.is_enum {
            return;
        }
        let mut named: HashSet<&str> = HashSet::default();
        for v in cases.iter().flat_map(|c| &c.values) {
            match &v.kind {
                E::InferredMember(n) | E::Member(_, n) => {
                    named.insert(n.name.as_str());
                }
                E::Ident(n) => {
                    named.insert(n.as_str());
                }
                _ => return,
            }
        }
        let missing: Vec<&str> = members
            .members
            .iter()
            .map(|(n, _)| n.as_str())
            .filter(|n| !named.contains(n))
            .collect();
        if missing.is_empty() {
            return;
        }
        let (sa, sb) = stmt_extent(self.text, stmt);
        if self.text.get(sb.saturating_sub(1)..sb) != Some("}") {
            return;
        }
        // New cases go before a `case;` that takes the rest, else before the closing brace.
        let before = match cases.iter().find(|c| c.values.is_empty()) {
            Some(default) => default.span.start as usize,
            None => sb - 1,
        };
        let outer = indent_at(self.text, sa).to_string();
        let indent = match cases.first() {
            Some(c) if starts_line(self.text, c.span.start as usize) => {
                indent_at(self.text, c.span.start as usize).to_string()
            }
            _ => format!("{outer}{}", self.unit),
        };
        let nl = self.nl;
        let edit = if starts_line(self.text, before) {
            // A line of its own: whole lines go in above it.
            let at = line_start(self.text, before);
            let text: String = missing
                .iter()
                .map(|n| format!("{indent}case .{n};{nl}"))
                .collect();
            self.edit(at, at, text)
        } else {
            // Something shares the line (`{ }`, `{ case .A; }`): break the line before `before`
            // and put what follows on a line of its own, indented like the line it was on.
            let keep = if before == sb - 1 {
                &outer
            } else {
                &indent
            };
            let from = before
                - (self.text[..before].len()
                    - self.text[..before].trim_end_matches([' ', '\t']).len());
            let mut text: String = missing
                .iter()
                .map(|n| format!("{nl}{indent}case .{n};"))
                .collect();
            text.push_str(nl);
            text.push_str(keep);
            self.edit(from, before, text)
        };
        let title = format!("Add missing cases ({})", missing.len());
        out.extend(self.action(title, REWRITE, vec![edit]));
    }

    /// The members of the enum or struct at `at`.
    fn members(&self, at: usize, as_type: bool) -> Option<jaic::ide::meta::IdeMembers> {
        self.session
            .checked(self.uri, |a, f| {
                a.compiler.ide_members_at(f, at as u32, as_type)
            })
            .flatten()
    }

    /// Add `name = value` for each field a `Type.{...}` literal leaves out.
    pub(super) fn fill_struct(&self, out: &mut Vec<CodeAction>) {
        let Some(((ty, fields, lit), _)) = self.innermost(true, |n| {
            let e = n.expr()?;
            let E::StructLit {
                ty: Some(ty),
                fields,
            } = &e.kind
            else {
                return None;
            };
            Some(((&**ty, fields, e), self.ex(e)))
        }) else {
            return;
        };
        if fields
            .iter()
            .any(|f| f.name.is_none() || f.target.is_some() || f.spread)
        {
            return;
        }
        let (_, te) = self.ex(ty);
        let Some(members) = self.members(te.saturating_sub(1), true) else {
            return;
        };
        if members.is_enum {
            return;
        }
        let present: HashSet<&str> = fields
            .iter()
            .filter_map(|f| f.name.as_ref().map(|n| n.name.as_str()))
            .collect();
        let missing: Vec<(&str, &str)> = members
            .members
            .iter()
            .filter(|(n, _)| !present.contains(n.as_str()))
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();
        if missing.is_empty() {
            return;
        }
        let (la, lb) = self.ex(lit);
        let Some(open) = self
            .text
            .get(te..lb)
            .and_then(|t| t.find(".{"))
            .map(|i| te + i + 2)
        else {
            return;
        };
        if self.text.get(lb.saturating_sub(1)..lb) != Some("}") {
            return;
        }
        let close = lb - 1;
        if open > close {
            return;
        }
        let multiline = self.text[open..close].contains('\n');
        let pairs = |sep: &str| -> String {
            missing
                .iter()
                .map(|(n, v)| format!("{n} = {v}"))
                .collect::<Vec<_>>()
                .join(sep)
        };
        let edit = match fields.last() {
            None if !multiline => self.edit(open, close, format!(" {} ", pairs(", "))),
            None => {
                let indent = format!("{}{}", indent_at(self.text, la), self.unit);
                let lines: String = missing
                    .iter()
                    .map(|(n, v)| format!("{indent}{n} = {v},{}", self.nl))
                    .collect();
                let at = line_start(self.text, close);
                if !starts_line(self.text, close) {
                    return;
                }
                self.edit(at, at, lines)
            }
            Some(last) if !multiline => {
                let (_, le) = self.ex(&last.value);
                self.edit(le, le, format!(", {}", pairs(", ")))
            }
            Some(last) => {
                let (le, trailing) = {
                    let (_, le) = self.ex(&last.value);
                    let comma = self
                        .text
                        .get(le..close)
                        .unwrap_or("")
                        .trim_start()
                        .starts_with(',');
                    (le, comma)
                };
                let Some(name) = &last.name else {
                    return;
                };
                if !starts_line(self.text, name.span.start as usize) {
                    return;
                }
                let indent = indent_at(self.text, name.span.start as usize);
                if trailing {
                    let comma = le
                        + self.text[le..]
                            .find(',')
                            .expect("invariant: `trailing` was read from a comma after the value")
                        + 1;
                    let lines: String = missing
                        .iter()
                        .map(|(n, v)| format!("{}{indent}{n} = {v},", self.nl))
                        .collect();
                    self.edit(comma, comma, lines)
                } else {
                    let lines: String = missing
                        .iter()
                        .map(|(n, v)| format!(",{}{indent}{n} = {v}", self.nl))
                        .collect();
                    self.edit(le, le, lines)
                }
            }
        };
        let title = format!("Add missing fields ({})", missing.len());
        out.extend(self.action(title, REWRITE, vec![edit]));
    }
}
