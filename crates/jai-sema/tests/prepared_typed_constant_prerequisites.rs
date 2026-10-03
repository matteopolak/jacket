//! Canonical original typed values must precede dependent alias argument preparation.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use std::path::Path;
fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/prepared-typed-constant/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}
fn options() -> ResolveOptions {
    ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..Default::default()
    }
}
fn run(source: &str) -> i128 {
    let graph = graph(source);
    let mut session = PreparedLibrarySession::new(&graph, &options()).unwrap();
    let library = match session.drive(&mut jai_vm::NoEffects) {
        LibraryReadiness::Complete(library) => *library,
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
        LibraryReadiness::Pending(wait) => panic!("{wait:?}"),
    };
    let main = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "main")
        .unwrap();
    let id = library.procedure(main.id()).unwrap().id;
    let program = library.into_program(jai_ir::EntryPoint::Int(id)).unwrap();
    match jai_vm::execute(&program, Default::default()).outcome {
        jai_vm::Outcome::Complete(values) => values[0].integer().unwrap().value(),
        other => panic!("{other:?}"),
    }
}
#[test]
fn a_named_original_distinct_value_is_ready_before_its_alias_argument() {
    assert_eq!(
        run("First::#type,distinct u8; initial:First:21; \
        Box::struct(T:Type,Fill:T){value:T=Fill;} \
        Ready::#type Box(First,initial); saved:Ready; main::()->int{return cast(int)saved.value;}"),
        21
    );
}
#[test]
fn accepted_modifier_preserves_a_genuinely_checked_initial_distinct_value() {
    assert_eq!(
        run("First::#type,distinct u8; initial:First:21; \
        Box::struct(T:Type,Fill:T) #modify {return true;} {value:T=Fill;} \
        Ready::#type Box(First,initial); saved:Ready; main::()->int{return cast(int)saved.value;}"),
        21
    );
}
#[test]
fn accepted_modifier_rejects_the_checked_original_nominal_at_its_application() {
    let graph = graph(
        "First::#type,distinct u8; Other::#type,distinct u8; initial:First:21; \
        Box::struct(T:Type,Fill:T) #modify {T=Other;return true;} {value:T=Fill;} \
        Rejected::#type Box(First,initial); main::()->int{return 0;}",
    );
    let error = match PreparedLibrarySession::new(&graph, &options()) {
        Err(error) => error,
        Ok(mut session) => match session.drive(&mut jai_vm::NoEffects) {
            LibraryReadiness::Failed(error) => error,
            _ => panic!("the accepted incompatible nominal must be rejected"),
        },
    };
    assert!(
        error
            .message
            .contains("baked record argument differs from its formal parameter type"),
        "{error:?}"
    );
    let source = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "Rejected")
        .unwrap();
    let jai_syntax::FileDeclarationKind::TypeAlias(alias) = &source.syntax().kind else {
        panic!("source alias")
    };
    let jai_syntax::TypeSyntax::Application(application) = &alias.ty else {
        panic!("source application")
    };
    assert_eq!(error.location.source, source.location().source);
    assert_eq!(error.location.span, application.span);
}
#[test]
fn a_checked_distinct_value_never_becomes_a_scalar_count_seed() {
    let source = "First::#type,distinct u8; initial:First:21; Bad::#type [initial]u8; main::()->int{return 0;}";
    let graph = graph(source);
    let error = match PreparedLibrarySession::new(&graph, &options()) {
        Err(error) => error,
        Ok(mut session) => match session.drive(&mut jai_vm::NoEffects) {
            LibraryReadiness::Failed(error) => error,
            _ => panic!("a distinct nominal is not an integer scalar count"),
        },
    };
    assert!(
        error.message.contains("cannot supply a scalar"),
        "{error:?}"
    );
    let start = source.find("[initial]").unwrap() + 1;
    assert_eq!(
        error.location.span,
        jai_source::Span::new(start, start + "initial".len())
    );
}
