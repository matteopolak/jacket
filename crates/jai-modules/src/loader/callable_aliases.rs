use super::*;
use jai_syntax::{ExpressionKind, NamePath};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Destination {
    File(FileInstanceId),
    Module(ModuleId),
    Exports(ModuleId),
}
impl Destination {
    fn module(self, graph: &ModuleGraph) -> ModuleId {
        match self {
            Self::File(file) => graph.files[file.index()].module,
            Self::Module(module) | Self::Exports(module) => module,
        }
    }
}
#[derive(Clone)]
pub(super) struct DeferredAlias {
    destination: Destination,
    name: Symbol,
    binding: Binding,
    location: SourceSpan,
}
#[derive(Clone, Copy)]
enum AliasPhase {
    Discovery,
    Complete,
}
impl Builder<'_> {
    pub(super) fn merge_callable_binding(
        &mut self,
        destination: Destination,
        previous: Option<Binding>,
        binding: Binding,
        location: SourceSpan,
        name: Symbol,
        idempotent: bool,
    ) -> Result<(), GraphError> {
        match self.merge_binding(previous, binding, location, name, idempotent) {
            Ok(merged) => self.store_callable_binding(destination, name, merged),
            Err(_)
                if previous.is_some_and(|previous| {
                    self.callable_candidates(previous)
                        && self.callable_candidates(binding)
                        && (self.has_callable_alias(previous) || self.has_callable_alias(binding))
                }) =>
            {
                self.callable_aliases.push(DeferredAlias {
                    destination,
                    name,
                    binding,
                    location,
                });
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }
    fn store_callable_binding(&mut self, destination: Destination, name: Symbol, binding: Binding) {
        match destination {
            Destination::File(file) => {
                self.graph.files[file.index()].private.insert(name, binding);
            }
            Destination::Module(module) => {
                self.graph.modules[module.index()]
                    .bindings
                    .insert(name, binding);
            }
            Destination::Exports(module) => {
                self.graph.modules[module.index()]
                    .exports
                    .insert(name, binding);
            }
        }
    }
    fn callable_binding(&self, destination: Destination, name: Symbol) -> Option<Binding> {
        match destination {
            Destination::File(file) => self.graph.files[file.index()].private.get(&name),
            Destination::Module(module) => self.graph.modules[module.index()].bindings.get(&name),
            Destination::Exports(module) => self.graph.modules[module.index()].exports.get(&name),
        }
        .copied()
    }
    fn members(&self, binding: Binding) -> Option<Vec<DeclarationId>> {
        match binding {
            Binding::Declaration(id) => Some(vec![id]),
            Binding::OverloadSet(id) => Some(self.graph.overload_set(id)?.declarations().to_vec()),
            _ => None,
        }
    }
    fn alias_path(&self, id: DeclarationId) -> Option<NamePath> {
        let FileDeclarationKind::Constant(constant) = &self.graph.declaration(id)?.syntax().kind
        else {
            return None;
        };
        match &constant.initializer.kind {
            ExpressionKind::Name(root) => Some(NamePath {
                root: *root,
                members: vec![],
            }),
            ExpressionKind::QualifiedName(path) => Some(path.clone()),
            _ => None,
        }
    }
    fn callable_candidates(&self, binding: Binding) -> bool {
        self.members(binding).is_some_and(|members| {
            members.into_iter().all(|id| {
                self.graph.declaration(id).is_some_and(|declaration| {
                    matches!(
                        declaration.syntax().kind,
                        FileDeclarationKind::Procedure(_)
                            | FileDeclarationKind::ProcedurePrototype(_)
                    ) || self.alias_path(id).is_some()
                })
            })
        })
    }
    fn has_callable_alias(&self, binding: Binding) -> bool {
        self.members(binding)
            .is_some_and(|members| members.into_iter().any(|id| self.alias_path(id).is_some()))
    }
    fn alias_members(
        &self,
        binding: Binding,
        destination: Destination,
        name: Symbol,
    ) -> Vec<DeclarationId> {
        let mut members = self.members(binding).unwrap_or_default();
        for alias in &self.callable_aliases {
            if alias.destination == destination
                && alias.name == name
                && self.callable_binding(destination, name) == Some(binding)
            {
                members.extend(self.members(alias.binding).unwrap_or_default());
            }
        }
        members.sort_unstable_by_key(|id| id.index());
        members.dedup();
        members
    }
    fn alias_destination(&self, file: FileInstanceId, path: &NamePath) -> Destination {
        if !path.members.is_empty() {
            let mut namespace = path.clone();
            namespace.members.pop();
            let Binding::Module(module) = self
                .graph
                .lookup(file, &namespace)
                .expect("checked namespace")
            else {
                unreachable!("successful member lookup requires a module")
            };
            return Destination::Exports(module);
        }
        let file_scope = &self.graph.files[file.index()];
        if file_scope.private.contains_key(&path.root) {
            return Destination::File(file);
        }
        if self.graph.modules[file_scope.module.index()]
            .bindings
            .contains_key(&path.root)
        {
            return Destination::Module(file_scope.module);
        }
        if let Some(module) = self.graph.prelude
            && self.graph.modules[module.index()]
                .exports
                .contains_key(&path.root)
        {
            return Destination::Exports(module);
        }
        Destination::Exports(self.graph.runtime_support.expect("checked root lookup"))
    }
    fn pending_callable_alias(&self, location: SourceSpan) -> GraphError {
        let diagnostic = self.graph.diagnostic(
            location,
            "procedure overload aliases await source dependency discovery",
        );
        let rendered = diagnostic.render(&self.graph.sources);
        GraphError::Pending {
            diagnostic,
            rendered,
        }
    }
    fn resolve_callable_binding(
        &self,
        binding: Binding,
        destination: Destination,
        name: Symbol,
        phase: AliasPhase,
    ) -> Result<Vec<DeclarationId>, GraphError> {
        enum Work {
            Enter(DeclarationId),
            Leave(DeclarationId),
        }
        let mut pending: Vec<_> = self
            .alias_members(binding, destination, name)
            .into_iter()
            .rev()
            .map(Work::Enter)
            .collect();
        let mut active = HashSet::new();
        let mut complete = HashSet::new();
        let mut targets = Vec::new();
        while let Some(work) = pending.pop() {
            let id = match work {
                Work::Leave(id) => {
                    active.remove(&id);
                    complete.insert(id);
                    continue;
                }
                Work::Enter(id) => id,
            };
            if complete.contains(&id) {
                continue;
            }
            let declaration = self
                .graph
                .declaration(id)
                .expect("graph binding declaration");
            if matches!(
                declaration.syntax().kind,
                FileDeclarationKind::Procedure(_) | FileDeclarationKind::ProcedurePrototype(_)
            ) {
                targets.push(id);
                complete.insert(id);
                continue;
            }
            let Some(path) = self.alias_path(id) else {
                return Err(self.located(
                    declaration.syntax().location,
                    "overload alias does not denote a procedure",
                ));
            };
            if !active.insert(id) || (path.members.is_empty() && path.root == declaration.name()) {
                return Err(self.located(
                    declaration.syntax().location,
                    "cyclic procedure overload alias",
                ));
            }
            let target = self
                .graph
                .lookup(declaration.file(), &path)
                .map_err(|error| {
                    if matches!(phase, AliasPhase::Discovery)
                        && matches!(
                            error,
                            LookupError::UnknownName(_) | LookupError::UnknownMember { .. }
                        )
                    {
                        return self.pending_callable_alias(declaration.syntax().location);
                    }
                    let reason = match error {
                        LookupError::InvalidFile => "definition file is unavailable",
                        LookupError::UnknownName(_) => "target name is unknown",
                        LookupError::NotNamespace(_) => "target is not a namespace",
                        LookupError::UnknownMember {
                            ..
                        } => "target member is unknown",
                        LookupError::PrivateMember {
                            ..
                        } => "target member is private",
                        LookupError::UnfilledPlaceholder(_) => {
                            return self.pending_callable_alias(declaration.syntax().location);
                        }
                    };
                    self.located(
                        declaration.syntax().location,
                        format!("cannot resolve procedure overload alias: {reason}"),
                    )
                })?;
            let members = self.alias_members(
                target,
                self.alias_destination(declaration.file(), &path),
                path.members.last().copied().unwrap_or(path.root),
            );
            if members.is_empty() {
                return Err(self.located(
                    declaration.syntax().location,
                    "overload alias does not denote a procedure",
                ));
            }
            pending.push(Work::Leave(id));
            pending.extend(members.into_iter().rev().map(Work::Enter));
        }
        Ok(targets)
    }
    fn publish_callable_aliases(
        &mut self,
        module: ModuleId,
        phase: AliasPhase,
    ) -> Result<Option<GraphError>, GraphError> {
        let mut resolved = Vec::new();
        let mut first_pending = None;
        for (index, alias) in self.callable_aliases.iter().enumerate() {
            if alias.destination.module(&self.graph) != module {
                continue;
            }
            let previous = self
                .callable_binding(alias.destination, alias.name)
                .ok_or_else(|| {
                    self.located(alias.location, "procedure overload binding is unavailable")
                })?;
            let result =
                self.resolve_callable_binding(previous, alias.destination, alias.name, phase);
            let mut members = match result {
                Ok(members) => members,
                Err(
                    error @ GraphError::Pending {
                        ..
                    },
                ) => {
                    first_pending.get_or_insert(error);
                    continue;
                }
                Err(error) => return Err(error),
            };
            members.extend(self.resolve_callable_binding(
                alias.binding,
                alias.destination,
                alias.name,
                phase,
            )?);
            members.sort_unstable_by_key(|id| id.index());
            members.dedup();
            resolved.push((index, alias.destination, alias.name, members));
        }
        let mut published = HashSet::new();
        for (index, destination, name, members) in resolved {
            let binding = if let Some(existing) = self
                .graph
                .overload_sets
                .iter()
                .find(|set| set.declarations.as_ref() == members.as_slice())
            {
                Binding::OverloadSet(existing.id)
            } else {
                let id = OverloadSetId(self.graph.overload_sets.len());
                self.graph.overload_sets.push(OverloadSet {
                    id,
                    declarations: members.into_boxed_slice(),
                });
                Binding::OverloadSet(id)
            };
            self.store_callable_binding(destination, name, binding);
            published.insert(index);
        }
        let mut index = 0;
        self.callable_aliases.retain(|_| {
            let keep = !published.contains(&index);
            index += 1;
            keep
        });
        Ok(first_pending)
    }
    pub(crate) fn prepare_callable_aliases(
        &mut self,
        module: ModuleId,
    ) -> Result<Option<GraphError>, GraphError> {
        self.publish_callable_aliases(module, AliasPhase::Discovery)
    }
    pub(super) fn finish_callable_aliases(&mut self, module: ModuleId) -> Result<(), GraphError> {
        self.publish_callable_aliases(module, AliasPhase::Complete)?;
        Ok(())
    }
}
