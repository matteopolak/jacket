//! Creating scopes and binding entities into them.
use super::expand::may_overload;
use super::model::{
    Entity, EntityId, EntityKind, EntityState, ImportEntry, Pending, PendingState, Resolved, Scope,
    ScopeId, ScopeKind, UsingEntry,
};
use crate::sema::{
    Compiler, Diagnostic, FileId, HashMap, ModuleId, Rc, Result, Span, Sym, Value, ast,
};
use crate::types::TypeId;

impl Compiler {
    pub fn new_scope(
        &mut self,
        kind: ScopeKind,
        parent: Option<ScopeId>,
        module: ModuleId,
        file: Option<FileId>,
    ) -> ScopeId {
        let proc_depth = parent.map_or(0, |p| self.ns.scopes[p.0 as usize].proc_depth)
            + u32::from(kind == ScopeKind::Proc);
        self.ns.scopes.push(Scope {
            kind,
            parent,
            inserted_into: None,
            module,
            file,
            names: HashMap::default(),
            imports: Vec::new(),
            pending: Vec::new(),
            pending_done: 0,
            usings: Vec::new(),
            proc_depth,
            proc: None,
            fallbacks: Vec::new(),
        });
        let id = ScopeId(self.ns.scopes.len() as u32 - 1);
        if matches!(kind, ScopeKind::Module | ScopeKind::File) {
            self.ns.unsettled.insert(id);
        }
        id
    }

    /// Queue a top-level item of `scope` for expansion.
    pub fn push_pending(&mut self, scope: ScopeId, item: Pending) {
        self.scope_mut(scope).pending.push(item);
        self.namespace_changed(scope);
        self.unsettle(scope);
    }

    /// Queue an unnamed (or `using`) import of `scope` for loading.
    pub fn push_import(&mut self, scope: ScopeId, entry: ImportEntry) {
        self.scope_mut(scope).imports.push(entry);
        self.namespace_changed(scope);
        self.unsettle(scope);
    }

    /// Add a `using` entry to `scope`.
    pub(in crate::sema) fn push_using(&mut self, scope: ScopeId, entry: UsingEntry) {
        self.scope_mut(scope).usings.push(entry);
        self.namespace_changed(scope);
    }

    /// Bind `name` in `scope` to `id` as well.
    pub(in crate::sema) fn bind_name(&mut self, scope: ScopeId, name: Sym, id: EntityId) {
        self.scope_mut(scope)
            .names
            .entry(name)
            .or_default()
            .push(id);
        self.namespace_changed(scope);
    }

    /// Something a name lookup through `scope` can see changed (a name, an import, a `using`,
    /// an expanded item). Bodies' own scopes are remade by every attempt to lower them, so
    /// only the others count: see `lower_epoch`.
    pub(in crate::sema) fn namespace_changed(&mut self, scope: ScopeId) {
        if !matches!(
            self.scope(scope).kind,
            ScopeKind::Proc | ScopeKind::Block | ScopeKind::Macro
        ) {
            self.epochs.namespace_generation += 1;
        }
    }

    fn unsettle(&mut self, scope: ScopeId) {
        if matches!(self.scope(scope).kind, ScopeKind::Module | ScopeKind::File) {
            self.ns.unsettled.insert(scope);
        }
    }

    /// Has `expand_all` nothing left to do in `scope`: every item expanded, every import
    /// loaded (or being loaded by a caller further up)?
    pub(in crate::sema) fn settled(&self, scope: ScopeId) -> bool {
        let s = self.scope(scope);
        s.pending[s.pending_done..]
            .iter()
            .all(|p| p.state == PendingState::Done)
            && s.imports.iter().all(|i| i.module.is_some() || i.loading)
    }

    pub fn scope(&self, id: ScopeId) -> &Scope {
        &self.ns.scopes[id.0 as usize]
    }

