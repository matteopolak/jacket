//! Shared helpers for the workspace's Rust tests. A dev-dependency only (`publish = false`).
//!
//! - [`tmp`]: per-test scratch directories that clean up after themselves.
//! - [`process`]: run a child process and fail with its output unless it succeeds.
//! - [`toolchain`]: find clang, `wasm-ld`, `node` and MinGW, and skip a test when one is absent.
//! - [`repo`]: paths inside the repository.
//! - [`snapshot`]: [`insta`] snapshots with machine-independent paths.
//! - [`ir`]: assertions over textual LLVM IR.
//! - [`plugin`]: a small metaprogram plugin and a program that uses it.
//! - `compiler` (feature `compiler`): compile helpers over `jaic`.
//! - `backend` (feature `backend`): emit, link and run programs natively and as WebAssembly.
//! - `lsp` (feature `lsp`): language-server sessions and position helpers.
//!
//! The crate depends on `jaic` only behind features, and `jaic` dev-depends on it, so a
//! test inside `jaic`'s own `src/` (which sees a different build of `jaic` than this crate
//! links) uses the feature-less parts only. Integration tests may use every feature.

pub mod ir;
pub mod plugin;
pub mod process;
pub mod repo;
pub mod snapshot;
pub mod tmp;
pub mod toolchain;

#[cfg(feature = "backend")]
pub mod backend;
#[cfg(feature = "compiler")]
pub mod compiler;
#[cfg(feature = "lsp")]
pub mod lsp;

pub use insta;
pub use tmp::TempDir;
