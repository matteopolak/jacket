//! The LLVM version jaic is built against: the one place Rust spells it.
//!
//! `.github/pins.json` holds the full version for CI and the Python tools; a test in the parent
//! module keeps this in step. The `llvm23-1` features in `Cargo.toml` and
//! `crates/jaic-llvm/Cargo.toml` name it too. This file is also included by the `jaic-llvm`
//! build script, so it must stand alone.
#![allow(dead_code)]

macro_rules! version {
    (major) => {
        "23"
    };
    (minor) => {
        "1"
    };
}

/// The LLVM major version, as it appears in `llvm-config-<major>` and `/usr/lib/llvm-<major>`.
pub const MAJOR: &str = version!(major);

/// The variable `llvm-sys` reads for the install prefix (`LLVM_SYS_<major><minor>_PREFIX`).
pub const SYS_PREFIX_VAR: &str = concat!("LLVM_SYS_", version!(major), version!(minor), "_PREFIX");
