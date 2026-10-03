use super::*;
use jai_modules::{GraphOptions, ModuleGraph};
use jai_types::{CallingConvention, ContextMode, TypeKind, Types};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-lexical-nominals-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.jai"), source).unwrap();
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
fn run(source: &str) -> i128 {
    let fixture = Fixture::new(source);
    let graph = fixture.graph();
    let program = crate::resolve_graph(&graph).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("VM did not complete: {execution:?}")
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}")
    };
    value.value()
}

#[test]
fn scalar_adapter_publishes_explicit_no_context_nested_body() {
    let module = syntax::parse(
        "main :: () -> int { add :: (value: int) -> int #no_context { return value + 7; } return add(5); }",
    ).unwrap();
    let program = crate::resolve(&module).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("VM did not complete: {execution:?}");
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    assert_eq!(value.value(), 12);
    assert_eq!(program.procedures().len(), 2);
}
fn record_with_scalar_field(types: &Types, integer: IntegerType) -> TypeId {
    types
        .iter()
        .find_map(|(ty, kind)| {
            if !matches!(kind, TypeKind::Record(_)) {
                return None;
            }
            let record = types.record_definition(ty).unwrap();
            let [field] = record.fields.as_ref() else {
                return None;
            };
            (types.kind(*field).unwrap() == &TypeKind::Integer(integer)).then_some(ty)
        })
        .expect("record with requested integer field")
}
#[test]
fn nested_annotation_shapes_use_local_alias_instead_of_equal_spelled_file_type() {
    let fixture = Fixture::new(
        "Outer :: struct { value: int; } main :: () -> int { Outer :: struct { value: u32; } Alias :: Outer; Container :: struct { fixed: [2] *Alias; view: [] Alias; growing: [..] Alias; callback: #type (*Alias) -> u32 #c_call #no_context; } return 0; }",
    );
    let graph = fixture.graph();
    let program = crate::resolve_graph(&graph).unwrap();
    let types = program.types();
    let local = record_with_scalar_field(types, IntegerType::U32);
    let file = record_with_scalar_field(types, IntegerType::S64);
    assert_ne!(local, file);
    let container = types
        .iter()
        .find_map(|(ty, kind)| {
            matches!(kind, TypeKind::Record(_))
                .then(|| types.record_definition(ty).unwrap())
                .filter(|record| record.fields.len() == 4)
        })
        .unwrap();
    let TypeKind::FixedArray {
        element,
        count,
    } = *types.kind(container.fields[0]).unwrap()
    else {
        panic!("expected fixed array")
    };
    assert_eq!(count, 2);
    assert_eq!(types.kind(element).unwrap(), &TypeKind::Pointer(local));
    assert_eq!(
        types.kind(container.fields[1]).unwrap(),
        &TypeKind::Slice(local)
    );
    assert_eq!(
        types.kind(container.fields[2]).unwrap(),
        &TypeKind::DynamicArray(local)
    );
    let callback = types.procedure_definition(container.fields[3]).unwrap();
    assert_eq!(callback.parameters.as_ref(), &[element]);
    assert_eq!(
        callback.results.as_ref(),
        &[types.scalar(ScalarType::Int(IntegerType::U32))]
    );
    assert_eq!(callback.convention, CallingConvention::C);
    assert_eq!(callback.context, ContextMode::None);
}
#[test]
fn local_record_defaults_and_nested_shadowing_restore_outer_type() {
    assert_eq!(
        run(
            "main :: () -> int { Point :: struct { x: int = 3; } total := 0; { Point :: struct { x: int = 7; } p: Point; total = p.x; } p: Point; return total + p.x; }"
        ),
        10
    );
}
#[test]
fn same_named_records_in_sibling_scopes_keep_distinct_identity() {
    let fixture = Fixture::new(
        "main :: () -> int { { Node :: struct { value: int; } p: *Node = null; } { Node :: struct { value: int; } p: *Node = null; } return 0; }",
    );
    let graph = fixture.graph();
    let program = crate::resolve_graph(&graph).unwrap();
    let records: Vec<_> = program
        .types()
        .iter()
        .filter_map(|(ty, kind)| {
            if !matches!(kind, TypeKind::Record(_)) {
                return None;
            }
            let record = program.types().record_definition(ty).unwrap();
            let [field] = record.fields.as_ref() else {
                return None;
            };
            (program.types().kind(*field).unwrap() == &TypeKind::Integer(IntegerType::S64))
                .then_some(ty)
        })
        .collect();
    assert_eq!(records.len(), 2);
    assert_ne!(records[0], records[1]);
}
#[test]
fn local_enum_members_and_record_defaults_preserve_nominal_values() {
    assert_eq!(
        run(
            "main :: () -> int { Fruit :: enum u32 #specified { APPLE :: 5; PEAR :: APPLE + 2; } Choice :: struct { fruit := Fruit.PEAR; } choice: Choice; if choice.fruit == Fruit.PEAR { return 7; } return 0; }"
        ),
        7
    );
}
#[test]
fn nested_procedure_captures_constant_without_runtime_environment() {
    assert_eq!(
        run(
            "main :: () -> int { BASE :: 7; add :: (value: int) -> int { return value + BASE; } return add(5); }"
        ),
        12
    );
}
#[test]
fn local_alias_cycle_is_a_diagnostic_without_recursive_type_allocation() {
    let fixture = Fixture::new(
        "main :: () -> int { A :: *B; B :: *A; Holder :: struct { value: A; } return 0; }",
    );
    let graph = fixture.graph();
    let error = crate::resolve_graph(&graph).unwrap_err();
    assert!(error.message.contains("cycl"), "{}", error.message);
    assert_eq!(error.location.source, graph.files()[0].source());
}

