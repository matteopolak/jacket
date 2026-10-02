use std::fs;

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-modules-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Self(root)
    }
    fn graph(&self) -> Result<ModuleGraph, GraphError> {
        ModuleGraph::load(
            &self.0.join("main.jai"),
            GraphOptions {
                import_dirs: vec![self.0.join("modules")],
            },
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn lookup(
    graph: &ModuleGraph,
    file: FileInstanceId,
    names: &[&str],
) -> Result<Binding, LookupError> {
    let symbols: Vec<_> = names
        .iter()
        .map(|name| graph.symbols().find(name).unwrap())
        .collect();
    graph.lookup(
        file,
        &NamePath {
            root: symbols[0],
            members: symbols[1..].to_vec(),
        },
    )
}
fn declaration(graph: &ModuleGraph, binding: Binding) -> &Declaration {
    let Binding::Declaration(id) = binding else {
        panic!("expected declaration")
    };
    graph.declaration(id).unwrap()
}
fn root_file(graph: &ModuleGraph) -> FileInstanceId {
    graph.module(graph.root()).unwrap().entry()
}
#[test]
fn modules_are_isolated_and_qualification_filters_exports() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "app_only :: 90; Lib :: #import \"Lib\"; main :: () -> int { return Lib.answer(); }",
        ),
        (
            "modules/Lib.jai",
            "answer :: () -> int { return hidden; } #scope_module hidden :: 42;",
        ),
    ]);
    let graph = f.graph().unwrap();
    let app = root_file(&graph);
    let Binding::Module(lib) = lookup(&graph, app, &["Lib"]).unwrap() else {
        panic!()
    };
    let lib_file = graph.module(lib).unwrap().entry();
    assert!(matches!(
        lookup(&graph, lib_file, &["app_only"]),
        Err(LookupError::UnknownName(_))
    ));
    assert!(lookup(&graph, lib_file, &["hidden"]).is_ok());
    assert!(matches!(
        lookup(&graph, app, &["Lib", "hidden"]),
        Err(LookupError::PrivateMember { .. })
    ));
    assert_eq!(
        declaration(&graph, lookup(&graph, app, &["Lib", "answer"]).unwrap()).file(),
        lib_file
    );
    assert_eq!(graph.modules().len(), 2);
}
#[test]
fn loads_create_independent_file_scopes_and_share_module_exports() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "#scope_file private :: 1; #load \"a.jai\"; #load \"b.jai\"; #load \"./a.jai\"; main :: () {}",
        ),
        ("a.jai", "exported_a :: 10; #scope_file private :: 2;"),
        ("b.jai", "exported_b :: 20; #scope_file private :: 3;"),
    ]);
    let graph = f.graph().unwrap();
    assert_eq!(graph.files().len(), 3);
    assert_eq!(graph.loads().len(), 3);
    assert_eq!(graph.loads()[0].target(), graph.loads()[2].target());
    let files = graph.module(graph.root()).unwrap().files();
    let bindings: Vec<_> = files
        .iter()
        .map(|&file| lookup(&graph, file, &["private"]).unwrap())
        .collect();
    assert_ne!(bindings[0], bindings[1]);
    assert_ne!(bindings[1], bindings[2]);
    for &file in files {
        assert!(lookup(&graph, file, &["exported_a"]).is_ok());
        assert!(lookup(&graph, file, &["exported_b"]).is_ok());
    }
    assert!(
        !graph
            .module(graph.root())
            .unwrap()
            .exports()
            .contains_key(&graph.symbols().find("private").unwrap())
    );
}
#[test]
fn anonymous_alias_using_and_reexports_preserve_declaration_identity() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "#import \"Wrapper\"; First :: #import \"Leaf\"; Second :: #import \"Leaf\"; using Also :: #import \"Leaf\"; main :: () {}",
        ),
        (
            "modules/Wrapper/module.jai",
            "#import \"Leaf\"; Alias :: #import \"Leaf\";",
        ),
        (
            "modules/Leaf.jai",
            "value :: 42; #scope_module hidden :: 9;",
        ),
    ]);
    let graph = f.graph().unwrap();
    let file = root_file(&graph);
    let value = lookup(&graph, file, &["value"]).unwrap();
    assert_eq!(value, lookup(&graph, file, &["First", "value"]).unwrap());
    assert_eq!(value, lookup(&graph, file, &["Second", "value"]).unwrap());
    assert_eq!(value, lookup(&graph, file, &["Alias", "value"]).unwrap());
    assert_eq!(
        lookup(&graph, file, &["First"]).unwrap(),
        lookup(&graph, file, &["Second"]).unwrap()
    );
    assert_eq!(graph.modules().len(), 3);
    assert!(matches!(
        lookup(&graph, file, &["hidden"]),
        Err(LookupError::UnknownName(_))
    ));
}
#[test]
fn file_private_import_does_not_leak_and_same_source_has_separate_instances() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "#scope_file #import \"Leaf\"; #load \"other.jai\"; Local :: #import,file \"shared.jai\"; #load \"shared.jai\"; main :: () {}",
        ),
        ("other.jai", "other :: 3;"),
        ("shared.jai", "shared :: 2;"),
        ("modules/Leaf.jai", "value :: 42;"),
    ]);
    let graph = f.graph().unwrap();
    let root = root_file(&graph);
    let other = graph
        .files()
        .iter()
        .find(|file| {
            graph
                .sources()
                .get(file.source())
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                == "other.jai"
        })
        .unwrap();
    assert!(lookup(&graph, root, &["value"]).is_ok());
    assert!(matches!(
        lookup(&graph, other.id(), &["value"]),
        Err(LookupError::UnknownName(_))
    ));
    let app = declaration(&graph, lookup(&graph, root, &["shared"]).unwrap());
    let imported = declaration(&graph, lookup(&graph, root, &["Local", "shared"]).unwrap());
    assert_ne!(app.id(), imported.id());
    assert_ne!(app.file(), imported.file());
    assert_eq!(
        graph.file(app.file()).unwrap().source(),
        graph.file(imported.file()).unwrap().source()
    );
}
#[test]
fn file_directory_imports_and_exported_loads() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "One :: #import,file \"local/entry.jai\"; Two :: #import,dir \"local\"; main :: () {}",
        ),
        ("local/entry.jai", "value :: 1;"),
        ("local/module.jai", "#scope_file #load \"loaded.jai\";"),
        ("local/loaded.jai", "value :: 2;"),
    ]);
    let graph = f.graph().unwrap();
    let root = root_file(&graph);
    let one = declaration(&graph, lookup(&graph, root, &["One", "value"]).unwrap());
    let two = declaration(&graph, lookup(&graph, root, &["Two", "value"]).unwrap());
    assert_ne!(one.id(), two.id());
    assert_eq!(
        graph
            .sources()
            .get(two.location().source)
            .unwrap()
            .path()
            .file_name()
            .unwrap(),
        "loaded.jai"
    );
}
#[test]
fn collisions_cycles_and_unsupported_forms_fail_explicitly() {
    let collision = Fixture::new(&[
        ("main.jai", "value :: 1;\n#import \"Leaf\";"),
        ("modules/Leaf.jai", "value :: 2;"),
    ])
    .graph()
    .unwrap_err();
    assert!(
        matches!(&collision, GraphError::Located { diagnostic, .. } if diagnostic.location.span.start > 0)
    );
    assert!(collision.to_string().contains("main.jai:2:"));
    for (files, kind) in [
        (
            vec![("main.jai", "#load \"main.jai\";")],
            DependencyKind::Load,
        ),
        (
            vec![
                ("main.jai", "#import \"A\";"),
                ("modules/A.jai", "#import \"B\";"),
                ("modules/B.jai", "#import \"A\";"),
            ],
            DependencyKind::Import,
        ),
    ] {
        let error = Fixture::new(&files).graph().unwrap_err();
        assert!(error.to_string().contains(":1:"), "{error}");
        assert!(
            matches!(error, GraphError::Cycle { kind: actual, location: Some(_), .. } if actual == kind)
        );
    }
    for source in [
        "#import \"Leaf\"();",
        "#import \"Leaf\"()(DEBUG=true);",
        "#module_parameters(DEBUG := false);",
        "#import,string \"value :: 1;\";",
        "#if true { #load \"other.jai\"; }",
    ] {
        assert!(
            Fixture::new(&[("main.jai", source), ("modules/Leaf.jai", "value :: 1;")])
                .graph()
                .is_err(),
            "{source}"
        );
    }
}
#[test]
fn imported_file_parse_errors_keep_original_source_path() {
    let error = Fixture::new(&[
        ("main.jai", "#import \"Bad\";"),
        ("modules/Bad.jai", "\nbad :: () -> int { return +; }"),
    ])
    .graph()
    .unwrap_err();
    assert!(matches!(error, GraphError::Located { .. }));
    assert!(error.to_string().contains("Bad.jai:2:"), "{error}");
}
