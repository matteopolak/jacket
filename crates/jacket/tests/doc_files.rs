//! `[doc] files`: arbitrary files published at the site root. These tests share the integration
//! suite's gate: jaic must be built beside jacket.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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

/// Documents a one-module library whose `[doc]` lists `files`, into `site/`.
fn document(temp: &TempDir, files: &str) -> Option<Output> {
    let jacket = tools()?;
    std::fs::create_dir_all(temp.join("lib")).unwrap();
    std::fs::write(
        temp.join("lib/module.jai"),
        "// Says hello.\nhello :: () {}\n",
    )
    .unwrap();
    std::fs::create_dir_all(temp.join("scripts")).unwrap();
    std::fs::write(temp.join("scripts/install.sh"), "#!/bin/sh\necho hi\n").unwrap();
    std::fs::write(
        temp.join("jacket.toml"),
        format!("[doc]\npath = \"lib\"\nfiles = {files}\n"),
    )
    .unwrap();
    let stdlib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Some(
        Command::new(jacket)
            .args(["doc", "--no-test-examples", "--out", "site"])
            .env("JAIC_STDLIB", stdlib)
            .env("XDG_CACHE_HOME", temp.join("cache"))
            .current_dir(temp)
            .output()
            .unwrap(),
    )
}

#[test]
fn listed_files_are_published_at_the_site_root_under_their_names() {
    let temp = TempDir::new("doc-files-copied");
    let Some(output) = document(&temp, "[\"scripts/install.sh\"]") else {
        return;
    };
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(temp.join("site/install.sh")).unwrap(),
        "#!/bin/sh\necho hi\n"
    );
}

#[test]
fn a_file_named_like_a_generated_page_is_an_error() {
    let temp = TempDir::new("doc-files-clash");
    std::fs::create_dir_all(temp.join("extra")).unwrap();
    std::fs::write(temp.join("extra/index.html"), "mine").unwrap();
    let Some(output) = document(&temp, "[\"extra/index.html\"]") else {
        return;
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("index.html"), "{stderr}");
}

#[test]
fn a_missing_file_is_an_error() {
    let temp = TempDir::new("doc-files-missing");
    let Some(output) = document(&temp, "[\"scripts/nope.sh\"]") else {
        return;
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("nope.sh"), "{stderr}");
}