    pub fn scope_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.ns.scopes[id.0 as usize]
    }

    /// The declaration namespace, excluding synthetic Code lookup wrappers.
    pub(in crate::sema) fn binding_scope(&self, mut scope: ScopeId) -> ScopeId {
        while let Some(target) = self.scope(scope).inserted_into {
            scope = target;
        }
        scope
    }

    pub fn entity(&self, id: EntityId) -> &Entity {
        &self.ns.entities[id.0 as usize]
    }

    pub fn entity_mut(&mut self, id: EntityId) -> &mut Entity {
        &mut self.ns.entities[id.0 as usize]
    }

    pub fn add_entity(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
        kind: EntityKind,
        exported: bool,
    ) -> EntityId {
        let id = self.new_entity(scope, name, span, kind, exported);
        if self.ed.is_live() {
            self.ide_note_entity(id);
        }
        id
    }

    /// [`Self::add_entity`] without the editor's record of the declaration, for a caller that knows
    /// where the name is written (or that it is not).
    pub(in crate::sema) fn new_entity(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
        kind: EntityKind,
        exported: bool,
    ) -> EntityId {
        let placeholder = matches!(
            &kind,
            EntityKind::Decl { .. } | EntityKind::Import(_) | EntityKind::Const { .. }
        )
        .then(|| self.generated_placeholder(scope, name, span))
        .flatten();
        let id = EntityId(self.ns.entities.len() as u32);
        self.ns.entities.push(Entity {
            name,
            span,
            scope,
            home: scope,
            exported,
            kind,
            state: EntityState::Unresolved,
            file_private: false,
            untyped_const: false,
        });
        self.bind_name(scope, name, id);
        if let Some(placeholder) = placeholder {
            self.diag
                .dependencies
                .bindings
                .entry(placeholder)
                .or_default()
                .push(id);
        }
        id
    }

    /// A scoped source definition fulfills the visible explicit placeholder,
    /// including when a file Code scope sees its module's declaration. Keep
    /// the entity identity so other files/importers observe that same binding.
    fn generated_placeholder(&self, scope: ScopeId, name: Sym, span: Span) -> Option<EntityId> {
        if let Some(placeholder) = self.scope(scope).names.get(&name).and_then(|entities| {
            entities
                .iter()
                .copied()
                .find(|&entity| matches!(self.entity(entity).kind, EntityKind::Placeholder))
        }) {
            return Some(placeholder);
        }
        if self.diag.dependencies.source_scopes.get(&span.file) != Some(&scope) {
            return None;
        }
        let module = self.scope(scope).module;
        let mut current = Some(scope);
        while let Some(scope) = current {
            let info = self.scope(scope);
            if info.module != module {
                break;
            }
            if let Some(entities) = info.names.get(&name) {
                if let Some(placeholder) = entities
                    .iter()
                    .copied()
                    .find(|&entity| matches!(self.entity(entity).kind, EntityKind::Placeholder))
                {
                    return Some(placeholder);
                }
                return self.diag.dependencies.bindings.iter().find_map(
                    |(&placeholder, definitions)| {
                        let declaration = self.entity(placeholder);
                        (declaration.name == name
                            && self.scope(declaration.scope).module == module
                            && entities.iter().all(|entity| definitions.contains(entity)))
                        .then_some(placeholder)
                    },
                );
            }
            current = info.parent;
        }
        None
    }

    pub(in crate::sema) fn bound_entities(&self, entities: &[EntityId]) -> Vec<EntityId> {
        let mut result = Vec::with_capacity(entities.len());
        for &entity in entities {
            for &definition in self
                .diag
                .dependencies
                .bindings
                .get(&entity)
                .map_or(std::slice::from_ref(&entity), Vec::as_slice)
            {
                if !result.contains(&definition) {
                    result.push(definition);
                }
            }
        }
        result
    }

    /// A declaration may not reuse a name declared in the same scope, except for procedures,
    /// which form an overload set (identical signatures are caught when the set is used).
    pub(in crate::sema) fn check_redeclared(
        &self,
        scope: ScopeId,
        name: Sym,
        span: Span,
        decl: Option<&Rc<ast::Decl>>,
    ) -> Result<()> {
        if name.as_str() == "_" {
            return Ok(());
        }
        let mut ids = self
            .scope(scope)
            .names
            .get(&name)
            .cloned()
            .unwrap_or_default();
        if let Some(placeholder) = self.generated_placeholder(scope, name, span)
            && let Some(definitions) = self.diag.dependencies.bindings.get(&placeholder)
        {
            ids.extend(definitions);
        }
        // A placeholder permits its definition, not additional definitions after it.
        // Real declarations retain the ordinary duplicate/overload rules.
        for id in self.bound_entities(&ids) {
            match (&self.entity(id).kind, decl) {
                (
                    EntityKind::Decl {
                        decl: old, ..
                    },
                    Some(new),
                ) if Rc::ptr_eq(old, new) || (may_overload(old) && may_overload(new)) => continue,
                (
                    EntityKind::Decl {
                        ..
                    }
                    | EntityKind::Import(_),
                    _,
                ) => {}
                _ => continue,
            }
            return Err(Box::new(
                Diagnostic::error(span, format!("`{name}` is already declared in this scope"))
                    .with_label("declared again here")
                    .with_note(
                        self.entity(id).span,
                        format!("`{name}` is first declared here"),
                    ),
            ));
        }
        Ok(())
    }

    /// Declare a constant whose value is already known.
    pub fn add_const(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
        value: Value,
        ty: TypeId,
    ) -> EntityId {
        let id = self.new_const(scope, name, span, value, ty);
        if self.ed.is_live() {
            self.ide_note_entity(id);
        }
        id
    }

    /// [`Self::add_const`] without the editor's record of the declaration: a constant the
    /// source does not declare at `span`.
    pub(in crate::sema) fn new_const(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
        value: Value,
        ty: TypeId,
    ) -> EntityId {
        let id = self.new_entity(
            scope,
            name,
            span,
            EntityKind::Const {
                value: value.clone(),
                ty,
            },
            true,
        );
        self.entity_mut(id).state = EntityState::Done(Resolved::Const {
            value,
            ty,
        });
        id
    }
}
