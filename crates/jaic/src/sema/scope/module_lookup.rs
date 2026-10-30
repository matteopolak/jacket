//! Looking names up in a module's declarations and `using` members.
use super::model::{EntityId, Found, UsingEntry};
use crate::sema::Namespaces;
use crate::sema::{Compiler, ModuleId, Result, Span, Sym, ast};

impl Compiler {
    /// Exported declarations named `name` in a module or the modules it re-exports (after
    /// expanding pending items). Only for fixed names the compiler itself looks up
    /// (`__arithmetic_overflow` in `Runtime_Support`); resolving a user's name through a
    /// module needs `module_lookup`, which also sees members of exported `using` globals.
    pub fn module_declarations(&mut self, module: ModuleId, name: Sym) -> Result<Vec<EntityId>> {
        // A search inside a re-export cycle skips modules, so only whole answers are kept.
        if !self.ns.reexport_visiting.is_empty() {
            let ids = self.module_declarations_unbound(module, name)?;
            return Ok(self.diag.bound_entities(&ids));
        }
        let generation = self.epochs.namespace_generation;
        if let Some((at, ids)) = self.caches.module_names.get(&(module, name))
            && *at == generation
        {
            return Ok(self.diag.bound_entities(ids));
        }
        let outer = std::mem::replace(&mut self.caches.module_names_unsettled, false);
        let ids = self.module_declarations_unbound(module, name);
        let unsettled = std::mem::replace(&mut self.caches.module_names_unsettled, outer);
        let ids = ids?;
        if !unsettled && self.epochs.namespace_generation == generation {
            self.caches
                .module_names
                .insert((module, name), (generation, ids.as_slice().into()));
        }
        Ok(self.diag.bound_entities(&ids))
    }

