//! A module lambda retains its actual constant declaration and defining file.
use super::*;

impl<'a> FileScope<'a> {
    pub(crate) fn source_results_required(&self, declaration: DeclarationId) -> Option<bool> {
        let results = match &self
            .declarations
            .graph
            .declaration(declaration)?
            .syntax()
            .kind
        {
            FileDeclarationKind::Procedure(source) => &source.results,
            FileDeclarationKind::ProcedurePrototype(source) => &source.results,
            _ => return None,
        };
        Some(
            results
                .iter()
                .any(|result| result.usage == syntax::ResultUsage::Required),
        )
    }
    pub(crate) fn short_lambda_source(
        self,
        path: &NamePath,
        imported: Option<jai_modules::Binding>,
    ) -> Option<(DeclarationId, FileScope<'a>, syntax::ConstantDeclaration)> {
        let mut binding =
            imported.or_else(|| self.declarations.graph.lookup(self.file, path).ok())?;
        let mut seen = std::collections::HashSet::new();
        loop {
            let jai_modules::Binding::Declaration(id) = binding else {
                return None;
            };
            if !seen.insert(id) {
                return None;
            }
            let declaration = self.declarations.graph.declaration(id)?;
            let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                return None;
            };
            if matches!(
                constant.initializer.kind,
                syntax::ExpressionKind::ShortLambda(_)
            ) {
                return Some((id, self.code_file(declaration.file()), constant.clone()));
            }
            if constant.ty.is_some() {
                return None;
            }
            let alias = match &constant.initializer.kind {
                syntax::ExpressionKind::Name(name) => NamePath {
                    root: *name,
                    members: vec![],
                },
                syntax::ExpressionKind::QualifiedName(path) => path.clone(),
                _ => return None,
            };
            binding = self
                .declarations
                .graph
                .lookup(declaration.file(), &alias)
                .ok()?;
        }
    }
}
