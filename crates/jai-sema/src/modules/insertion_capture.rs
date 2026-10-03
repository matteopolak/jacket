//! Source insertion captures are rebound through genuine graph declarations.
use super::*;

impl FileScope<'_> {
    pub(crate) fn insertion_capture_code(
        &self,
        code: jai_types::CodeValueId,
        location: SourceSpan,
    ) -> Result<jai_modules::Binding, Diagnostic> {
        let mut declarations = self
            .declarations
            .values
            .iter()
            .filter_map(|(id, binding)| {
                if !matches!(binding, Binding::Code(candidate) if *candidate == code) {
                    return None;
                }
                let declaration = self.declarations.graph.declaration(*id)?;
                let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                    return None;
                };
                let quoted = matches!(constant.initializer.kind, syntax::ExpressionKind::Code(_));
                Some((!quoted, *id))
            })
            .collect::<Vec<_>>();
        // Prefer the retained quote declaration over an immutable alias. Both
        // candidates must already bind this exact registry-owned Code handle.
        declarations.sort_by_key(|(alias, id)| (*alias, id.index()));
        declarations
            .first()
            .map(|(_, id)| jai_modules::Binding::Declaration(*id))
            .ok_or_else(|| {
                Diagnostic::at_source(
                    location,
                    "local Code capture requires a graph source Code declaration receipt",
                )
            })
    }

    pub(crate) fn insertion_capture_type(
        &self,
        types: &TypeRegistry,
        records: &aggregates::parameterized::RecordSpecializations,
        ty: TypeId,
        location: SourceSpan,
    ) -> Result<jai_modules::ModuleType, Diagnostic> {
        parameter_discovery::encode_type(
            self.declarations.graph,
            &self.declarations.nominals,
            types,
            records,
            ty,
            location,
            0,
        )
        .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }

    pub(crate) fn insertion_capture_procedure(
        &self,
        procedure: ProcedureId,
        ty: TypeId,
        location: SourceSpan,
    ) -> Result<jai_modules::Binding, Diagnostic> {
        let mut declarations = self
            .declarations
            .signatures
            .iter()
            .filter_map(|(id, signature)| {
                (signature.id == procedure && signature.ty == ty)
                    .then(|| self.declarations.graph.declaration(*id))
                    .flatten()
                    .filter(|declaration| {
                        matches!(
                            declaration.syntax().kind,
                            FileDeclarationKind::Procedure(_)
                                | FileDeclarationKind::ProcedurePrototype(_)
                        )
                    })
                    .map(|_| *id)
            })
            .collect::<Vec<_>>();
        declarations.sort_by_key(|id| id.index());
        declarations.first().copied().map(jai_modules::Binding::Declaration).ok_or_else(|| Diagnostic::at_source(
            location,
            "declaration insertion cannot transport a local or specialized procedure without a retained graph callable binding",
        ))
    }
}