    /// `module_declarations` before aliases are replaced by what they are bound to.
    fn module_declarations_unbound(
        &mut self,
        module: ModuleId,
        name: Sym,
    ) -> Result<Vec<EntityId>> {
        let scope = self.ns.modules[module.0 as usize].scope;
        self.expand_pending(scope)?;
        if !self.ns.modules[module.0 as usize].loading {
            self.validate_module_arguments(module, Span::NONE)?;
        }
        let s = self.scope(scope);
        if s.pending_done < s.pending.len()
            || self.ns.modules[module.0 as usize].loading
            || !self.ns.modules[module.0 as usize].arguments_validated
        {
            self.caches.module_names_unsettled = true;
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
        // Procedures the module declares and procedures it re-exports under the same name form
        // one overload set; any other declaration hides what is re-exported.
        let overloads = !own.is_empty() && own.iter().all(|&e| self.entity_is_overloadable(e));
        if !own.is_empty() && !overloads {
            return Ok(own);
        }
        // Names of modules this one re-exports with `using M :: #import "M";` (each module
        // once per lookup, so re-export cycles end).
        if self.ns.reexport_visiting.contains(&module) {
            return Ok(own);
        }
        self.ns.reexport_visiting.push(module);
        let mut result = Ok(if overloads {
            Vec::new()
        } else {
            own.clone()
        });
        for k in 0..self.ns.modules[module.0 as usize]
            .exported_using_imports
            .len()
        {
            let Some(&index) = self.ns.modules[module.0 as usize]
                .exported_using_imports
                .get(k)
            else {
                break;
            };
            let ids = match self.lookup_import(scope, index) {
                Ok(Some(inner)) => {
                    let Some(target) =
                        filter_name(Some(&self.scope(scope).imports[index].filter), name)
                    else {
                        continue;
                    };
                    self.module_declarations_unbound(inner, target)
                }
                Ok(None) => {
                    self.caches.module_names_unsettled = true;
                    continue;
                }
                Err(e) => Err(e),
            };
            if !matches!(&ids, Ok(ids) if ids.is_empty()) {
                result = ids;
                break;
            }
        }
        if matches!(&result, Ok(ids) if ids.is_empty()) {
            for (used, target) in self.ns.using_modules(module, name, true) {
                let ids = self.module_declarations_unbound(used, target);
                if !matches!(&ids, Ok(ids) if ids.is_empty()) {
                    result = ids;
                    break;
                }
            }
        }
        self.ns.reexport_visiting.pop();
        if overloads {
            let mut merged = own;
            for id in result? {
                if self.entity_is_overloadable(id) && !merged.contains(&id) {
                    merged.push(id);
                }
            }
            return Ok(merged);
        }
        result
    }

    /// What an anonymous import or `using M` exposes: exported declarations and
    /// re-exports, exported using members, then module parameters. Qualified `M.name`
    /// uses `module_member_lookup` to include M's own module-scope declarations too.
    /// Keep imported-name lookup here so export filters and its caches agree.
    pub fn module_lookup(&mut self, module: ModuleId, name: Sym) -> Result<Found> {
        // Looking up a name a module lacks walks its re-exports and `using` globals, and a
        // scope asks every module it imports for every name it uses: remember the misses.
        let generation = self.epochs.namespace_generation;
        if self.ns.reexport_visiting.is_empty()
            && self.caches.module_misses.get(&(module, name)) == Some(&generation)
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
        let params: Vec<EntityId> = self.ns.modules[module.0 as usize]
            .param_entities
            .iter()
            .copied()
            .filter(|&e| self.entity(e).name == name)
            .collect();
        if params.is_empty()
            && self.ns.reexport_visiting.is_empty()
            && self.epochs.namespace_generation == generation
            && matches!(self.caches.module_names.get(&(module, name)), Some((at, _)) if *at == generation)
        {
            self.caches.module_misses.insert((module, name), generation);
        }
        Ok(Found::Entities(params))
    }

    /// A qualified name sees this module's declarations and using members. Following a
    /// using edge still imports only the other module's exports, not its private names.
    pub fn module_member_lookup(&mut self, module: ModuleId, name: Sym) -> Result<Found> {
        // Settle declarations and arguments before examining the module's own scope.
        self.module_declarations(module, name)?;
        let scope = self.ns.modules[module.0 as usize].scope;
        let own: Vec<_> = self
            .scope(scope)
            .names
            .get(&name)
            .into_iter()
            .flatten()
            .copied()
            .filter(|&id| !self.entity(id).file_private)
            .collect();
        if !own.is_empty() {
            return Ok(Found::Entities(self.diag.bound_entities(&own)));
        }
        let found = self.module_lookup(module, name)?;
        if !matches!(&found, Found::Entities(ids) if ids.is_empty()) {
            return Ok(found);
        }
        for entry in self.scope(scope).usings.clone() {
            if let UsingEntry::Module(inner, filter) = &entry {
                if let Some(target) = filter_name(filter.as_deref(), name) {
                    let found = self.module_lookup(*inner, target)?;
                    if !matches!(&found, Found::Entities(ids) if ids.is_empty()) {
                        return Ok(found);
                    }
                }
            } else if let Some((entry, member)) = self.aggregate_using_member(&entry, name)? {
                return Ok(Found::Using(entry, member));
            }
        }
        for index in self.ns.modules[module.0 as usize]
            .member_using_imports
            .clone()
        {
            let Some(target) = filter_name(Some(&self.scope(scope).imports[index].filter), name)
            else {
                continue;
            };
            if let Some(inner) = self.lookup_import(scope, index)? {
                let found = self.module_lookup(inner, target)?;
                if !matches!(&found, Found::Entities(ids) if ids.is_empty()) {
                    return Ok(found);
                }
            }
        }
        Ok(found)
    }

    /// Can `module` or a module it re-exports hold an exported `using` of a value? A
    /// `false` answer is remembered until the namespace next changes; anything not yet
    /// settled (a module still loading, an import not resolved) answers `true`.
    fn module_may_have_using_members(&mut self, module: ModuleId) -> bool {
        let generation = self.epochs.namespace_generation;
        if self.caches.using_member_free.get(&module) == Some(&generation) {
            return false;
        }
        let mut visiting = Vec::new();
        let may = self.module_may_have_using_members_inner(module, &mut visiting);
        if !may && self.epochs.namespace_generation == generation {
            for m in visiting {
                self.caches.using_member_free.insert(m, generation);
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
        let data = &self.ns.modules[module.0 as usize];
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
        for used in &data.using_modules {
            if used.exported && self.module_may_have_using_members_inner(used.module, visiting) {
                return true;
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
        for k in 0..self.ns.modules[module.0 as usize].exported_usings.len() {
            let Some(entry) = self.ns.modules[module.0 as usize].exported_usings.get(k) else {
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
                    let entry = self.ns.modules[module.0 as usize].exported_usings[k].clone();
                    return Ok(Some((entry, name)));
                }
                continue;
            }
            let entry = entry.clone();
            if let Some(found) = self.aggregate_using_member(&entry, name)? {
                return Ok(Some(found));
            }
        }
        if self.ns.reexport_visiting.contains(&module) {
            return Ok(None);
        }
        self.ns.reexport_visiting.push(module);
        let scope = self.ns.modules[module.0 as usize].scope;
        let mut result = Ok(None);
        for k in 0..self.ns.modules[module.0 as usize]
            .exported_using_imports
            .len()
        {
            let Some(&index) = self.ns.modules[module.0 as usize]
                .exported_using_imports
                .get(k)
            else {
                break;
            };
            match self.lookup_import(scope, index) {
                Ok(Some(inner)) => {
                    let Some(target) =
                        filter_name(Some(&self.scope(scope).imports[index].filter), name)
                    else {
                        continue;
                    };
                    match self.module_using_member(inner, target) {
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
        if matches!(result, Ok(None)) {
            for (used, target) in self.ns.using_modules(module, name, true) {
                match self.module_using_member(used, target) {
                    Ok(None) => {}
                    other => {
                        result = other;
                        break;
                    }
                }
            }
        }
        self.ns.reexport_visiting.pop();
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
        ast::UsingFilter::Only(_) | ast::UsingFilter::Except(_) | ast::UsingFilter::Map(_) => {
            crate::sema::using::filter_target(filter, name).is_none()
        }
        _ => false,
    }
}

/// The name a module declares for `name` as seen through an optional `using` filter;
/// `None` when the filter keeps it out. Mapping filters answer with the original name.
pub(in crate::sema) fn filter_name(filter: Option<&ast::UsingFilter>, name: Sym) -> Option<Sym> {
    match filter {
        Some(
            filter @ (ast::UsingFilter::Only(_)
            | ast::UsingFilter::Except(_)
            | ast::UsingFilter::Map(_)),
        ) => crate::sema::using::filter_target(filter, name),
        _ => Some(name),
    }
}

impl Namespaces {
    /// Modules `module` says `using M;` to whose filters let `name` through: only those at
    /// export scope when `exported_only`.
    fn using_modules(
        &self,
        module: ModuleId,
        name: Sym,
        exported_only: bool,
    ) -> Vec<(ModuleId, Sym)> {
        self.modules[module.0 as usize]
            .using_modules
            .iter()
            .filter(|used| used.exported || !exported_only)
            .filter_map(|used| Some((used.module, filter_name(used.filter.as_deref(), name)?)))
            .collect()
    }
}
