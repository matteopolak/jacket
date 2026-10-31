//! Records the LLVM install prefix jaic is built against as `LLVM_PREFIX_AT_BUILD`, so the
//! compiler can find `clang` and `wasm-ld` there at run time (`jacket_config::llvm`).

// A build script reads cargo's variables and the build environment's, not the tools' own.
#![allow(clippy::disallowed_methods)]

// The one place the LLVM version is spelled; shared so the variable name is never typed twice.
#[path = "../jacket-config/src/llvm/version.rs"]
mod version;

fn main() {
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
