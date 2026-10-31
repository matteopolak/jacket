//! Jai compiler core.
//!
//! Pipeline: `lexer` → `parser` (→ `ast`) → `sema` (demand-driven typing into
//! the checked tree) → `ir` (lowered procedures) → `interp` (compile-time
//! execution, scripting, browser) or the LLVM backend in `jaic-llvm`.

// What the compiler prints to stderr takes the build status line down first (`progress`).
macro_rules! eprintln {
    ($($arg:tt)*) => {{
        $crate::progress::before_output();
        ::std::eprintln!($($arg)*)
    }};
}

pub mod abi;
pub mod build;
pub mod builtin_keys;
pub mod fiber;
pub mod flags;
pub mod format_string;
pub mod ide;
pub mod int_arith;
pub mod interp;
pub mod interpolate;
pub mod ir;
pub mod lang_items;
pub mod layout;
pub mod memory_limit;
pub mod parse_ahead;
pub mod platform;
pub mod pool;
pub mod procedures;
pub mod progress;
pub mod project;
pub mod record_build;
pub mod records;
pub mod sema;
pub mod service;
pub mod stack_trace;
pub mod target;
pub mod toolchain;
pub mod types;
pub mod wide_float;

// The front of the compiler lives in `jaic-syntax`; its modules and functions are re-exported at
// this crate's root, so the rest of the compiler (and every downstream crate) names them as before.
pub use jaic_syntax::*;