#[test]
fn lexical_procedure_type_keeps_jai_pack_element_and_c_fixed_prefix() {
    let fixture = Fixture::new(
        "main :: () -> int { Element :: struct { value: u32; } Callbacks :: struct { jai: #type (..Element) -> int; c: #type (int, ..Any) -> int #c_call #no_context; } return 0; }",
    );
    let graph = fixture.graph();
    let program = crate::resolve_graph(&graph).unwrap();
    let types = program.types();
    let element = record_with_scalar_field(types, IntegerType::U32);
    let callbacks = types
        .iter()
        .find_map(|(ty, kind)| {
            matches!(kind, TypeKind::Record(_))
                .then(|| types.record_definition(ty).unwrap())
                .filter(|record| record.fields.len() == 2)
        })
        .unwrap();
    let jai = types.procedure_definition(callbacks.fields[0]).unwrap();
    assert_eq!(
        jai.variadic,
        jai_types::Variadic::Jai {
            parameter: 0,
            element
        }
    );
    assert_eq!(jai.parameters.len(), 1);
    assert_eq!(
        types.kind(jai.parameters[0]).unwrap(),
        &TypeKind::Slice(element)
    );
    let c = types.procedure_definition(callbacks.fields[1]).unwrap();
    assert_eq!(
        c.variadic,
        jai_types::Variadic::C {
            fixed_parameters: 1
        }
    );
    assert_eq!(
        c.parameters.as_ref(),
        &[types.scalar(ScalarType::Int(IntegerType::S64))]
    );
    assert_eq!(c.convention, CallingConvention::C);
}

#[test]
fn recursive_local_record_pointer_defaults_to_null() {
    assert_eq!(
        run(
            "main :: () -> int { Node :: struct { next: *Node; value: int = 9; } node: Node; if node.next == null { return node.value; } return 0; }"
        ),
        9
    );
}

#[test]
fn forward_local_constant_dependencies_determine_array_field_extent() {
    assert_eq!(
        run(
            "main :: () -> int { Count :: Later + 1; Later :: 2; Batch :: struct { values: [Count] int; } batch: Batch; batch.values[2] = 7; return batch.values.count + batch.values[2]; }"
        ),
        10
    );
}

#[test]
fn forward_type_of_uses_annotated_runtime_type_without_loading_storage() {
    assert_eq!(
        run(
            "main :: () -> int { Alias :: type_of(value); value: u32 = 11; other: Alias = 5; return cast(int) other + cast(int) value; }"
        ),
        16
    );
}

