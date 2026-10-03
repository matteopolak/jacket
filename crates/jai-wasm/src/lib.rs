//! Pointer-free WebAssembly boundary for the same LLVM-independent scripting engine.
mod bridge;
// Rust marks symbol-export attributes unsafe. The exported functions themselves
// are safe and exchange checked scalar bytes rather than dereferencing pointers.
#[allow(unsafe_code)]
mod exports;
pub use exports::*;

#[allow(unsafe_code)]
mod language_server;
pub use language_server::*;
