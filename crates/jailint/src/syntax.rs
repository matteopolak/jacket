//! The file a rule looks at: its syntax tree, tokens and text, the procedures in it, and
//! helpers to walk statements and expressions and to read source text.
use crate::facts::{Facts, Recorded};
use jaic::ast::{
    AsmItem, AsmOperand, Block, CodeBody, Decl, Expr, ExprKind as E, File, ForOver, ProcHeader,
    RunBody, Stmt, StmtKind as S,
};
use jaic::intern::Sym;
use jaic::lexer::{P, Tok, Token};
use jaic::sema::{Compiler, EntityId};
use jaic::source::{FileId, Span};

pub(crate) struct Cx<'a> {
    pub compiler: &'a Compiler,
    pub facts: &'a Facts,
    pub file: FileId,
    pub text: &'a str,
    pub tokens: &'a [Token],
    pub procs: Vec<ProcSite<'a>>,
    pub imports: Vec<crate::rules::unused_import::Unused>,
    /// The compile finished without errors (otherwise some code went unchecked).
    pub complete: bool,
}

/// A procedure literal with a body.
pub(crate) struct ProcSite<'a> {
    pub header: &'a ProcHeader,
    pub body: &'a Block,
    /// The declaration naming it (`name :: (...) { ... }`), if any.
    pub decl: Option<&'a Decl>,
    /// Polymorphic, or nested in a polymorphic procedure: checked once per instance, so types
    /// may differ between checks.
    pub poly: bool,
    /// An `#expand` macro (or nested in one): checked where it is expanded, with names that
    /// may belong to the caller.
    pub is_macro: bool,
    /// The compiler checked the whole body (in every instance).
    pub clean: bool,
}

impl ProcSite<'_> {
    /// Type facts recorded for this body describe it fully and mean the same in every check.
    pub fn typed(&self) -> bool {
        self.clean && !self.poly && !self.is_macro
    }
}

impl<'a> Cx<'a> {
    pub fn new(
        compiler: &'a Compiler,
        facts: &'a Facts,
        file: FileId,
        text: &'a str,
        ast: &'a File,
        tokens: &'a [Token],
        imports: Vec<crate::rules::unused_import::Unused>,
    ) -> Self {
        let mut procs = Vec::new();
        collect_procs(&ast.stmts, None, false, false, facts, &mut procs);
        Self {
            compiler,
            facts,
            file,
            text,
            tokens,
            procs,
            imports,
            complete: true,
        }
    }