#[test]
fn nested_procedure_alias_preserves_named_arguments_and_default_values() {
    assert_eq!(
        run(
            "main :: () -> int { BASE :: 7; add :: (value: int = BASE, extra: int = 1) -> int { return value + extra; } Alias :: add; return Alias(extra = 5); }"
        ),
        12
    );
}

#[test]
fn nested_procedure_rejects_runtime_local_capture_at_source_reference() {
    let source =
        "main :: () -> int { runtime := 9; read :: () -> int { return runtime; } return read(); }";
    let fixture = Fixture::new(source);
    let graph = fixture.graph();
    let error = crate::resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("capture") && error.message.contains("runtime"),
        "{}",
        error.message
    );
    assert_eq!(error.location.source, graph.files()[0].source());
    assert_eq!(error.location.span.text(source), "runtime");
}

#[test]
fn local_record_namespace_types_aliases_and_constants_feed_fields_and_defaults() {
    assert_eq!(
        run(
            "main :: () -> int { Outer :: struct { Count :: 3; Nested :: struct { value: int = Count; } Alias :: Nested; nested: Alias; values: [Count] int; value: int = Count; } outer: Outer; outer.values[2] = 5; return outer.nested.value + outer.values.count + outer.value + outer.values[2]; }"
        ),
        14
    );
}

#[test]
fn forward_qualified_local_record_constant_resolves_before_record_definition() {
    assert_eq!(
        run(
            "main :: () -> int { Value :: Outer.CONST; Outer :: struct { CONST :: 8; number: int = CONST; } outer: Outer; return Value + outer.number; }"
        ),
        16
    );
}

#[test]
fn local_record_namespace_enum_alias_keeps_qualified_member_identity() {
    assert_eq!(
        run(
            "main :: () -> int { Outer :: struct { State :: enum u8 #specified { NONE :: 0; READY :: 4; } Alias :: State; state: Alias = State.READY; } choice: Outer; if choice.state == Outer.State.READY { return cast(int) choice.state; } return 0; }"
        ),
        4
    );
}

#[test]
fn local_record_namespace_defaults_keep_definition_scope_under_caller_shadow() {
    assert_eq!(
        run(
            "main :: () -> int { VALUE :: 3; Container :: struct { VALUE :: 7; Inner :: struct { x: int = VALUE; } child: Inner; } total := 0; { VALUE :: 99; child: Container.Inner; total = child.x; } box: Container; return total + box.child.x + Container.VALUE; }"
        ),
        21
    );
}

#[test]
fn local_record_insert_requires_record_member_expansion() {
    let fixture =
        Fixture::new("main :: () -> int { Holder :: struct { #insert generated; } return 0; }");
    let graph = fixture.graph();
    let error = crate::resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("record") && error.message.contains("insert"),
        "unexpected record insertion diagnostic: {}",
        error.message
    );
    assert_eq!(error.location.source, graph.files()[0].source());
}

#[test]
fn local_record_reflection_preserves_field_note_local_flag_and_initializer() {
    let fixture = Fixture::new(
        r#"
        main :: () -> int {
            Pair :: struct { value: int = 42; @JsonIgnore }
            info := type_info(Pair);
            if info.members.count != 1 return 1;
            if info.members[0].notes.count != 1 return 2;
            note := info.members[0].notes[0];
            expected := "JsonIgnore";
            if note.count != expected.count return 3;
            for i: 0..expected.count-1 {
                if note[i] != expected[i] return 4;
            }
            if (cast(int) info.status_flags & 4) != 4 return 5;
            pair := initializer_of(Pair);
            return pair.value;
        }
    "#,
    );
    let graph = fixture.graph();
    let options = crate::ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..crate::ResolveOptions::default()
    };
    let program =
        crate::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    assert!(
        matches!(&outcome, jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
        "reflection execution: {outcome:?}"
    );
}

