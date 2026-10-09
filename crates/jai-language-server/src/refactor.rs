//! Refactoring code actions (`refactor.extract`, `refactor.inline`, `refactor.rewrite`).
//!
//! Every action is worked out from the parse tree of the document, plus a few facts of the
//! type-checked compile: the declared types of locals, the references of a name, and the members
//! of an enum or struct. An action is offered only when its edit is safe; anything unclear (a
//! variable another statement modifies, a type nobody knows, a selection that is not whole
//! statements) is declined rather than guessed.
//!
//! The text an action writes follows jaifmt's canonical layout: one statement per line, braces
//! joined to their headers, and one indentation unit (the one the document uses) per level.
use crate::analysis::Span;
use crate::session::Session;
use crate::{CodeAction, DocumentUri, TextEdit};
use jaic::ast::{
    AssignOp, BinOp, Block, DeclKind, Expr, ExprKind as E, File, ForOver, ProcHeader, Stmt,
    StmtKind as S, UnOp,
};
use jaic::intern::Sym;
use jaic::source::FileId;
use jailint::syntax::{Node, walk};
use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};

type Sp = jaic::source::Span;

// The refactoring kinds.

pub(crate) const EXTRACT: &str = "refactor.extract";
pub(crate) const INLINE: &str = "refactor.inline";
pub(crate) const REWRITE: &str = "refactor.rewrite";

impl Session {
    /// The refactorings that apply to the selection `start..end` (bytes) of `uri`.
    pub(crate) fn refactor_actions(
        &self,
        uri: &DocumentUri,
        start: usize,
        end: usize,
    ) -> Vec<CodeAction> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        if !self.analyses.get(uri).is_some_and(|a| a.complete) {
            return Vec::new();
        }
        let Ok(file) = jaic::parser::parse_file(FileId(0), &doc.text) else {
            return Vec::new();
        };
        let mut bodies = Vec::new();
        proc_bodies(&file.stmts, &mut bodies);
        let mut roots: Vec<Root> = vec![Root {
            stmts: &file.stmts,
            in_proc: false,
        }];
        roots.extend(bodies.iter().map(|b| Root {
            stmts: &b.1.stmts,
            in_proc: true,
        }));
        let cx = Cx {
            session: self,
            uri,
            text: &doc.text,
            file: &file,
            roots,
            sel: trim(&doc.text, start, end),
            cursor: start,
            unit: indent_unit(&doc.text),
            nl: if doc.text.contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            },
            types: OnceCell::new(),
        };
        let mut out = Vec::new();
        cx.extract_variable(&mut out);
        cx.extract_procedure(&mut out);
        cx.inline_variable(&mut out);
        cx.fill_switch(&mut out);
        cx.fill_struct(&mut out);
        cx.ifx_to_if(&mut out);
        cx.if_to_ifx(&mut out);
        out
    }
}

struct Root<'a> {
    stmts: &'a [Stmt],
    /// The statements are a procedure's body (a place for a local to live).
    in_proc: bool,
}

struct Cx<'a> {
    session: &'a Session,
    uri: &'a DocumentUri,
    text: &'a str,
    file: &'a File,
    roots: Vec<Root<'a>>,
    /// The selection, trimmed of whitespace.
    sel: (usize, usize),
    cursor: usize,
    unit: String,
    nl: &'static str,
    /// Declared types of locals, by the start of their names.
    types: OnceCell<HashMap<usize, String>>,
}

/// A local variable that is visible where a selection starts.
struct Local {
    name: Sym,
    ty: Option<String>,
    constant: bool,
}

// -------------------------------------------------------------------------------------------
// Text helpers
// -------------------------------------------------------------------------------------------

pub(crate) fn trim(text: &str, start: usize, end: usize) -> (usize, usize) {
    let end = end.min(text.len());
    let start = start.min(end);
    let slice = &text[start..end];
    let lead = slice.len() - slice.trim_start().len();
    let trail = slice.trim_start().len() - slice.trim().len();
    (start + lead, end - trail)
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |n| n + 1)
}

