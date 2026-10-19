//! Looking names up in a module's declarations and `using` members.
use super::model::{EntityId, Found, UsingEntry};
use crate::sema::{Compiler, ModuleId, Result, Span, Sym, ast};

impl Compiler {
    /// Exported declarations named `name` in a module or the modules it re-exports (after
    /// expanding pending items). Only for fixed names the compiler itself looks up
    /// (`__arithmetic_overflow` in `Runtime_Support`); resolving a user's name through a
    /// module needs `module_lookup`, which also sees members of exported `using` globals.
    pub fn module_declarations(&mut self, module: ModuleId, name: Sym) -> Result<Vec<EntityId>> {
        // A search inside a re-export cycle skips modules, so only whole answers are kept.
        if !self.reexport_visiting.is_empty() {
            let ids = self.module_declarations_unbound(module, name)?;
            return Ok(self.bound_entities(&ids));
        }
        let generation = self.namespace_generation;
        if let Some((at, ids)) = self.module_names.get(&(module, name))
            && *at == generation
        {
            return Ok(self.bound_entities(ids));
        }
        let outer = std::mem::replace(&mut self.module_names_unsettled, false);
        let ids = self.module_declarations_unbound(module, name);
        let unsettled = std::mem::replace(&mut self.module_names_unsettled, outer);
        let ids = ids?;
        if !unsettled && self.namespace_generation == generation {
            self.module_names
                .insert((module, name), (generation, ids.as_slice().into()));
        }
        Ok(self.bound_entities(&ids))
    }

    /// `module_declarations` before aliases are replaced by what they are bound to.
    fn module_declarations_unbound(
        &mut self,
        module: ModuleId,
        name: Sym,
    ) -> Result<Vec<EntityId>> {
        let scope = self.modules[module.0 as usize].scope;
        self.expand_pending(scope)?;
        if !self.modules[module.0 as usize].loading {
            self.validate_module_arguments(module, Span::NONE)?;
        }
        let s = self.scope(scope);
        if s.pending_done < s.pending.len()
            || self.modules[module.0 as usize].loading
            || !self.modules[module.0 as usize].arguments_validated
        {
            self.module_names_unsettled = true;
        }
        let own: Vec<EntityId> = self
            .scope(scope)
            .names
            .get(&name)
            .map(|ids| {
                ids.iter()
                    .copied()
                    .filter(|&e| self.entity(e).exported)
                    .collect()
            })
            .unwrap_or_default();
        if !own.is_empty() {
            return Ok(own);
        }
        // Names of modules this one re-exports with `using M :: #import "M";` (each module
        // once per lookup, so re-export cycles end).
        if self.reexport_visiting.contains(&module) {
            return Ok(own);
        }
        self.reexport_visiting.push(module);
        let mut result = Ok(own);
        for k in 0..self.modules[module.0 as usize].exported_using_imports.len() {
            let Some(&index) = self.modules[module.0 as usize]
                .exported_using_imports
                .get(k)
            else {
                break;
            };
            let ids = match self.lookup_import(scope, index) {
                Ok(Some(inner)) => {
                    if filter_hides(&self.scope(scope).imports[index].filter, name) {
                        continue;
                    }
                    self.module_declarations_unbound(inner, name)
                }
                Ok(None) => {
                    self.module_names_unsettled = true;
                    continue;
                }
                Err(e) => Err(e),
            };
            if !matches!(&ids, Ok(ids) if ids.is_empty()) {
                result = ids;
                break;
            }
        }
        self.reexport_visiting.pop();
        result
    }

    /// What `name` means as a member of `module`: `M.name` through a named import, `name`
    /// in a scope that imports M or says `using M;`, and the names M re-exports. In order:
    /// M's exported declarations (and those of modules it re-exports), members of globals
    /// it exports with `using` (GL's `using gl;` makes `GL.glViewport` work), then module
    /// parameters (`Basic.MEMORY_DEBUGGER`).
    ///
    /// Every path that resolves a name through a module must use this (or
    /// `merge_module_lookup`), never `module_declarations` alone: separate copies of these
    /// rules drifted apart once (`GL.glViewport` failed while `glViewport` worked).
    /// `tests/compiler/regressions/module-member-resolution.jai` checks that all paths agree.
    pub fn module_lookup(&mut self, module: ModuleId, name: Sym) -> Result<Found> {
        // Looking up a name a module lacks walks its re-exports and `using` globals, and a
        // scope asks every module it imports for every name it uses: remember the misses.
        let generation = self.namespace_generation;
        if self.reexport_visiting.is_empty()
            && self.module_misses.get(&(module, name)) == Some(&generation)
        {
            return Ok(Found::Entities(Vec::new()));
        }
        let ids = self.module_declarations(module, name)?;
        if !ids.is_empty() {
            return Ok(Found::Entities(ids));
        }
        if let Some((entry, member)) = self.module_using_member(module, name)? {
            return Ok(Found::Using(entry, member));
        }
        let params: Vec<EntityId> = self.modules[module.0 as usize]
            .param_entities
            .iter()
            .copied()
            .filter(|&e| self.entity(e).name == name)
            .collect();
        if params.is_empty()
            && self.reexport_visiting.is_empty()
            && self.namespace_generation == generation
            && matches!(self.module_names.get(&(module, name)), Some((at, _)) if *at == generation)
        {
            self.module_misses.insert((module, name), generation);
        }
        Ok(Found::Entities(params))
    }

