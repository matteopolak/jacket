//! Resolve lexical source overrides before constructing typed operation nodes.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ActiveChecks {
    pub(crate) array_bounds: jai_ir::CheckMode,
    pub(crate) arithmetic_overflow: jai_ir::CheckMode,
}

impl Default for ActiveChecks {
    fn default() -> Self {
        Self {
            array_bounds: jai_ir::CheckMode::Enabled,
            arithmetic_overflow: jai_ir::CheckMode::Enabled,
        }
    }
}

impl ActiveChecks {
    pub(crate) fn overridden(self, checks: syntax::SafetyChecks) -> Self {
        fn resolve(current: jai_ir::CheckMode, policy: syntax::CheckPolicy) -> jai_ir::CheckMode {
            match policy {
                syntax::CheckPolicy::Inherited => current,
                syntax::CheckPolicy::Disabled => jai_ir::CheckMode::Disabled,
            }
        }
        Self {
            array_bounds: resolve(self.array_bounds, checks.array_bounds),
            arithmetic_overflow: resolve(self.arithmetic_overflow, checks.arithmetic_overflow),
        }
    }
}

impl Resolver<'_> {
    pub(crate) fn check_static_index(
        &mut self,
        base: TypeId,
        index: &IntExpr,
        source: &syntax::Expression,
    ) -> Result<(), Diagnostic> {
        if !self.checks.array_bounds.enabled() {
            return Ok(());
        }
        let Ok(jai_types::TypeKind::FixedArray {
            count, ..
        }) = self.types.kind(base)
        else {
            return Ok(());
        };
        let count = *count;
        // Only the pure evaluator is queried here. Calls, #run, storage reads,
        // and other execution-dependent operands retain their runtime guard.
        let value = match index.kind() {
            IntExprKind::Constant(index) => Some(index.value()),
            _ => match self.local_scalar_expression(source) {
                Ok(ScalarConstant::Int(index)) => Some(index.value()),
                Ok(ScalarConstant::Literal(index)) => Some(index),
                _ => None,
            },
        };
        if value.is_some_and(|value| value < 0 || value >= i128::from(count)) {
            return Err(Diagnostic::new(
                source.span,
                "constant array index is outside its fixed array bounds",
            ));
        }
        Ok(())
    }

    pub(crate) fn checked_block(
        &mut self,
        checks: syntax::SafetyChecks,
        body: &[syntax::Statement],
        scoped: bool,
    ) -> Result<Block, Diagnostic> {
        let previous = self.checks;
        self.checks = previous.overridden(checks);
        let result = self.block(body, scoped);
        self.checks = previous;
        result
    }
}
