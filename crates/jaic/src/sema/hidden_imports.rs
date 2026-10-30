//! Declarations of the program that a module it imports also exports.
//!
//! The official compiler refuses a program that declares a name one of its own imports exports,
//! unless both are procedures whose parameters differ (an overload set). Jacket lets the
//! program's own declaration win, which is what a library module relies on and the official
//! compiler allows there. This sweep warns once per declaration of the program's own files, after
//! the program has settled, so it does not depend on the name being used.
use crate::diagnostics::Code;
use crate::sema::scope::{EntityId, Found, Resolved, ScopeId, ScopeKind, filter_name};
use crate::sema::{Compiler, Diagnostic, ModuleId, Sym, Value};
use crate::types::TypeId;

/// What a procedure declaration is told apart by.
type Parameters = (Vec<TypeId>, bool);

impl Compiler {
    /// Warn at each declaration of the program's own files that an import of the program also
    /// exports (see the module comment).
    pub(in crate::sema) fn warn_declarations_hiding_imports(&mut self) {
        let Some(main) = self.ns.main_module else {
            return;
        };
        let scopes: Vec<ScopeId> = (0..self.ns.scopes.len() as u32)
            .map(ScopeId)
            .filter(|&s| {
                let scope = self.scope(s);
                scope.module == main && matches!(scope.kind, ScopeKind::Module | ScopeKind::File)
            })
            .collect();
        for &scope in &scopes {
            let reaching: Vec<ScopeId> = if self.scope(scope).kind == ScopeKind::Module {
                scopes.clone()
            } else {
                scopes
                    .iter()
                    .copied()
                    .filter(|&s| s == scope || self.scope(s).kind == ScopeKind::Module)
                    .collect()
            };
            let mut names: Vec<(Sym, Vec<EntityId>)> = self
                .scope(scope)
                .names
                .iter()
                .map(|(&name, ids)| (name, ids.clone()))
                .collect();
            names.sort_by_key(|(name, _)| name.as_str());
            for (name, ids) in names {
                self.warn_if_imports_export(&reaching, name, &ids);
            }
        }
    }

    fn warn_if_imports_export(&mut self, reaching: &[ScopeId], name: Sym, ids: &[EntityId]) {
        let own: Vec<EntityId> = ids
            .iter()
            .copied()
            .filter(|&id| {
                matches!(
                    self.entity(id).kind,
                    crate::sema::scope::EntityKind::Decl { .. }
                        | crate::sema::scope::EntityKind::Const { .. }
                )
            })
            .collect();
        let Some(&first) = own.first() else {
            return;
        };
        let mut imported: Vec<(ModuleId, Sym)> = Vec::new();
        for &scope in reaching {
            for entry in &self.scope(scope).imports {
                let (Some(module), Some(target)) =
                    (entry.module, filter_name(Some(&entry.filter), name))
                else {
                    continue;
                };
                if !imported.contains(&(module, target)) {
                    imported.push((module, target));
                }
            }
        }
        for (module, target) in imported {
            let Ok(Found::Entities(theirs)) = self.module_lookup(module, target) else {
                continue;
            };
            let Some(&their) = theirs.first() else {
                continue;
            };
            let procedures = own.iter().all(|&id| self.overloadable_entity(id))
                && theirs.iter().all(|&id| self.overloadable_entity(id));
            if procedures {
                let mine = self.parameter_lists(&own);
                let same = self
                    .parameter_lists(&theirs)
                    .iter()
                    .any(|signature| mine.contains(signature));
                if !same {
                    continue;
                }
            }
            self.report_hidden_import(first, their, module, name, procedures);
            return;
        }
    }

    fn report_hidden_import(
        &mut self,
        own: EntityId,
        their: EntityId,
        module: ModuleId,
        name: Sym,
        procedures: bool,
    ) {
        let at = self.entity(own).span;
        let their_span = self.entity(their).span;
        let scope = self.entity(own).scope;
        let module_name = self.ns.modules[module.0 as usize].name.clone();
        let message = if procedures {
            format!("`{name}` has the same parameters as the one `{module_name}` exports")
        } else {
            format!("`{name}` is also exported by `{module_name}`, which this program imports")
        };
        let Some(d) =
            self.leniency_warning(self.scope(scope).module, at, Code::HidesImport, message)
        else {
            return;
        };
        let alias = if self.module_name(module).is_some() {
            "M"
        } else {
            "Imported"
        };
        let d: Diagnostic = d
            .with_label("this declaration is the one used")
            .with_note(their_span, "also declared here")
            .with_help(format!(
                "rename it, or import the module under a name (`{alias} :: #import \"{module_name}\";`) and write `{alias}.{name}` for the imported one"
            ));
        self.warn(d);
    }

    /// Is the entity a procedure, or an alias of one?
    fn overloadable_entity(&mut self, id: EntityId) -> bool {
        self.entity_is_overloadable(id)
            || matches!(
                self.resolve_entity(id),
                Ok(Resolved::Proc(_)
                    | Resolved::ProcSet(_)
                    | Resolved::Const {
                        value: Value::Proc(_),
                        ..
                    })
            )
    }

    /// The parameter types of the monomorphic procedures among `ids`.
    fn parameter_lists(&mut self, ids: &[EntityId]) -> Vec<Parameters> {
        let mut lists = Vec::new();
        for &id in ids {
            let procs = match self.resolve_entity(id) {
                Ok(Resolved::Proc(p)) => vec![p],
                Ok(Resolved::ProcSet(set)) => set,
                Ok(Resolved::Const {
                    value: Value::Proc(p),
                    ..
                }) => vec![p],
                _ => continue,
            };
            for p in procs {
                if self.proc(p).is_poly {
                    continue;
                }
                let span = self.entity(id).span;
                let Ok(signature) = self.signature(p, span) else {
                    continue;
                };
                let crate::types::TypeKind::Proc(ty) = self.ty.types.kind(signature.ty).clone()
                else {
                    continue;
                };
                lists.push((ty.params.clone(), ty.variadic));
            }
        }
        lists
    }
}