#[test]
fn generic_body_local_record_identity_is_reused_for_repeated_concrete_run() {
    let fixture = Fixture::new(
        "nominal :: (value:$T) -> T { Local :: struct { value:T; } result:Local = .{value=value}; return result.value; } main :: () -> int { first:s32 = nominal(cast(s32) 3); second:u32 = nominal(cast(u32) 5); repeated :: #run nominal(cast(u32) 8); return cast(int) first + cast(int) second + cast(int) repeated; }",
    );
    let graph = fixture.graph();
    let program = crate::resolve_graph(&graph).unwrap();
    let types = program.types();
    let field_kinds: Vec<_> = types
        .iter()
        .filter_map(|(ty, kind)| {
            if !matches!(kind, TypeKind::Record(_)) {
                return None;
            }
            let definition = types.record_definition(ty).unwrap();
            let [field] = definition.fields.as_ref() else {
                return None;
            };
            match types.kind(*field).unwrap() {
                TypeKind::Integer(integer @ (IntegerType::S32 | IntegerType::U32)) => {
                    Some(*integer)
                }
                _ => None,
            }
        })
        .collect();
    assert_eq!(
        field_kinds.len(),
        2,
        "one local nominal per concrete owner: {field_kinds:?}"
    );
    assert_eq!(
        field_kinds
            .iter()
            .filter(|&&integer| integer == IntegerType::S32)
            .count(),
        1
    );
    assert_eq!(
        field_kinds
            .iter()
            .filter(|&&integer| integer == IntegerType::U32)
            .count(),
        1
    );
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    assert!(
        matches!(&outcome, jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 16)),
        "generic execution: {outcome:?}"
    );
}

#[test]
fn record_application_arguments_read_local_alias_and_forward_weak_constant() {
    assert_eq!(
        run(
            "Box :: struct(T:Type,N:u32) { values:[N]T; } main :: ()->int { Element :: u32; Count :: Later+1; Later :: 2; value:Box(Element,Count); return value.values.count; }"
        ),
        3
    );
}

#[test]
fn record_application_arguments_read_qualified_local_namespace_members() {
    assert_eq!(
        run(
            "Box :: struct(T:Type,N:u32) { values:[N]T; } main :: ()->int { Types :: struct { Element :: u32; Count :: Later+1; Later :: 3; } value:Box(Types.Element,Types.Count); return value.values.count; }"
        ),
        4
    );
}

#[test]
fn record_template_default_reads_defining_file_instead_of_caller_shadow() {
    assert_eq!(
        run(
            "Count :: 3; Box :: struct(N:int=Count) { values:[N]int; } main :: ()->int { Count :: 7; value:Box(); return value.values.count; }"
        ),
        3
    );
}

#[test]
fn local_type_and_runtime_declarations_shadow_file_template_names() {
    for source in [
        "Box :: struct(T:Type) { value:T; } main :: () { Box :: struct { value:int; } value:Box(int); }",
        "Box :: struct(T:Type) { value:T; } main :: () { Box := 3; value:Box(int); }",
    ] {
        let fixture = Fixture::new(source);
        let graph = fixture.graph();
        let error = crate::resolve_graph(&graph).unwrap_err();
        assert!(
            error.message.contains("lexical declaration")
                && error.message.contains("parameterized record template"),
            "unexpected shadow diagnostic: {error:?}"
        );
        assert_eq!(error.location.source, graph.files()[0].source());
    }
}

#[test]
fn record_application_preserves_explicit_local_nominal_element_initializer() {
    assert_eq!(
        run(
            "Box :: struct(T:Type,N:u32) { values:[N]T; } main :: ()->int { Item :: struct { value:u32; } box:Box(Item,1)=.{values=.[.{value=13}]}; return cast(int) box.values[0].value; }"
        ),
        13
    );
}

#[test]
fn initializer_of_in_own_local_field_default_diagnoses_a_cycle() {
    let fixture = Fixture::new(
        "main :: () { Node :: struct { value:int=initializer_of(Node).value; } value:Node; }",
    );
    let graph = fixture.graph();
    let error = crate::resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("cycle") || error.message.contains("cyclic"),
        "expected recursive initializer diagnostic: {error:?}"
    );
    assert_eq!(error.location.source, graph.files()[0].source());
}

