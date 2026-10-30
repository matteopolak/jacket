//! Scopes, entities and name lookup.
mod bindings;
mod builtins;
mod expand;
mod lookup;
mod model;
mod module_lookup;

pub(super) use self::expand::may_overload;
pub use self::model::{
    Builtin, BuiltinProc, Entity, EntityId, EntityKind, EntityState, Found, ImportEntry, Pending,
    PendingState, Resolved, Scope, ScopeId, ScopeKind, UsingEntry,
};
pub use self::module_lookup::filter_hides;
pub(in crate::sema) use self::module_lookup::filter_name;
