//! Expanding pending declarations and plain static ifs.
use super::model::{EntityState, PendingState, ScopeId};
use crate::sema::Epochs;
use crate::sema::{Compiler, Result, ast};

/// A procedure-valued declaration can join an overload set.
/// A constant that can join an overload set: a procedure, or an alias of one (`print :: f;`).
pub(in crate::sema) fn may_overload(decl: &ast::Decl) -> bool {
    decl_is_proc(decl)
        || decl.kind == ast::DeclKind::Const
            && decl.ty.is_none()
            && matches!(
                decl.value.as_ref().map(|v| &v.kind),
                Some(ast::ExprKind::Ident(_) | ast::ExprKind::Member(..))
            )
}

pub(super) fn decl_is_proc(decl: &ast::Decl) -> bool {
    decl.kind == ast::DeclKind::Const
        && matches!(
            decl.value.as_ref().map(|v| &v.kind),
            Some(ast::ExprKind::Proc(_) | ast::ExprKind::Lambda { .. })
        )
}

/// A condition made of names, literals and operators only.
fn plain_condition(expr: &ast::Expr) -> bool {
    use ast::ExprKind as E;
    match &expr.kind {
        E::Ident(_) | E::Int(_) | E::Bool(_) | E::Str(_) | E::InferredMember(_) | E::Null => true,
        E::Member(base, _) => plain_condition(base),
        E::Unary(_, a) => plain_condition(a),
        E::Binary(_, a, b) => plain_condition(a) && plain_condition(b),
        _ => false,
    }
}

impl Compiler {
    /// Expand pending conditional items of a scope (each at most once).
    pub fn expand_pending(&mut self, scope: ScopeId) -> Result<()> {
        // Readiness probes may resolve existing bindings, but must not drain
        // dependencies whose failures the probe treats as "not ready yet".
        if self.epochs.lookup_without_expansion {
            return Ok(());
        }
        let s = self.scope_mut(scope);
        let done = s.pending_done;
        while s
            .pending
            .get(s.pending_done)
            .is_some_and(|p| p.state == PendingState::Done)
        {
            s.pending_done += 1;
        }
        let mut i = s.pending_done;
        if i != done {
            self.namespace_changed(scope);
        }
        while i < self.scope(scope).pending.len() {
            // Items expand in source order: a re-entrant expansion (from inside an
            // item's own condition or lookup) must not run later items early.
            if self.scope(scope).pending[i].state == PendingState::Expanding {
                return Ok(());
            }
            if self.scope(scope).pending[i].state == PendingState::Waiting {
                self.scope_mut(scope).pending[i].state = PendingState::Expanding;
                let stmt = self.scope(scope).pending[i].stmt.clone();
                let exported = self.scope(scope).pending[i].exported;
                let file_scope = self.scope(scope).pending[i].file_scope;
                // Compile-time code run from inside a body that is being lowered
                // (a lookup reached this scope) cannot call procedures that are
                // themselves mid-lowering. Put the item back and retry it later.
                let reentrant = !self.epochs.retrying_pending
                    && (self.epochs.resolving_depth > 0
                        || self
                            .pr
                            .procs
                            .iter()
                            .any(|p| p.body_state == crate::sema::procs::BodyState::Lowering));
                let misses = self.epochs.placeholder_misses;
                let executions = self.ct.executions;
                let in_progress = self.epochs.in_progress_misses;
                let result = self.expand_pending_item(scope, file_scope, &stmt, exported);
                // Retry only unresolved semantic work. A generator that executed
                // and failed is terminal, even while its enclosing body is lowering.
                // Compile-time code that spent the whole budget fails the same way every time.
                let spent = self.ct.interp.block_budget == Some(0);
                if result.is_err()
                    && !spent
                    && self.ct.executions == executions
                    && self.epochs.waits_for_placeholder(misses)
                {
                    self.scope_mut(scope).pending[i].state = PendingState::Waiting;
                    return Ok(());
                }
                // A condition that reaches a constant still being resolved (the lookup that
                // brought us here is part of resolving it) fails only because of that
                // outer resolution; the item is retried once the constant is done.
                if let Err(e) = &result
                    && !spent
                    && self.ct.executions == executions
                    && self.epochs.in_progress_misses != in_progress
                    && e.message.starts_with("circular dependency while resolving")
                    && self
                        .ns
                        .entities
                        .iter()
                        .any(|e| matches!(e.state, EntityState::Resolving))
                {
                    self.scope_mut(scope).pending[i].state = PendingState::Waiting;
                    return Ok(());
                }
                if reentrant
                    && !spent
                    && self.ct.executions == executions
                    && self.epochs.in_progress_misses != in_progress
                    && let Err(e) = result
                {
                    self.scope_mut(scope).pending[i].state = PendingState::Waiting;
                    self.ns.deferred_pending.push(scope);
                    self.diag.deferred_errors.push(e);
                    return Ok(());
                }
                self.scope_mut(scope).pending[i].state = PendingState::Done;
                result?;
            }
            i += 1;
        }
        Ok(())
    }

