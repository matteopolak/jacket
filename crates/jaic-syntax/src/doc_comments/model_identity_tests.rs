use super::{library, print};
use crate::doc_comments::{module_key, stdlib_module_key};

#[test]
fn compiler_doc_key_matches_model_links_for_nested_and_external_modules() {
    let dir = library(
        "doc-key-authority",
        &[
            (
                "stdlib/Extensions/Widget/module.jai",
                "// See [Thing].\n\nThing :: 1;\n",
            ),
            (
                "project/nested/Widget/module.jai",
                "// See [Thing].\n\nThing :: 1;\n",
            ),
        ],
    );
    let stdlib = dir.join("stdlib");
    for (target, expected) in [
        (
            stdlib.join("Extensions/../Extensions/Widget"),
            "Extensions/Widget",
        ),
        (dir.join("project/nested/Widget"), "Widget"),
    ] {
        let key = module_key(&target, &stdlib);
        assert_eq!(key, expected);
        assert_eq!(
            stdlib_module_key(&target, &stdlib).as_deref(),
            (expected == "Extensions/Widget").then_some(expected)
        );
        assert!(print(&target, &stdlib).contains(&format!("jaidoc:{key}/Thing")));
    }
}

#[cfg(unix)]
#[test]
fn compiler_doc_key_canonicalizes_symlinked_library_roots() {
    let dir = library(
        "doc-key-symlink",
        &[(
            "stdlib/Extensions/Widget/module.jai",
            "// See [Thing].\n\nThing :: 1;\n",
        )],
    );
    std::os::unix::fs::symlink(dir.join("stdlib"), dir.join("linked-stdlib")).unwrap();
    let target = dir.join("linked-stdlib/Extensions/Widget");
    assert_eq!(
        stdlib_module_key(&target, &dir.join("stdlib")).as_deref(),
        Some("Extensions/Widget")
    );
    assert_eq!(
        module_key(&target, &dir.join("stdlib")),
        "Extensions/Widget"
    );
    assert_eq!(
        print(&target, &dir.join("linked-stdlib")),
        print(&dir.join("stdlib/Extensions/Widget"), &dir.join("stdlib"))
    );
}
