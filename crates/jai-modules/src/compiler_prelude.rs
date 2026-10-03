//! Independently authored source contracts for the compiler's bootstrap module.
//!
//! The physical entry uses ordinary `#load` files. Overlay consumers use the
//! composed form below to retain one source snapshot and the same declarations.

/// Repository-relative entry for explicitly selecting the authored prelude.
pub const COMPILER_PRELUDE_ENTRY: &str = "stdlib/prelude/module.jai";

/// Complete source-defined bootstrap, without filesystem or standard library IO.
pub fn compiler_prelude_source() -> &'static str {
    concat!(
        include_str!("../../../prelude/platform.jai"),
        "\n",
        include_str!("../../../prelude/reflection.jai"),
        "\n",
        include_str!("../../../prelude/allocation.jai"),
        "\n",
        include_str!("../../../prelude/diagnostics.jai"),
        "\n",
        include_str!("../../../prelude/runtime-storage.jai"),
        "\n",
        include_str!("../../../prelude/intrinsics.jai"),
        "\n",
        include_str!("../../../prelude/context.jai"),
    )
}
