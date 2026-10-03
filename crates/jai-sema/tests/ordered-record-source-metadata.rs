//! Complete source nodes bind canonical layouts and keep physical policies distinct.
use jai_modules::{ModuleGraph, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::{LayoutEngine, LayoutPolicy, RecordReflectionFlag, TypeKind};
use jai_vm::NoEffects;
use std::path::Path;

fn checked(source: &str) -> Result<jai_ir::Program, jai_source::LocatedDiagnostic> {
    let path = Path::new("/ordered-record-source/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, Default::default(), &overlay).unwrap();
    resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut NoEffects,
    )
}

#[test]
fn graph_local_and_specialized_cursor_shapes_use_actual_owner_bound_anchors() {
    for source in [
        "Placed::struct {anchor:u64; #place anchor; view:u32=---; tail:u8=---;} main::()->int{return size_of(Placed);}",
        "main::()->int{Placed::struct {anchor:u64; #place anchor; view:u32=---; tail:u8=---;} return size_of(Placed);}",
        "Placed::struct(T:Type){anchor:u64; #place anchor; view:T=---; tail:u8=---;} main::()->int{p:*Placed(u32); return size_of(Placed(u32));}",
    ] {
        let program = checked(source).unwrap();
        let types = program.types();
        let (ty, record) = types
            .iter()
            .find_map(|(ty, kind)| {
                if !matches!(kind, TypeKind::Record(_)) {
                    return None;
                }
                let record = types.record_definition(ty).ok()?;
                (record.layout.field_placements.iter().any(Option::is_some)).then_some((ty, record))
            })
            .expect("actual completed placed source record");
        assert_eq!(record.fields.len(), 3);
        assert_eq!(
            record.layout.field_placements[1],
            Some(types.field(ty, 0).unwrap().id)
        );
        assert_eq!(
            types
                .validate_field(ty, record.layout.field_placements[1].unwrap())
                .unwrap(),
            record.fields[0]
        );
        let layout = LayoutEngine::new(types, LayoutPolicy::lp64())
            .layout(ty)
            .unwrap()
            .clone();
        assert_eq!(layout.field_offsets.as_ref(), &[0, 0, 4]);
        assert_eq!(layout.size, 8);
    }
}

#[test]
fn chosen_members_preserve_cursor_chronology_and_inactive_anchors_do_not_bind() {
    checked("main::()->int{Placed::struct{anchor:u64; #if true {#place anchor; view:u32=---;} else {#place missing; bad:MissingType;} tail:u8=---;} return size_of(Placed);}").unwrap();
}

#[test]
fn source_reflection_reduction_keeps_real_procedure_field_types() {
    for source in [
        "Procs::struct #type_info_procedures_are_void_pointers #type_info_no_size_complaint { callback:(value:u32)->u32 #c_call #no_context; } main::()->int{return size_of(Procs);}",
        "main::()->int{Procs::struct #type_info_procedures_are_void_pointers #type_info_no_size_complaint {callback:(value:u32)->u32 #c_call #no_context;} return size_of(Procs);}",
    ] {
        let program = checked(source).unwrap();
        let types = program.types();
        let (ty, record) = types
            .iter()
            .find_map(|(ty, kind)| {
                if !matches!(kind, TypeKind::Record(_)) {
                    return None;
                }
                let record = types.record_definition(ty).ok()?;
                let policy = types.record_reflection_policy(ty).ok()?;
                policy
                    .contains(RecordReflectionFlag::ProceduresAreVoidPointers)
                    .then_some((ty, record))
            })
            .expect("source policy on original nominal record");
        assert!(
            types
                .record_reflection_policy(ty)
                .unwrap()
                .contains(RecordReflectionFlag::NoSizeComplaint)
        );
        assert!(matches!(
            types.kind(record.fields[0]).unwrap(),
            TypeKind::Procedure(_)
        ));
    }
}

#[test]
fn forward_cursor_anchor_is_rejected_at_actual_target() {
    let source = "Placed::struct{#place later; later:u64;} main::()->int{return 0;}";
    let error = checked(source).unwrap_err();
    assert_eq!(error.location.span.text(source), "later");
    assert!(error.message.contains("earlier directly declared"));
}

#[test]
fn overlay_is_parsed_but_does_not_invent_a_cursor_policy() {
    let source = "Alias::struct{anchor:u64; #overlay(anchor) view:u32;} main::()->int{return 0;}";
    let error = checked(source).unwrap_err();
    assert_eq!(error.location.span.text(source), "anchor");
    assert!(error.message.contains("checked overlay cursor policy"));
}

#[test]
fn whole_placed_default_and_explicit_constructors_require_ordered_storage_recipe() {
    for body in ["value:Placed;", "value:=Placed.{anchor=1,view=2,tail=3};"] {
        let source = format!(
            "Placed::struct{{anchor:u64; #place anchor; view:u32=---; tail:u8=---;}} main::()->int{{{body} return 0;}}"
        );
        let error = checked(&source).unwrap_err();
        assert!(
            error
                .message
                .contains("ordered storage initialization recipe"),
            "{error:?}"
        );
    }
}
