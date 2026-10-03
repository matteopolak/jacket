//! Deferred weak-float errors retain the source that owns the failing arithmetic.
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn deferred_integer_condition_error_points_to_the_constant_definition() {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-weak-float-origin-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(root);
    std::fs::create_dir_all(&scratch.0).unwrap();
    let definition = scratch.0.join("definition.jai");
    std::fs::write(&definition, "BAD :: ifx 1 / 0 == 0 then 1.0 else 2.0;").unwrap();
    std::fs::write(
        scratch.0.join("alias.jai"),
        "#load \"definition.jai\"; ALIAS :: BAD + 0.0;",
    )
    .unwrap();
    let main = scratch.0.join("main.jai");
    std::fs::write(
        &main,
        "#load \"alias.jai\"; main :: ()->int { x:float64=ALIAS; return 0; }",
    )
    .unwrap();
    let graph = jai_modules::ModuleGraph::load(&main, Default::default()).unwrap();
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    let actual = graph.sources().get(error.location.source).unwrap();
    assert!(error.message.contains("zero divisor"), "{error}");
    assert_eq!(actual.path(), std::fs::canonicalize(&definition).unwrap());

    std::fs::write(&definition, "BIG :: 1e39;").unwrap();
    let materializing_source =
        "#load \"definition.jai\"; main :: ()->int { x:float32=BIG; return 0; }";
    std::fs::write(&main, materializing_source).unwrap();
    let graph = jai_modules::ModuleGraph::load(&main, Default::default()).unwrap();
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    let actual = graph.sources().get(error.location.source).unwrap();
    assert_eq!(actual.path(), std::fs::canonicalize(main).unwrap());
    assert_eq!(
        error.location.span.text(materializing_source),
        "x:float32=BIG;"
    );
}
