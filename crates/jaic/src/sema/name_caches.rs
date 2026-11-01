//! Memoised name lookups, each stamped with the `Epochs` counters it was computed under.

use super::body_probe::{Probed, W};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::fxhash::HashMap;
use crate::intern::Sym;

use super::stdlib_index::StdlibView;
use super::{EntityId, ModuleId, ScopeId};
use crate::source::ImportSuggestion;

/// What an import suggestion is looked up by.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(in crate::sema) enum DeclaringKey {
    /// A name that is not declared anywhere in reach.
    Name(String),
    /// An operator, the struct names of its operands and the modules those structs live in.
    Operator(String, Vec<String>, Vec<String>),
}

/// A cached `module_declarations` answer and the `namespace_generation` it holds for.
type ModuleNames = (u64, Arc<[EntityId]>);

#[derive(Default)]
pub struct NameCaches {
    /// `module_declarations` answers (before `bound_entities`) and the `namespace_generation`
    /// they hold for. Only answers that no later expansion or import can change are kept.
    pub(in crate::sema) module_names:
        Probed<HashMap<(ModuleId, Sym), ModuleNames>, { W::CachesModuleNames as usize }>,
    /// Set while computing a `module_declarations` answer that may still change.
    pub(in crate::sema) module_names_unsettled: bool,
    /// Names `module_lookup` found nothing for, with the `namespace_generation` that holds
    /// for. Only names whose `module_names` entry is settled are kept.
    pub(in crate::sema) module_misses:
        Probed<HashMap<(ModuleId, Sym), u64>, { W::CachesModuleMisses as usize }>,
    /// Scopes whose `using`s, imports and implicit modules hold nothing under a name, with
    /// the `namespace_generation` that holds for.
    pub(in crate::sema) scope_misses:
        Probed<HashMap<(ScopeId, Sym), u64>, { W::CachesScopeMisses as usize }>,
    /// The imports of a scope with many that a lookup has to ask (`Compiler::import_plan`), with
    /// the `namespace_generation` and import count they were made for.
    pub(in crate::sema) import_plans: HashMap<ScopeId, (u64, usize, Arc<[u32]>)>,
    /// Modules without any exported `using` member (here or re-exported), and the
    /// `namespace_generation` that holds for.
    pub(in crate::sema) using_member_free:
        Probed<HashMap<ModuleId, u64>, { W::CachesUsingMemberFree as usize }>,
    /// The imports found for a name or an operator that nothing declares, by what was asked
    /// for. A program that misses the same one many times would read the whole standard
    /// library each time.
    pub(in crate::sema) declaring_imports: RefCell<HashMap<DeclaringKey, Vec<ImportSuggestion>>>,
    /// The standard library's index of declared names for this compile, once asked for (`None`
    /// inside when there is no standard library folder).
    pub(in crate::sema) stdlib_view: RefCell<Option<Option<Rc<StdlibView>>>>,
}
