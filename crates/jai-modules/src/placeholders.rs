//! Source reservations are separate from real declarations and runtime bindings.
use super::FileInstanceId;
use jai_source::{ModuleId, SourceSpan, Symbol};
use jai_syntax::Visibility;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_PLACEHOLDER_SESSION: AtomicU64 = AtomicU64::new(1);

/// An authored marker identity, scoped to one graph discovery session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlaceholderId {
    session: u64,
    index: usize,
}
impl PlaceholderId {
    pub fn index(self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum PlaceholderScope {
    File(FileInstanceId),
    Module(ModuleId),
}

/// Original source metadata; a marker has no type, value, or procedure identity.
#[derive(Clone, Debug)]
pub struct Placeholder {
    id: PlaceholderId,
    file: FileInstanceId,
    scope: PlaceholderScope,
    name: Symbol,
    visibility: Visibility,
    location: SourceSpan,
}
impl Placeholder {
    pub fn id(&self) -> PlaceholderId {
        self.id
    }
    pub fn file(&self) -> FileInstanceId {
        self.file
    }
    pub fn name(&self) -> Symbol {
        self.name
    }
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }
    pub(super) fn scope(&self) -> PlaceholderScope {
        self.scope
    }
}

/// Cloning preserves session identities for atomic insertion rollback.
#[derive(Clone, Debug)]
pub(super) struct Placeholders {
    session: u64,
    entries: Vec<Placeholder>,
    namespaces: HashMap<(PlaceholderScope, Symbol), PlaceholderId>,
    imports: HashMap<(PlaceholderScope, Symbol), PlaceholderImport>,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct PlaceholderImport {
    pub(super) placeholder: PlaceholderId,
    pub(super) visibility: Visibility,
    pub(super) location: SourceSpan,
}
impl Default for Placeholders {
    fn default() -> Self {
        Self {
            session: NEXT_PLACEHOLDER_SESSION.fetch_add(1, Ordering::Relaxed),
            entries: vec![],
            namespaces: HashMap::new(),
            imports: HashMap::new(),
        }
    }
}
impl Placeholders {
    pub(super) fn get(&self, id: PlaceholderId) -> Option<&Placeholder> {
        (id.session == self.session)
            .then(|| self.entries.get(id.index))
            .flatten()
    }
    pub(super) fn entries(&self) -> &[Placeholder] {
        &self.entries
    }
    pub(super) fn find(&self, scope: PlaceholderScope, name: Symbol) -> Option<&Placeholder> {
        self.get(*self.namespaces.get(&(scope, name))?)
    }
    pub(super) fn imported(
        &self,
        scope: PlaceholderScope,
        name: Symbol,
    ) -> Option<&PlaceholderImport> {
        self.imports.get(&(scope, name))
    }
    pub(super) fn imported_names(
        &self,
        scope: PlaceholderScope,
    ) -> impl Iterator<Item = (Symbol, &PlaceholderImport)> {
        self.imports
            .iter()
            .filter_map(move |(&(candidate, name), link)| {
                (candidate == scope).then_some((name, link))
            })
    }
    pub(super) fn link(
        &mut self,
        scope: PlaceholderScope,
        name: Symbol,
        placeholder: PlaceholderId,
        visibility: Visibility,
        location: SourceSpan,
    ) -> Result<(), PlaceholderId> {
        if let Some(previous) = self.imported(scope, name) {
            if previous.placeholder == placeholder {
                return Ok(());
            }
            return Err(previous.placeholder);
        }
        self.imports.insert(
            (scope, name),
            PlaceholderImport {
                placeholder,
                visibility,
                location,
            },
        );
        Ok(())
    }
    /// Returns the previous marker on a conflicting reservation. Identical
    /// source re-registration is idempotent; distinct marker statements are not.
    pub(super) fn reserve(
        &mut self,
        scope: PlaceholderScope,
        file: FileInstanceId,
        name: Symbol,
        visibility: Visibility,
        location: SourceSpan,
    ) -> Result<PlaceholderId, PlaceholderId> {
        if let Some(previous) = self.find(scope, name) {
            return if previous.file == file
                && previous.location == location
                && previous.visibility == visibility
            {
                Ok(previous.id)
            } else {
                Err(previous.id)
            };
        }
        let id = PlaceholderId {
            session: self.session,
            index: self.entries.len(),
        };
        self.entries.push(Placeholder {
            id,
            file,
            scope,
            name,
            visibility,
            location,
        });
        self.namespaces.insert((scope, name), id);
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{Identities, SourceMap, Span, Symbols};

    #[test]
    fn reservations_use_namespace_and_graph_session_identity() {
        let mut symbols = Symbols::default();
        let name = symbols.intern("TRUTH");
        let mut sources = SourceMap::default();
        let source = sources.insert("placeholder.jai".into(), "#placeholder TRUTH;".into());
        let location = SourceSpan {
            source,
            span: Span::new(0, 19),
        };
        let mut identities = Identities::default();
        let module = identities.module();
        let other_module = identities.module();
        let file = FileInstanceId(0);
        let mut reservations = Placeholders::default();
        let original = reservations
            .reserve(
                PlaceholderScope::Module(module),
                file,
                name,
                Visibility::Export,
                location,
            )
            .unwrap();
        let private = reservations
            .reserve(
                PlaceholderScope::File(file),
                file,
                name,
                Visibility::File,
                location,
            )
            .unwrap();
        let unrelated = reservations
            .reserve(
                PlaceholderScope::Module(other_module),
                file,
                name,
                Visibility::Export,
                location,
            )
            .unwrap();
        assert_ne!(original, private);
        assert_ne!(original, unrelated);
        assert_eq!(reservations.get(original).unwrap().location(), location);
        assert!(Placeholders::default().get(original).is_none());
        let checkpoint = reservations.clone();
        assert_eq!(checkpoint.get(original).unwrap().id(), original);
    }

    #[test]
    fn only_the_same_source_marker_is_idempotent() {
        let mut symbols = Symbols::default();
        let name = symbols.intern("TRUTH");
        let mut sources = SourceMap::default();
        let source = sources.insert(
            "placeholder.jai".into(),
            "#placeholder TRUTH; #placeholder TRUTH;".into(),
        );
        let location = SourceSpan {
            source,
            span: Span::new(0, 19),
        };
        let file = FileInstanceId(0);
        let scope = PlaceholderScope::File(file);
        let mut reservations = Placeholders::default();
        let id = reservations
            .reserve(scope, file, name, Visibility::File, location)
            .unwrap();
        assert_eq!(
            reservations.reserve(scope, file, name, Visibility::File, location),
            Ok(id)
        );
        assert_eq!(
            reservations.reserve(
                scope,
                file,
                name,
                Visibility::File,
                SourceSpan {
                    source,
                    span: Span::new(20, 39)
                }
            ),
            Err(id)
        );
        assert_eq!(reservations.entries().len(), 1);
        assert_eq!(reservations.get(id).unwrap().scope(), scope);
    }
}