    /// Source text of `span` (empty when out of range).
    pub fn src(&self, span: Span) -> &'a str {
        self.text
            .get(span.start as usize..span.end as usize)
            .unwrap_or("")
    }

    /// Byte range of `span` widened to balance its parentheses (`jaic::lexer::balanced`) and to
    /// take in parentheses written around the whole expression.
    pub fn whole(&self, span: Span) -> (usize, usize) {
        let balanced = jaic::lexer::balanced(self.tokens, span);
        let mut start = balanced.start as usize;
        let mut end = balanced.end as usize;
        // Parentheses written around the whole expression.
        let mut a = self
            .tokens
            .partition_point(|t| (t.span.start as usize) < start);
        let mut b = self
            .tokens
            .partition_point(|t| (t.span.end as usize) <= end);
        while a > 0
            && matches!(self.tokens[a - 1].tok, Tok::Punct(P::LParen))
            && matches!(
                self.tokens.get(b).map(|t| &t.tok),
                Some(Tok::Punct(P::RParen))
            )
            && !self.call_paren(a - 1)
        {
            a -= 1;
            start = self.tokens[a].span.start as usize;
            end = self.tokens[b].span.end as usize;
            b += 1;
        }
        (start, end)
    }

    /// The `(` at token `i` opens an argument list or a header, not a group.
    fn call_paren(&self, i: usize) -> bool {
        i > 0
            && matches!(
                self.tokens[i - 1].tok,
                Tok::Ident(n) if !matches!(
                    n.as_str(),
                    "if" | "ifx" | "then" | "else" | "return" | "while" | "case" | "xx" | "for"
                )
            )
            || matches!(
                self.tokens[i - 1].tok,
                Tok::Punct(P::RParen | P::RBracket | P::RBrace) | Tok::Directive(_)
            )
    }

    /// The source text of [`Cx::whole`].
    pub fn whole_src(&self, span: Span) -> &'a str {
        let (start, end) = self.whole(span);
        self.text.get(start..end).unwrap_or("")
    }

    /// The entities the identifier at `span` resolved to.
    pub fn entities(&self, span: Span) -> &[EntityId] {
        self.facts.idents.get(&span).map_or(&[], |v| v.as_slice())
    }

    /// The type recorded for an expression, if all its checks agreed.
    pub fn ty(&self, e: &Expr) -> Option<jaic::types::TypeId> {
        self.compiler.type_at(e.span)
    }

    /// The expression was a compile-time constant in every check.
    pub fn constant(&self, e: &Expr) -> bool {
        self.compiler
            .expr_fact(e.span)
            .is_some_and(|f| f.constant && !f.conflicting)
    }

    /// Token indices of identifiers spelled `name` within `start..end`, not counting member
    /// names (`x.name`).
    pub fn name_tokens(&self, name: Sym, start: u32, end: u32) -> impl Iterator<Item = usize> {
        let first = self.tokens.partition_point(|t| t.span.start < start);
        (first..self.tokens.len())
            .take_while(move |&i| self.tokens[i].span.end <= end)
            .filter(move |&i| {
                matches!(self.tokens[i].tok, Tok::Ident(n) if n == name)
                    && !(i > 0 && matches!(self.tokens[i - 1].tok, Tok::Punct(P::Dot)))
            })
    }

    /// The name occurs as an identifier within `start..end` anywhere but `except`.
    pub fn mentions(&self, name: Sym, start: u32, end: u32, except: Span) -> bool {
        self.name_tokens(name, start, end).any(|i| {
            let s = self.tokens[i].span;
            !(except.start <= s.start && s.end <= except.end)
        })
    }

    /// Start of the line holding `offset`, and the indentation written there.
    pub fn line_start(&self, offset: usize) -> (usize, &'a str) {
        let start = self.text[..offset.min(self.text.len())]
            .rfind('\n')
            .map_or(0, |n| n + 1);
        let rest = &self.text[start..];
        let indent = &rest[..rest.len() - rest.trim_start_matches([' ', '\t']).len()];
        (start, indent)
    }

    /// The byte range of a whole statement line (with its newline) when the statement is alone
    /// on its lines, else just the statement.
    pub fn statement_removal(&self, span: Span) -> (usize, usize) {
        let (line, _) = self.line_start(span.start as usize);
        let before = &self.text[line..span.start as usize];
        let mut end = span.end as usize;
        let rest = &self.text[end..];
        let line_end = rest.find('\n').map_or(self.text.len(), |n| end + n);
        let after = &self.text[end..line_end];
        // A `;` the statement's span stops before.
        let after_trim = after.trim_start();
        let after = if after_trim.starts_with(';') {
            end += after.len() - after_trim.len() + 1;
            &self.text[end..line_end]
        } else {
            after
        };
        if before.trim().is_empty() && (after.trim().is_empty() || after.trim().starts_with("//")) {
            let stop = (line_end + 1).min(self.text.len());
            (line, stop)
        } else {
            (span.start as usize, end)
        }
    }
}

/// A node of the tree, with its parents when walked.
#[derive(Clone, Copy)]
pub(crate) enum Node<'a> {
    Stmt(&'a Stmt),
    Expr(&'a Expr),
}

impl<'a> Node<'a> {
    pub fn expr(self) -> Option<&'a Expr> {
        match self {
            Node::Expr(e) => Some(e),
            Node::Stmt(_) => None,
        }
    }

    pub fn stmt(self) -> Option<&'a Stmt> {
        match self {
            Node::Stmt(s) => Some(s),
            Node::Expr(_) => None,
        }
    }
}

/// Visit every statement and expression under `stmts` (parents first) with the chain of
/// its ancestors (outermost first). `visit` returns whether to look inside the node. Nested
/// procedure bodies, struct and enum bodies are not entered.
pub(crate) fn walk<'a>(stmts: &'a [Stmt], visit: &mut dyn FnMut(Node<'a>, &[Node<'a>]) -> bool) {
    let mut stack = Vec::new();
    for s in stmts {
        walk_node(Node::Stmt(s), &mut stack, visit);
    }
}

pub(crate) fn walk_node<'a>(
    node: Node<'a>,
    stack: &mut Vec<Node<'a>>,
    visit: &mut dyn FnMut(Node<'a>, &[Node<'a>]) -> bool,
) {
    if !visit(node, stack) {
        return;
    }
    stack.push(node);
    for child in children(node) {
        walk_node(child, stack, visit);
    }
    stack.pop();
}

/// The direct children of a node, in source order.
pub(crate) fn children(node: Node<'_>) -> Vec<Node<'_>> {
    let mut out = Vec::new();
    match node {
        Node::Stmt(s) => stmt_children(s, &mut out),
        Node::Expr(e) => expr_children(e, &mut out),
    }
    out
}

fn stmts<'a>(list: &'a [Stmt], out: &mut Vec<Node<'a>>) {
    out.extend(list.iter().map(Node::Stmt));
}

