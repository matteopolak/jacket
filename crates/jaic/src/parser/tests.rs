//! Parser unit tests, focused on the ambiguous corners of the grammar.
mod expressions;
mod headers_and_types;
mod recovery;
mod statements;

use super::parse_file;
use crate::ast::*;
use crate::source::FileId;
use std::rc::Rc;

fn parse(src: &str) -> Vec<Stmt> {
    match parse_file(FileId(0), src) {
        Ok(file) => file.stmts,
        Err(d) => panic!(
            "parse error: {} (bytes {}..{})\n{src}",
            d.message, d.span.start, d.span.end
        ),
    }
}

fn error(src: &str) -> String {
    match parse_file(FileId(0), src) {
        Ok(_) => panic!("expected a parse error for:\n{src}"),
        Err(d) => d.message,
    }
}

fn decl(stmt: &Stmt) -> &Rc<Decl> {
    match &stmt.kind {
        StmtKind::Decl(decl) => decl,
        other => panic!("expected a declaration, found {other:?}"),
    }
}

/// The value of the first declaration in `src`.
fn value(src: &str) -> Expr {
    let stmts = parse(src);
    decl(&stmts[0])
        .value
        .clone()
        .expect("declaration has a value")
}

fn proc_of(expr: &Expr) -> &ProcLit {
    match &expr.kind {
        ExprKind::Proc(lit) => lit,
        other => panic!("expected a procedure, found {other:?}"),
    }
}

/// Statements of the body of `f :: () { ... }`.
fn body_of(src: &str) -> Vec<Stmt> {
    let stmts = parse(&format!("f :: () {{\n{src}\n}}"));
    proc_of(decl(&stmts[0]).value.as_ref().unwrap())
        .body
        .clone()
        .unwrap()
        .stmts
}
