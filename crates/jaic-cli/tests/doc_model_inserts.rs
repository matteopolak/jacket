//! `jaic -doc_model` documents what a module's top-level `#insert` declares.
use std::{path::PathBuf, process::Command};
use test_support::TempDir;

fn stdlib() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")
}

/// The model of the module `name` in a library holding `source` as its `module.jai`.
fn model(tag: &str, source: &str) -> (TempDir, serde_json::Value) {
    let dir = TempDir::new(tag);
    let module = dir.path().join("Gen");
    std::fs::create_dir_all(&module).unwrap();
    std::fs::write(module.join("module.jai"), source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .env("JAIC_STDLIB", stdlib())
        .arg("-import_dir")
        .arg(dir.path())
        .arg("-doc_model")
        .arg(&module)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = serde_json::from_slice(&output.stdout).unwrap();
    (dir, json)
}

const GENERATOR: &str = r#"
Local :: struct { n: int; }

make :: () -> string {
    return "/// A point the generator made.\nPoint :: struct { x: int; at: Local; }\n\n/// Moves a point.\nmove :: (p: *Point) {}\n";
}

#insert #run make();
"#;

#[test]
fn generated_declarations_are_items_marked_generated_with_the_insert_as_origin() {
    let (dir, json) = model("doc-model-generated", GENERATOR);
    let items = json["items"].as_array().unwrap();
    let find = |name: &str| items.iter().find(|i| i["name"] == name).unwrap();
    let point = find("Point");
    assert_eq!(point["generated"], true);
    assert_eq!(point["docs"], "A point the generator made.");
    assert_eq!(find("move")["generated"], true);
    assert!(find("Local").get("generated").is_none());
    let file = point["file"].as_str().unwrap();
    let origin = dir.path().join("Gen/module.jai");
    assert_eq!(
        file,
        format!("{}#insert:8", origin.to_str().unwrap()),
        "{point}"
    );
    assert_eq!(json["unexpanded"].as_array().unwrap().len(), 0, "{json}");
}

#[test]
fn an_insert_that_expands_to_nothing_is_silent() {
    let (_dir, json) = model(
        "doc-model-empty",
        "make :: () -> string { return \"\"; }\n\n#insert #run make();\n\nkept :: () {}\n",
    );
    let items = json["items"].as_array().unwrap();
    assert!(items.iter().any(|i| i["name"] == "kept"));
    assert!(items.iter().all(|i| i.get("generated").is_none()));
    assert_eq!(json["unexpanded"].as_array().unwrap().len(), 0, "{json}");
}

#[test]
fn an_insert_that_fails_to_expand_is_reported_with_the_reason() {
    let (_dir, json) = model(
        "doc-model-failed",
        "make :: () -> string { return missing_name; }\n\n#insert #run make();\n",
    );
    let failed = json["unexpanded"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{json}");
    assert_eq!(failed[0]["line"], 3);
    let reason = failed[0]["reason"].as_str().unwrap();
    assert!(reason.contains("missing_name"), "{reason}");
}