#[test]
fn reserved_runtime_local_does_not_fall_back_to_same_spelled_file_constant() {
    let source = "Value::9; main::()->int{Answer::Value;Value:=3;return Answer;}";
    let fixture = Fixture::new(source);
    let graph = fixture.graph();
    let error = crate::resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("runtime local") && error.message.contains("before its declaration"),
        "unexpected forward runtime lookup: {error:?}"
    );
    assert_eq!(error.location.source, graph.files()[0].source());
    assert_eq!(error.location.span.text(source), "Value");
}

#[test]
fn local_record_methods_reserve_mutual_signatures_and_keep_definition_scope() {
    assert_eq!(
        run(r#"
main :: () -> int {
    OFFSET :: 3;
    Numbers :: struct {
        OFFSET :: 7;
        Item :: struct { value: int = OFFSET; }
        even :: (n: int) -> bool { if n == 0 return true; return odd(n-1); }
        odd :: (n: int) -> bool { if n == 0 return false; return even(n-1); }
        answer :: (amount: int = 5) -> int { item: Item; return item.value+amount; }
    }
    Alias :: Numbers.answer;
    { OFFSET :: 99; if Numbers.even(4) return Alias(amount=11)+Numbers.answer(); }
    return 0;
}
"#),
        30
    );
}

#[test]
fn local_record_method_rejects_capture_of_runtime_outer_storage() {
    let source = "main :: () -> int { runtime := 9; Holder :: struct { read :: () -> int { return runtime; } } return Holder.read(); }";
    let fixture = Fixture::new(source);
    let graph = fixture.graph();
    let error = crate::resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("runtime") && error.message.contains("capture"),
        "unexpected method capture diagnostic: {error:?}"
    );
    assert_eq!(error.location.source, graph.files()[0].source());
    assert_eq!(error.location.span.text(source), "runtime");
}

#[test]
fn local_record_intrinsic_prototype_publishes_namespace_signature_and_executes() {
    let fixture = Fixture::new(
        r#"
main :: () -> int {
    Memory :: struct { fill :: (dest: *void, value: u8, count: s64) #intrinsic "memset"; }
    Alias :: Memory.fill;
    value: u64 = 0;
    Alias(cast(*void) *value, 42, 8);
    return cast(int) (value & 255);
}
"#,
    );
    let graph = fixture.graph();
    let options = crate::ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..crate::ResolveOptions::default()
    };
    let program =
        crate::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
    let [prototype] = program.library().prototypes() else {
        panic!("expected one local intrinsic prototype");
    };
    assert!(matches!(prototype.origin, PrototypeOrigin::Intrinsic(_)));
    assert!(program.procedure_by_id(prototype.id).is_none());
    let descriptor = program
        .types()
        .procedure_definition(prototype.signature)
        .unwrap();
    assert_eq!(descriptor.context, ContextMode::None);
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(&execution.outcome, jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
        "local intrinsic execution: {execution:?}"
    );
}

