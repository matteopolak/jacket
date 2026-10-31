//! Records two things at build time.
//!
//! `LLVM_PREFIX_AT_BUILD`: the LLVM install prefix jaic is built against, so the compiler can find
//! `clang` and `wasm-ld` there at run time (`jacket_config::llvm`).
//!
//! `BACKEND_STAMP_AT_BUILD`: what the driver and the code generation backend must agree on to share an
//! `ir::Program` in memory (`src/backend.rs`): the crate version, the compiler that built them and
//! a hash of the sources that define the shared types.

// A build script reads cargo's variables and the build environment's, not the tools' own.
#![allow(clippy::disallowed_methods)]

// The one place the LLVM version is spelled; shared so the variable name is never typed twice.
#[path = "../jacket-config/src/llvm/version.rs"]
mod version;

use std::path::PathBuf;

/// FNV-1a over the bytes: no dependency, and the same on every machine.
fn fnv(hash: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *hash ^= u64::from(b);
        *hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
}

/// The sources whose types cross to the backend: the IR, targets and ABI of `jaic`, and this
/// crate's options and loader.
fn stamp() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let mut files = vec![
        dir.join("src/options.rs"),
        dir.join("src/backend.rs"),
        dir.join("../jaic/src/ir.rs"),
        dir.join("../jaic/src/target.rs"),
        dir.join("../jaic/src/abi.rs"),
    ];
    if let Ok(entries) = std::fs::read_dir(dir.join("../jaic/src/ir")) {
        let mut ir: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        ir.sort();
        files.extend(ir);
    }
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
        fnv(&mut hash, &std::fs::read(file).unwrap_or_default());
    }
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let version = std::process::Command::new(rustc)
        .arg("-vV")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    fnv(&mut hash, version.as_bytes());
    let release = version
        .lines()
        .find_map(|l| l.strip_prefix("release: "))
        .unwrap_or("unknown");
    let package = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    println!("cargo:rustc-env=BACKEND_STAMP_AT_BUILD={package}/rustc-{release}/{hash:016x}");
}

fn main() {
    stamp();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed={}", version::SYS_PREFIX_VAR);
    println!("cargo:rerun-if-env-changed=LLVM_PREFIX");
    let prefix = [version::SYS_PREFIX_VAR, "LLVM_PREFIX"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()));
    if let Some(prefix) = prefix {
        println!("cargo:rustc-env=LLVM_PREFIX_AT_BUILD={prefix}");
    }
}
