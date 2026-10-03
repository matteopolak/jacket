use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use jai_source::{LocatedDiagnostic, SourceSpan};
use jai_types::LayoutPolicy;
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/prepared-nested-layout/main.jai");
    let mut provider = SourceOverlay::new();
    provider.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &provider).unwrap()
}

fn options() -> ResolveOptions {
    ResolveOptions {
        layout: Some(LayoutPolicy::lp64()),
        ..Default::default()
    }
}

fn run(source: &str) -> i128 {
    let graph = graph(source);
    let mut session = PreparedLibrarySession::new(&graph, &options()).unwrap();
    let library = match session.drive(&mut jai_vm::NoEffects) {
        LibraryReadiness::Complete(library) => *library,
        LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
    };
    let main = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "main")
        .unwrap();
    let entry = library.procedure(main.id()).unwrap().id;
    let program = library
        .into_program(jai_ir::EntryPoint::Int(entry))
        .unwrap();
    match jai_vm::execute(&program, Default::default()).outcome {
        jai_vm::Outcome::Complete(values) => values[0].integer().unwrap().value(),
        outcome => panic!("{outcome:?}"),
    }
}

fn fail(source: &str) -> (ModuleGraph, LocatedDiagnostic) {
    let graph = graph(source);
    let error = {
        let mut session = PreparedLibrarySession::new(&graph, &options()).unwrap();
        match session.drive(&mut jai_vm::NoEffects) {
            LibraryReadiness::Failed(error) => error,
            _ => panic!("a cyclic layout prerequisite must not complete or wait for effects"),
        }
    };
    (graph, error)
}

#[test]
fn independent_nested_buffer_layout_precedes_its_outer_array() {
    assert_eq!(
        run(r#"
        ENABLE_ASSERT :: true;
        N :: 4096 - size_of(Builder.Buffer);
        Builder :: struct {
            Buffer :: struct {
                count: s64;
                allocated: s64;
                #if ENABLE_ASSERT ensured_count: s64;
                next: *Buffer;
            }
            current: *Buffer;
            bytes: [N]u8;
        }
        saved: Builder;
        main :: () -> int { return saved.bytes.count + size_of(Builder.Buffer); }
    "#),
        4096
    );
}

#[test]
fn selected_ordinary_call_constant_finishes_its_actual_body() {
    assert_eq!(
        run(
            "count::()->int #no_context{return 3;} Count::count(); Alias::#type [Count]u8; saved:Alias; main::()->int{return saved.count;}"
        ),
        3
    );
}

#[test]
fn a_nested_count_that_reads_its_own_layout_constant_is_a_real_cycle() {
    let source = "N::size_of(Outer.Inner); Outer::struct {Inner::struct {bytes:[N]u8;} bytes:[N]u8;} saved:Outer;";
    let (graph, error) = fail(source);
    let n = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "N")
        .unwrap();
    assert_eq!(error.location.source, n.location().source);
    assert_eq!(error.location.span.text(source), "N");
    assert!(
        error
            .message
            .contains("unresolved or cyclic #run dependencies"),
        "{error}"
    );
    assert!(
        error.message.contains(&format!("constants [{:?}]", n.id())),
        "{error}"
    );
}

#[test]
fn a_nested_by_value_outer_dependency_cannot_certify_an_independent_layout() {
    let source = "N::size_of(Outer.Inner); Outer::struct {Inner::struct {parent:Outer;} bytes:[N]u8;} saved:Outer;";
    let (graph, error) = fail(source);
    let n = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "N")
        .unwrap();
    let jai_syntax::FileDeclarationKind::Constant(constant) = &n.syntax().kind else {
        unreachable!()
    };
    assert_eq!(
        error.location,
        SourceSpan {
            source: n.location().source,
            span: constant.initializer.span
        }
    );
    assert!(
        error
            .message
            .contains("size_of is waiting for target or nominal layout dependencies"),
        "{error}"
    );
    assert!(error.message.contains("Definition("), "{error}");
}