#[test]
fn local_inline_enum_members_use_lexical_constants_and_prior_members() {
    assert_eq!(
        run(r#"
main :: () -> int {
    BASE :: 5;
    value: enum u32 #specified { ZERO :: 0; FIRST :: BASE; NEXT :: FIRST+2; } = .NEXT;
    Holder :: struct {
        BASE :: 7;
        mode: enum u32 #specified { ZERO :: 0; FIRST :: BASE; NEXT :: FIRST+2; } = .NEXT;
    }
    holder: Holder;
    return cast(int) value+cast(int) holder.mode;
}
"#),
        16
    );
}

#[test]
fn caller_scope_insertion_keeps_imported_quote_notes_and_local_source_provenance() {
    let fixture = Fixture::new(
        "Quoted :: #import \"Quoted\"; main :: () -> int { CALLER :: 42; #insert,scope() Quoted.Quote; }",
    );
    std::fs::create_dir_all(fixture.0.join("modules")).unwrap();
    std::fs::write(
        fixture.0.join("modules/Quoted.jai"),
        r#"
CALLER :: 99;
Quote :: #code {
    Local :: struct { value: int = CALLER; @JsonIgnore }
    value: Local;
    info := type_info(Local);
    if info.members[0].notes.count != 1 return 1;
    note := info.members[0].notes[0];
    expected := "JsonIgnore";
    if note.count != expected.count return 2;
    for i: 0..expected.count-1 { if note[i] != expected[i] return 3; }
    return value.value;
};
"#,
    )
    .unwrap();
    let graph = fixture.graph();
    let options = crate::ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..crate::ResolveOptions::default()
    };
    let program =
        crate::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
    let debug = program.library().debug_sources().unwrap();
    let (_, value_source) = debug
        .locals()
        .find(|(_, source)| source.name == "value")
        .unwrap();
    assert!(value_source.location.path().ends_with("modules/Quoted.jai"));
    assert_ne!(
        value_source.location.span().source,
        graph.files()[0].source()
    );
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(&execution.outcome, jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
        "cross-file quote execution: {execution:?}"
    );
}

#[test]
fn local_record_root_default_override_constructs_independent_values() {
    assert_eq!(
        run(
            "main :: ()->int { Local :: struct { value:int=3; value=17; untouched:int=5; } first:Local; first.value=99; second:Local; return second.value+second.untouched; }"
        ),
        22
    );
}

#[test]
fn local_record_nested_enum_override_preserves_embedded_defaults_and_base_types() {
    assert_eq!(
        run(r#"
main :: () -> int {
    State :: enum u8 #specified { NONE :: 0; READY :: 5; }
    Base :: struct { kind := State.NONE; untouched: int = 11; }
    Middle :: struct { using base: Base; other: int = 13; }
    Derived :: struct { using middle: Middle; own: int = 17; middle.base.kind = .READY; }
    base: Base;
    middle: Middle;
    derived: Derived;
    if base.kind != State.NONE || middle.kind != State.NONE return 1;
    if derived.kind != State.READY return 2;
    if derived.untouched != 11 || derived.other != 13 || derived.own != 17 return 3;
    if base.untouched != 11 || middle.untouched != 11 || middle.other != 13 return 4;
    return 42;
}
"#),
        42
    );
}

#[test]
fn local_record_default_overrides_apply_selected_writes_in_source_order() {
    assert_eq!(
        run(r#"
main :: () -> int {
    Local :: struct {
        value: int = 3;
        value = 7;
        #if ACTIVE { value = 11; } else { value = Missing; }
        value = 13;
        #if false { value = Missing; } else { value = 17; }
        ACTIVE :: true;
    }
    value: Local;
    return value.value;
}
"#),
        17
    );
}

#[test]
fn local_record_default_override_rejects_array_index_targets() {
    let source = "main :: () { Local :: struct { values:[2]int; values[0]=7; } value:Local; }";
    let fixture = Fixture::new(source);
    let graph = fixture.graph();
    let error = crate::resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("override") && error.message.contains("field"),
        "unexpected array override diagnostic: {error:?}"
    );
    assert_eq!(error.location.source, graph.files()[0].source());
    assert!(error.location.span.text(source).contains("values[0]"));
}

#[test]
fn record_method_omitted_record_default_uses_complete_field_initializers() {
    assert_eq!(
        run(
            "R :: struct { value:int=21; } Methods :: struct { read :: (value:R=R.{}) -> int { return value.value; } } Alias :: Methods.read; main :: ()->int { return Methods.read()+Alias(); }"
        ),
        42
    );
}

#[test]
fn record_method_context_default_uses_complete_context_registration() {
    assert_eq!(
        run(
            "#add_context marker:int=7; Methods :: struct { read :: (saved:#Context=.{})->int { return saved.marker; } } Alias :: Methods.read; main :: ()->int { return Methods.read()+Alias(); }"
        ),
        14
    );
}

#[test]
fn record_method_run_default_waits_for_source_body_readiness() {
    assert_eq!(
        run(
            "R :: struct { value:int=7; } make :: ()->R { return R.{value=21}; } Methods :: struct { read :: (value:R=#run make())->int { return value.value; } } Alias :: Methods.read; main :: ()->int { return Methods.read()+Alias(); }"
        ),
        42
    );
}

#[test]
fn local_record_method_own_literal_default_waits_for_owner_field_defaults() {
    assert_eq!(
        run(
            "main :: ()->int { R :: struct { value:int=21; read :: (input:R=R.{}) ->int { return input.value; } } return R.read()+R.read(); }"
        ),
        42
    );
}

#[test]
fn local_record_method_own_literal_default_waits_for_owner_run_field_default() {
    assert_eq!(
        run(
            "main :: ()->int { R :: struct { value:int=#run seed(); seed :: ()->int { return 21; } read :: (input:R=R.{}) ->int { return input.value; } } return R.read()+R.read(); }"
        ),
        42
    );
}

#[test]
fn exported_record_method_alias_retains_private_owner_initializer() {
    let fixture = Fixture::new(
        "Lib::#import,file \"library.jai\"; main::()->int{return Lib.answer()+Lib.alias();}",
    );
    fs::write(
        fixture.0.join("library.jai"),
        "#scope_module; R::struct{value:int=#run seed();seed::()->int{return 21;}read::(input:R=R.{}) ->int{return input.value;}} #scope_export; answer::()->int{return R.read();} alias::R.read;",
    )
    .unwrap();
    let graph = fixture.graph();
    let program = crate::resolve_graph(&graph).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values)
            if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
}

