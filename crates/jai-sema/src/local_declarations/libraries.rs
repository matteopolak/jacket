//! Lexical foreign libraries share the local declaration arena, never fake graph IDs.
use super::*;
use jai_ir::{ForeignLibrary, ForeignLibraryId};

impl Resolver<'_> {
    pub(super) fn define_local_library(
        &mut self,
        declaration: LocalDeclarationId,
        syntax: &syntax::LibraryDeclaration,
    ) -> Result<Binding, Diagnostic> {
        let LexicalScopeOwner::Procedure(owner) = declaration.scope.owner else {
            return Err(Diagnostic::new(
                syntax.span,
                "record members cannot declare a foreign library",
            ));
        };
        let id = if let Some(id) = self.meta.local_declarations.library_ids.get(&declaration) {
            *id
        } else {
            let next = self
                .meta
                .local_declarations
                .next_library
                .entry(owner)
                .or_default();
            let id = ForeignLibraryId::local(owner, *next);
            *next = next.checked_add(1).ok_or_else(|| {
                Diagnostic::new(syntax.span, "local library identity space exhausted")
            })?;
            self.meta
                .local_declarations
                .library_ids
                .insert(declaration, id);
            id
        };
        let source_path = self
            .graph_scope
            .map(|scope| scope.source_path())
            .unwrap_or_else(|| std::path::Path::new("."));
        let library = crate::modules::foreign_libraries::metadata(id, syntax, source_path)?;
        self.meta
            .local_declarations
            .foreign_libraries
            .insert(id, library);
        Ok(Binding::Library(id))
    }

    pub(crate) fn foreign_library_path(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<ForeignLibrary, Diagnostic> {
        if let Some(binding) = self.lexical_graph_binding(path, span)? {
            return self
                .graph_scope
                .ok_or_else(|| Diagnostic::new(span, "imported libraries require a source graph"))?
                .imported_foreign_library(binding, span);
        }
        if let Some(binding) = self.resolve_local_name(path.root, span)? {
            return match binding {
                Binding::Library(id) if path.members.is_empty() => self
                    .meta
                    .local_declarations
                    .foreign_libraries
                    .get(&id)
                    .cloned()
                    .ok_or_else(|| Diagnostic::new(span, "local library metadata is not ready")),
                _ => Err(Diagnostic::new(
                    span,
                    "local declaration does not denote a foreign library",
                )),
            };
        }
        if self
            .local_scopes
            .frames
            .iter()
            .any(|frame| frame.runtime.contains_key(&path.root))
        {
            return Err(Diagnostic::new(
                span,
                "local runtime storage does not denote a foreign library",
            ));
        }
        self.graph_scope
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "foreign library name requires a source declaration scope",
                )
            })?
            .foreign_library(path, span)
    }
}
