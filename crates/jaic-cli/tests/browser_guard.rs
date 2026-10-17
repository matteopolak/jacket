//! Public module target guards remain strict despite the playground's private scheduler provider.
use std::{path::PathBuf, process::Command};

#[test]
fn public_thread_rejects_wasm_in_cli_source_checking() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/semantics/browser-thread-target-rejection.jai");
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(source)
        .args(["-target", "wasm64-unknown-wasi", "-check"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("OS") && stderr.contains("#assert failed"),
        "{stderr}"
    );
}
