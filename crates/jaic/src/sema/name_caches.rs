//! Memoised name lookups, each stamped with the `Epochs` counters it was computed under.

use std::cell::RefCell;
use std::rc::Rc;

use crate::fxhash::HashMap;
use crate::intern::Sym;

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
type ModuleNames = (u64, Rc<[EntityId]>);

#[derive(Default)]
pub struct NameCaches {
    /// `module_declarations` answers (before `bound_entities`) and the `namespace_generation`
    /// they hold for. Only answers that no later expansion or import can change are kept.
    pub(in crate::sema) module_names: HashMap<(ModuleId, Sym), ModuleNames>,
    /// Set while computing a `module_declarations` answer that may still change.
    pub(in crate::sema) module_names_unsettled: bool,
    /// Names `module_lookup` found nothing for, with the `namespace_generation` that holds
    /// for. Only names whose `module_names` entry is settled are kept.
    pub(in crate::sema) module_misses: HashMap<(ModuleId, Sym), u64>,
    /// Scopes whose `using`s, imports and implicit modules hold nothing under a name, with
    /// the `namespace_generation` that holds for.
    pub(in crate::sema) scope_misses: HashMap<(ScopeId, Sym), u64>,
    /// Modules without any exported `using` member (here or re-exported), and the
    /// `namespace_generation` that holds for.
    pub(in crate::sema) using_member_free: HashMap<ModuleId, u64>,
    /// The imports found for a name or an operator that nothing declares, by what was asked
    /// for. A program that misses the same one many times would read the whole standard
    /// library each time.
    pub(in crate::sema) declaring_imports: RefCell<HashMap<DeclaringKey, Vec<ImportSuggestion>>>,
}
