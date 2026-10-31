//! Walking a syntax tree: every statement and expression under a list of statements, with the
//! chain of ancestors.
use super::{
    AsmItem, AsmOperand, CodeBody, Expr, ExprKind as E, ForOver, RunBody, Stmt, StmtKind as S,
};

/// A node of the tree, with its parents when walked.
#[derive(Clone, Copy)]
pub enum Node<'a> {
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

impl<'a> From<&'a Expr> for Node<'a> {
    fn from(e: &'a Expr) -> Self {
        Node::Expr(e)
    }
}

impl<'a> From<&'a Box<Expr>> for Node<'a> {
    fn from(e: &'a Box<Expr>) -> Self {
        Node::Expr(e)
    }
}

impl<'a> From<&'a Box<Stmt>> for Node<'a> {
    fn from(s: &'a Box<Stmt>) -> Self {
        Node::Stmt(s)
    }
}

impl<'a> From<&'a Stmt> for Node<'a> {
    fn from(s: &'a Stmt) -> Self {
        Node::Stmt(s)
    }
}

/// The full source range of a node: its span with the parentheses written around it, and the
/// parentheses around its first and last children that the span leaves out (`(a + b) * c`
/// spans `a + b) * c`).
pub fn extent(node: Node<'_>) -> crate::source::Span {
    let mut out = body_extent(node);
    if let Node::Expr(e) = node {
        let outer = e.outer_span();
        out.start = out.start.min(outer.start);
        out.end = out.end.max(outer.end);
    }
    out
}

/// [`extent`] without the parentheses written around the node itself.
pub fn body_extent(node: Node<'_>) -> crate::source::Span {
    let mut out = match node {
        Node::Expr(e) => e.span,
        Node::Stmt(s) => s.span,
    };
    let kids = children(node);
    for kid in [kids.first(), kids.last()].into_iter().flatten() {
        let inner = extent(*kid);
        if inner.file == out.file {
            out.start = out.start.min(inner.start);
            out.end = out.end.max(inner.end);
        }
    }
    out
}

/// Visit every statement and expression under `stmts` (parents first) with the chain of
/// its ancestors (outermost first). `visit` returns whether to look inside the node. Nested
/// procedure bodies, struct and enum bodies are not entered.
pub fn walk<'a>(stmts: &'a [Stmt], visit: &mut dyn FnMut(Node<'a>, &[Node<'a>]) -> bool) {
    let mut stack = Vec::new();
    for s in stmts {
        walk_node(Node::Stmt(s), &mut stack, visit);
    }
}

pub fn walk_node<'a>(
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
pub fn children(node: Node<'_>) -> Vec<Node<'_>> {
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
            for v in d.extra_values() {
                e(v);
            }
        }
        S::Expr(x) => e(x),
        S::Run(x) | S::Place(x) | S::Overlay(x) => e(x),
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
        S::Switch(boxed) | S::StaticSwitch(boxed) => {
            let super::SwitchStmt {
                value,
                cases,
                ..
            } = &**boxed;
            e(value);
            for c in cases {
                for v in &c.values {
                    out.push(Node::Expr(v));
                }
                stmts(&c.body, out);
            }
        }
        S::While(boxed) => {
            let super::WhileStmt {
                cond,
                body,
                ..
            } = &**boxed;
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
        S::Using(boxed) => {
            let super::UsingStmt {
                value, ..
            } = &**boxed;
            e(value);
        }
        S::PushContext(boxed) => {
            let super::PushContextStmt {
                context,
                body,
                ..
            } = &**boxed;
            e(context);
            out.push(Node::Stmt(body));
        }
        S::StaticIf(boxed) => {
            let super::StaticIfStmt {
                cond,
                then_branch,
                else_branch,
                ..
            } = &**boxed;
            e(cond);
            stmts(then_branch, out);
            stmts(else_branch, out);
        }
        S::Insert(boxed) => {
            let super::InsertStmt {
                value,
                scope,
                replacements,
                ..
            } = &**boxed;
            e(value);
            if let Some(x) = scope {
                e(x);
            }
            for a in replacements {
                e(&a.value);
            }
        }
        S::Assert(boxed) => {
            let super::AssertStmt {
                cond,
                message,
                args,
            } = &**boxed;
            e(cond);
            if let Some(m) = message {
                e(m);
            }
            for x in args {
                e(x);
            }
        }
        S::PushContextDefer(boxed) => {
            let super::PushContextDeferStmt {
                context: x, ..
            } = &**boxed;
            e(x);
        }
        S::Directive(boxed) => {
            let super::DirectiveStmt {
                args, ..
            } = &**boxed;
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
        E::Bytes(a) => e(a),
        E::Exists(a, other) => {
            e(a);
            if let Some(other) = other {
                e(other);
            }
        }
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
            if let super::ArraySize::Fixed(n) = size {
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
            CodeBody::Stmt(s) => out.push(Node::Stmt(s)),
            CodeBody::Block(b) => stmts(&b.stmts, out),
        },
        E::Insert(boxed) => {
            let super::InsertExpr {
                value,
                scope,
                replacements,
                ..
            } = &**boxed;
            e(value);
            if let Some(s) = scope {
                e(s);
            }
            for a in replacements {
                e(&a.value);
            }
        }
        E::Location(Some(a)) | E::ProcedureName(Some(a)) => e(a),
        E::UnknownDirective(d) => {
            if let Some(a) = &d.operand {
                e(a);
            }
        }
        E::Asm(block) => {
            let operands = block.items.iter().filter_map(|item| match item {
                AsmItem::Inst(inst) => Some(&inst.operands),
                AsmItem::Decl(_) => None,
            });
            for op in operands.flatten() {
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
        // Procedures, lambdas, structs and enums are not part of the enclosing body.
        _ => {}
    }
}
