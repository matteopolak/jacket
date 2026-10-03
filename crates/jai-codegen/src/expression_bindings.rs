//! Lexically scoped immutable SSA captures for checked expression bindings.
use crate::*;
use jai_ir::ExpressionBindingId;
use std::collections::HashSet;

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn native_condition(&self, value: &BoolExpr) -> Option<bool> {
        execution_phase::native_condition_with_bindings(value, &self.phase_bindings)
    }

    pub(super) fn native_boolean(&self, value: &BoolExpr) -> Option<bool> {
        execution_phase::native_boolean_with_bindings(value, &self.phase_bindings)
    }

    pub(super) fn bound_value(
        &mut self,
        binding: ExpressionBindingId,
        ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        if binding.procedure() != self.procedure {
            return Err(Error::Invariant);
        }
        let value = self
            .expression_bindings
            .get(&binding)
            .copied()
            .ok_or(Error::Invariant)?;
        if value.get_type() != self.lowerer.basic(ty)? {
            return Err(Error::Invariant);
        }
        Ok(value)
    }

    pub(super) fn bind_values(
        &mut self,
        bindings: &[(ExpressionBindingId, ValueExpr)],
        body: &ValueExpr,
        ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        if body.type_id(self.types) != ty {
            return Err(Error::Invariant);
        }
        let mut declared = HashSet::with_capacity(bindings.len());
        for (binding, _) in bindings {
            if binding.procedure() != self.procedure
                || self.expression_bindings.contains_key(binding)
                || !declared.insert(*binding)
            {
                return Err(Error::Invariant);
            }
        }
        let mut installed = Vec::with_capacity(bindings.len());
        let result = (|| {
            for (binding, producer) in bindings {
                // Install after evaluating the producer: earlier captures remain
                // readable, while this capture cannot refer to itself.
                let phase = execution_phase::native_value_condition(producer, &self.phase_bindings);
                let value = self.value(producer)?;
                self.expression_bindings.insert(*binding, value);
                self.phase_bindings.insert(*binding, phase);
                installed.push(*binding);
            }
            self.value(body)
        })();
        // Nested binds restore themselves before returning. Remove only this
        // scope, preserving captures belonging to an enclosing expression.
        for binding in installed.into_iter().rev() {
            self.expression_bindings.remove(&binding);
            self.phase_bindings.remove(&binding);
        }
        result
    }
}