#[test]
fn local_record_method_default_uses_final_construction_overrides() {
    assert_eq!(
        run(
            "main::()->int{R::struct{value:int=7; value=21; read::(input:R=R.{}) ->int{return input.value;}} return R.read()+R.read();}",
        ),
        42,
    );
}

#[test]
fn local_inferred_run_field_gets_its_type_before_its_seed_body() {
    assert_eq!(
        run(
            "main::()->int{R::struct{value:=#run seed();seed::()->int{return 21;}read::(input:R=R.{}) ->int{return input.value;}} return R.read()+R.read();}",
        ),
        42,
    );
}

#[test]
fn demanded_record_field_method_resolves_nested_local_seed_default() {
    assert_eq!(
        run(r#"
Factory :: struct {
    value: int = #run make();
    make :: () -> int {
        Inner :: struct {
            value: int = #run seed();
            seed :: () -> int { return 21; }
        }
        item: Inner;
        return item.value;
    }
}
main :: () -> int { value: Factory; return value.value*2; }
"#),
        42
    );
}

#[test]
fn full_pass_checks_unused_nested_method_after_outer_field_method_is_demanded() {
    let source = r#"
Factory :: struct {
    value: int = #run make();
    make :: () -> int {
        Inner :: struct {
            value: int = #run seed();
            seed :: () -> int { return 21; }
            unused :: () -> int { return MISSING; }
        }
        item: Inner;
        return item.value;
    }
}
main :: () -> int { value: Factory; return value.value*2; }
"#;
    let fixture = Fixture::new(source);
    let graph = fixture.graph();
    let error = crate::resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("MISSING"),
        "expected unused nested method body diagnostic: {error:?}"
    );
    assert_eq!(error.location.source, graph.files()[0].source());
    assert_eq!(error.location.span.text(source), "MISSING");
}

#[test]
fn local_required_and_optional_baking_report_missing_lexical_specialization_origin() {
    for marker in ["$", "$$"] {
        let source = format!(
            "main :: ()->int {{ local :: ({marker}value:int)->int {{ return value; }} return 0; }}"
        );
        let fixture = Fixture::new(&source);
        let graph = fixture.graph();
        let error = crate::resolve_graph(&graph).unwrap_err();
        assert!(
            error.message.contains("lexical specialization origin"),
            "unexpected {marker} local header diagnostic: {error:?}"
        );
        assert_eq!(error.location.source, graph.files()[0].source());
        assert_eq!(
            error.location.span.text(&source),
            format!("{marker}value:int")
        );
    }
}