/// The end of the line holding `at`, before its line break.
fn line_end(text: &str, at: usize) -> usize {
    let end = text[at..].find('\n').map_or(text.len(), |n| at + n);
    if text[..end].ends_with('\r') {
        end - 1
    } else {
        end
    }
}

/// The leading whitespace of the line holding `at`.
fn indent_at(text: &str, at: usize) -> &str {
    let start = line_start(text, at);
    let line = &text[start..];
    let width = line
        .find(|c: char| c != ' ' && c != '\t')
        .unwrap_or(line.len());
    &line[..width]
}

/// Nothing but whitespace lies between the start of the line and `at`.
fn starts_line(text: &str, at: usize) -> bool {
    text[line_start(text, at)..at].trim().is_empty()
}

/// The indentation one level deeper is made of: the document's smallest indent step.
fn indent_unit(text: &str) -> String {
    let mut step: Option<usize> = None;
    for line in text.lines() {
        if line.starts_with('\t') {
            return "\t".into();
        }
        let width = line.len() - line.trim_start_matches(' ').len();
        if width > 0 && width < line.len() {
            step = Some(step.map_or(width, |s| s.min(width)));
        }
    }
    " ".repeat(step.filter(|s| (2..=8).contains(s)).unwrap_or(4))
}

/// Does `text` use `name` as a whole word?
fn mentions(text: &str, name: &str) -> bool {
    text.match_indices(name).any(|(at, _)| {
        let word = |c: char| c.is_alphanumeric() || c == '_';
        !text[..at].chars().next_back().is_some_and(word)
            && !text[at + name.len()..].chars().next().is_some_and(word)
    })
}

/// `base`, or `base2`, `base3` ... when the document already uses the name.
fn fresh(text: &str, base: &str) -> String {
    if !mentions(text, base) {
        return base.into();
    }
    (2..)
        .map(|n| format!("{base}{n}"))
        .find(|c| !mentions(text, c))
        .unwrap_or_else(|| base.into())
}

/// Parentheses and brackets of `text` outside string literals: the count of `(` minus `)`.
fn paren_balance(text: &str) -> i32 {
    let mut balance = 0;
    let mut in_string = false;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' if in_string => {
                chars.next();
            }
            '"' => in_string = !in_string,
            '(' if !in_string => balance += 1,
            ')' if !in_string => balance -= 1,
            _ => {}
        }
    }
    balance
}

/// The source extent of an expression. The parser's span can leave out parentheses around the
/// expression's first or last operand, so they are taken back in.
pub(crate) fn extent(text: &str, span: Sp) -> (usize, usize) {
    let (mut a, mut b) = (span.start as usize, span.end as usize);
    if a > b || b > text.len() || !text.is_char_boundary(a) || !text.is_char_boundary(b) {
        return (a, b);
    }
    let balance = paren_balance(&text[a..b]);
    for _ in 0..balance.unsigned_abs() {
        if balance < 0 {
            let before = text[..a].trim_end();
            if before.ends_with('(') {
                a = before.len() - 1;
            }
        } else {
            let after = text[b..].trim_start();
            if after.starts_with(')') {
                b = text.len() - after.len() + 1;
            }
        }
    }
    (a, b)
}

/// A statement's extent, through its `;`.
pub(crate) fn stmt_extent(text: &str, s: &Stmt) -> (usize, usize) {
    let (a, mut b) = (s.span.start as usize, s.span.end as usize);
    if b <= text.len() && a <= b {
        let rest = text[b..].trim_start_matches([' ', '\t']);
        if rest.starts_with(';') {
            b = text.len() - rest.len() + 1;
        }
    }
    (a, b)
}

