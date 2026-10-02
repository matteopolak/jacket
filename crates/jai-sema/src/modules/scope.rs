//! Adapt graph lookup into semantic declaration metadata.
use super::*;
use jai_modules::{Binding as GraphBinding, LookupError};
#[derive(Clone, Copy)]
pub(crate) struct FileScope<'a> {
    pub(super) declarations: &'a ScopedDeclarations<'a>,
    pub(super) file: FileInstanceId,
}
impl<'a> FileScope<'a> {
    fn declaration(&self, path: &NamePath, span: Span) -> Result<DeclarationId, Diagnostic> {
        declaration_id(self.declarations.graph, self.file, path, span)
    }
    pub(crate) fn value(&self, path: &NamePath, span: Span) -> Result<Binding, Diagnostic> {
        let id = self.declaration(path, span)?;
        self.declarations
            .values
            .get(&id)
            .copied()
            .ok_or_else(|| Diagnostic::new(span, "procedure cannot supply a scalar value"))
    }
    pub(crate) fn signature(
        &self,
        path: &NamePath,
        span: Span,
    ) -> Result<&'a Signature, Diagnostic> {
        let id = self.declaration(path, span)?;
        self.declarations
            .signatures
            .get(&id)
            .ok_or_else(|| Diagnostic::new(span, "scalar value is not a procedure"))
    }
}
pub(super) fn path(name: Symbol) -> NamePath {
    NamePath {
        root: name,
        members: Vec::new(),
    }
}
pub(super) fn declaration_id(
    graph: &ModuleGraph,
    file: FileInstanceId,
    path: &NamePath,
    span: Span,
) -> Result<DeclarationId, Diagnostic> {
    match graph.lookup(file, path) {
        Ok(GraphBinding::Declaration(id)) => Ok(id),
        Ok(GraphBinding::Module(_)) => {
            Err(Diagnostic::new(span, "namespace cannot supply a value"))
        }
        Err(error) => Err(Diagnostic::new(
            span,
            match error {
                LookupError::InvalidFile => "invalid defining file".into(),
                LookupError::UnknownName(name) => {
                    format!("unknown name '{}'", graph.symbols().name(name))
                }
                LookupError::NotNamespace(name) => {
                    format!("'{}' is not a namespace", graph.symbols().name(name))
                }
                LookupError::UnknownMember { name, .. } => {
                    format!("unknown module member '{}'", graph.symbols().name(name))
                }
                LookupError::PrivateMember { name, .. } => {
                    format!("module member '{}' is private", graph.symbols().name(name))
                }
            },
        )),
    }
}
pub(super) fn located(
    graph: &ModuleGraph,
    file: FileInstanceId,
    diagnostic: Diagnostic,
) -> LocatedDiagnostic {
    LocatedDiagnostic::new(
        graph
            .file(file)
            .expect("graph declaration has a defining file")
            .source(),
        diagnostic,
    )
}
