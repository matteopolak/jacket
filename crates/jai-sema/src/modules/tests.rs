use super::*;
use jai_modules::GraphOptions;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-scoped-sema-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        for (path, text) in files {
            let path = directory.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Self(directory)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(
            &self.0.join("main.jai"),
            GraphOptions {
                import_dirs: vec![self.0.join("modules")],
            },
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn returned_integer(procedure: &Procedure) -> &IntExpr {
    let Statement::Exit(exit) = procedure.body.statements.last().unwrap() else {
        panic!("expected return")
    };
    let Transfer::ReturnInt(value) = &exit.transfer else {
        panic!("expected integer return")
    };
    value
}
#[test]
fn same_spelling_has_distinct_declaration_storage_and_procedure_identity() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "A :: #import \"A\"; B :: #import \"B\"; value := A.VALUE + B.VALUE; main :: () -> int { return A.answer() + B.answer(); }",
        ),
        (
            "modules/A.jai",
            "VALUE :: 10; value := VALUE; answer :: () -> int { return value; }",
        ),
        (
            "modules/B.jai",
            "VALUE :: 20; value := VALUE; answer :: () -> int { return value; }",
        ),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    let initializers: Vec<_> = program
        .globals()
        .iter()
        .map(|global| match global.initializer() {
            GlobalInitializer::Int(value) => value.value(),
            GlobalInitializer::Bool(_) => panic!("expected integer global"),
        })
        .collect();
    assert_eq!(initializers, vec![30, 10, 20]);
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let IntExprKind::Binary(_, left, right) =
        returned_integer(&program.procedures()[main.index()]).kind()
    else {
        panic!()
    };
    let (IntExprKind::Call(left), IntExprKind::Call(right)) = (left.kind(), right.kind()) else {
        panic!()
    };
    assert_ne!(left.procedure, right.procedure);
    for call in [left, right] {
        let IntExprKind::Load(place) =
            returned_integer(&program.procedures()[call.procedure.index()]).kind()
        else {
            panic!()
        };
        assert!(matches!(place.place().kind(), PlaceKind::Global(_)));
    }
}
#[test]
fn cross_module_constant_dependencies_and_defaults_use_defining_file() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Math :: #import \"Math\"; VALUE :: 99; result := Math.TOTAL; main :: () -> int { return Math.answer(); }",
        ),
        (
            "modules/Math.jai",
            "Other :: #import \"Other\"; VALUE :: 5; TOTAL :: Other.VALUE + VALUE; answer :: (amount := TOTAL) -> int { return amount; }",
        ),
        ("modules/Other.jai", "VALUE :: 37;"),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    assert!(
        matches!(program.globals()[0].initializer(), GlobalInitializer::Int(value) if value.value() == 42)
    );
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let IntExprKind::Call(call) = returned_integer(&program.procedures()[main.index()]).kind()
    else {
        panic!()
    };
    let ValueExpr::Int(argument) = &call.arguments[0].1 else {
        panic!()
    };
    assert!(matches!(argument.kind(), IntExprKind::Constant(value) if value.value() == 42));
}
#[test]
fn block_constants_resolve_module_paths_and_forward_lexical_dependencies() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Library :: #import \"Library\"; main :: () -> int { return Library.answer(); }",
        ),
        (
            "modules/Library.jai",
            "Other :: #import \"Other\"; VALUE :: 5; answer :: () -> int { LOCAL :: VALUE + Other.VALUE + LATER; LATER :: 1; return LOCAL; }",
        ),
        ("modules/Other.jai", "VALUE :: 36;"),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    let answer = program.procedures().iter().find(|procedure| matches!(returned_integer(procedure).kind(), IntExprKind::Constant(value) if value.value() == 42)).unwrap();
    assert!(answer.parameters.is_empty());
}
#[test]
fn imported_libraries_need_no_main_and_cannot_supply_application_main() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Library :: #import \"Library\"; main :: () -> int { return Library.answer(); }",
        ),
        ("modules/Library.jai", "answer :: () -> int { return 42; }"),
    ]);
    assert!(resolve_graph(&fixture.graph()).is_ok());
    let fixture = Fixture::new(&[
        ("main.jai", "#import \"Library\";"),
        ("modules/Library.jai", "main :: () -> int { return 42; }"),
    ]);
    let graph = fixture.graph();
    assert!(resolve_library(&graph).is_ok());
    assert!(
        resolve_graph(&graph)
            .unwrap_err()
            .message
            .contains("application main")
    );
}
#[test]
fn body_errors_and_constant_cycles_keep_original_source() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Library :: #import \"Library\"; main :: () -> int { return Library.answer(); }",
        ),
        (
            "modules/Library.jai",
            "\nanswer :: () -> int { return missing; }",
        ),
    ]);
    let graph = fixture.graph();
    let error = resolve_graph(&graph).unwrap_err();
    assert!(error.render(graph.sources()).contains("Library.jai:2:"));
    let fixture = Fixture::new(&[
        ("main.jai", "Library :: #import \"Library\"; main :: () {}"),
        ("modules/Library.jai", "LEFT :: RIGHT;\nRIGHT :: LEFT;"),
    ]);
    let graph = fixture.graph();
    let error = resolve_graph(&graph).unwrap_err();
    assert!(error.message.contains("cyclic constant"));
    assert!(error.render(graph.sources()).contains("Library.jai:2:"));
}
#[test]
fn qualified_privacy_and_local_namespace_shadowing_fail_at_use_site() {
    for source in [
        "return Library.hidden;",
        "Library := 1; return Library.answer();",
        "return Library.file_only;",
    ] {
        let fixture = Fixture::new(&[
            (
                "main.jai",
                &format!("Library :: #import \"Library\"; main :: () -> int {{ {source} }}"),
            ),
            (
                "modules/Library.jai",
                "#scope_module hidden :: 1; #scope_file file_only :: 2; #scope_export answer :: () -> int { return hidden + file_only; }",
            ),
        ]);
        let graph = fixture.graph();
        let error = resolve_graph(&graph).unwrap_err();
        assert!(
            error.render(graph.sources()).contains("main.jai:1:"),
            "{error}"
        );
    }
}

