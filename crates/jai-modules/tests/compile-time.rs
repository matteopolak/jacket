use jai_modules::{GraphOptions, ModuleGraph};
use jai_syntax::CompileTimeBody;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn active_run_requests_keep_the_loaded_file_instance() {
    let root = std::env::temp_dir().join(format!(
        "jai-run-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("main.jai"),
        "#load \"helper.jai\"; #if false { #run absent(); } #run build(); main :: () {}",
    )
    .unwrap();
    std::fs::write(
        root.join("helper.jai"),
        "#scope_file; build :: () {} #run { build(); }",
    )
    .unwrap();
    let graph = ModuleGraph::load(&root.join("main.jai"), GraphOptions::default()).unwrap();
    assert_eq!(graph.runs().len(), 2);
    for request in graph.runs() {
        assert_eq!(
            graph.file(request.file).unwrap().source(),
            request.syntax.location.source
        );
    }
    assert!(
        graph
            .runs()
            .iter()
            .any(|request| matches!(request.syntax.body, CompileTimeBody::Block(_)))
    );
    assert!(
        graph
            .runs()
            .iter()
            .any(|request| matches!(request.syntax.body, CompileTimeBody::Expression(_)))
    );
    std::fs::remove_dir_all(root).unwrap();
}
