//! Publish one original alias only after its representation is ready.
use super::super::parameterized::{TypePreparation, TypeRequest, prepare_type_paired};
use super::*;

impl Nominals<'_> {
    pub(crate) fn prepare_alias(
        &mut self,
        graph: &ModuleGraph,
        id: DeclarationId,
        types: &mut TypeRegistry,
        records: &mut super::super::parameterized::RecordSpecializations,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<
            crate::modules::constants::ScalarPreparation,
            LocatedDiagnostic,
        >,
    ) -> Result<Option<TypePreparation>, LocatedDiagnostic> {
        let declaration = graph
            .declaration(id)
            .expect("alias origin belongs to graph");
        let annotation = match &declaration.syntax().kind {
            FileDeclarationKind::TypeAlias(alias) => alias.ty.clone(),
            FileDeclarationKind::Constant(constant) if self.is_type_alias(graph, id) => {
                expression_type(&constant.initializer).expect("classified alias has type syntax")
            }
            _ => return Ok(None),
        };
        let representation = match &annotation {
            TypeSyntax::Variant { base, .. } => base.as_ref(),
            annotation => annotation,
        };
        if let TypeSyntax::Named(path) = representation
            && let Ok(origin) =
                declaration_id(graph, declaration.file(), path, declaration.location().span)
            && matches!(&graph.declaration(origin).unwrap().syntax().kind,
                FileDeclarationKind::Record(record) if !record.parameters.is_empty())
        {
            return Ok(None);
        }
        let preparation = prepare_type_paired(
            graph,
            TypeRequest::new(
                declaration.file(),
                representation,
                declaration.location().span,
            ),
            types,
            self,
            records,
            evaluate,
        )?;
        let TypePreparation::Ready(resolved) = preparation else {
            return Ok(Some(preparation));
        };
        let ty = if matches!(annotation, TypeSyntax::Variant { .. }) {
            let owner = self.declarations[&id];
            match types.distinct_definition(owner) {
                Ok(definition) if definition.representation == resolved => {}
                Ok(_) => {
                    return Err(located(
                        graph,
                        declaration.file(),
                        Diagnostic::new(
                            declaration.location().span,
                            "a prepared alias changed its canonical representation",
                        ),
                    ));
                }
                Err(jai_types::TypeError::Incomplete(incomplete)) if incomplete == owner => {
                    types.define_distinct(owner, resolved).map_err(|error| {
                        located(
                            graph,
                            declaration.file(),
                            Diagnostic::new(declaration.location().span, error.to_string()),
                        )
                    })?;
                }
                Err(error) => {
                    return Err(located(
                        graph,
                        declaration.file(),
                        Diagnostic::new(declaration.location().span, error.to_string()),
                    ));
                }
            }
            owner
        } else {
            resolved
        };
        self.declarations.insert(id, ty);
        Ok(Some(TypePreparation::Ready(ty)))
    }
}
