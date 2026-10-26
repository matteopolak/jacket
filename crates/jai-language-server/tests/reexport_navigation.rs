//! Names a module passes on with `using Name :: #import "X"` resolve through it for hover and
//! go to definition, as for the module's own names.
mod common;
use common::{at, file_uri, virtual_session};

#[test]
fn names_a_module_reexports_resolve_through_it() {
    let uri = file_uri("/reexport-nav/main.jai");
    let inner = "// Adds one.\nsucc :: (x: int) -> int { return x + 1; }\n";
    let source = "F :: #import \"Facade\";\nmain :: () {\n    y := F.succ(1);\n}\n";
    let mut s = virtual_session(
        &[
            (
                "/reexport-nav/modules/Facade/module.jai",
                "using Inner :: #import \"internal/inner\";\n",
            ),
            ("/reexport-nav/modules/internal/inner/module.jai", inner),
        ],
        &["/reexport-nav/modules"],
        None,
    );
    s.open(uri.clone(), 1, source.into()).unwrap();
    let found = s.definition(&uri, at(source, "succ(1)")).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].uri.ends_with("internal/inner/module.jai"),
        "{found:?}"
    );
    let hover = s.hover(&uri, at(source, "succ(1)")).unwrap().unwrap();
    assert!(format!("{hover:?}").contains("succ"), "{hover:?}");
}