/// `selection` without one pair of outer parentheses that enclose all of it.
fn unparenthesized(text: &str, (a, b): (usize, usize)) -> Option<(usize, usize)> {
    let slice = text.get(a..b)?;
    if slice.starts_with('(')
        && slice.ends_with(')')
        && paren_balance(&slice[1..slice.len() - 1]) == 0
    {
        // `(a) + (b)` has balanced insides too; the first `(` must close at the very end.
        let mut depth = 0;
        for (i, c) in slice.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 && i + 1 != slice.len() {
                        return None;
                    }
                }
                _ => {}
            }
        }
        return Some(trim(text, a + 1, b - 1));
    }
    None
}

/// The root variable of an assignable place: `a` of `a.b[2]`.
fn root(e: &Expr) -> Option<Sym> {
    match &e.kind {
        E::Ident(n) => Some(*n),
        E::Member(a, _) | E::Index(a, _) | E::Unary(UnOp::Deref | UnOp::Star, a) => root(a),
        _ => None,
    }
}

/// Every procedure literal with a body under `stmts`, at any depth.
fn proc_bodies<'a>(stmts: &'a [Stmt], out: &mut Vec<(&'a ProcHeader, &'a Block)>) {
    walk(stmts, &mut |n, _| {
        let Some(x) = n.expr() else {
            return true;
        };
        match &x.kind {
            E::Proc(lit) => {
                if let Some(body) = &lit.body {
                    out.push((&lit.header, body));
                    proc_bodies(&body.stmts, out);
                }
                false
            }
            E::Struct(st) => {
                proc_bodies(&st.body, out);
                false
            }
            _ => true,
        }
    });
}