fn stmt_children<'a>(s: &'a Stmt, out: &mut Vec<Node<'a>>) {
    let mut e = |x: &'a Expr| out.push(Node::Expr(x));
    match &s.kind {
        S::Decl(d) | S::AddContext(d) => {
            if let Some(t) = &d.ty {
                e(t);
            }
            if let Some(v) = &d.value {
                e(v);
            }
            for v in &d.extra_values {
                e(v);
            }
        }
        S::Expr(x) | S::Run(x) | S::Place(x) | S::Overlay(x) => e(x),
        S::Assign {
            lhs,
            rhs,
            ..
        } => {
            for x in lhs.iter().chain(rhs) {
                e(x);
            }
        }
        S::Block(b) => stmts(&b.stmts, out),
        S::If {
            cond,
            then_branch,
            else_branch,
        } => {
            e(cond);
            out.push(Node::Stmt(then_branch));
            if let Some(b) = else_branch {
                out.push(Node::Stmt(b));
            }
        }
        S::Switch {
            value,
            cases,
            ..
        }
        | S::StaticSwitch {
            value,
            cases,
        } => {
            e(value);
            for c in cases {
                for v in &c.values {
                    out.push(Node::Expr(v));
                }
                stmts(&c.body, out);
            }
        }
        S::While {
            cond,
            body,
            ..
        } => {
            e(cond);
            out.push(Node::Stmt(body));
        }
        S::For(f) => {
            match &f.over {
                ForOver::Range(a, b) => {
                    e(a);
                    e(b);
                }
                ForOver::Collection(c) => e(c),
            }
            for x in f.pointer_if.iter().chain(&f.reverse_if) {
                e(x);
            }
            out.push(Node::Stmt(&f.body));
        }
        S::Return {
            values, ..
        } => {
            for a in values {
                e(&a.value);
            }
        }
        S::Defer {
            body, ..
        } => out.push(Node::Stmt(body)),
        S::Using {
            value, ..
        } => e(value),
        S::PushContext {
            context,
            body,
        } => {
            e(context);
            out.push(Node::Stmt(body));
        }
        S::StaticIf {
            cond,
            then_branch,
            else_branch,
        } => {
            e(cond);
            stmts(then_branch, out);
            stmts(else_branch, out);
        }
        S::Insert {
            value,
            scope,
            replacements,
            ..
        } => {
            e(value);
            if let Some(x) = scope {
                e(x);
            }
            for a in replacements {
                e(&a.value);
            }
        }
        S::Assert {
            cond,
            message,
            args,
        } => {
            e(cond);
            if let Some(m) = message {
                e(m);
            }
            for x in args {
                e(x);
            }
        }
        S::PushContextDefer {
            context: x,
        } => e(x),
        S::Directive {
            args, ..
        } => {
            for x in args {
                e(x);
            }
        }
        S::Case(c) => {
            for v in &c.values {
                e(v);
            }
            stmts(&c.body, out);
        }
        _ => {}
    }
}

fn expr_children<'a>(x: &'a Expr, out: &mut Vec<Node<'a>>) {
    let mut e = |x: &'a Expr| out.push(Node::Expr(x));
    match &x.kind {
        E::Binary(_, a, b) | E::Index(a, b) => {
            e(a);
            e(b);
        }
        E::Unary(_, a) | E::Member(a, _) | E::Backtick(a) | E::ProcedureOfCall(a) => e(a),
        E::Exists(a) | E::Bytes(a) => e(a),
        E::PolyRestricted {
            restriction, ..
        } => e(restriction),
        E::Call {
            callee,
            args,
            ..
        }
        | E::Bake {
            callee,
            args,
            ..
        } => {
            e(callee);
            for a in args {
                if let Some(t) = &a.target {
                    e(t);
                }
                e(&a.value);
            }
        }
        E::Cast {
            ty,
            value,
            ..
        } => {
            if let Some(t) = ty {
                e(t);
            }
            e(value);
        }
        E::Ifx {
            cond,
            then_value,
            else_value,
            ..
        } => {
            e(cond);
            for v in then_value.iter().chain(else_value) {
                e(v);
            }
        }
        E::StructLit {
            ty,
            fields,
        } => {
            if let Some(t) = ty {
                e(t);
            }
            for a in fields {
                if let Some(t) = &a.target {
                    e(t);
                }
                e(&a.value);
            }
        }
        E::ArrayLit {
            ty,
            elems,
        } => {
            if let Some(t) = ty {
                e(t);
            }
            for v in elems {
                e(v);
            }
        }
        E::ArrayType {
            size,
            elem,
        } => {
            if let jaic::ast::ArraySize::Fixed(n) = size {
                e(n);
            }
            e(elem);
        }
        E::Block(b) => stmts(&b.stmts, out),
        E::TypeDirective {
            ty, ..
        } => e(ty),
        E::Run {
            body, ..
        } => match &**body {
            RunBody::Expr(x) => e(x),
            RunBody::Block(b) => stmts(&b.stmts, out),
        },
        E::Code(body) => match &**body {
            CodeBody::Expr(x) => e(x),
            CodeBody::Block(b) => stmts(&b.stmts, out),
        },
        E::Insert {
            value,
            scope,
            replacements,
            ..
        } => {
            e(value);
            if let Some(s) = scope {
                e(s);
            }
            for a in replacements {
                e(&a.value);
            }
        }
        E::Location(Some(a)) | E::ProcedureName(Some(a)) => e(a),
        E::UnknownDirective {
            operand: Some(a), ..
        } => e(a),
        E::Asm(block) => {
            for item in &block.items {
                if let AsmItem::Inst(inst) = item {
                    for op in &inst.operands {
                        match op {
                            AsmOperand::Value(v) => e(v),
                            AsmOperand::Mem(m) => {
                                for t in &m.terms {
                                    e(&t.value);
                                    if let Some(s) = &t.scale {
                                        e(s);
                                    }
                                }
                            }
                            AsmOperand::Decl(_) => {}
                        }
                    }
                }
            }
        }
        // Procedures, lambdas, structs and enums are not part of the enclosing body.
        _ => {}
    }
}

