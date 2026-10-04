//! Jai compiler core.
//!
//! Pipeline: `lexer` → `parser` (→ `ast`) → `sema` (demand-driven typing into
//! the checked tree) → `ir` (lowered procedures) → `interp` (compile-time
//! execution, scripting, browser) or the LLVM backend in `jaic-llvm`.
pub mod abi;
pub mod ast;
pub mod build;
pub mod clang;
pub mod intern;
pub mod interp;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod records;
pub mod sema;
pub mod source;
pub mod stack_trace;
pub mod types;
