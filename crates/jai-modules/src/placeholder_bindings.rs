//! Resolve placeholder reservations through their actual namespace tables.
use super::*;
use crate::placeholders::{PlaceholderId, PlaceholderScope};

impl ModuleGraph {
    pub fn placeholder(&self, id: PlaceholderId) -> Option<&placeholders::Placeholder> {
        self.placeholders.get(id)
    }
    pub fn placeholders(&self) -> &[placeholders::Placeholder] {
        self.placeholders.entries()
    }
    pub fn placeholder_binding(&self, id: PlaceholderId) -> Option<Binding> {
        let marker = self.placeholder(id)?;
        self.placeholder_scope_binding(marker.scope(), marker.name())
    }
    pub fn lookup_placeholder_export(&self, id: PlaceholderId) -> Result<Binding, LookupError> {
        let marker = self.placeholder(id).ok_or(LookupError::InvalidFile)?;
        let module = match marker.scope() {
            PlaceholderScope::Module(module) => module,
            PlaceholderScope::File(file) => {
                return Err(LookupError::PrivateMember {
                    module: self.files[file.index()].module,
                    name: marker.name(),
                });
            }
        };
        let source = &self.modules[module.index()];
        if let Some(&binding) = source.exports.get(&marker.name()) {
            return Ok(binding);
        }
        if source.bindings.contains_key(&marker.name()) || marker.visibility() != Visibility::Export
        {
            return Err(LookupError::PrivateMember {
                module,
                name: marker.name(),
            });
        }
        Err(LookupError::UnfilledPlaceholder(id))
    }
    /// Ready ordinary exports remain available while individual reservations wait.
    pub fn module_placeholder_exports(&self, module: ModuleId) -> Vec<(Symbol, PlaceholderId)> {
        let scope = PlaceholderScope::Module(module);
        let mut exports = self
            .placeholders
            .entries()
            .iter()
            .filter_map(|marker| {
                (marker.scope() == scope && marker.visibility() == Visibility::Export)
                    .then_some((marker.name(), marker.id()))
            })
            .chain(
                self.placeholders
                    .imported_names(scope)
                    .filter_map(|(name, link)| {
                        (link.visibility == Visibility::Export).then_some((name, link.placeholder))
                    }),
            )
            .filter(|(_, id)| {
                matches!(
                    self.lookup_placeholder_export(*id),
                    Ok(_) | Err(LookupError::UnfilledPlaceholder(_))
                )
            })
            .collect::<Vec<_>>();
        exports.sort_by_key(|(name, _)| self.symbols.name(*name));
        exports.dedup();
        exports
    }
    /// Canonical pending imports are source links, not fabricated value bindings.
    pub fn imported_placeholder(
        &self,
        file: FileInstanceId,
        name: Symbol,
    ) -> Option<PlaceholderId> {
        let defining = self.file(file)?;
        self.placeholders
            .imported(
                PlaceholderScope::File(self.canonical_binding_file(file)),
                name,
            )
            .or_else(|| {
                self.placeholders
                    .imported(PlaceholderScope::Module(defining.module), name)
            })
            .map(|link| link.placeholder)
    }
    pub(super) fn placeholder_scope(
        &self,
        file: FileInstanceId,
        visibility: Visibility,
    ) -> PlaceholderScope {
        if visibility == Visibility::File {
            PlaceholderScope::File(self.canonical_binding_file(file))
        } else {
            PlaceholderScope::Module(self.files[file.index()].module)
        }
    }
    fn placeholder_scope_binding(&self, scope: PlaceholderScope, name: Symbol) -> Option<Binding> {
        match scope {
            PlaceholderScope::File(file) => self.files[file.index()].private.get(&name),
            PlaceholderScope::Module(module) => self.modules[module.index()].bindings.get(&name),
        }
        .copied()
    }
    pub(super) fn placeholder_namespace_lookup(
        &self,
        scope: PlaceholderScope,
        name: Symbol,
    ) -> Option<Result<Binding, LookupError>> {
        if let Some(marker) = self.placeholders.find(scope, name) {
            return Some(
                self.placeholder_scope_binding(scope, name)
                    .ok_or(LookupError::UnfilledPlaceholder(marker.id())),
            );
        }
        self.placeholders
            .imported(scope, name)
            .map(|link| self.lookup_placeholder_export(link.placeholder))
    }
    pub(super) fn placeholder_module_lookup(
        &self,
        module: ModuleId,
        name: Symbol,
    ) -> Option<Result<Binding, LookupError>> {
        let scope = PlaceholderScope::Module(module);
        if let Some(marker) = self.placeholders.find(scope, name) {
            return Some(self.lookup_placeholder_export(marker.id()));
        }
        self.placeholders.imported(scope, name).map(|link| {
            if link.visibility == Visibility::Export {
                self.lookup_placeholder_export(link.placeholder)
            } else {
                Err(LookupError::PrivateMember { module, name })
            }
        })
    }
    fn placeholder_filler(&self, scope: PlaceholderScope, name: Symbol, binding: Binding) -> bool {
        let ids = match binding {
            Binding::Declaration(id) => vec![id],
            Binding::OverloadSet(id) => match self.overload_set(id) {
                Some(group) => group.declarations().to_vec(),
                None => return false,
            },
            _ => return false,
        };
        !ids.is_empty()
            && ids.into_iter().all(|id| {
                self.declaration(id).is_some_and(|declaration| {
                    declaration.name() == name
                        && self
                            .placeholder_scope(declaration.file(), declaration.syntax().visibility)
                            == scope
                })
            })
    }
}

