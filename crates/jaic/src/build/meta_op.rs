//! The primitives a metaprogram can call on the compiler, and the version it reports.

use crate::macros::raw_enum;

raw_enum! {
/// The `#compiler` primitives the stdlib declares itself, bound by name in `stdlib/Compiler`.
/// Every other compiler operation is a public `Compiler` procedure (`public_api::Operation`).
pub enum MetaOp by name {
    SyntaxInt = "__jaic_syntax_int",
    SyntaxString = "__jaic_syntax_string",
    SyntaxChild = "__jaic_syntax_child",
}
}

/// Id of the program being compiled by the command line (workspace 1 is reserved, as in `jai`).
pub const TOP_LEVEL_WORKSPACE: i64 = 2;

/// Jai language version reported to programs, as major / minor / micro. Build scripts check it
/// against a supported range, so it stays in the 0.2 line and comes first in the version text.
pub const LANGUAGE_VERSION: (u32, u32, u32) = (0, 2, 99);

/// Version text returned by `compiler_get_version_info`: the language version, whose three
/// integers fill `Version_Info`, then the Jacket product version.
pub fn compiler_version() -> String {
    let (major, minor, micro) = LANGUAGE_VERSION;
    format!(
        "Jai {major}.{minor}.{micro} (Jacket {})",
        env!("CARGO_PKG_VERSION")
    )
}
