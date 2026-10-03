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
        declaration: LocalDeclarationId,
        result: &ConstantResult,
    ) -> Result<Binding, Diagnostic> {
        let values = self.constant_result_values(&result.declaration)?;
        let siblings: Vec<_> = self
            .local_scopes
            .frames
            .last()
            .expect("constant projections retain their defining frame")
            .declarations
            .values()
            .filter_map(|declaration| match &declaration.syntax {
                DeclarationSyntax::ConstantResult(sibling)
                    if Arc::ptr_eq(&sibling.declaration, &result.declaration) =>
                {
                    Some((declaration.id, sibling.index, sibling.span()))
                }
                _ => None,
            })
            .collect();
        let mut bindings = Vec::with_capacity(siblings.len());
        for (id, index, span) in siblings {
            let value = values.get(index).cloned().ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "constant result count does not match declaration count",
                )
            })?;
            let binding = crate::compile_time::materialized_binding(value, None, span, self.meta)?;
            bindings.push((id, binding));
        }
        let selected = bindings
            .iter()
            .find(|(id, _)| *id == declaration)
            .expect("registered result projection belongs to its group")
            .1
            .clone();
        for (id, binding) in bindings {
            let entry = self.meta.local_declarations.entries.get_mut(&id).unwrap();
            entry.binding = Some(binding);
            entry.scope_watermark = self.local_scopes.next_scope;
        }
        Ok(selected)
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
