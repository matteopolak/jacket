//! Compilation identities and immutable source records.
use crate::{Diagnostic, Span};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(usize);
        impl $name {
            pub fn index(self) -> usize {
                self.0
            }
        }
    };
}
identity!(SourceId);
identity!(UnitId);
identity!(ModuleId);
identity!(ScopeId);
identity!(DeclarationId);

/// Allocate identities within one compilation session; IDs never identify spellings.
#[derive(Clone, Debug, Default)]
pub struct Identities {
    units: usize,
    modules: usize,
    scopes: usize,
    declarations: usize,
}
impl Identities {
    pub fn unit(&mut self) -> UnitId {
        let id = UnitId(self.units);
        self.units += 1;
        id
    }
    pub fn module(&mut self) -> ModuleId {
        let id = ModuleId(self.modules);
        self.modules += 1;
        id
    }
    pub fn scope(&mut self) -> ScopeId {
        let id = ScopeId(self.scopes);
        self.scopes += 1;
        id
    }
    pub fn declaration(&mut self) -> DeclarationId {
        let id = DeclarationId(self.declarations);
        self.declarations += 1;
        id
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    pub source: SourceId,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedDiagnostic {
    pub location: SourceSpan,
    pub message: String,
}
impl LocatedDiagnostic {
    pub fn new(source: SourceId, diagnostic: Diagnostic) -> Self {
        Self {
            location: SourceSpan {
                source: diagnostic.source.unwrap_or(source),
                span: diagnostic.span,
            },
            message: diagnostic.message,
        }
    }
    pub fn render(&self, sources: &SourceMap) -> String {
        let diagnostic = Diagnostic::at_source(self.location, &self.message);
        match sources.get(self.location.source) {
            Some(source) => diagnostic.render(&source.path.to_string_lossy(), &source.text),
            None => format!(
                "source {}: error: {}",
                self.location.source.index(),
                self.message
            ),
        }
    }
}
impl std::fmt::Display for LocatedDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for LocatedDiagnostic {}
#[derive(Clone, Debug)]
pub struct SourceRecord {
    id: SourceId,
    path: PathBuf,
    text: Arc<str>,
}
impl SourceRecord {
    pub fn id(&self) -> SourceId {
        self.id
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Retain this source's immutable text and allocation provenance across phases.
    pub fn shared_text(&self) -> Arc<str> {
        Arc::clone(&self.text)
    }
}
#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    records: Vec<SourceRecord>,
}
impl SourceMap {
    pub fn insert(&mut self, path: PathBuf, text: String) -> SourceId {
        let id = SourceId(self.records.len());
        self.records.push(SourceRecord {
            id,
            path,
            text: text.into(),
        });
        id
    }
    pub fn get(&self, id: SourceId) -> Option<&SourceRecord> {
        self.records.get(id.0)
    }
    pub fn records(&self) -> &[SourceRecord] {
        &self.records
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_owns_identity_and_source_provenance() {
        let mut identities = Identities::default();
        assert_ne!(identities.module(), identities.module());
        assert_eq!(identities.scope().index(), 0);
        let mut sources = SourceMap::default();
        let id = sources.insert("a.jai".into(), "é\nbad".into());
        let diagnostic = LocatedDiagnostic::new(id, Diagnostic::new(Span::new(3, 6), "bad name"));
        assert_eq!(diagnostic.render(&sources), "a.jai:2:1: error: bad name");
        assert_eq!(sources.get(id).unwrap().id(), id);
    }
    #[test]
    fn explicit_diagnostic_origin_survives_lexical_source_fallbacks() {
        let mut sources = SourceMap::default();
        let caller = sources.insert("caller.jai".into(), "caller".into());
        let quote = sources.insert("quote.jai".into(), "é\r\nquoted".into());
        let location = SourceSpan {
            source: quote,
            span: Span::new(4, 10),
        };
        let diagnostic = Diagnostic::at_source(location, "bad quote").with_fallback_source(caller);
        let located = LocatedDiagnostic::new(caller, diagnostic);
        assert_eq!(located.location, location);
        assert_eq!(located.render(&sources), "quote.jai:2:1: error: bad quote");
        let caller_diagnostic = Diagnostic::new(Span::new(0, 6), "bad caller");
        assert_eq!(caller_diagnostic.source, None);
        assert_eq!(
            LocatedDiagnostic::new(caller, caller_diagnostic)
                .location
                .source,
            caller
        );
    }
}
