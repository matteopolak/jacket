//! `jacket doc` documents what a module's top-level `#insert` declares and tags it generated.
//! These tests share the integration suite's gate: jaic must be built beside jacket.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use test_support::{TempDir, toolchain};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const MADE: &str = r#"// Things a generator makes.

make :: () -> string {
    return "/// A point the generator made.\nPoint :: struct { x: int; }\n\n/// Moves a point.\nmove :: (p: *Point) {}\n";
}

#insert #run make();
"#;

const QUIET: &str = r#"// Expands to nothing.

nothing :: () -> string { return ""; }

#insert #run nothing();
"#;

const BROKEN: &str = r"// Cannot be expanded.

broken :: () -> string { return missing_name; }

#insert #run broken();
";

const BARE: &str = r#"// Generates a struct without docs.

make :: () -> string { return "Bare :: struct { x: int; }\n"; }

#insert #run make();
"#;

fn run(jacket: &Path, temp: &TempDir, extra: &[&str]) -> Output {
    Command::new(jacket)
        .arg("doc")
        .args(extra)
        .env("JAIC_STDLIB", repo().join("stdlib"))
        .env("XDG_CACHE_HOME", temp.join("cache"))
        .current_dir(temp)
        .output()
        .unwrap()
}

#[test]
fn generated_items_are_documented_tagged_and_searchable() {
    let Some(jacket) = Some(PathBuf::from(env!("CARGO_BIN_EXE_jacket")))
        .filter(|jacket| toolchain::jaic_beside(jacket).is_some())
    else {
        return;
    };
    let temp = TempDir::new("doc-generated");
    temp.write("jacket.toml", "[doc]\npath = 'lib'\nname = 'Lib'\n");
    temp.write("lib/Made/module.jai", MADE);
    temp.write("lib/Quiet/module.jai", QUIET);
    temp.write("lib/Broken/module.jai", BROKEN);
    let output = run(&jacket, &temp, &["--out", "site"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    // Only the insert that fails is a warning, and it says why.
    let warnings: Vec<&str> = stderr.lines().filter(|l| l.contains("#insert")).collect();
    assert_eq!(warnings.len(), 1, "{stderr}");
    assert!(
        warnings[0].contains("Broken: what the #insert at") && warnings[0].contains("missing_name"),
        "{stderr}"
    );
    let read = |path: &str| std::fs::read_to_string(temp.join("site").join(path)).unwrap();
    // The page of a generated item carries its docs and the tag; so does its row.
    let page = read("api/Made/Point.html");
    assert!(page.contains("A point the generator made."), "{page}");
    assert!(page.contains("<span class=\"badge generated\""), "{page}");
    assert!(read("api/Made/move.html").contains("Moves a point."));
    let module = read("api/Made.html");
    assert!(module.contains("badge sm generated"), "{module}");
    assert!(module.contains("href=\"/api/Made/Point\""), "{module}");
    // The item the module writes itself is not tagged.
    let make = read("api/Made/make.html");
    assert!(!make.contains("badge generated"), "{make}");
    // Search flags it (bit 4 of the last column).
    let index: serde_json::Value = serde_json::from_str(&read("search-index.json")).unwrap();
    let flags = |name: &str| {
        index["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row[1] == name)
            .map(|row| row[6].as_u64().unwrap())
    };
    assert_eq!(flags("Point").map(|f| f & 4), Some(4));
    assert_eq!(flags("move").map(|f| f & 4), Some(4));
    assert_eq!(flags("make").map(|f| f & 4), Some(0));

    // A generated item with no docs fails the check and says where the docs go.
    temp.write("lib/Bare/module.jai", BARE);
    let output = run(&jacket, &temp, &["--check"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(
        stderr.contains("Bare.Bare has no docs") && stderr.contains("the text the generator emits"),
        "{stderr}"
    );
}
