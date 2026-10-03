//! A source declaration witness originates in an actual graph or its frozen provenance table.
use super::*;
use jai_ir::{ForeignLibrary, Library, SourceProcedureIdentity};

#[derive(Clone, Debug)]
pub struct NativeSourceLibraryBinding {
    library: ForeignLibrary,
    source: SourceProcedureIdentity,
    environment: std::sync::Arc<[u8]>,
}
impl NativeSourceLibraryBinding {
    /// No ID/path/body constructor: only the graph's actual immutable source allocation is captured.
    pub fn from_graph(
        graph: &ModuleGraph,
        declaration: DeclarationId,
    ) -> Result<Self, LocatedDiagnostic> {
        let declaration = graph.declaration(declaration).ok_or_else(|| {
            graph.diagnostic(
                graph
                    .locate(
                        graph.module(graph.root()).expect("root module").entry(),
                        Span::default(),
                    )
                    .expect("root source"),
                "native binding requires an actual current source declaration",
            )
        })?;
        let library = foreign_libraries::declaration(graph, declaration)?;
        let location = declaration.location();
        let source = graph
            .sources()
            .get(location.source)
            .expect("actual declaration source");
        let source = SourceProcedureIdentity::new(source, location)
            .map_err(|error| graph.diagnostic(location, error.to_string()))?;
        let environment = graph
            .module_environment_origin(declaration.file())
            .map_err(|error| graph.diagnostic(location, error.to_string()))?;
        Ok(Self {
            library,
            source,
            environment: environment.into(),
        })
    }
    /// Recover the same producer witness after the semantic library has frozen.
    pub fn from_frozen_library(library: &Library, row: &ForeignLibrary) -> Option<Self> {
        if !library
            .foreign_libraries()
            .iter()
            .any(|actual| std::ptr::eq(actual, row))
        {
            return None;
        }
        Some(Self {
            library: row.clone(),
            source: library.foreign_library_sources().get(row.id)?.clone(),
            environment: library
                .foreign_library_sources()
                .environment(row.id)?
                .into(),
        })
    }
    /// Rebase only another graph-owned witness with the same actual source and environment.
    pub fn same_source_declaration(&self, other: &Self) -> bool {
        self.library.kind == other.library.kind
            && self.library.options == other.library.options
            && self.source.matches_identity(&other.source)
            && self.environment == other.environment
    }
    /// Select the reached row by its actual retained source owner. Full metadata is
    /// validated by `can_rebase_to_frozen`; changed metadata must fail after selection.
    pub fn owns_frozen_declaration(&self, library: &Library, row: &ForeignLibrary) -> bool {
        library
            .foreign_libraries()
            .iter()
            .any(|actual| std::ptr::eq(actual, row))
            && library
                .foreign_library_sources()
                .get(row.id)
                .is_some_and(|identity| identity.matches_identity(&self.source))
    }
    pub fn metadata(&self) -> &ForeignLibrary {
        &self.library
    }
    pub fn source_identity(&self) -> &SourceProcedureIdentity {
        &self.source
    }
    /// Explicit rebasing across a genuine retained-source graph rebuild. Dense
    /// IDs may change; source allocation/span, metadata and environment cannot.
    pub fn can_rebase_to_frozen(&self, library: &Library, row: &ForeignLibrary) -> bool {
        library
            .foreign_libraries()
            .iter()
            .any(|actual| std::ptr::eq(actual, row))
            && row.kind == self.library.kind
            && row.options == self.library.options
            && library
                .foreign_library_sources()
                .get(row.id)
                .is_some_and(|identity| identity.matches_identity(&self.source))
            && library.foreign_library_sources().environment(row.id)
                == Some(self.environment.as_ref())
    }
    pub fn matches_frozen(&self, library: &Library, row: &ForeignLibrary) -> bool {
        library
            .foreign_libraries()
            .iter()
            .any(|actual| std::ptr::eq(actual, row))
            && row == &self.library
            && library.foreign_library_sources().environment(row.id)
                == Some(self.environment.as_ref())
            && library
                .foreign_library_sources()
                .get(row.id)
                .is_some_and(|identity| identity.matches_identity(&self.source))
    }
}
