//! Finds which optional LLVM backends the linked LLVM was built with and sets
//! `llvm_backend = "<name>"` for each, so `machine.rs` registers only backends that link, and
//! sets `JAIC_LLVM_VERSION` to the linked LLVM's version (`jaic_llvm::VERSION`).
//! Official Windows releases, for one, have no `AVR`, `PowerPC` or `SystemZ` backend.

// Cargo's variables for a build script are not the user's settings `jacket_config::env` declares.
#![allow(clippy::disallowed_methods)]

use std::process::Command;

/// The backends jaic can use that some LLVM builds leave out, as `llvm-config --targets-built`
/// names them, and the cfg value each sets.
const OPTIONAL: [(&str, &str); 3] = [
    ("AVR", "avr"),
    ("PowerPC", "powerpc"),
    ("SystemZ", "systemz"),
];

fn main() {
    let values: Vec<String> = OPTIONAL.iter().map(|(_, v)| format!("\"{v}\"")).collect();

    println!(
        "cargo::rustc-check-cfg=cfg(llvm_backend, values({}))",
        values.join(", ")
    );
    // llvm-sys exports the llvm-config it linked against (its `links = "llvm-23"`).
    let Ok(config) = std::env::var("DEP_LLVM_23_CONFIG_PATH") else {
        return;
    };

    println!("cargo::rerun-if-changed={config}");
    let version = Command::new(&config)
        .arg("--version")
        .output()
        .unwrap_or_else(|e| panic!("could not run {config} --version: {e}"));

    assert!(version.status.success(), "{config} --version failed");
    println!(
        "cargo::rustc-env=JAIC_LLVM_VERSION={}",
        String::from_utf8_lossy(&version.stdout).trim()
    );
    let output = Command::new(&config)
        .arg("--targets-built")
        .output()
        .unwrap_or_else(|e| panic!("could not run {config} --targets-built: {e}"));

    assert!(
        output.status.success(),
        "{config} --targets-built failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let built = String::from_utf8_lossy(&output.stdout);

    for (name, value) in OPTIONAL {
        if built.split_whitespace().any(|t| t == name) {
            println!("cargo::rustc-cfg=llvm_backend=\"{value}\"");
        }
    }
}
