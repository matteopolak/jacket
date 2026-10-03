//! Resolve lexical loop names once, before LLVM block construction.
use super::{
    Diagnostic, Exit, HashMap, LoopBinding, LoopCondition, LoopId, RangeLoop, Resolver, Span,
    Statement, Symbol, Transfer, ValueExpr, syntax,
};

impl Resolver<'_> {
    pub(super) fn enter_loop(&mut self, name: Option<Symbol>) -> LoopId {
        let id = LoopId::new(self.next_loop);
        self.next_loop += 1;
        self.loops.push(LoopBinding {
            id,
            name,
            cleanup_depth: self.deferred_scopes.len(),
            iteration: None,
        });
        id
    }
    pub(super) fn resolve_while(
        &mut self,
        condition: &syntax::WhileCondition,
        body: &[syntax::Statement],
    ) -> Result<Statement, Diagnostic> {
        self.scopes.push(HashMap::new());
        let (condition, name) = match condition {
            syntax::WhileCondition::Expression(e) => {
                (LoopCondition::Value(self.condition_expression(e)?), None)
            }
            syntax::WhileCondition::Binding {
                name,
                export_span,
                initializer,
            } => {
                let value = self.expr(initializer)?.value(initializer.span)?;
                let condition = match value {
                    ValueExpr::Int(e) => {
                        LoopCondition::BoundInt(self.declare_int(*name, e.ty())?, e)
                    }
                    ValueExpr::Bool(e) => LoopCondition::BoundBool(self.declare_bool(*name)?, e),
                    _ => {
                        return Err(Diagnostic::new(
                            initializer.span,
                            "while binding requires an integer or bool value",
                        ));
                    }
                };
                if let Some(span) = export_span {
                    self.export_bound_name(*name, *span)?;
                }
                (condition, Some(*name))
            }
        };
        let id = self.enter_loop(name);
        let body = self.block(body, true)?;
        match &condition {
            LoopCondition::BoundInt(local, _) => {
                self.debug.local_in_completed_block(local.local().id())
            }
            LoopCondition::BoundBool(local, _) => {
                self.debug.local_in_completed_block(local.local().id())
            }
            LoopCondition::Value(_) => {}
        }
        self.debug
            .attach_block(&[jai_ir::DebugPathStep::Child(jai_ir::DebugBranch::While)]);
        self.loops.pop();
        self.scopes.pop();
        Ok(Statement::While {
            id,
            condition,
            body,
        })
    }
    pub(super) fn resolve_range(
        &mut self,
        range: &syntax::RangeLoop,
    ) -> Result<Statement, Diagnostic> {
        // Resolve endpoints before introducing the iterator; outer names stay visible.
        let direction =
            self.iteration_direction(range.direction, range.reverse_control.as_ref())?;
        let (start, end) = Self::integer_pair(
            self.expr(&range.start)?,
            self.expr(&range.end)?,
            range.start.span,
        )?;
        self.scopes.push(HashMap::new());
        let iterator = self.declare_int(range.iterator, start.ty())?;
        if range.iterator_export {
            self.export_bound_name(range.iterator, range.start.span)?;
        }
        let id = self.enter_loop(Some(range.iterator));
        let body = self.block(&range.body, true)?;
        self.debug.local_in_completed_block(iterator.local().id());
        self.debug
            .attach_block(&[jai_ir::DebugPathStep::Child(jai_ir::DebugBranch::Range)]);
        self.loops.pop();
        self.scopes.pop();
        Ok(Statement::Range(RangeLoop {
            id,
            iterator,
            start,
            end,
            direction,
            body,
        }))
    }
    pub(super) fn resolve_jump(
        &mut self,
        kind: syntax::JumpKind,
        target: syntax::LoopTarget,
        span: Span,
    ) -> Result<Statement, Diagnostic> {
        if let Some(replacement) = self.resolve_loop_replacement(kind, target, span)? {
            return Ok(replacement);
        }
        if kind == syntax::JumpKind::Remove {
            return self.resolve_remove(target, span);
        }
        let (index, active) = match target {
            syntax::LoopTarget::Innermost => self.loops.last().map(|l| (self.loops.len() - 1, l)),
            syntax::LoopTarget::Named(name) => self
                .loops
                .iter()
                .enumerate()
                .rev()
                .find(|(_, l)| l.name == Some(name)),
        }
        .ok_or_else(|| {
            Diagnostic::new(span, "break/continue must target an active enclosing loop")
        })?;
        if self
            .cleanup_context
            .is_some_and(|context| index < context.loop_depth)
        {
            return Err(Diagnostic::new(
                span,
                "a deferred body cannot exit an enclosing loop",
            ));
        }
        let mut cleanups = self.pending_cleanups(active.cleanup_depth);
        if kind == syntax::JumpKind::Continue
            && let Some(iteration) = &active.iteration
        {
            cleanups.push(iteration.latch);
        }
        Ok(Statement::Exit(Exit {
            cleanups,
            transfer: match kind {
                syntax::JumpKind::Break => Transfer::Break(active.id),
                syntax::JumpKind::Continue => Transfer::Continue(active.id),
                syntax::JumpKind::Remove => unreachable!("removal resolved before transfers"),
            },
        }))
    }
}
