//! These tests share the integration suite's gate: jaic must be built beside jacket.
use std::path::{Path, PathBuf};
use std::process::Command;
use test_support::{TempDir, toolchain};

fn tools() -> Option<PathBuf> {
    let jacket = PathBuf::from(env!("CARGO_BIN_EXE_jacket"));
    toolchain::jaic_beside(&jacket)?;
    Some(jacket)
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const SHAPES: &str =
    "/// A point on the plane.\nPoint :: struct {\n    x: float;\n    y: float;\n}\n";

const DRAW: &str = "Shapes :: #import \"Shapes\";\n\n/// Moves `p` to the origin.\nreset :: (p: Shapes.Point) -> Shapes.Point {\n    return .{};\n}\n";

#[test]
fn a_type_of_another_module_of_the_library_links() {
    let Some(jacket) = tools() else {
        return;
    };
    let temp = TempDir::new("doc-import-dir-sibling");
    temp.write("lib/Shapes/module.jai", SHAPES);
    temp.write("lib/Draw/module.jai", DRAW);
    let output = Command::new(jacket)
        .args(["doc", "lib", "--check", "--emit-input"])
        .arg(temp.join("input"))
        .env("JAIC_STDLIB", repo().join("stdlib"))
        .env("XDG_CACHE_HOME", temp.join("cache"))
        .current_dir(&temp)
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{said}");
    let mut pages = String::new();
    for entry in std::fs::read_dir(temp.join("input/modules")).unwrap() {
        pages += &std::fs::read_to_string(entry.unwrap().path()).unwrap();
    }
    assert!(
        pages.contains("/api/Shapes/Point"),
        "no link to Point in {pages}"
    );
}