#[test]
fn file_private_constants_keep_each_loaded_procedures_defining_scope() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "#scope_file VALUE :: 1; #scope_export #load \"helper.jai\"; main :: () -> int { return answer(); }",
        ),
        (
            "helper.jai",
            "#scope_file VALUE :: 42; #scope_export answer :: () -> int { return VALUE; }",
        ),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let IntExprKind::Call(call) = returned_integer(&program.procedures()[main.index()]).kind()
    else {
        panic!()
    };
    assert!(
        matches!(returned_integer(&program.procedures()[call.procedure.index()]).kind(), IntExprKind::Constant(value) if value.value() == 42)
    );
}

#[test]
fn one_source_loaded_and_imported_has_independent_runtime_storage() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "#load \"shared.jai\"; Library :: #import,file \"shared.jai\"; main :: () -> int { return answer() + Library.answer(); }",
        ),
        (
            "shared.jai",
            "value := 0; answer :: () -> int { return value; }",
        ),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    assert_eq!(program.globals().len(), 2);
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let IntExprKind::Binary(_, left, right) =
        returned_integer(&program.procedures()[main.index()]).kind()
    else {
        panic!()
    };
    let (IntExprKind::Call(left), IntExprKind::Call(right)) = (left.kind(), right.kind()) else {
        panic!()
    };
    let places: Vec<_> = [left, right]
        .into_iter()
        .map(
            |call| match returned_integer(&program.procedures()[call.procedure.index()]).kind() {
                IntExprKind::Load(place) => place.place(),
                _ => panic!("expected global load"),
            },
        )
        .collect();
    assert_ne!(places[0], places[1]);
}
