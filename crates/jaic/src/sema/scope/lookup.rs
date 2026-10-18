//! Name lookup through scopes, imports and using entries.
use super::model::{EntityId, EntityKind, Found, ScopeId, ScopeKind, UsingEntry};
use super::module_lookup::filter_hides;
use crate::sema::*;

impl Compiler {
    /// Look up `name` starting at `scope`, ignoring `using` struct members.
    pub fn lookup(&mut self, scope: ScopeId, name: Sym) -> Result<Vec<EntityId>> {
        match self.lookup_full(scope, name)? {
            Found::Entities(ids) => Ok(ids),
            Found::Using(..) => Ok(Vec::new()),
        }
    }

    /// Full lookup. Procedures from several scopes merge into one overload
    /// set; any other binding shadows everything outside it. `using` entries
    /// of a scope are consulted after its own names.
    pub fn lookup_full(&mut self, scope: ScopeId, name: Sym) -> Result<Found> {
        let found = self.lookup_full_raw(scope, name)?;
        // A `#placeholder` reached through an import gives way to its definition (added
        // to the module by a metaprogram) found along with it.
        Ok(match found {
            Found::Entities(ids)
                if ids.len() > 1
                    && ids
                        .iter()
                        .any(|&e| matches!(self.entity(e).kind, EntityKind::Placeholder))
                    && ids
                        .iter()
                        .any(|&e| !matches!(self.entity(e).kind, EntityKind::Placeholder)) =>
            {
                Found::Entities(
                    ids.into_iter()
                        .filter(|&e| !matches!(self.entity(e).kind, EntityKind::Placeholder))
                        .collect(),
                )
            }
            other => other,
        })
    }

    fn lookup_full_raw(&mut self, scope: ScopeId, name: Sym) -> Result<Found> {
        let mut found: Vec<EntityId> = Vec::new();
        let mut current = Some(scope);
        while let Some(sid) = current {
            // A name this scope already binds to a non-procedure is final (another
            // declaration would be a redefinition): pending items need not run first.
            let settled = self.scope(sid).names.get(&name).is_some_and(|ids| {
                ids.iter().any(|&e| {
                    !matches!(self.entity(e).kind, EntityKind::Placeholder)
                        && !self.entity_is_overloadable(e)
                })
            });
            if !settled && !self.lookup_without_expansion {
                self.expand_pending(sid)?;
            }
            if let Some(ids) = self.scope(sid).names.get(&name) {
                let mut ids = self.bound_entities(ids);
                // A `#placeholder` gives way to the real declaration (possibly
                // added later by a metaprogram).
                let is_placeholder =
                    |e: &EntityId| matches!(self.entity(*e).kind, EntityKind::Placeholder);
                if ids.iter().any(is_placeholder) {
                    let defined = ids.iter().any(|e| !is_placeholder(e));
                    ids.retain(|e| !is_placeholder(e) || (!defined && found.is_empty()));
                }
                if !ids.is_empty() && self.collect(&mut found, ids) {
                    return Ok(Found::Entities(found));
                }
            }
            // What `sid`'s `using`s, imports and the implicit modules hold under `name` can
            // only change with the namespace: a miss is remembered for this generation.
            let generation = self.namespace_generation;
            let key = (sid, name);
            // Body scopes gain `using`s and imports without moving the generation.
            let stable = !matches!(
                self.scope(sid).kind,
                ScopeKind::Proc | ScopeKind::Block | ScopeKind::Macro
            );
            let known_miss = stable
                && self.scope_misses.get(&key) == Some(&generation)
                && self.reexport_visiting.is_empty();
            let try_remember =
                stable && !known_miss && found.is_empty() && self.reexport_visiting.is_empty();
            let mut cacheable = try_remember;
            if !known_miss {
                // Entries added while these are searched are not seen by this lookup.
                for k in 0..self.scope(sid).usings.len() {
                    let Some(entry) = self.scope(sid).usings.get(k) else {
                        break;
                    };
                    match entry {
                        UsingEntry::Module(m, filter) => {
                            if filter.as_deref().is_some_and(|f| filter_hides(f, name)) {
                                continue;
                            }
                            let m = *m;
                            if let Some(result) = self.merge_module_lookup(&mut found, m, name)? {
                                return Ok(result);
                            }
                        }
                        _ => {
                            cacheable = false;
                            let entry = entry.clone();
                            if found.is_empty()
                                && let Some((target, member)) =
                                    self.aggregate_using_member(&entry, name)?
                            {
                                return Ok(Found::Using(target, member));
                            }
                        }
                    }
                }
                let imports = self.scope(sid).imports.len();
                for i in 0..imports {
                    let resolved = match self.scope(sid).imports[i].module {
                        Some(module) => Some(module),
                        None => self.lookup_import(sid, i)?,
                    };
                    let Some(module) = resolved else {
                        cacheable = false;
                        continue;
                    };
                    {
                        if filter_hides(&self.scope(sid).imports[i].filter, name) {
                            continue;
                        }
                        let before = found.len();
                        let result = self.merge_module_lookup(&mut found, module, name)?;
                        if result.is_some() || found.len() > before {
                            self.ide_note_import_use(sid, i);
                        }
                        if let Some(result) = result {
                            return Ok(result);
                        }
                    }
                }
                // Preload and Runtime_Support are implicitly visible to every module.
                if self.scope(sid).kind == ScopeKind::Module {
                    for implicit in [self.preload, self.runtime_support].into_iter().flatten() {
                        if self.scope(sid).module == implicit {
                            continue;
                        }
                        if let Some(result) =
                            self.merge_module_lookup(&mut found, implicit, name)?
                        {
                            return Ok(result);
                        }
                    }
                }
                if cacheable && found.is_empty() && self.namespace_generation == generation {
                    self.scope_misses.insert(key, generation);
                }
            }
            current = self.scope(sid).parent;
        }
        if found.is_empty() {
            let mut current = Some(scope);
            while let Some(sid) = current {
                for fallback in self.scope(sid).fallbacks.clone() {
                    if let Found::Entities(ids) = self.lookup_full(fallback, name)?
                        && !ids.is_empty()
                    {
                        return Ok(Found::Entities(ids));
                    }
                }
                current = self.scope(sid).parent;
            }
            return Ok(Found::Entities(found));
        }
        Ok(Found::Entities(found))
    }

    /// Merge `ids` into `found`; returns true when the lookup must stop
    /// (a non-procedure binding shadows everything outside it).
    pub(super) fn collect(&mut self, found: &mut Vec<EntityId>, ids: Vec<EntityId>) -> bool {
        let overloadable = ids.iter().all(|&e| self.overloadable_or_alias(e));
        if found.is_empty() {
            *found = ids;
            return !overloadable;
        }
        if overloadable {
            for id in ids {
                if !found.contains(&id) {
                    found.push(id);
                }
            }
        }
        false
    }
}
