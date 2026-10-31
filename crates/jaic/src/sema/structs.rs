//! Structs, enums and polymorphic structs: declaration, layout, member
//! access, default initialization and aggregate literals.
mod access;
mod defaults;
mod fields;
mod layout;
mod literal_hints;
pub(in crate::sema) use literal_hints::LitMisfit;
mod literals;
mod members;
mod model;
mod poly;
mod using_members;

pub use self::literals::write_agg;
pub(super) use self::model::{FieldUsing, UsingFilters};
pub use self::model::{MemberAlias, PathStep, PolyStruct, StructSource};
