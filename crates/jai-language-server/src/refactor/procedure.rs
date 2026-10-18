//! Extract procedure: the variables a selection reads and writes.
use super::exprs::{header_poly, mentioned_in};
use super::text::{fresh, stmt_extent};
use super::tree::root;
use super::{Cx, EXTRACT, Local};
use crate::CodeAction;
use jaic::ast::{DeclKind, ExprKind as E, ForOver, Stmt, StmtKind as S, UnOp};
use jaic::fxhash::HashSet;
use jaic::intern::Sym;
use jaic::source::{line_end, line_start};
use jailint::syntax::{Node, walk};

impl Cx<'_> {
    // Extract into procedure

    /// Whole statements selected become a procedure; the variables they read are its
    /// parameters, and the ones declared in them and used afterwards are its results.
    pub(super) fn extract_procedure(&self, out: &mut Vec<CodeAction>) {
        let (a, b) = self.sel;
        if a >= b {
            return;
        }
        // The top-level procedure holding the selection.
        let Some((top, header, body)) = self.file.stmts.iter().find_map(|s| {
            let S::Decl(d) = &s.kind else {
                return None;
            };
            let E::Proc(lit) = &d.value.as_ref()?.kind else {
                return None;
            };
            let body = lit.body.as_ref()?;
            let (ba, bb) = (body.span.start as usize, body.span.end as usize);
            (d.names.len() == 1 && ba <= a && b <= bb).then_some((s, &*lit.header, body))
        }) else {
            return;
        };
        let flags = &header.flags;
        if flags.c_call()
            || flags.no_context()
            || flags.expand()
            || flags.compiler()
            || flags.intrinsic()
            || header.foreign.is_some()
            || header_poly(header)
        {
            return;
        }
        let mut visible: Vec<Local> = Vec::new();
        for p in &header.params {
            let Some(name) = p.name else {
                continue;
            };
            let usable =
                !p.baked && !p.auto_bake && !p.variadic && !p.using && p.using_filter.is_none();
            visible.push(Local {
                name: name.name,
                ty: p
                    .ty
                    .as_ref()
                    .filter(|_| usable)
                    .map(|t| self.slice(self.ex(t)).to_string()),
                constant: false,
            });
        }
        let Some((list, i, j)) = self.descend(&body.stmts, &mut visible) else {
            return;
        };
        let selected = &list[i..=j];
        let (first, last) = (
            stmt_extent(self.text, &selected[0]).0,
            stmt_extent(self.text, &selected[selected.len() - 1]).1,
        );
        // Whole lines, so the code moves as it is written.
        let (la, lb) = (line_start(self.text, first), line_end(self.text, last));
        if !self.text[la..first].trim().is_empty() || !self.text[last..lb].trim().is_empty() {
            return;
        }
        let base = &self.text[la..first];
        let Some(code) = self.facts(selected, &list[j + 1..], &visible) else {
            return;
        };
        let name = fresh(self.text, "extracted");
        let params: Vec<String> = code
            .params
            .iter()
            .map(|(n, t)| format!("{n}: {t}"))
            .collect();
        let results: Vec<&str> = code.results.iter().map(|(_, t)| t.as_str()).collect();
        let args: Vec<&str> = code.params.iter().map(|(n, _)| n.as_str()).collect();
        let names: Vec<&str> = code.results.iter().map(|(n, _)| n.as_str()).collect();
        let mut proc = format!("{name} :: ({})", params.join(", "));
        if !results.is_empty() {
            proc.push_str(&format!(" -> {}", results.join(", ")));
        }
        proc.push_str(&format!(" {{{}", self.nl));
        for line in self.text[la..lb].lines() {
            if line.trim().is_empty() {
                proc.push_str(self.nl);
                continue;
            }
            let Some(rest) = line.strip_prefix(base) else {
                return;
            };
            proc.push_str(&format!("{}{rest}{}", self.unit, self.nl));
        }
        if !names.is_empty() {
            proc.push_str(&format!(
                "{}return {};{}",
                self.unit,
                names.join(", "),
                self.nl
            ));
        }
        proc.push('}');
        let call = format!("{name}({})", args.join(", "));
        let call = if names.is_empty() {
            format!("{base}{call};")
        } else {
            format!("{base}{} := {call};", names.join(", "))
        };
        let (_, top_end) = stmt_extent(self.text, top);
        let edits = vec![
            self.edit(la, lb, call),
            self.edit(top_end, top_end, format!("{}{}{proc}", self.nl, self.nl)),
        ];
        out.extend(self.action("Extract into procedure".into(), EXTRACT, edits));
    }

    /// Search `list` for the statements the selection covers: the list, and the first and last
    /// index. The locals declared before them (in this list and the ones around) go in `visible`.
    fn descend<'s>(
        &self,
        list: &'s [Stmt],
        visible: &mut Vec<Local>,
    ) -> Option<(&'s [Stmt], usize, usize)> {
        let (a, b) = self.sel;
        let extents: Vec<(usize, usize)> = list.iter().map(|s| stmt_extent(self.text, s)).collect();
        let i = extents.iter().position(|e| e.0 == a);
        let j = extents.iter().rposition(|e| e.1 == b);
        if let (Some(i), Some(j)) = (i, j)
            && i <= j
        {
            for s in &list[..i] {
                self.declare(s, visible);
            }
            return Some((list, i, j));
        }
        for (k, s) in list.iter().enumerate() {
            if extents[k].0 <= a && b <= extents[k].1 {
                for earlier in &list[..k] {
                    self.declare(earlier, visible);
                }
                let mark = visible.len();
                let found = match &s.kind {
                    S::Block(blk) => self.descend(&blk.stmts, visible),
                    S::If {
                        then_branch,
                        else_branch,
                        ..
                    } => self
                        .descend(std::slice::from_ref(then_branch), visible)
                        .or_else(|| {
                            else_branch
                                .as_ref()
                                .and_then(|e| self.descend(std::slice::from_ref(e), visible))
                        }),
                    S::While(boxed) => {
                        let jaic::ast::WhileStmt {
                            bind_label,
                            label,
                            body,
                            ..
                        } = &**boxed;
                        if *bind_label && let Some(l) = label {
                            visible.push(Local {
                                name: l.name,
                                ty: self.declared(l.span.start as usize),
                                constant: false,
                            });
                        }
                        self.descend(std::slice::from_ref(body), visible)
                    }
                    S::For(f) => {
                        for (given, default) in [(&f.it, "it"), (&f.index, "it_index")] {
                            visible.push(Local {
                                name: given.map_or_else(|| Sym::intern(default), |g| g.name),
                                ty: given.and_then(|g| self.declared(g.span.start as usize)),
                                constant: false,
                            });
                        }
                        self.descend(std::slice::from_ref(&f.body), visible)
                    }
                    S::Switch(boxed) => {
                        let jaic::ast::SwitchStmt {
                            cases, ..
                        } = &**boxed;
                        cases.iter().find_map(|c| self.descend(&c.body, visible))
                    }
                    S::StaticIf(boxed) => {
                        let jaic::ast::StaticIfStmt {
                            then_branch,
                            else_branch,
                            ..
                        } = &**boxed;
                        self.descend(then_branch, visible)
                            .or_else(|| self.descend(else_branch, visible))
                    }
                    _ => None,
                };
                if found.is_none() {
                    visible.truncate(mark);
                }
                return found;
            }
        }
        None
    }

    /// The locals a statement declares into its list.
    fn declare(&self, s: &Stmt, visible: &mut Vec<Local>) {
        if let S::Decl(d) = &s.kind {
            for (index, name) in d.names.iter().enumerate() {
                visible.push(Local {
                    name: name.name,
                    ty: self.type_of_decl(d, index),
                    constant: d.kind == DeclKind::Const,
                });
            }
        }
    }

    /// What extracting `selected` needs: its parameters and results, or `None` if it cannot be
    /// extracted (it returns, breaks out of the selection, changes an outer variable, ...).
    fn facts(&self, selected: &[Stmt], after: &[Stmt], visible: &[Local]) -> Option<Extraction> {
        let mut declared: Vec<Sym> = Vec::new();
        let mut used: Vec<Sym> = Vec::new();
        let mut written: HashSet<Sym> = HashSet::default();
        let mut ok = true;
        walk(selected, &mut |n, parents| {
            match n {
                Node::Stmt(s) => match &s.kind {
                    S::Return {
                        ..
                    }
                    | S::Defer {
                        ..
                    }
                    | S::Using(_)
                    | S::Insert(_)
                    | S::Directive(_)
                    | S::Through
                    | S::Remove(_)
                    | S::AddContext(_)
                    | S::Case(_)
                    | S::Run(_)
                    | S::Import(_)
                    | S::Load {
                        ..
                    }
                    | S::PushContextDefer(_) => ok = false,
                    S::Break(label) | S::Continue(label) => {
                        let inside = parents.iter().any(|p| {
                            matches!(p.stmt().map(|s| &s.kind), Some(S::While(_) | S::For(_)))
                        });
                        ok &= inside && label.is_none();
                    }
                    S::Decl(d) => {
                        if d.using
                            || d.backtick
                            || d.as_
                            || !d.existing().is_empty()
                            || !d.backtick_names().is_empty()
                            || d.using_filter().is_some()
                        {
                            ok = false;
                        }
                        declared.extend(d.names.iter().map(|n| n.name));
                    }
                    S::Assign {
                        lhs, ..
                    } => {
                        for l in lhs {
                            match root(l) {
                                Some(r) => {
                                    written.insert(r);
                                }
                                None => ok = false,
                            }
                        }
                    }
                    S::For(f) => {
                        if f.backtick_names || f.iterator.is_some() {
                            ok = false;
                        }
                        declared.push(f.it.map_or_else(|| Sym::intern("it"), |i| i.name));
                        declared.push(f.index.map_or_else(|| Sym::intern("it_index"), |i| i.name));
                        if f.by_pointer
                            && let ForOver::Collection(c) = &f.over
                            && let Some(r) = root(c)
                        {
                            written.insert(r);
                        }
                    }
                    S::While(w) if w.bind_label => {
                        if let Some(l) = w.label {
                            declared.push(l.name);
                        }
                    }
                    _ => {}
                },
                Node::Expr(x) => match &x.kind {
                    E::Ident(name) => used.push(*name),
                    E::Unary(UnOp::Star, inner) => {
                        if let Some(r) = root(inner) {
                            written.insert(r);
                        }
                    }
                    E::Proc(_)
                    | E::Lambda {
                        ..
                    }
                    | E::Struct(_)
                    | E::Enum(_)
                    | E::Code(_)
                    | E::Insert(_)
                    | E::Asm(_)
                    | E::Backtick(_)
                    | E::This
                    | E::Run {
                        ..
                    }
                    | E::PolyVar {
                        ..
                    }
                    | E::PolyRestricted {
                        ..
                    } => ok = false,
                    _ => {}
                },
            }
            ok
        });
        if !ok {
            return None;
        }
        // Parameters: the outer locals read, once each, in order of use.
        let mut params: Vec<(String, String)> = Vec::new();
        for name in &used {
            let Some(local) = visible.iter().rev().find(|l| l.name == *name) else {
                continue;
            };
            if params.iter().any(|(n, _)| n == name.as_str()) {
                continue;
            }
            if local.constant || written.contains(name) || declared.contains(name) {
                return None;
            }
            let ty = local.ty.clone().filter(|t| !t.contains('$'))?;
            params.push((name.as_str().to_string(), ty));
        }
        // An outer variable assigned but never read as a plain name.
        if written
            .iter()
            .any(|w| visible.iter().any(|l| l.name == *w) && !declared.contains(w))
        {
            return None;
        }
        // Results: declared by the selection itself and used after it.
        let mut results: Vec<(String, String)> = Vec::new();
        for s in selected {
            let S::Decl(d) = &s.kind else {
                continue;
            };
            for (index, name) in d.names.iter().enumerate() {
                if !mentioned_in(after, name.name) {
                    continue;
                }
                if d.kind == DeclKind::Const {
                    return None;
                }
                let ty = self.type_of_decl(d, index).filter(|t| !t.contains('$'))?;
                results.push((name.name.as_str().to_string(), ty));
            }
        }
        Some(Extraction {
            params,
            results,
        })
    }
}

struct Extraction {
    params: Vec<(String, String)>,
    results: Vec<(String, String)>,
}
