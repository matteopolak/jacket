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
        let root = std::env::temp_dir().join(format!(
            "jai-nominal-types-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        for (name, source) in files {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source).unwrap();
        }
        Self(root)
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
fn evaluate(
    graph: &ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
) -> Result<ConstantValue, LocatedDiagnostic> {
    jai_eval::evaluate_paths(expression, |_, span| {
        Err(Diagnostic::new(span, "unexpected constant dependency"))
    })
    .map_err(|error| located(graph, file, error))
}
#[test]
fn distinct_variants_preserve_nominal_identity_through_transparent_aliases() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Handle :: #type,distinct u32; Other :: #type,distinct u32; Child :: #type,isa Handle; Alias :: Handle;",
    )]);
    let graph = fixture.graph();
    let mut types = TypeRegistry::new();
    let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
    nominals
        .define_aliases(&graph, &mut types, &mut |file, expression| {
            evaluate(&graph, file, expression)
        })
        .unwrap();
    let declared = |name: &str| {
        let id = graph
            .declarations()
            .iter()
            .find(|declaration| graph.symbols().name(declaration.name()) == name)
            .unwrap()
            .id();
        nominals.declarations[&id]
    };
    let handle = declared("Handle");
    assert_ne!(handle, declared("Other"));
    assert_eq!(handle, declared("Alias"));
    assert_eq!(
        types.distinct_definition(handle).unwrap().representation,
        types.scalar(ScalarType::Int(IntegerType::U32))
    );
    let child = types.distinct_definition(declared("Child")).unwrap();
    assert_eq!(child.kind, jai_types::DistinctKind::IsA);
    assert_eq!(child.representation, handle);
    let frozen = types.freeze().unwrap();
    let mut layout = jai_types::LayoutEngine::new(&frozen, jai_types::LayoutPolicy::lp64());
    assert_eq!(layout.layout(handle).unwrap().size, 4);
    assert_eq!(layout.layout(declared("Child")).unwrap().size, 4);
}
#[test]
fn record_attributes_preserve_field_alignment_constraints() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Record :: struct { tag: u32 #align 4; pointer: *u8 #align (2 * 2); tail: u32 #align 4; } #no_padding",
    )]);
    let graph = fixture.graph();
    let mut types = TypeRegistry::new();
    let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
    nominals
        .define_records(&graph, &mut types, &mut |file, expression| {
            evaluate(&graph, file, expression)
        })
        .unwrap();
    let ty = *nominals.records.keys().next().unwrap();
    assert_eq!(
        types
            .record_definition(ty)
            .unwrap()
            .layout
            .field_alignments
            .as_ref(),
        &[Some(4), Some(4), Some(4)]
    );
    let frozen = types.freeze().unwrap();
    let mut layout = jai_types::LayoutEngine::new(&frozen, jai_types::LayoutPolicy::lp64());
    assert_eq!(
        layout.layout(ty).unwrap().field_offsets.as_ref(),
        &[0, 4, 12]
    );
    assert_eq!(layout.layout(ty).unwrap().alignment, 4);
}
#[test]
fn invalid_source_alignment_is_diagnosed_before_layout() {
    for value in ["0", "3", "-1", "true", "4294967296"] {
        let source = format!("Record :: struct {{ tag: u8 #align {value}; }}");
        let fixture = Fixture::new(&[("main.jai", &source)]);
        let graph = fixture.graph();
        let mut types = TypeRegistry::new();
        let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let error = nominals
            .define_records(&graph, &mut types, &mut |file, expression| {
                evaluate(&graph, file, expression)
            })
            .unwrap_err();
        assert!(error.message.contains("alignment requires"));
    }
}
#[test]
fn inferred_record_string_and_typed_array_fields_keep_aggregate_types() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Point :: struct { x: int; } Holder :: struct { named := Point.{x = 1}; positional := Point.{2}; text := \"hello\"; bytes := u8.[1, 2]; }",
    )]);
    let graph = fixture.graph();
    let mut types = TypeRegistry::new();
    let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
    nominals
        .define_records(&graph, &mut types, &mut |file, expression| {
            evaluate(&graph, file, expression)
        })
        .unwrap();
    let holder = nominals
        .records
        .values()
        .find(|record| {
            graph
                .symbols()
                .name(graph.declaration(record.declaration).unwrap().name())
                == "Holder"
        })
        .unwrap();
    assert_eq!(holder.fields[0].ty, holder.fields[1].ty);
    assert!(matches!(
        types.kind(holder.fields[0].ty),
        Ok(TypeKind::Record(_))
    ));
    assert_eq!(holder.fields[2].ty, types.string());
    assert!(matches!(
        types.kind(holder.fields[3].ty),
        Ok(TypeKind::FixedArray { count: 2, .. })
    ));
}
#[test]
fn same_spelled_records_keep_module_identity_and_self_pointer_fields() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "A :: #import \"A\"; B :: #import \"B\"; Wrapper :: struct { a: A.Point; b: B.Point; next: *Wrapper; callback: #type (*Wrapper) -> int; }",
        ),
        ("modules/A.jai", "Point :: struct { x: int; }"),
        ("modules/B.jai", "Point :: struct { x: u32; }"),
    ]);
    let graph = fixture.graph();
    let mut types = TypeRegistry::new();
    let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
    nominals
        .define_records(&graph, &mut types, &mut |file, expression| {
            evaluate(&graph, file, expression)
        })
        .unwrap();
    let wrapper = nominals
        .records
        .values()
        .find(|record| {
            graph
                .symbols()
                .name(graph.declaration(record.declaration).unwrap().name())
                == "Wrapper"
        })
        .unwrap();
    assert_ne!(wrapper.fields[0].ty, wrapper.fields[1].ty);
    let wrapper_ty = nominals.declarations[&wrapper.declaration];
    assert_eq!(
        types.kind(wrapper.fields[2].ty).unwrap(),
        &TypeKind::Pointer(wrapper_ty)
    );
    for (ordinal, field) in wrapper.fields.iter().enumerate() {
        assert_eq!(types.field(wrapper_ty, ordinal).unwrap().id, field.id);
        assert_eq!(types.field_type(field.id).unwrap(), field.ty);
    }
    types.freeze().unwrap();
}
#[test]
fn transparent_aliases_follow_defining_file_and_leave_scalar_names_scalar() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "A :: #import \"A\"; Width :: s64; Point :: struct { local: Width; remote: A.Alias; bytes: [3] u8; } N :: 7; Copy :: N;",
        ),
        (
            "modules/A.jai",
            "Width :: u32; Node :: struct { x: Width; next: *Node; } Alias :: Node; Ptr :: *Node;",
        ),
    ]);
    let graph = fixture.graph();
    let mut types = TypeRegistry::new();
    let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
    nominals
        .define_aliases(&graph, &mut types, &mut |file, expression| {
            evaluate(&graph, file, expression)
        })
        .unwrap();
    nominals
        .define_records(&graph, &mut types, &mut |file, expression| {
            evaluate(&graph, file, expression)
        })
        .unwrap();
    let lookup = |name: &str| {
        graph
            .declarations()
            .iter()
            .find(|declaration| graph.symbols().name(declaration.name()) == name)
            .unwrap()
            .id()
    };
    assert_eq!(
        nominals.declarations[&lookup("Alias")],
        nominals.declarations[&lookup("Node")]
    );
    assert!(matches!(
        types.kind(nominals.declarations[&lookup("Ptr")]).unwrap(),
        TypeKind::Pointer(_)
    ));
    assert!(!nominals.is_type_alias(&graph, lookup("Copy")));
    let node = &nominals.records[&nominals.declarations[&lookup("Node")]];
    assert_eq!(
        types.kind(node.fields[0].ty).unwrap(),
        &TypeKind::Integer(IntegerType::U32)
    );
    let point = &nominals.records[&nominals.declarations[&lookup("Point")]];
    assert_eq!(
        types.kind(point.fields[0].ty).unwrap(),
        &TypeKind::Integer(IntegerType::S64)
    );
    types.freeze().unwrap();
}
#[test]
fn pointer_alias_classification_follows_types_and_preserves_storage_addresses() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Number::int; Pointer::*Number; Again::Pointer; Node::struct{value:int;} NodePointer::*Node; storage:int; Address::*storage; N::7; ScalarCopy::N;",
    )]);
    let graph = fixture.graph();
    let mut types = TypeRegistry::new();
    let nominals = Nominals::reserve(&graph, &mut types).unwrap();
    for declaration in graph.declarations() {
        let name = graph.symbols().name(declaration.name());
        let expected = matches!(
            name,
            "Number" | "Pointer" | "Again" | "Node" | "NodePointer"
        );
        assert_eq!(
            nominals.is_type_alias(&graph, declaration.id()),
            expected,
            "{name}"
        );
    }
}
#[test]
fn enum_values_use_earlier_members_and_integer_representation_alias() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Width :: u8; Fruit :: enum Width #specified { APPLE :: 5; PEAR :: APPLE + 2; LAST :: Fruit.PEAR + 1; } Bits :: enum_flags u8 { A; B; C; BOTH :: A | B; } Choice :: struct { fruit := Fruit.PEAR; }",
    )]);
    let graph = fixture.graph();
    let mut types = TypeRegistry::new();
    let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
    nominals
        .define_enums(&graph, &mut types, &mut |_, _, span| {
            Err(Diagnostic::new(span, "unexpected enum dependency"))
        })
        .unwrap();
    nominals
        .define_records(&graph, &mut types, &mut |file, expression| {
            evaluate(&graph, file, expression)
        })
        .unwrap();
    let choice = nominals.records.values().next().unwrap();
    assert!(matches!(
        types.kind(choice.fields[0].ty).unwrap(),
        TypeKind::Enum(_)
    ));
    let fruit = nominals.enums.values().find(|info| !info.flags).unwrap();
    assert_eq!(fruit.representation, IntegerType::U8);
    let mut values: Vec<_> = fruit.members.values().map(|value| value.value()).collect();
    values.sort();
    assert_eq!(values, [5, 7, 8]);
    let bits = nominals.enums.values().find(|info| info.flags).unwrap();
    let mut values: Vec<_> = bits.members.values().map(|value| value.value()).collect();
    values.sort();
    assert_eq!(values, [1, 2, 3, 4]);
    types.freeze().unwrap();
}
#[test]
fn type_alias_cycles_report_the_file_that_closes_the_cycle() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Alias :: *Other; Other :: *Alias; Holder :: struct { value: Alias; }",
    )]);
    let graph = fixture.graph();
    let mut types = TypeRegistry::new();
    let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
    let error = nominals
        .define_records(&graph, &mut types, &mut |file, expression| {
            evaluate(&graph, file, expression)
        })
        .unwrap_err();
    assert!(error.message.contains("cyclic type alias"));
    assert_eq!(
        error.location.source,
        graph.files().first().unwrap().source()
    );
}
#[test]
fn terminal_enum_maximum_does_not_require_a_representable_successor() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Maximum :: enum u64 #specified { LAST :: 18446744073709551615; } Bits :: enum_flags u64 #specified { HIGH :: 9223372036854775808; }",
    )]);
    let graph = fixture.graph();
    let mut types = TypeRegistry::new();
    let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
    nominals
        .define_enums(&graph, &mut types, &mut |_, _, span| {
            Err(Diagnostic::new(span, "unexpected enum dependency"))
        })
        .unwrap();
    types.freeze().unwrap();
}