impl Builder<'_> {
    pub(super) fn reserve_placeholder(
        &mut self,
        file: FileInstanceId,
        name: Symbol,
        visibility: Visibility,
        location: SourceSpan,
    ) -> Result<(), GraphError> {
        let scope = self.graph.placeholder_scope(file, visibility);
        // Replay checks the authored marker before a filler introduced by an
        // insertion with a different SourceId can look like a collision.
        if self
            .graph
            .placeholders
            .find(scope, name)
            .is_some_and(|marker| {
                marker.file() == file
                    && marker.location() == location
                    && marker.visibility() == visibility
            })
        {
            return Ok(());
        }
        if self.graph.placeholders.imported(scope, name).is_some() {
            return Err(self.located(
                location,
                format!(
                    "placeholder '{}' conflicts with an imported reservation",
                    self.graph.symbols.name(name)
                ),
            ));
        }
        if let Some(binding) = self.graph.placeholder_scope_binding(scope, name) {
            // Selected file branches are registered after the ordinary file
            // prepass. Authored later fillers may consequently already exist.
            let later_source = self.graph.placeholder_filler(scope, name, binding)
                && match binding {
                    Binding::Declaration(id) => {
                        self.graph.declaration(id).is_some_and(|declaration| {
                            declaration.location().source == location.source
                                && declaration.location().span.start >= location.span.end
                        })
                    }
                    Binding::OverloadSet(id) => self.graph.overload_set(id).is_some_and(|group| {
                        group.declarations().iter().all(|id| {
                            self.graph.declaration(*id).is_some_and(|declaration| {
                                declaration.location().source == location.source
                                    && declaration.location().span.start >= location.span.end
                            })
                        })
                    }),
                    _ => false,
                };
            if !later_source {
                return Err(self.located(
                    location,
                    format!(
                        "placeholder '{}' conflicts with an existing declaration or import",
                        self.graph.symbols.name(name)
                    ),
                ));
            }
        }
        self.graph
            .placeholders
            .reserve(scope, file, name, visibility, location)
            .map_err(|id| {
                let marker = self.graph.placeholder(id).expect("same graph reservation");
                let original = self
                    .graph
                    .sources
                    .get(marker.location().source)
                    .expect("marker source");
                self.located(
                    location,
                    format!(
                        "duplicate placeholder '{}'; original marker at {} byte {}",
                        self.graph.symbols.name(name),
                        original.path().display(),
                        marker.location().span.start
                    ),
                )
            })?;
        if let Some(publication) = self
            .graph
            .insertion_publications
            .iter_mut()
            .find(|publication| publication.file == file)
        {
            publication.own_names.insert(name);
        }
        Ok(())
    }
    pub(super) fn validate_placeholder_binding(
        &self,
        file: FileInstanceId,
        visibility: Visibility,
        name: Symbol,
        binding: Binding,
        location: SourceSpan,
        idempotent: bool,
    ) -> Result<(), GraphError> {
        let scope = self.graph.placeholder_scope(file, visibility);
        if self.graph.placeholders.find(scope, name).is_some()
            && !self.graph.placeholder_filler(scope, name, binding)
        {
            return Err(self.located(
                location,
                format!(
                    "placeholder '{}' requires a real declaration in its defining namespace",
                    self.graph.symbols.name(name)
                ),
            ));
        }
        if let Some(link) = self.graph.placeholders.imported(scope, name)
            && !(idempotent
                && self.graph.lookup_placeholder_export(link.placeholder) == Ok(binding))
        {
            let original = self
                .graph
                .sources
                .get(link.location.source)
                .expect("import reservation source");
            return Err(self.located(
                location,
                format!(
                    "declaration or import '{}' conflicts with an imported placeholder; original import at {} byte {}",
                    self.graph.symbols.name(name), original.path().display(), link.location.span.start
                ),
            ));
        }
        Ok(())
    }
    pub(super) fn link_placeholder_import(
        &mut self,
        file: FileInstanceId,
        visibility: Visibility,
        name: Symbol,
        placeholder: PlaceholderId,
        location: SourceSpan,
    ) -> Result<(), GraphError> {
        if self.graph.placeholder(placeholder).is_none() {
            return Err(self.located(
                location,
                "imported placeholder belongs to another graph session",
            ));
        }
        let scope = self.graph.placeholder_scope(file, visibility);
        if self.graph.placeholders.find(scope, name).is_some() {
            return Err(self.located(
                location,
                format!(
                    "import '{}' conflicts with a placeholder in this namespace",
                    self.graph.symbols.name(name)
                ),
            ));
        }
        if let Some(binding) = self.graph.placeholder_scope_binding(scope, name)
            && self.graph.lookup_placeholder_export(placeholder) != Ok(binding)
        {
            return Err(self.located(
                location,
                format!(
                    "imported placeholder '{}' conflicts with an existing declaration or import",
                    self.graph.symbols.name(name)
                ),
            ));
        }
        self.graph
            .placeholders
            .link(scope, name, placeholder, visibility, location)
            .map_err(|_| {
                self.located(
                    location,
                    format!(
                        "conflicting imported placeholder '{}'",
                        self.graph.symbols.name(name)
                    ),
                )
            })
    }
}
