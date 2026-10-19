//! What Jai reads from the shape of a procedure body without evaluating it: whether every path
//! returns (`not all control paths return a value`) and statements that follow a `return`,
//! `break` or `continue` in the same block. The second is `reachability.rs`.
//!
//! Both follow the structure of the code, not the values in it: `if true return 1;` can still
//! fall through, `while true {}` ends the body, and a `while` counts as returning when its body
//! does. Only `#if` is decided, by the branch the checker took (`FnCtx::static_taken`).
use super::{Compiler, ast, lower};
use ast::StmtKind as S;
use lower::FnCtx;

impl Compiler {
    /// Whether leaving the end of `stmts` is impossible because one of them returns.
    pub(super) fn stmts_return(&self, f: &FnCtx, stmts: &[ast::Stmt]) -> bool {
        stmts.iter().any(|s| self.stmt_returns(f, s))
    }

    fn stmt_returns(&self, f: &FnCtx, stmt: &ast::Stmt) -> bool {
        match &stmt.kind {
            S::Return {
                ..
            } => true,
            S::Block(b) => self.stmts_return(f, &b.stmts),
            S::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.stmt_returns(f, then_branch)
                    && else_branch
                        .as_ref()
                        .is_some_and(|e| self.stmt_returns(f, e))
            }
            S::Switch(boxed) => {
                let crate::ast::SwitchStmt {
                    cases,
                    complete,
                    ..
                } = &**boxed;
                !cases.is_empty()
                    && (*complete || cases.iter().any(|c| c.values.is_empty()))
                    && cases
                        .iter()
                        .all(|c| c.through || self.stmts_return(f, &c.body))
            }
            S::While(w) => self.stmt_returns(f, &w.body),
            S::PushContext(p) => self.stmt_returns(f, &p.body),
            S::StaticIf(boxed) => {
                let crate::ast::StaticIfStmt {
                    then_branch,
                    else_branch,
                    ..
                } = &**boxed;
                match f.static_taken.get(&stmt.span) {
                    Some(true) => self.stmts_return(f, then_branch),
                    Some(false) => self.stmts_return(f, else_branch),
                    None => true,
                }
            }
            // Chosen or generated at compile time: assume the best.
            S::StaticSwitch(_) | S::Insert(_) => true,
            _ => false,
        }
    }
}