    /// Expand a scope's `#if` items whose conditions are plain constant expressions (no
    /// calls, no `#run`), ahead of items that run compile-time code. `expand_all` does
    /// this for every scope first, so a module's `#if FLAG #load "x.jai"` (and the
    /// `#add_context` in it) is in before any `#run` lays out the Context.
    pub fn expand_plain_ifs(&mut self, scope: ScopeId) -> Result<()> {
        // A constant reached by a readiness probe can re-enter expansion.
        // Committing pending declarations still belongs to the outer pass.
        if self.epochs.lookup_without_expansion {
            return Ok(());
        }
        for i in 0..self.scope(scope).pending.len() {
            let item = &self.scope(scope).pending[i];
            if item.state != PendingState::Waiting {
                continue;
            }
            // `#if OS == .MACOS` and `#if OS == { case .MACOS; ... }` alike.
            let plain = match &item.stmt.kind {
                ast::StmtKind::StaticIf(boxed) => {
                    let crate::ast::StaticIfStmt {
                        cond, ..
                    } = &**boxed;
                    plain_condition(cond)
                }
                ast::StmtKind::StaticSwitch(boxed) => {
                    let crate::ast::SwitchStmt {
                        value,
                        cases,
                        ..
                    } = &**boxed;
                    plain_condition(value)
                        && cases.iter().all(|c| c.values.iter().all(plain_condition))
                }
                _ => false,
            };
            if !plain {
                continue;
            }
            self.scope_mut(scope).pending[i].state = PendingState::Expanding;
            let item = &self.scope(scope).pending[i];
            let (stmt, exported, file_scope) = (item.stmt.clone(), item.exported, item.file_scope);
            // The condition must resolve from what is already declared; if it needs a name that
            // another pending item declares, the item waits for the ordinary lazy expansion.
            let saved = std::mem::replace(&mut self.epochs.lookup_without_expansion, true);
            let branch = self.select_plain_branch(scope, file_scope, &stmt);
            let ready = branch.is_some();
            self.epochs.lookup_without_expansion = saved;
            let misses = self.epochs.placeholder_misses;
            let result = if let Some(branch) = branch {
                // Commit the selection already made without draining unrelated
                // pending items. Declaration itself resumes ordinary lookup.
                let vis = if exported {
                    ast::ScopeKind::Export
                } else if scope == file_scope {
                    ast::ScopeKind::File
                } else {
                    ast::ScopeKind::Module
                };
                self.declare_stmts(scope, file_scope, &branch, vis)
                    .map(|_| ())
            } else {
                Ok(())
            };
            if !ready || (result.is_err() && self.epochs.waits_for_placeholder(misses)) {
                self.scope_mut(scope).pending[i].state = PendingState::Waiting;
                continue;
            }
            self.scope_mut(scope).pending[i].state = PendingState::Done;
            result?;
        }
        Ok(())
    }

    /// Select a plain conditional once, using only already declared names.
    /// Failure leaves it for ordinary expansion, which reports real errors.
    fn select_plain_branch(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmt: &ast::Stmt,
    ) -> Option<Vec<ast::Stmt>> {
        let eval_scope = crate::sema::modules::file_scope_for_eval(self, scope, file_scope);
        match &stmt.kind {
            ast::StmtKind::StaticIf(boxed) => {
                let ast::StaticIfStmt {
                    cond,
                    then_branch,
                    else_branch,
                } = &**boxed;
                let taken = self.eval_static_condition(eval_scope, cond).ok()?;
                let (branch, skipped) = if taken {
                    (then_branch, else_branch)
                } else {
                    (else_branch, then_branch)
                };
                self.ide_note_branches(cond, branch, skipped);
                Some(branch.clone())
            }
            ast::StmtKind::StaticSwitch(boxed) => {
                let ast::SwitchStmt {
                    value,
                    cases,
                    ..
                } = &**boxed;
                Some(
                    self.static_switch_case(eval_scope, value, cases)
                        .ok()?
                        .map_or_else(Vec::new, |case| case.body.clone()),
                )
            }
            _ => None,
        }
    }
}

impl Epochs {
    /// Did an item's failed expansion reach an undefined `#placeholder` (since `before`)?
    fn waits_for_placeholder(&self, before: u64) -> bool {
        !self.placeholders_final && self.placeholder_misses != before
    }
}