/// Is the header polymorphic (`$T` binders, baked parameters)?
pub(crate) fn header_is_poly(h: &ProcHeader) -> bool {
    h.params
        .iter()
        .any(|p| p.baked || p.auto_bake || p.ty.as_ref().is_some_and(jaic::sema::procs::has_poly))
        || h.returns
            .iter()
            .any(|r| r.ty.as_ref().is_some_and(jaic::sema::procs::has_poly))
}

/// Every procedure literal with a body under `list`, at any depth (inside other procedures,
/// structs, `#if` branches, `#run` blocks).
fn collect_procs<'a>(
    list: &'a [Stmt],
    decl: Option<&'a Decl>,
    poly: bool,
    is_macro: bool,
    facts: &Facts,
    out: &mut Vec<ProcSite<'a>>,
) {
    for s in list {
        let decl = match &s.kind {
            S::Decl(d) => Some(&**d),
            _ => decl.filter(|_| false),
        };
        let mut stack = Vec::new();
        find_procs(Node::Stmt(s), decl, poly, is_macro, facts, out, &mut stack);
    }
}

fn find_procs<'a>(
    node: Node<'a>,
    decl: Option<&'a Decl>,
    poly: bool,
    is_macro: bool,
    facts: &Facts,
    out: &mut Vec<ProcSite<'a>>,
    stack: &mut Vec<Node<'a>>,
) {
    walk_node(node, stack, &mut |n, _| {
        let Some(x) = n.expr() else {
            return true;
        };
        match &x.kind {
            E::Proc(lit) => {
                let header = &*lit.header;
                let p = poly || header_is_poly(header);
                let m = is_macro || header.flags.expand;
                // The declaration names this procedure only when it is its value.
                let named = decl.filter(|d| d.value.as_ref().is_some_and(|v| v.span == x.span));
                if let Some(body) = &lit.body {
                    out.push(ProcSite {
                        header,
                        body,
                        decl: named,
                        poly: p,
                        is_macro: m,
                        clean: facts.clean(body.span),
                    });
                    collect_procs(&body.stmts, None, p, m, facts, out);
                }
                false
            }
            E::Struct(st) => {
                collect_procs(&st.body, None, poly, is_macro, facts, out);
                false
            }
            _ => true,
        }
    });
}

/// Strip parentheses the parser kept in a span: the text of an expression as written.
pub(crate) fn is_atom(e: &Expr) -> bool {
    matches!(
        e.kind,
        E::Ident(_)
            | E::Member(..)
            | E::Call { .. }
            | E::Index(..)
            | E::Int(_)
            | E::Float(_)
            | E::Str(_)
            | E::Bool(_)
            | E::Null
    )
}

/// The identifier an access path starts from: `a` of `a.b[i].c`.
pub(crate) fn root_ident(e: &Expr) -> Option<(Sym, Span)> {
    match &e.kind {
        E::Ident(n) => Some((*n, e.span)),
        E::Member(base, _) | E::Index(base, _) => root_ident(base),
        E::Unary(jaic::ast::UnOp::Deref, base) => root_ident(base),
        _ => None,
    }
}

/// `a`, `a.b`, `a.b.c`: names and member accesses only.
pub(crate) fn is_path(e: &Expr) -> bool {
    match &e.kind {
        E::Ident(_) => true,
        E::Member(base, _) => is_path(base),
        _ => false,
    }
}

/// Text with whitespace removed, to compare two spellings of an expression.
pub(crate) fn squash(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
