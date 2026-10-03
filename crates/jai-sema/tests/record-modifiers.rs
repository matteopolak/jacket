//! Genuine source modifier recipes must finish before dependent type publication.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use jai_syntax as syntax;
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/jai-record-modifier-source/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}

fn run(source: &str) -> i128 {
    run_with_options(source, &ResolveOptions::default())
}

fn run_with_options(source: &str, options: &ResolveOptions) -> i128 {
    let graph = graph(source);
    let mut session = PreparedLibrarySession::new(&graph, options).unwrap();
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

#[test]
fn accepted_count_recipe_precedes_alias_global_header_and_recursive_identity() {
    assert_eq!(
        run(r#"
            Buffer::struct(N:int=3) #modify {if N<8 N=8;return true;} {
                #assert N>=8;
                values:[N]int;
                next:*#this;
            }
            Alias::#type Buffer();
            Other::#type Buffer(1);
            saved:Alias;
            read::(value:Buffer()=.{})->int {return value.values.count;}
            main::()->int {
                another:Buffer(1)=saved;
                saved.values[0]=21;
                saved.next=*saved;
                return saved.next.values[0]+read(value=another)+13;
            }
        "#,),
        42,
    );
}

#[test]
fn a_late_body_type_application_retains_its_genuine_modifier_dependency() {
    assert_eq!(
        run(r#"
            Box::struct(N:int) #modify {if N<21 N=21;return true;} {value:int=N;}
            First::#type Box(1);
            saved:First;
            main::()->int {other:Box(2)=saved;return saved.value+other.value;}
        "#,),
        42,
    );
}

#[test]
fn omitted_default_recipe_does_not_reapply_a_non_idempotent_modifier() {
    assert_eq!(
        run(r#"
            Buffer::struct(N:int=3) #modify {N+=1;return true;} {values:[N]int;}
            Default::#type Buffer();
            Explicit::#type Buffer(4);
            saved:Default;
            read::(value:Buffer()=.{})->int {return value.values.count;}
            main::()->int {
                other:Explicit;
                return saved.values.count+other.values.count+read()+29;
            }
        "#,),
        42,
    );
}

#[test]
fn modifier_waits_for_the_original_checked_helper_body() {
    assert_eq!(
        run(r#"
            choose::(value:int)->int #no_context {if value<21 return 21;return value;}
            Box::struct(N:int) #modify {N=choose(N);return true;} {value:int=N;}
            First::#type Box(1);
            Second::#type Box(2);
            saved:First;
            main::()->int {other:Second=saved;return saved.value+other.value;}
        "#,),
        42,
    );
}

#[test]
fn accepted_type_slot_rechecks_its_dependent_baked_value() {
    assert_eq!(
        run_with_options(
            r#"
            Box::struct(T:Type,Fill:T) #modify {T=s64;return true;} {value:T=Fill;}
            First::#type Box(u8,21);
            Second::#type Box(u16,21);
            saved:First;
            main::()->int {other:Second=saved;return saved.value+other.value;}
        "#,
            &ResolveOptions {
                layout: Some(jai_types::LayoutPolicy::lp64()),
                ..ResolveOptions::default()
            },
        ),
        42,
    );
}

#[test]
fn rejected_recipe_retains_its_original_source_reason() {
    let graph = graph(
        "Box::struct(N:int) #modify{return false,\"count is forbidden\";} {value:int=N;} Alias::#type Box(7); main::()->int{return 0;}",
    );
    let error = match PreparedLibrarySession::new(&graph, &ResolveOptions::default()) {
        Err(error) => error,
        Ok(mut session) => match session.drive(&mut jai_vm::NoEffects) {
            LibraryReadiness::Failed(error) => error,
            _ => panic!("rejected modifier must fail preparation"),
        },
    };
    assert!(error.message.contains("count is forbidden"), "{error:?}");
    assert_eq!(
        error.location.source,
        graph.declarations()[0].location().source
    );
}

#[test]
fn inferred_modified_default_pattern_requires_its_own_checked_recipe() {
    let graph = graph(
        "Box::struct(T:Type,N:int=3) #modify{N+=1;return true;} {value:T;values:[N]int;} read::(value:Box($T))->int{return value.values.count;} main::()->int{return 0;}",
    );
    let error = match PreparedLibrarySession::new(&graph, &ResolveOptions::default()) {
        Err(error) => error,
        Ok(mut session) => match session.drive(&mut jai_vm::NoEffects) {
            LibraryReadiness::Failed(error) => error,
            _ => panic!("an inferred modified default needs its own recipe proof"),
        },
    };
    assert!(
        error.message.contains("checked default recipe"),
        "{error:?}"
    );
    let syntax::FileDeclarationKind::Procedure(read) = &graph.declarations()[1].syntax().kind
    else {
        panic!("the diagnostic belongs to the original procedure header");
    };
    assert_eq!(
        error.location.source,
        graph.declarations()[1].location().source
    );
    assert!(error.location.span.start >= read.span.start);
    assert!(error.location.span.end <= read.span.end);
}

fn rejected_type_modifier(source: &str) -> jai_source::LocatedDiagnostic {
    let graph = graph(source);
    let options = ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..ResolveOptions::default()
    };
    let error = match PreparedLibrarySession::new(&graph, &options) {
        Err(error) => error,
        Ok(mut session) => match session.drive(&mut jai_vm::NoEffects) {
            LibraryReadiness::Failed(error) => error,
            LibraryReadiness::Complete(_) => panic!("an incompatible accepted binding must fail"),
            LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
        },
    };
    let rejected = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "Rejected")
        .unwrap();
    let syntax::FileDeclarationKind::TypeAlias(alias) = &rejected.syntax().kind else {
        panic!("the original application owns this rejection")
    };
    let syntax::TypeSyntax::Application(application) = &alias.ty else {
        panic!("the original application owns this rejection")
    };
    assert!(
        error
            .message
            .contains("baked record argument differs from its formal parameter type"),
        "{error:?}"
    );
    assert_eq!(
        error.location.source,
        rejected.location().source,
        "{error:?}"
    );
    assert_eq!(error.location.span, application.span, "{error:?}");
    error
}

#[test]
fn accepted_type_slot_rejects_narrowing_the_dependent_typed_value() {
    rejected_type_modifier(
        "Box::struct(T:Type,Fill:T) #modify {T=u8;return true;} {value:T=Fill;} \
         Rejected::#type Box(u16,21); main::()->int{return 0;}",
    );
}
