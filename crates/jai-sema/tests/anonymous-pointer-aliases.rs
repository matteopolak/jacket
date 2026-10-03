//! Actual opaque pointer aliases retain source-owned anonymous nominal identities.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_types::{RecordKind, TypeKind};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let mut overlay = SourceOverlay::new();
    overlay
        .insert(
            Path::new("/jai-opaque-alias/main.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    ModuleGraph::load_with_provider(
        Path::new("/jai-opaque-alias/main.jai"),
        GraphOptions::default(),
        &overlay,
    )
    .unwrap()
}
fn complete(source: &str) -> jai_sema::Program {
    let program = jai_sema::resolve_graph(&graph(source)).unwrap();
    assert!(
        matches!(jai_vm::execute(&program,jai_vm::Limits::default()).outcome,jai_vm::Outcome::Complete(values) if matches!(values.as_slice(),[jai_vm::Value::Int(value)] if value.value()==42))
    );
    program
}

#[test]
fn opaque_pointer_alias_repeated_uses_share_the_real_empty_record() {
    let program = complete(
        "EventStreamRef::*struct {}; Again::#type EventStreamRef; accept::(value:EventStreamRef)->int{return 42;} main::()->int{first:EventStreamRef;second:Again=first;return accept(second);}",
    );
    let argument = program
        .procedures()
        .iter()
        .find_map(|procedure| {
            let signature = program
                .types()
                .procedure_definition(procedure.signature)
                .unwrap();
            (signature.parameters.len() == 1).then(|| signature.parameters[0])
        })
        .unwrap();
    let TypeKind::Pointer(target) = *program.types().kind(argument).unwrap() else {
        panic!("pointer alias required");
    };
    let record = program.types().record_definition(target).unwrap();
    assert_eq!(record.kind, RecordKind::Struct);
    assert!(record.fields.is_empty());
    let pointer_locals = program
        .procedures()
        .iter()
        .flat_map(|procedure| procedure.locals.iter())
        .filter(|local| matches!(program.types().kind(local.ty()), Ok(TypeKind::Pointer(_))))
        .collect::<Vec<_>>();
    assert!(pointer_locals.len() >= 2);
    assert!(pointer_locals.iter().all(|local| local.ty() == argument));
}

#[test]
fn local_nominal_pointer_aliases_accept_union_and_enum_bodies() {
    complete(
        "main::()->int{UnionRef::**union{integer:int;number:float64;}; EnumRef::*enum u8{ZERO::0;};FlagsRef::*enum_flags u8{A::1;}; a:UnionRef;b:UnionRef=a;c:EnumRef;d:FlagsRef;if b!=null || c!=null || d!=null return 1;return 42;}",
    );
}

#[test]
fn distinct_anonymous_alias_origins_do_not_merge_equal_layouts() {
    for source in [
        "Left::*struct{};Right::*struct{};main::(){left:Left;right:Right;left=right;}",
        "main::(){Left::*enum u8{ZERO::0;};Right::*enum u8{ZERO::0;};left:Left;right:Right;left=right;}",
    ] {
        let error = jai_sema::resolve_graph(&graph(source)).unwrap_err();
        assert!(error.message.contains("pointee"), "{error:?}");
    }
}
