//! Pointer-free WebAssembly boundary for the browser playground: a `jaic` compile-and-run entry
//! point (`jai_play_*`) and the shared language server (`jai_lsp_*`).
// Rust marks symbol-export attributes unsafe. The exported functions themselves
// are safe and exchange checked scalar bytes rather than dereferencing pointers.
#[allow(unsafe_code)]
mod language_server;
pub use language_server::*;

/// Playground run through the `jaic` core (interpreter backend) with the bundled stdlib.
pub mod play;
#[allow(unsafe_code)]
mod play_exports;
pub use play_exports::*;
