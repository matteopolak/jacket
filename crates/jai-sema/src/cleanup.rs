//! Lower lexically activated defers into explicit cleanup IDs on each exit path.
use super::{
    CleanupControlContext, CleanupId, Diagnostic, Exit, Resolver, Statement, Transfer, syntax,
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
        self.cleanup_context = Some(CleanupControlContext {
            loop_depth: self.loops.len(),
        });
        let shadows = self.local_scopes.capture_cleanup_runtime(&self.scopes);
        let cleanup = self.block(body, true);
        self.local_scopes.restore_cleanup_runtime(shadows);
        self.cleanup_context = previous;
        let cleanup = cleanup?;
        let id = CleanupId::new(self.cleanups.len());
        self.debug.cleanup(id);
        self.cleanups.push(jai_ir::Cleanup {
            body: cleanup,
            context: match self.active_push {
                Some(id) => jai_ir::CleanupContext::Push(id),
                None => jai_ir::CleanupContext::Procedure,
            },
        });
        self.deferred_scopes
            .last_mut()
            .expect("defer belongs to a block")
            .push(id);
        Ok(())
    }
    pub(super) fn resolve_return(
        &mut self,
        expression: Option<&syntax::Expression>,
    ) -> Result<Statement, Diagnostic> {
        if let Some(expression) = expression {
            if self.results.len() == 1 && self.results[0].name.is_none() {
                if self.cleanup_context.is_some() {
                    return Err(
                        self.error("a deferred body cannot return from the enclosing procedure")
                    );
                }
                let ty = self.results[0].ty;
                let value = self.expr_expected(expression, ty)?;
                let transfer = match self
                    .types
                    .kind(ty)
                    .map_err(|error| self.error(error.to_string()))?
                {
                    jai_types::TypeKind::Integer(integer) => {
                        Transfer::ReturnInt(value.int_as(*integer, expression.span)?)
                    }
                    jai_types::TypeKind::Bool => Transfer::ReturnBool(value.bool(expression.span)?),
                    _ => {
                        let value = self.coerce_value(value, ty, expression.span)?;
                        self.reject_returned_pack_alias(&value, expression.span)?;
                        super::sequences::reject_returned_temporary(
                            &value,
                            self.types,
                            expression.span,
                        )?;
                        Transfer::ReturnValues(vec![value])
                    }
                };
                return Ok(Statement::Exit(Exit {
                    cleanups: self.pending_cleanups(0),
                    transfer,
                }));
            }
            self.resolve_return_values(&[syntax::ReturnValue {
                name: None,
                value: expression.clone(),
            }])
        } else {
            self.resolve_return_values(&[])
        }
    }
    pub(super) fn resolve_return_values(
        &mut self,
        source: &[syntax::ReturnValue],
    ) -> Result<Statement, Diagnostic> {
        if self.cleanup_context.is_some() {
            return Err(self.error("a deferred body cannot return from the enclosing procedure"));
        }
        if self.results.is_empty() {
            if !source.is_empty() {
                return Err(self.error("return value does not match procedure signature"));
            }
            return Ok(Statement::Exit(Exit {
                cleanups: self.pending_cleanups(0),
                transfer: Transfer::ReturnVoid,
            }));
        }
        let mut bound = vec![false; self.results.len()];
        let mut values: Vec<Option<super::ValueExpr>> =
            (0..self.results.len()).map(|_| None).collect();
        let mut statements = Vec::new();
        let mut positional = 0;
        let mut named = false;
        for value in source {
            let index = if let Some(name) = value.name {
                named = true;
                self.results
                    .iter()
                    .position(|result| result.name == Some(name))
                    .ok_or_else(|| {
                        super::Diagnostic::new(value.value.span, "unknown named return")
                    })?
            } else {
                if named {
                    return Err(super::Diagnostic::new(
                        value.value.span,
                        "positional return cannot follow a named return",
                    ));
                }
                let index = positional;
                positional += 1;
                index
            };
            let result = self.results.get(index).ok_or_else(|| {
                super::Diagnostic::new(value.value.span, "too many return values")
            })?;
            if bound[index] {
                return Err(super::Diagnostic::new(
                    value.value.span,
                    "duplicate return for result",
                ));
            }
            bound[index] = true;
            let ty = result.ty;
            let expression = self.expr_expected(&value.value, ty)?;
            let expression = self.coerce_value(expression, ty, value.value.span)?;
            self.reject_returned_pack_alias(&expression, value.value.span)?;
            super::sequences::reject_returned_temporary(&expression, self.types, value.value.span)?;
            let local = self.allocate_typed(ty)?;
            statements.push(Statement::Store(local.place(), expression));
            values[index] = Some(super::ValueExpr::Load(local.place()));
        }
        for (index, result) in self.results.iter().enumerate() {
            if values[index].is_none() {
                values[index] = Some(
                    result
                        .default
                        .clone()
                        .ok_or_else(|| self.error("missing required return value"))?
                        .into_expression(),
                );
            }
        }
        statements.push(Statement::Exit(Exit {
            cleanups: self.pending_cleanups(0),
            transfer: Transfer::ReturnValues(values.into_iter().map(Option::unwrap).collect()),
        }));
        Ok(Statement::Block(super::Block {
            statements,
            flow: super::Flow::Terminates,
        }))
    }
}
