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

/// `jacket doc lib --check --emit-input input`, then every module page it wrote.
fn pages(jacket: &Path, temp: &TempDir, extra: &[&str]) -> String {
    let output = Command::new(jacket)
        .args(["doc", "lib", "--check", "--emit-input"])
        .arg(temp.join("input"))
        .args(extra)
        .env("JAIC_STDLIB", repo().join("stdlib"))
        .env("XDG_CACHE_HOME", temp.join("cache"))
        .current_dir(temp)
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
    pages
}

#[test]
fn a_type_of_another_module_of_the_library_links() {
    let Some(jacket) = tools() else {
        return;
    };
    let temp = TempDir::new("doc-import-dir-sibling");
    temp.write("lib/Shapes/module.jai", SHAPES);
    temp.write("lib/Draw/module.jai", DRAW);
    let pages = pages(&jacket, &temp, &[]);
    assert!(
        pages.contains("/api/Shapes/Point"),
        "no link to Point in {pages}"
    );
}

#[test]
fn flags_after_the_dashes_add_import_directories() {
    let Some(jacket) = tools() else {
        return;
    };
    let temp = TempDir::new("doc-import-dir-flag");
    temp.write(
        "vendor/Shapes/module.jai",
        "/// Makes a point.\nmake :: () {}\n",
    );
    temp.write(
        "lib/Draw/module.jai",
        "Shapes :: #import \"Shapes\";\n\nmake :: Shapes.make;\n",
    );
    // `Draw.make` takes the docs of the `Shapes` module that only the flag lets it find.
    assert!(!pages(&jacket, &temp, &[]).contains("Makes a point."));
    let pages = pages(&jacket, &temp, &["--", "-import_dir", "vendor"]);
    assert!(pages.contains("Makes a point."), "{pages}");
}
