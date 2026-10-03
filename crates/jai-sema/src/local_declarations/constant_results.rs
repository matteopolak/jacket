//! One lexical identity per named projection of one constant result group.
use super::*;
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct ConstantResult {
    declaration: Arc<syntax::ConstantResultsDeclaration>,
    index: usize,
}
impl ConstantResult {
    pub(super) fn name(&self) -> Symbol {
        self.declaration.names[self.index].0
    }
    pub(super) fn span(&self) -> Span {
        self.declaration.names[self.index].1
    }
}
pub(super) fn declarations(
    declaration: &syntax::ConstantResultsDeclaration,
    symbols: &Symbols,
) -> Vec<DeclarationSyntax> {
    let declaration = Arc::new(declaration.clone());
    declaration
        .names
        .iter()
        .enumerate()
        .filter(|(_, (name, _))| symbols.name(*name) != "_")
        .map(|(index, _)| {
            DeclarationSyntax::ConstantResult(ConstantResult {
                declaration: declaration.clone(),
                index,
            })
        })
        .collect()
}
impl Resolver<'_> {
    pub(super) fn local_constant_result_binding(
        &mut self,
        result: &ConstantResult,
    ) -> Result<Binding, Diagnostic> {
        let values = self.constant_result_values(&result.declaration)?;
        let value = values.get(result.index).cloned().ok_or_else(|| {
            Diagnostic::new(
                result.span(),
                "constant result count does not match declaration count",
            )
        })?;
        crate::compile_time::materialized_binding(value, None, result.span(), self.meta)
    }
    pub(crate) fn constant_result_values(
        &mut self,
        declaration: &syntax::ConstantResultsDeclaration,
    ) -> Result<Vec<jai_ir::ConstantValue>, Diagnostic> {
        let syntax::ExpressionKind::CompileTime(body) = &declaration.initializer.kind else {
            return Err(Diagnostic::new(
                declaration.initializer.span,
                "constant result groups require a #run initializer",
            ));
        };
        let used = declaration
            .names
            .iter()
            .map(|&(name, _)| self.symbols.name(name) != "_")
            .collect::<Vec<_>>();
        self.execute_compile_time_results(body, declaration.initializer.span, &used)
    }
}
