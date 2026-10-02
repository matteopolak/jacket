//! Resolve lexical loop names once, before LLVM block construction.
use super::{
    Diagnostic, Exit, HashMap, LoopBinding, LoopCondition, LoopId, RangeLoop, Resolver, Span,
    Statement, Symbol, Transfer, ValueExpr, syntax,
};

impl Resolver<'_> {
    fn enter_loop(&mut self, name: Option<Symbol>) -> LoopId {
        let id = LoopId(self.next_loop);
        self.next_loop += 1;
        self.loops.push(LoopBinding {
            id,
            name,
            cleanup_depth: self.deferred_scopes.len(),
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
                (LoopCondition::Value(self.expr(e)?.condition(e.span)?), None)
            }
            syntax::WhileCondition::Binding { name, initializer } => {
                let value = self.expr(initializer)?.value(initializer.span)?;
                let condition = match value {
                    ValueExpr::Int(e) => {
                        LoopCondition::BoundInt(self.declare_int(*name, e.ty())?, e)
                    }
                    ValueExpr::Bool(e) => LoopCondition::BoundBool(self.declare_bool(*name)?, e),
                };
                (condition, Some(*name))
            }
        };
        let id = self.enter_loop(name);
        let body = self.block(body, true)?;
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
        let (start, end) = Self::integer_pair(
            self.expr(&range.start)?,
            self.expr(&range.end)?,
            range.start.span,
        )?;
        self.scopes.push(HashMap::new());
        let iterator = self.declare_int(range.iterator, start.ty())?;
        let id = self.enter_loop(Some(range.iterator));
        let body = self.block(&range.body, true)?;
        self.loops.pop();
        self.scopes.pop();
        Ok(Statement::Range(RangeLoop {
            id,
            iterator,
            start,
            end,
            direction: range.direction,
            body,
        }))
    }
    pub(super) fn resolve_jump(
        &self,
        kind: syntax::JumpKind,
        target: syntax::LoopTarget,
        span: Span,
    ) -> Result<Statement, Diagnostic> {
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
        Ok(Statement::Exit(Exit {
            cleanups: self.pending_cleanups(active.cleanup_depth),
            transfer: match kind {
                syntax::JumpKind::Break => Transfer::Break(active.id),
                syntax::JumpKind::Continue => Transfer::Continue(active.id),
            },
        }))
    }
}
