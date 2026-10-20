//! Facts for editors. While `Compiler::ed` records, checking records what each identifier of the
//! user's files names and its type, and the extent of block and procedure scopes. The queries
//! below answer hover (`ide_hover`), completion (`ide_visible`, `ide_members`) and member
//! receivers (`ide_receiver`) from those facts and the compiler's scopes.
mod audit;
mod completion;
mod definition;
mod describe;
mod editor;
mod facts;
mod hover;
pub mod interval;
mod member_notes;
pub mod meta;
mod never_returns;
mod recording;
mod sketch;
mod values;

pub use self::editor::Editor;
pub(crate) use self::editor::StmtRecovery;
pub use self::facts::{
    IdeCall, IdeCast, IdeExprFact, IdeFacts, IdeFieldSite, IdeHover, IdeKind, IdeLayout, IdeName,
    IdeReceiver, IdeRef, IdeRoot, IdeWhat, SketchMark,
};
pub(crate) use self::member_notes::for_header_types;
pub(crate) use self::values::{AInfo, Narrowing};
