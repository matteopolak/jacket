//! Modules, scopes and entities: what names resolve against, and the work of filling them
//! (pending imports, `#poke_name`, `#add_context`, program parameters).

use super::body_probe::{Probed, W};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::ast;
use crate::diagnostics::Diagnostic;
use crate::fxhash::{HashMap, HashSet};
use crate::intern::Sym;

use super::{ConstArg, EntityId, Module, ModuleId, ScopeId, Value, decls, scope};

pub struct Namespaces {
    pub modules: Probed<Vec<Module>, { W::NsModules as usize }>,
    pub(in crate::sema) module_cache:
        Probed<HashMap<(PathBuf, Vec<ConstArg>), ModuleId>, { W::NsModuleCache as usize }>,
    pub(in crate::sema) module_failures:
        Probed<HashMap<ModuleId, Box<Diagnostic>>, { W::NsModuleFailures as usize }>,
    pub scopes: Probed<Vec<scope::Scope>, { W::NsScopes as usize }>,
    pub entities: Probed<Vec<scope::Entity>, { W::NsEntities as usize }>,
    pub libraries: Probed<Vec<decls::LibraryInfo>, { W::NsLibraries as usize }>,
    pub root_scope: ScopeId,
    pub preload: Option<ModuleId>,
    pub runtime_support: Option<ModuleId>,
    /// `__jaic_check_failed` was looked up (`note_check_handler`).
    pub check_handler_resolved: bool,
    pub main_module: Option<ModuleId>,
    /// `#add_context` declarations with their declaring scope, in load order.
    pub add_contexts: Probed<Vec<(Arc<ast::Decl>, ScopeId)>, { W::NsAddContexts as usize }>,
    /// `#poke_name Module name;` directives: (module expression, name, declaring file scope).
    pub pokes: Probed<Vec<(ast::Expr, Sym, ScopeId)>, { W::NsPokes as usize }>,
    /// Scopes holding the bindings that a polymorphic parameter's default is evaluated in.
    pub(in crate::sema) default_scopes:
        Probed<HashMap<(ScopeId, Vec<Value>), ScopeId>, { W::NsDefaultScopes as usize }>,
    /// Overload sets already checked for identical parameter lists.
    pub overload_sets_checked:
        Probed<HashSet<Vec<EntityId>>, { W::NsOverloadSetsChecked as usize }>,
    /// Imports that found no module: (importing module, wanted module). Metaprograms that
    /// intercept the workspace get a `FAILED_IMPORT` message for each.
    pub failed_imports: Probed<Vec<(String, String)>, { W::NsFailedImports as usize }>,
    /// Modules whose re-exports a `module_declarations` call is searching.
    pub reexport_visiting: Probed<Vec<ModuleId>, { W::NsReexportVisiting as usize }>,
    pub thunk_scopes: Probed<HashMap<ScopeId, ScopeId>, { W::NsThunkScopes as usize }>,
    /// Declarations the driver resolves even when unreferenced: `#program_export`
    /// procedures and `link_always` libraries.
    pub export_entities: Probed<Vec<EntityId>, { W::NsExportEntities as usize }>,
    /// Scopes with top-level items put back to waiting because they failed while
    /// procedure bodies were mid-lowering (retried by `expand_all`).
    pub deferred_pending: Probed<Vec<ScopeId>, { W::NsDeferredPending as usize }>,
    /// Module and file scopes that may still have top-level items to expand or imports to load.
    /// `expand_all` visits only these (in scope order) and drops the ones it finishes, so each
    /// round a metaprogram drives costs what is new rather than everything declared so far.
    pub unsettled: Probed<BTreeSet<ScopeId>, { W::NsUnsettled as usize }>,
    /// Entities below this index have had their named imports (`X :: #import`) resolved.
    pub named_imports_done: usize,
    /// Program parameters set by imports (`#import "Basic"()(MEMORY_DEBUGGER = DEBUG)`), by
    /// module entry path: (name, value expression, scope it is evaluated in). Applied to
    /// the module's instance as soon as both exist, before anything reads the parameter.
    pub program_param_settings: Vec<(PathBuf, Sym, ast::Expr, ScopeId)>,
}

impl Default for Namespaces {
    fn default() -> Self {
        Self {
            modules: Probed::default(),
            module_cache: Probed::default(),
            module_failures: Probed::default(),
            scopes: Probed::default(),
            entities: Probed::default(),
            libraries: Probed::default(),
            root_scope: ScopeId(0),
            preload: None,
            runtime_support: None,
            check_handler_resolved: false,
            main_module: None,
            add_contexts: Probed::default(),
            pokes: Probed::default(),
            default_scopes: Probed::default(),
            overload_sets_checked: Probed::default(),
            failed_imports: Probed::default(),
            reexport_visiting: Probed::default(),
            thunk_scopes: Probed::default(),
            export_entities: Probed::default(),
            deferred_pending: Probed::default(),
            unsettled: Probed::default(),
            named_imports_done: 0,
            program_param_settings: Vec::default(),
        }
    }
}
