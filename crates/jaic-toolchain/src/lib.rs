//! Finding what a build needs from the machine: the linker, archiver, sysroot, builtins and
//! runner for a target, and the one note that says how to get a missing piece.
//!
//! See `docs/native/toolchain-discovery.md`. Every lookup reads the machine through [`Host`], so
//! tests describe a machine with [`testing::FakeHost`] instead of needing one.
pub mod builtins;
pub mod cache;
pub mod found;
pub mod host;
pub mod linker;
pub mod lookup;
pub mod names;
pub mod overrides;
pub mod package;
pub mod pins;
pub mod testing;

pub use found::{Found, Missing, Origin, tilde};
pub use host::{Host, Os, System};
pub use lookup::Lookup;
pub use overrides::Overrides;
