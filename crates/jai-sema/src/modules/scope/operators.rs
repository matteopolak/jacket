//! Typed operator lookup never manufactures a callable name path.
use super::*;

impl FileScope<'_> {
    pub(crate) fn operator_template_patterns(
        &self,
        source: &syntax::Procedure,
        types: &mut TypeRegistry,
        records: &mut aggregates::parameterized::RecordSpecializations,
        lexical: &aggregates::parameterized::LexicalTypeArguments,
    ) -> Result<HashMap<(usize, usize), crate::overloads::TypePattern>, Diagnostic> {
        aggregates::parameterized::procedure_application_patterns(
            self.declarations.graph,
            self.file,
            source,
            types,
            &self.declarations.nominals,
            records,
            &mut |file, expression| {
                let scope = FileScope {
                    file,
                    ..*self
                };
                jai_eval::evaluate_paths(expression, |path, span| {
                    match scope.value(path, span)? {
                        Binding::Constant(value) => Ok(value),
                        Binding::Enum(value) => Ok(ConstantValue::Int(value.value)),
                        _ => Err(Diagnostic::new(
                            span,
                            "operator template count requires a definition-site constant",
                        )),
                    }
                })
                .map_err(|error| located(self.declarations.graph, file, error))
            },
            lexical,
        )
        .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }

    pub(crate) fn operator_declarations(&self, kind: syntax::OperatorKind) -> Vec<DeclarationId> {
        self.declarations
            .graph
            .operator_declarations(self.file, kind)
    }
    pub(crate) fn exported_operator_declarations(
        &self,
        module: jai_source::ModuleId,
        kind: syntax::OperatorKind,
    ) -> Vec<DeclarationId> {
        self.declarations
            .graph
            .exported_operator_declarations(module, kind)
    }
    pub(crate) fn operator_metadata(
        &self,
        declaration: DeclarationId,
    ) -> Option<syntax::OperatorDeclaration> {
        match &self
            .declarations
            .graph
            .declaration(declaration)?
            .syntax()
            .kind
        {
            FileDeclarationKind::Procedure(procedure) => procedure.operator,
            _ => None,
        }
    }
}

impl FileScope<'_> {
    pub(crate) fn using_exported_operators(
        &self,
        module: jai_source::ModuleId,
    ) -> Vec<(syntax::OperatorKind, Vec<DeclarationId>)> {
        let graph = self.declarations.graph;
        let kinds = graph
            .declarations()
            .iter()
            .filter_map(|declaration| {
                let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind else {
                    return None;
                };
                procedure.operator.map(|operator| operator.kind)
            })
            .collect::<std::collections::HashSet<_>>();
        kinds
            .into_iter()
            .filter_map(|kind| {
                let declarations = graph.exported_operator_declarations(module, kind);
                (!declarations.is_empty()).then_some((kind, declarations))
            })
            .collect()
    }
}
