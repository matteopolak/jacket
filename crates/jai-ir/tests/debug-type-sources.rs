use jai_ir::{DebugSourceLocation, DebugSources, FieldSource, ProgramBuilder, TypeSource};
use jai_source::{SourceMap, SourceRecord, SourceSpan, Span};
use jai_types::{IntegerType, RecordKind, ScalarType, TypeRegistry};

fn source(map: &mut SourceMap) -> &SourceRecord {
    let id = map.insert("record.jai".into(), "Pair :: struct { value: int; }".into());
    map.get(id).unwrap()
}

fn location(source: &SourceRecord) -> DebugSourceLocation {
    DebugSourceLocation::from_source(
        source,
        SourceSpan {
            source: source.id(),
            span: Span::new(0, source.text().len()),
        },
    )
    .unwrap()
}

fn record(types: &mut TypeRegistry) -> jai_types::TypeId {
    let record = types.reserve_record(RecordKind::Struct);
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    types.define_record(record, [integer]).unwrap();
    record
}

#[test]
fn named_and_anonymous_records_publish_only_their_actual_field_identities() {
    for name in [Some("Pair".to_owned()), None] {
        let mut map = SourceMap::default();
        let original = source(&mut map);
        let mut types = TypeRegistry::new();
        let ty = record(&mut types);
        let field = types.field(ty, 0).unwrap();
        let mut sources = DebugSources::default();
        sources.retain_source(original);
        sources.insert_type_source(
            ty,
            TypeSource {
                name: name.clone(),
                location: location(original),
            },
        );
        sources.insert_field_source(
            field.id,
            FieldSource {
                name: "value".into(),
                location: location(original),
            },
        );
        let library = ProgramBuilder::new(types.freeze().unwrap())
            .debug_sources(sources)
            .finish_library()
            .unwrap();
        let sources = library.debug_sources().unwrap();
        assert_eq!(sources.type_source(ty).unwrap().name, name);
        assert_eq!(sources.field_source(field.id).unwrap().name, "value");
        assert_eq!(library.types().field_type(field.id).unwrap(), field.ty);
    }
}

#[test]
fn foreign_type_and_field_arenas_are_rejected_even_with_equal_indices() {
    for foreign_field in [false, true] {
        let mut map = SourceMap::default();
        let original = source(&mut map);
        let mut retained = TypeRegistry::new();
        let retained_ty = record(&mut retained);
        let mut foreign = TypeRegistry::new();
        let foreign_ty = record(&mut foreign);
        assert_eq!(retained_ty.index(), foreign_ty.index());
        let mut sources = DebugSources::default();
        sources.retain_source(original);
        if foreign_field {
            sources.insert_field_source(
                foreign.field(foreign_ty, 0).unwrap().id,
                FieldSource {
                    name: "value".into(),
                    location: location(original),
                },
            );
        } else {
            sources.insert_type_source(
                foreign_ty,
                TypeSource {
                    name: Some("Pair".into()),
                    location: location(original),
                },
            );
        }
        assert!(
            ProgramBuilder::new(retained.freeze().unwrap())
                .debug_sources(sources)
                .finish_library()
                .is_err()
        );
    }
}

#[test]
fn type_spelling_cannot_rename_a_shared_primitive_identity() {
    let mut map = SourceMap::default();
    let original = source(&mut map);
    let types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    let mut sources = DebugSources::default();
    sources.retain_source(original);
    sources.insert_type_source(
        integer,
        TypeSource {
            name: Some("Alias".into()),
            location: location(original),
        },
    );
    assert!(
        ProgramBuilder::new(types.freeze().unwrap())
            .debug_sources(sources)
            .finish_library()
            .is_err()
    );
}

#[test]
fn source_identity_and_spelling_are_checked_at_publication() {
    for invalid in [0, 1, 2] {
        let mut map = SourceMap::default();
        let original = source(&mut map);
        let mut foreign = SourceMap::default();
        let replacement = source(&mut foreign);
        let mut types = TypeRegistry::new();
        let ty = record(&mut types);
        let mut sources = DebugSources::default();
        sources.retain_source(original);
        sources.insert_type_source(
            ty,
            TypeSource {
                name: Some(match invalid {
                    1 => "".into(),
                    2 => "Pair\0Alias".into(),
                    _ => "Pair".into(),
                }),
                location: location(if invalid == 0 { replacement } else { original }),
            },
        );
        assert!(
            ProgramBuilder::new(types.freeze().unwrap())
                .debug_sources(sources)
                .finish_library()
                .is_err()
        );
    }
}
