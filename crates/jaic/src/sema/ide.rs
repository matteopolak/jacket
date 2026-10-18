//! Facts for editors. With `Compiler::ide` set, checking records what each identifier of the
//! user's files names and its type, and the extent of block and procedure scopes. The queries
//! below answer hover (`ide_hover`), completion (`ide_visible`, `ide_members`) and member
//! receivers (`ide_receiver`) from those facts and the compiler's scopes.
mod audit;
#[cfg(test)]
mod body_failure_tests;
mod completion;
mod definition;
mod describe;
mod facts;
mod hover;
mod member_notes;
mod recording;

pub use self::facts::{
    IdeCast, IdeExprFact, IdeFacts, IdeFieldSite, IdeHover, IdeKind, IdeLayout, IdeName,
    IdeReceiver, IdeRef, IdeRoot, IdeWhat, SketchMark,
};
pub(super) use self::member_notes::for_header_types;