    /// Qualified lookup observes the same export boundary as an anonymous import.
    pub fn module_member_lookup(&mut self, module: ModuleId, name: Sym) -> Result<Found> {
        self.module_lookup(module, name)
    }

    /// Can `module` or a module it re-exports hold an exported `using` of a value? A
    /// `false` answer is remembered until the namespace next changes; anything not yet
    /// settled (a module still loading, an import not resolved) answers `true`.
    fn module_may_have_using_members(&mut self, module: ModuleId) -> bool {
        let generation = self.namespace_generation;
        if self.using_member_free.get(&module) == Some(&generation) {
            return false;
        }
        let mut visiting = Vec::new();
        let may = self.module_may_have_using_members_inner(module, &mut visiting);
        if !may && self.namespace_generation == generation {
            for m in visiting {
                self.using_member_free.insert(m, generation);
            }
        }
        may
    }

    fn module_may_have_using_members_inner(
        &self,
        module: ModuleId,
        visiting: &mut Vec<ModuleId>,
    ) -> bool {
        if visiting.contains(&module) {
            return false;
        }
        let data = &self.modules[module.0 as usize];
        if data.loading || !data.exported_usings.is_empty() {
            return true;
        }
        visiting.push(module);
        let scope = self.scope(data.scope);
        for &index in &data.exported_using_imports {
            match scope.imports.get(index).and_then(|import| import.module) {
                Some(inner) => {
                    if self.module_may_have_using_members_inner(inner, visiting) {
                        return true;
                    }
                }
                None => return true,
            }
        }
        false
    }

    /// The exported `using` entry of `module` (or of a module it re-exports) whose type has
    /// a member `name`.
    fn module_using_member(
        &mut self,
        module: ModuleId,
        name: Sym,
    ) -> Result<Option<(UsingEntry, Sym)>> {
        if !self.module_may_have_using_members(module) {
            return Ok(None);
        }
        for k in 0..self.modules[module.0 as usize].exported_usings.len() {
            let Some(entry) = self.modules[module.0 as usize].exported_usings.get(k) else {
                break;
            };
            if let UsingEntry::Place {
                ty, ..
            }
            | UsingEntry::Type(ty) = entry
            {
                // The common case, answered without copying the entry on a miss.
                let ty = *ty;
                if self.type_has_member(ty, name)? {
                    let entry = self.modules[module.0 as usize].exported_usings[k].clone();
                    return Ok(Some((entry, name)));
                }
                continue;
            }
            let entry = entry.clone();
            if let Some(found) = self.aggregate_using_member(&entry, name)? {
                return Ok(Some(found));
            }
        }
        if self.reexport_visiting.contains(&module) {
            return Ok(None);
        }
        self.reexport_visiting.push(module);
        let scope = self.modules[module.0 as usize].scope;
        let mut result = Ok(None);
        for k in 0..self.modules[module.0 as usize].exported_using_imports.len() {
            let Some(&index) = self.modules[module.0 as usize]
                .exported_using_imports
                .get(k)
            else {
                break;
            };
            match self.lookup_import(scope, index) {
                Ok(Some(inner))
                    if !filter_hides(&self.scope(scope).imports[index].filter, name) =>
                {
                    match self.module_using_member(inner, name) {
                        Ok(None) => {}
                        other => {
                            result = other;
                            break;
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }
        self.reexport_visiting.pop();
        result
    }

    /// `module_lookup` as one step of a scope lookup: entities join the overload set in
    /// `found`; returns the final answer when the lookup is decided.
    pub(super) fn merge_module_lookup(
        &mut self,
        found: &mut Vec<EntityId>,
        module: ModuleId,
        name: Sym,
    ) -> Result<Option<Found>> {
        Ok(match self.module_lookup(module, name)? {
            Found::Entities(ids) => (!ids.is_empty() && self.collect(found, ids))
                .then(|| Found::Entities(std::mem::take(found))),
            Found::Using(entry, member) => found.is_empty().then_some(Found::Using(entry, member)),
        })
    }
}

/// Does a `using` filter keep `name` out? Computed and mapping filters hide nothing here.
pub fn filter_hides(filter: &ast::UsingFilter, name: Sym) -> bool {
    match filter {
        ast::UsingFilter::Only(_) | ast::UsingFilter::Except(_) => {
            crate::sema::using::filter_target(filter, name).is_none()
        }
        _ => false,
    }
}
