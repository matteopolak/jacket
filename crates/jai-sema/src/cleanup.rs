//! Lower lexically activated defers into explicit cleanup IDs on each exit path.
use super::{
    CleanupContext, CleanupId, Diagnostic, Exit, Resolver, ReturnType, ScalarType, Statement,
    Transfer, syntax,
};

impl Resolver<'_> {
    pub(super) fn pending_cleanups(&self, depth: usize) -> Vec<CleanupId> {
        self.deferred_scopes[depth..]
            .iter()
            .rev()
            .flat_map(|scope| scope.iter().rev().copied())
            .collect()
    }
    pub(super) fn register_defer(&mut self, body: &[syntax::Statement]) -> Result<(), Diagnostic> {
        let previous = self.cleanup_context;
        self.cleanup_context = Some(CleanupContext {
            loop_depth: self.loops.len(),
        });
        let cleanup = self.block(body, true)?;
        self.cleanup_context = previous;
        let id = CleanupId(self.cleanups.len());
        self.cleanups.push(cleanup);
        self.deferred_scopes
            .last_mut()
            .expect("defer belongs to a block")
            .push(id);
        Ok(())
    }
    pub(super) fn resolve_return(
        &self,
        expression: Option<&syntax::Expression>,
    ) -> Result<Statement, Diagnostic> {
        if self.cleanup_context.is_some() {
            return Err(self.error("a deferred body cannot return from the enclosing procedure"));
        }
        let transfer = match (self.result, expression) {
            (ReturnType::Void, None) => Transfer::ReturnVoid,
            (ReturnType::Value(ScalarType::Int(ty)), Some(e)) => {
                Transfer::ReturnInt(self.expr(e)?.int_as(ty, e.span)?)
            }
            (ReturnType::Value(ScalarType::Bool), Some(e)) => {
                Transfer::ReturnBool(self.expr(e)?.bool(e.span)?)
            }
            _ => return Err(self.error("return value does not match procedure signature")),
        };
        Ok(Statement::Exit(Exit {
            cleanups: self.pending_cleanups(0),
            transfer,
        }))
    }
}
