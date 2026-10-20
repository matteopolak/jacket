//! Modules, scopes and entities: what names resolve against, and the work of filling them
//! (pending imports, `#poke_name`, `#add_context`, program parameters).

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::rc::Rc;

use crate::ast;
use crate::fxhash::{HashMap, HashSet};
use crate::intern::Sym;
use crate::source::Diagnostic;

use super::{ConstArg, EntityId, Module, ModuleId, ScopeId, Value, decls, scope};

pub struct Namespaces {
    pub modules: Vec<Module>,
    pub(in crate::sema) module_cache: HashMap<(PathBuf, Vec<ConstArg>), ModuleId>,
    pub(in crate::sema) module_failures: HashMap<ModuleId, Box<Diagnostic>>,
    pub scopes: Vec<scope::Scope>,
    pub entities: Vec<scope::Entity>,
    pub libraries: Vec<decls::LibraryInfo>,
    pub root_scope: ScopeId,
    pub preload: Option<ModuleId>,
    pub runtime_support: Option<ModuleId>,
    /// `__jaic_check_failed` was looked up (`note_check_handler`).
    pub check_handler_resolved: bool,
    pub main_module: Option<ModuleId>,
    /// `#add_context` declarations with their declaring scope, in load order.
    pub add_contexts: Vec<(Rc<ast::Decl>, ScopeId)>,
    /// `#poke_name Module name;` directives: (module expression, name, declaring file scope).
    pub pokes: Vec<(ast::Expr, Sym, ScopeId)>,
    /// Scopes holding the bindings that a polymorphic parameter's default is evaluated in.
    pub(in crate::sema) default_scopes: HashMap<(ScopeId, Vec<Value>), ScopeId>,
    /// Overload sets already checked for identical parameter lists.
    pub overload_sets_checked: HashSet<Vec<EntityId>>,
    /// Imports that found no module: (importing module, wanted module). Metaprograms that
    /// intercept the workspace get a `FAILED_IMPORT` message for each.
    pub failed_imports: Vec<(String, String)>,
    /// Modules whose re-exports a `module_declarations` call is searching.
    pub reexport_visiting: Vec<ModuleId>,
    pub thunk_scopes: HashMap<ScopeId, ScopeId>,
    /// Declarations the driver resolves even when unreferenced: `#program_export`
    /// procedures and `link_always` libraries.
    pub export_entities: Vec<EntityId>,
    /// Scopes with top-level items put back to waiting because they failed while
    /// procedure bodies were mid-lowering (retried by `expand_all`).
    pub deferred_pending: Vec<ScopeId>,
    /// Module and file scopes that may still have top-level items to expand or imports to load.
    /// `expand_all` visits only these (in scope order) and drops the ones it finishes, so each
    /// round a metaprogram drives costs what is new rather than everything declared so far.
    pub unsettled: BTreeSet<ScopeId>,
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
            modules: Vec::new(),
            module_cache: HashMap::default(),
            module_failures: HashMap::default(),
            scopes: Vec::new(),
            entities: Vec::new(),
            libraries: Vec::new(),
            root_scope: ScopeId(0),
            preload: None,
            runtime_support: None,
            check_handler_resolved: false,
            main_module: None,
            add_contexts: Vec::new(),
            pokes: Vec::new(),
            default_scopes: HashMap::default(),
            overload_sets_checked: HashSet::default(),
            failed_imports: Vec::new(),
            reexport_visiting: Vec::new(),
            thunk_scopes: HashMap::default(),
            export_entities: Vec::new(),
            deferred_pending: Vec::new(),
            unsettled: BTreeSet::new(),
            named_imports_done: 0,
            program_param_settings: Vec::new(),
        }
    }
}
