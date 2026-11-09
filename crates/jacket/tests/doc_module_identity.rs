//! These tests share the integration suite's gate: jaic must be built beside jacket.
use std::path::{Path, PathBuf};
use std::process::Command;
use test_support::TempDir;

fn tools() -> Option<PathBuf> {
    let jacket = PathBuf::from(env!("CARGO_BIN_EXE_jacket"));
    let jaic = jacket
        .parent()?
        .join(format!("jaic{}", std::env::consts::EXE_SUFFIX));
    if !jaic.is_file() {
        eprintln!("skipped: build `jaic` first (`cargo build -p jaic-cli`)");
        return None;
    }
    Some(jacket)
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The `import` the module page of the first module shows, documenting `args`.
fn imported_as(name: &str, args: &[&str]) -> Option<String> {
    let jacket = tools()?;
    let temp = TempDir::new(name);
    let output = Command::new(jacket)
        .arg("doc")
        .args(args)
        .arg("--check")
        .arg("--emit-input")
        .arg(temp.join("input"))
        .env("JAIC_STDLIB", repo().join("stdlib"))
        .env("XDG_CACHE_HOME", temp.join("cache"))
        .current_dir(&temp)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let page: serde_json::Value =
        serde_json::from_slice(&std::fs::read(temp.join("input/modules/0.json")).unwrap()).unwrap();
    Some(
        page["page"]["content"]["import"]
            .as_str()
            .unwrap()
            .to_owned(),
    )
}

#[test]
fn a_module_documented_by_its_folder_is_imported_by_its_stdlib_path() {
    let folder = repo().join("stdlib/Extensions/Json");
    let Some(import) = imported_as("doc-identity-folder", &[folder.to_str().unwrap()]) else {
        return;
    };
    assert_eq!(import, "Extensions/Json");
}

#[test]
fn a_module_documented_with_others_is_imported_by_its_stdlib_path() {
    let Some(import) = imported_as("doc-identity-filter", &["--module", "Extensions/Json"]) else {
        return;
    };
    assert_eq!(import, "Extensions/Json");
}