/// A statement that sits in a list of statements, so more can be added before it: the nodes
/// above it end in a block, a case, or nothing (the top of a body).
fn in_list(parents: &[Node<'_>]) -> bool {
    match parents.last() {
        None => true,
        Some(Node::Stmt(s)) => matches!(
            s.kind,
            S::Block(_)
                | S::Switch { .. }
                | S::StaticSwitch { .. }
                | S::StaticIf { .. }
                | S::Case(_)
        ),
        Some(Node::Expr(e)) => matches!(e.kind, E::Block(_) | E::Run { .. } | E::Code(_)),
    }
}

/// The kinds of statement a hoisted or rewritten expression may belong to.
fn plain_statement(s: &Stmt) -> bool {
    matches!(
        s.kind,
        S::Decl(_)
            | S::Expr(_)
            | S::Assign { .. }
            | S::If { .. }
            | S::Switch { .. }
            | S::Return { .. }
            | S::For(_)
    )
}

impl<'a> Cx<'a> {
    fn ex(&self, e: &Expr) -> (usize, usize) {
        extent(self.text, e.span)
    }

    fn slice(&self, (a, b): (usize, usize)) -> &'a str {
        self.text.get(a..b).unwrap_or("")
    }

    fn edit(&self, start: usize, end: usize, new_text: String) -> Option<TextEdit> {
        Some(TextEdit {
            range: self.session.range_of(self.uri, Span::new(start, end))?,
            new_text,
        })
    }

    fn action(
        &self,
        title: String,
        kind: &'static str,
        edits: Vec<Option<TextEdit>>,
    ) -> Option<CodeAction> {
        let edits: Option<Vec<TextEdit>> = edits.into_iter().collect();
        Some(CodeAction {
            title,
            kind: Some(kind),
            edit: Some((self.uri.as_str().into(), edits?)),
            ..CodeAction::default()
        })
    }

    /// The declared type of every local (`x := 1` is `s64`), from the type-checked compile.
    fn declared(&self, at: usize) -> Option<String> {
        self.types
            .get_or_init(|| {
                self.session
                    .checked(self.uri, |a, f| a.compiler.ide_declared_types(f))
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(span, ty)| (span.start as usize, ty))
                    .collect()
            })
            .get(&at)
            .cloned()
    }

    /// The type of a declared name: as written, else as the compiler worked it out.
    fn type_of_decl(&self, d: &jaic::ast::Decl, index: usize) -> Option<String> {
        if let Some(ty) = &d.ty
            && d.names.len() == 1
        {
            return Some(self.slice(self.ex(ty)).to_string());
        }
        self.declared(d.names.get(index)?.span.start as usize)
    }

    /// The first expression (parents first) in the procedure bodies, or in every tree when
    /// `anywhere`, for which `pick` answers; with the nodes above it.
    fn find<T>(
        &self,
        anywhere: bool,
        mut pick: impl FnMut(Node<'a>, &[Node<'a>]) -> Option<T>,
    ) -> Option<(T, Vec<Node<'a>>)> {
        for root in self.roots.iter().filter(|r| anywhere || r.in_proc) {
            let mut found = None;
            walk(root.stmts, &mut |n, parents| {
                if found.is_some() {
                    return false;
                }
                if let Some(t) = pick(n, parents) {
                    found = Some((t, parents.to_vec()));
                    return false;
                }
                true
            });
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// The smallest node (an expression when `exprs`) around the cursor that `pick` accepts.
    fn innermost<T>(
        &self,
        exprs: bool,
        mut pick: impl FnMut(Node<'a>) -> Option<(T, (usize, usize))>,
    ) -> Option<(T, Vec<Node<'a>>)> {
        let mut best: Option<(usize, T, Vec<Node<'a>>)> = None;
        for root in &self.roots {
            walk(root.stmts, &mut |n, parents| {
                if n.expr().is_some() != exprs {
                    return true;
                }
                if let Some((t, (a, b))) = pick(n)
                    && a <= self.cursor
                    && self.cursor <= b
                    && best.as_ref().is_none_or(|(size, ..)| b - a <= *size)
                {
                    best = Some((b - a, t, parents.to_vec()));
                }
                true
            });
        }
        best.map(|(_, t, p)| (t, p))
    }

    // ---------------------------------------------------------------------------------------
    // Extract into variable
    // ---------------------------------------------------------------------------------------

    /// `foo(a + b)` with `a + b` selected becomes `value := a + b;` above and `foo(value)`.
    fn extract_variable(&self, out: &mut Vec<CodeAction>) {
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
            S::Switch {
                cases, ..
            } if cases
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

    // ---------------------------------------------------------------------------------------
    // Inline variable
    // ---------------------------------------------------------------------------------------

    /// `x := a + b;` with every use of `x` replaced by `a + b`, when `x` is never changed.
    fn inline_variable(&self, out: &mut Vec<CodeAction>) {
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
            || !d.extra_values.is_empty()
            || d.using
            || d.backtick
            || d.as_
            || d.align.is_some()
            || !d.flags.is_empty()
            || !d.notes.is_empty()
            || !d.existing.is_empty()
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

    // ---------------------------------------------------------------------------------------
    // Fill in missing cases and fields
    // ---------------------------------------------------------------------------------------

    /// Add a `case .NAME;` for each member an `if x == {` on an enum does not mention.
    fn fill_switch(&self, out: &mut Vec<CodeAction>) {
        let Some(((stmt, value, cases), _)) = self.innermost(false, |n| {
            let s = n.stmt()?;
            let S::Switch {
                value,
                cases,
                ..
            } = &s.kind
            else {
                return None;
            };
            Some(((s, value, cases), stmt_extent(self.text, s)))
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
        let mut named: HashSet<&str> = HashSet::new();
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
        // New cases go before a `case;` that takes the rest, else before the closing brace.
        let before = match cases.iter().find(|c| c.values.is_empty()) {
            Some(default) => default.span.start as usize,
            None => sb.saturating_sub(1),
        };
        if self.text.get(sb.saturating_sub(1)..sb) != Some("}") || !starts_line(self.text, before) {
            return;
        }
        let indent = match cases.first() {
            Some(c) => indent_at(self.text, c.span.start as usize).to_string(),
            None => format!("{}{}", indent_at(self.text, sa), self.unit),
        };
        let at = line_start(self.text, before);
        let text: String = missing
            .iter()
            .map(|n| format!("{indent}case .{n};{}", self.nl))
            .collect();
        let title = format!("Add missing cases ({})", missing.len());
        out.extend(self.action(title, REWRITE, vec![self.edit(at, at, text)]));
    }

    /// The members of the enum or struct at `at`.
    fn members(&self, at: usize, as_type: bool) -> Option<jaic::sema::ide_meta::IdeMembers> {
        self.session
            .checked(self.uri, |a, f| {
                a.compiler.ide_members_at(f, at as u32, as_type)
            })
            .flatten()
    }

    /// Add `name = value` for each field a `Type.{...}` literal leaves out.
    fn fill_struct(&self, out: &mut Vec<CodeAction>) {
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
                    let comma = le + self.text[le..].find(',').unwrap_or(0) + 1;
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

    // ---------------------------------------------------------------------------------------
    // ifx <-> if / else
    // ---------------------------------------------------------------------------------------

    /// `x = ifx c then a else b;` as an `if` that assigns (or returns) `a` or `b`.
    fn ifx_to_if(&self, out: &mut Vec<CodeAction>) {
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
                    && d.extra_values.is_empty()
                    && d.existing.is_empty()
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
    fn if_to_ifx(&self, out: &mut Vec<CodeAction>) {
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

    // ---------------------------------------------------------------------------------------
    // Extract into procedure
    // ---------------------------------------------------------------------------------------

    /// Whole statements selected become a procedure; the variables they read are its
    /// parameters, and the ones declared in them and used afterwards are its results.
    fn extract_procedure(&self, out: &mut Vec<CodeAction>) {
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
        if flags.c_call
            || flags.no_context
            || flags.expand
            || flags.compiler
            || flags.intrinsic
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
                    S::While {
                        bind_label,
                        label,
                        body,
                        ..
                    } => {
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
                    S::Switch {
                        cases, ..
                    } => cases.iter().find_map(|c| self.descend(&c.body, visible)),
                    S::StaticIf {
                        then_branch,
                        else_branch,
                        ..
                    } => self
                        .descend(then_branch, visible)
                        .or_else(|| self.descend(else_branch, visible)),
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
        let mut written: HashSet<Sym> = HashSet::new();
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
                    | S::Using {
                        ..
                    }
                    | S::Insert {
                        ..
                    }
                    | S::Directive {
                        ..
                    }
                    | S::Through
                    | S::Remove(_)
                    | S::AddContext(_)
                    | S::Case(_)
                    | S::Run(_)
                    | S::Import(_)
                    | S::Load {
                        ..
                    }
                    | S::PushContextDefer {
                        ..
                    } => ok = false,
                    S::Break(label) | S::Continue(label) => {
                        let inside = parents.iter().any(|p| {
                            matches!(p.stmt().map(|s| &s.kind), Some(S::While { .. } | S::For(_)))
                        });
                        ok &= inside && label.is_none();
                    }
                    S::Decl(d) => {
                        if d.using
                            || d.backtick
                            || d.as_
                            || !d.existing.is_empty()
                            || !d.backtick_names.is_empty()
                            || d.using_filter.is_some()
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
                    S::While {
                        bind_label: true,
                        label: Some(l),
                        ..
                    } => declared.push(l.name),
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
                    | E::Insert {
                        ..
                    }
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

fn header_poly(h: &ProcHeader) -> bool {
    h.params
        .iter()
        .any(|p| p.baked || p.auto_bake || p.ty.as_ref().is_some_and(jaic::sema::procs::has_poly))
        || h.returns
            .iter()
            .any(|r| r.ty.as_ref().is_some_and(jaic::sema::procs::has_poly))
}

/// Is `name` used by an expression in `stmts`?
fn mentioned_in(stmts: &[Stmt], name: Sym) -> bool {
    let mut found = false;
    walk(stmts, &mut |n, _| {
        if let Some(x) = n.expr()
            && matches!(x.kind, E::Ident(i) if i == name)
        {
            found = true;
        }
        !found
    });
    found
}

fn contains_range((a, b): (usize, usize), (x, y): (usize, usize)) -> bool {
    a <= x && y <= b
}

/// The only statement of a branch: itself, or the one in its block.
fn only(branch: &Stmt) -> Option<&Stmt> {
    match &branch.kind {
        S::Block(b) if b.stmts.len() == 1 => Some(&b.stmts[0]),
        S::Block(_) => None,
        _ => Some(branch),
    }
}

/// Expressions that name a value a variable can hold and that do not depend on where they run.
fn hoistable(e: &Expr) -> bool {
    match &e.kind {
        E::Ident(_)
        | E::Int(_)
        | E::Float(_)
        | E::Str(_)
        | E::Bool(_)
        | E::Null
        | E::Binary(..)
        | E::Call {
            ..
        }
        | E::Member(..)
        | E::Index(..)
        | E::Cast {
            ..
        }
        | E::Char(_)
        | E::Ifx {
            ..
        } => true,
        E::Unary(op, _) => !matches!(op, UnOp::Star),
        E::StructLit {
            ty, ..
        }
        | E::ArrayLit {
            ty, ..
        } => ty.is_some(),
        _ => false,
    }
}

/// A name that reads as a type (`Vector3`, `Hash_Table`): a variable cannot hold it.
fn type_like(e: &Expr) -> bool {
    let name = match &e.kind {
        E::Ident(n) => n.as_str(),
        E::Member(_, n) => n.name.as_str(),
        _ => return false,
    };
    name.starts_with(|c: char| c.is_ascii_uppercase())
        && name.chars().any(|c| c.is_ascii_lowercase())
}

/// A name for a value: what the call or member says it is.
fn suggest_name(e: &Expr) -> String {
    let name = match &e.kind {
        E::Call {
            callee, ..
        } => match &callee.kind {
            E::Ident(n) => n.as_str(),
            E::Member(_, n) => n.name.as_str(),
            _ => "",
        },
        E::Member(_, n) => n.name.as_str(),
        _ => "",
    };
    let name = name.strip_prefix("get_").unwrap_or(name);
    let valid = name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !crate::analysis::KEYWORDS.contains(&name);
    if valid && name.len() <= 24 {
        name.to_string()
    } else {
        "value".into()
    }
}

/// Does the expression call a procedure, so that running it twice (or later) matters?
fn has_call(e: &Expr) -> bool {
    let mut found = false;
    walk_expr(e, &mut |x| {
        found |= matches!(x.kind, E::Call { .. } | E::Run { .. } | E::Insert { .. });
    });
    found
}

fn walk_expr<'a>(e: &'a Expr, visit: &mut impl FnMut(&'a Expr)) {
    visit(e);
    for child in jailint::syntax::children(Node::Expr(e)) {
        if let Node::Expr(x) = child {
            walk_expr(x, visit);
        }
    }
}

/// Expressions that need no parentheses wherever they go.
fn atomic(e: &Expr) -> bool {
    matches!(
        e.kind,
        E::Ident(_)
            | E::Int(_)
            | E::Float(_)
            | E::Str(_)
            | E::Bool(_)
            | E::Null
            | E::Call { .. }
            | E::Member(..)
            | E::Index(..)
            | E::Char(_)
            | E::StructLit { .. }
            | E::ArrayLit { .. }
    )
}

/// Does the use `x`, under `parents`, assign to, or take the address of, its variable?
fn changes(parents: &[Node<'_>], x: &Expr) -> bool {
    let mut place = x.span;
    for node in parents.iter().rev() {
        match node {
            Node::Expr(up) => match &up.kind {
                E::Member(inner, _) | E::Index(inner, _) | E::Unary(UnOp::Deref, inner)
                    if inner.span == place =>
                {
                    place = up.span;
                }
                E::Unary(UnOp::Star, _) => return true,
                _ => return false,
            },
            Node::Stmt(s) => {
                return match &s.kind {
                    S::Assign {
                        lhs, ..
                    } => lhs.iter().any(|l| l.span == place),
                    S::For(f) => f.by_pointer,
                    _ => false,
                };
            }
        }
    }
    false
}
