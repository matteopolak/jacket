//! What Jai reads from the shape of a procedure body without evaluating it: whether every path
//! returns (`not all control paths return a value`) and statements that follow a `return`,
//! `break` or `continue` in the same block.
//!
//! Both follow the structure of the code, not the values in it: `if true return 1;` can still
//! fall through, `while true {}` ends the body, and a `while` counts as returning when its body
//! does. Only `#if` is decided, by the branch the checker took (`FnCtx::static_taken`).
use super::*;
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
            S::Switch {
                cases,
                complete,
                ..
            } => {
                !cases.is_empty()
                    && (*complete || cases.iter().any(|c| c.values.is_empty()))
                    && cases
                        .iter()
                        .all(|c| c.through || self.stmts_return(f, &c.body))
            }
            S::While {
                body, ..
            }
            | S::PushContext {
                body, ..
            } => self.stmt_returns(f, body),
            S::StaticIf {
                then_branch,
                else_branch,
                ..
            } => match f.static_taken.get(&stmt.span) {
                Some(true) => self.stmts_return(f, then_branch),
                Some(false) => self.stmts_return(f, else_branch),
                None => true,
            },
            // Chosen or generated at compile time: assume the best.
            S::StaticSwitch {
                ..
            }
            | S::Insert {
                ..
            } => true,
            _ => false,
        }
    }

    /// Warn at the first statement that follows a `return`, `break` or `continue` of its block.
    pub(super) fn warn_unreachable(&mut self, f: &FnCtx, stmts: &[ast::Stmt]) {
        for (i, stmt) in stmts.iter().enumerate() {
            let leaves = match &stmt.kind {
                S::Return {
                    ..
                } => Some("return"),
                S::Break(_) => Some("break"),
                S::Continue(_) => Some("continue"),
                _ => None,
            };
            if let Some(word) = leaves
                && let Some(next) = stmts[i + 1..].iter().find(|s| runs_in_order(s))
                && (next.span.file.0 as usize) < self.sources.len()
            {
                self.warn(
                    Diagnostic::warning(
                        next.span,
                        format!("statements in block after `{word}` are never reached"),
                    )
                    .with_label("this statement can not run")
                    .with_help(format!("remove it, or the `{word}` before it")),
                );
                // The rest of the block is dead too; one warning per block.
                return;
            }
            self.warn_unreachable_inside(f, stmt);
        }
    }

    fn warn_unreachable_inside(&mut self, f: &FnCtx, stmt: &ast::Stmt) {
        match &stmt.kind {
            S::Block(b) => self.warn_unreachable(f, &b.stmts),
            S::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.warn_unreachable_inside(f, then_branch);
                if let Some(e) = else_branch {
                    self.warn_unreachable_inside(f, e);
                }
            }
            S::While {
                body, ..
            }
            | S::Defer {
                body, ..
            }
            | S::PushContext {
                body, ..
            } => self.warn_unreachable_inside(f, body),
            S::For(for_) => self.warn_unreachable_inside(f, &for_.body),
            S::Switch {
                cases, ..
            } => {
                // A case body is not a block of its own: what follows a `return` there is
                // not reported, but the blocks inside it are looked at.
                for case in cases {
                    for s in &case.body {
                        self.warn_unreachable_inside(f, s);
                    }
                }
            }
            S::StaticIf {
                then_branch,
                else_branch,
                ..
            } => match f.static_taken.get(&stmt.span) {
                Some(true) => self.warn_unreachable(f, then_branch),
                Some(false) => self.warn_unreachable(f, else_branch),
                None => {}
            },
            _ => {}
        }
    }
}

/// Whether `stmt` is something that would run: constants, and statements that only configure
/// the program, do not count.
fn runs_in_order(stmt: &ast::Stmt) -> bool {
    match &stmt.kind {
        S::Decl(decl) => decl.kind != ast::DeclKind::Const,
        S::Empty
        | S::Scope(_)
        | S::AddContext(_)
        | S::ModuleParameters {
            ..
        }
        | S::Load {
            ..
        }
        | S::Case(_)
        | S::Overlay(_) => false,
        _ => true,
    }
}
