use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::resolve_graph;
use jai_vm::{Limits, Outcome, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-polymorphic-calls-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(source: &str) -> (jai_ir::Program, i128) {
    let fixture = Fixture::new(source);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let Outcome::Complete(values) = jai_vm::execute(&program, Limits::default()).outcome else {
        panic!("compiled generic call must execute");
    };
    let [Value::Int(result)] = values.as_slice() else {
        panic!("expected one int result");
    };
    let result = result.value();
    (program, result)
}

#[test]
fn scalar_generic_body_is_compiled_and_reused() {
    let (program, result) = run(
        "twice :: (x:$T) -> T { return x+x; } unused :: (x:$U) -> U { return x; } main :: () -> int { return twice(9) + twice(3); }",
    );
    assert_eq!(result, 24);
    assert_eq!(
        program.procedures().len(),
        2,
        "unused templates have no compiled procedure identity"
    );
}

#[test]
fn executable_callback_policies_separate_bodies_and_share_equal_source_contracts() {
    let (program, result) = run(
        "counter:int=0; next :: ()->int {counter+=1;return counter;} \
         left :: (#discard ignored:int,value:int)->int {return value;} \
         left_again :: (#discard ignored:int,value:int)->int {return value+3;} \
         right :: (value:int,#discard ignored:int)->int {return value;} \
         apply :: (callback:$T)->int {return callback(next(),next());} \
         main :: ()->int {a:=apply(left);b:=apply(left_again);c:=apply(right);return a*100+b*10+c+counter*1000;}",
    );
    assert_eq!(result, 3153);
    let bodies = program
        .procedures()
        .iter()
        .filter(|procedure| {
            let signature = program.types().procedure_definition(procedure.signature).unwrap();
            matches!(signature.parameters.as_ref(), [ty] if matches!(program.types().kind(*ty), Ok(jai_types::TypeKind::Procedure(_))))
        })
        .count();
    assert_eq!(
        bodies, 2,
        "equal source policies share a body; opposite discarded source slots do not"
    );
}

#[test]
fn overload_selection_and_aliases_preserve_original_declarations() {
    let (_, result) = run(
        "pick :: (x:int) -> int { return x+2; } pick :: (x:bool) -> int { if x return 19; return 3; } choose :: pick; main :: () -> int { return choose(true)+pick(5); }",
    );
    assert_eq!(result, 26);
}

#[test]
fn recursive_generic_call_uses_the_reserved_signature() {
    let (program, result) = run(
        "factorial :: (n:$T) -> T { if n <= 1 return 1; return n*factorial(n-1); } main :: () -> int { return factorial(5); }",
    );
    assert_eq!(result, 120);
    assert_eq!(program.procedures().len(), 2);
}

#[test]
fn baked_integer_is_definition_scope_constant_and_has_no_runtime_slot() {
    let (program, result) = run(
        "add :: ($N:int, value:int) -> int { return N+value; } main :: () -> int { return add(5,7)+add(cast(int) 5,2); }",
    );
    assert_eq!(result, 19);
    assert_eq!(
        program.procedures().len(),
        2,
        "literal and typed baked integers canonicalize to one specialization"
    );
    let specialized = program
        .procedures()
        .iter()
        .find(|procedure| procedure.parameters.len() == 1)
        .unwrap();
    assert_eq!(specialized.parameters.len(), 1);
}

#[test]
fn nominal_record_generic_uses_definition_type_overlay() {
    let (_, result) = run(
        "Box :: struct { value:int; } same :: (value:$T) -> T { copy:T=value; return copy; } main :: () -> int { box := same(Box.{value=17}); return box.value; }",
    );
    assert_eq!(result, 17);
}

#[test]
fn fixed_array_type_and_count_are_inferred_structurally() {
    let (_, result) = run(
        "first :: (values:[$N]$T) -> T { copy:[N]T=values; return copy[0]; } main :: () -> int { values:[3]int=int.[11,13,17]; return first(values); }",
    );
    assert_eq!(result, 11);
}

#[test]
fn callback_parameter_and_result_patterns_infer_from_the_real_signature() {
    let (_, result) = run(
        "combine :: (a:u8,b:u8) -> u16 { return cast(u16) a + cast(u16) b; } apply :: (values:[]$T, f:(T,T)->$R) -> R { return f(values[0],values[1]); } main :: () -> int { values:[2]u8=u8.[11,15]; return cast(int) apply(values,combine); }",
    );
    assert_eq!(result, 26);
}

#[test]
fn callback_result_pattern_is_inferred_from_a_contextual_lambda_body() {
    let (_, result) = run(
        "apply :: (values:[]$T, f:(T,T)->$R) -> R { return f(values[0],values[1]); } main :: () -> int { values:[2]u8=u8.[11,15]; return cast(int) apply(values,(a,b) => cast(u16) a+cast(u16) b); }",
    );
    assert_eq!(result, 26);
}

#[test]
fn callback_parameter_context_can_be_introduced_after_the_lambda() {
    let (_, result) = run(
        "apply :: (f:(T,T)->$R, values:[]$T) -> R { return f(values[0],values[1]); } main :: () -> int { values:[2]u8=u8.[7,9]; return cast(int) apply((a,b) => cast(u16) a+cast(u16) b,values); }",
    );
    assert_eq!(result, 16);
}

#[test]
fn named_dependent_lambdas_infer_results_in_their_definition_scope() {
    let (_, result) = run(
        "Output::u16; combine::(a,b)=>cast(Output)a+cast(Output)b; apply::(f:(T,T)->$R,values:[]$T)->R { return f(values[0],values[1]); } main::()->int { Output::bool; values:[2]u8=u8.[7,9]; return cast(int)apply(combine,values); }",
    );
    assert_eq!(result, 16);
    let (_, result) = run(
        "apply::(values:[]$T,f:(T,T)->$R)->R { return f(values[0],values[1]); } main::()->int { Output::u16; combine::(a,b)=>cast(Output)a+cast(Output)b; values:[2]u8=u8.[11,15]; return cast(int)apply(values,combine); }",
    );
    assert_eq!(result, 26);
}

#[test]
fn callback_result_variable_can_infer_the_canonical_void_result() {
    let (_, result) = run(
        "consume :: (value:int) {} accept :: (value:$T,f:(T)->$R) { f(value); } main :: () -> int { accept(9,(x) => consume(x)); return 7; }",
    );
    assert_eq!(result, 7);
}

#[test]
fn described_callbacks_retain_discarded_source_argument_slots() {
    let (_, result) = run(
        "counter:int=0; next::()->int { counter+=1; return 9; } choose::(#discard ignored:int,value:int)->int { return value; } skip::(#discard value:int) {} main::()->int { callback:=choose; skip(callback(next(),7)); return counter+3; }",
    );
    assert_eq!(result, 3);
}

#[test]
fn bound_layout_constants_fold_before_overload_and_baked_count_matching() {
    use jai_sema::{ResolveOptions, resolve_graph_with_options};
    use jai_types::{LayoutPolicy, ScalarLayout};
    let pointer32 = LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 8),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
        ScalarLayout::new(1, 1),
    )
    .unwrap();
    let fixture = Fixture::new(
        "pick::(count:int)->int { return count; } pick::(count:bool)->int { return 99; } add::($N:int,value:int)->int { storage:[N]u8; return N+value; } main::()->int { return pick(size_of(*int)*2)+add(size_of(*int)+1,3); }",
    );
    let graph = fixture.graph();
    for (layout, expected) in [(LayoutPolicy::lp64(), 28), (pointer32, 16)] {
        let program = resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                layout: Some(layout),
                ..ResolveOptions::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        let Outcome::Complete(values) = jai_vm::execute(&program, Limits::default()).outcome else {
            panic!("bound layout constants must execute");
        };
        let [Value::Int(value)] = values.as_slice() else {
            panic!("expected one integer result");
        };
        assert_eq!(value.value(), expected);
    }
}

#[test]
fn callback_patterns_preserve_parameter_identity_instead_of_widening() {
    let fixture = Fixture::new(
        "combine :: (a:u16,b:u16) -> u16 { return a+b; } apply :: (values:[]$T, f:(T,T)->$R) -> R { return f(values[0],values[1]); } main :: () -> int { values:[2]u8=u8.[11,15]; return cast(int) apply(values,combine); }",
    );
    assert!(resolve_graph(&fixture.graph()).is_err());
}

#[test]
fn discarded_generic_formals_infer_types_without_evaluating_arguments() {
    let (program, result) = run(
        "counter:int=0; next :: () -> int { counter+=1; return 9; } choose :: (#discard ignored:$T,value:T) -> T { return value; } main :: () -> int { return choose(next(),5)+counter*100; }",
    );
    assert_eq!(result, 5);
    assert!(
        program
            .procedures()
            .iter()
            .any(|procedure| procedure.parameters.len() == 1)
    );
}

#[test]
fn discarded_formals_cannot_be_read_from_a_specialized_body() {
    let fixture = Fixture::new(
        "read :: (#discard ignored:$T) -> T { return ignored; } main :: () -> int { return read(7); }",
    );
    assert!(
        resolve_graph(&fixture.graph())
            .unwrap_err()
            .message
            .contains("discard")
    );
}

#[test]
fn unrestricted_generic_and_equal_concrete_overload_are_ambiguous() {
    let fixture = Fixture::new(
        "pick :: (x:$T) -> T { return x; } pick :: (x:int) -> int { return x; } main :: () -> int { return pick(3); }",
    );
    assert!(
        resolve_graph(&fixture.graph())
            .unwrap_err()
            .message
            .contains("ambiguous")
    );
}

#[test]
fn selected_runtime_arguments_execute_once_in_source_order() {
    let (_, result) = run(
        "sequence:int=0; next :: () -> int { sequence=sequence*10+1; return sequence; } pair :: (a:$T,b:T)->T { return a*100+b; } main :: () -> int { result:=pair(b=next(),a=next()); return result+sequence*10000; }",
    );
    assert_eq!(result, 111101);
}

#[test]
fn weak_compound_float_keeps_the_selected_overload_context() {
    let (_, result) = run(
        "pick :: (x:float32) -> int { if x == 2.0 return 7; return 9; } pick :: (x:bool) -> int { return 42; } main :: () -> int { return pick(1.000000001+1.0); }",
    );
    assert_eq!(result, 7);
}

#[test]
fn typed_numeric_operand_determines_generic_expression_type() {
    let (program, result) = run(
        "same :: (x:$T) -> T { return x; } main :: () -> int { small:u8=7; result:=same(small+1); return cast(int) result; }",
    );
    assert_eq!(result, 8);
    let generic = program
        .procedures()
        .iter()
        .find(|procedure| !procedure.parameters.is_empty())
        .unwrap();
    let signature = program
        .types()
        .procedure_definition(generic.signature)
        .unwrap();
    assert_eq!(
        signature.results.as_ref(),
        [program
            .types()
            .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8))]
    );
}

#[test]
fn polymorphic_varargs_are_an_ordered_slice_and_fixed_overload_wins_ties() {
    let (_, result) = run(
        "min :: (a:$T,b:T)->T { if a<b return a; return b; } min :: (a:$T,rest:..T)->T { result:=a; for value:rest { if value<result result=value; } return result+100; } main :: () -> int { return min(9,3)+min(11,7,5); }",
    );
    assert_eq!(result, 108);
}

#[test]
fn forwarded_generic_slice_preserves_the_pack_and_trailing_parameter() {
    let (_, result) = run(
        "sum :: (values:..$T, extra:T) -> T { total:=extra; for value:values total+=value; return total; } forward :: (values:[]int) -> int { return sum(..values,5); } main :: () -> int { values:[2]int=int.[7,11]; return forward(values); }",
    );
    assert_eq!(result, 23);
}

#[test]
fn generic_intrinsic_prototype_publishes_metadata_without_a_body() {
    let fixture = Fixture::new(
        "compare_and_swap :: (pointer:*$T,old:T,new:T) -> (success:bool,old_value:T) #intrinsic; main :: () -> int { value:int=3; success,previous:=compare_and_swap(*value,3,7); if success && previous==3 return value; return 0; }",
    );
    let program = jai_sema::resolve_graph_with_options(
        &fixture.graph(),
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    assert_eq!(
        program.procedures().len(),
        1,
        "a specialized intrinsic never enters the source body queue"
    );
    assert_eq!(program.library().prototypes().len(), 1);
    assert!(matches!(
        program.library().prototypes()[0].origin,
        jai_ir::PrototypeOrigin::Intrinsic(jai_ir::RuntimeIntrinsic::CompareAndSwap { .. })
    ));
    let outcome = jai_vm::execute(&program, Limits::default()).outcome;
    let Outcome::Complete(values) = outcome else {
        panic!("checked generic intrinsic must execute: {outcome:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one result");
    };
    assert_eq!(value.value(), 7);
}

#[test]
fn contextual_enum_members_match_overloads_and_baked_nominal_arguments() {
    let (_, result) = run(
        "Choice :: enum { Red; Blue; } pick :: (value:Choice) -> int { if value==.Red return 13; return 2; } pick :: (value:bool) -> int { return 1; } baked :: ($value:Choice) -> int { if value==.Red return 7; return 0; } main :: () -> int { return pick(.Red)+baked(.Red); }",
    );
    assert_eq!(result, 20);
    let fixture =
        Fixture::new("same :: (value:$T) -> T { return value; } main :: () { same(.Red); }");
    assert!(
        resolve_graph(&fixture.graph())
            .unwrap_err()
            .message
            .contains("cannot infer")
    );
}

#[test]
fn record_applications_are_type_valued_baked_arguments() {
    let (_, result) = run(
        "Box :: struct(Element:Type) { value:Element; } size :: ($Element:Type) -> int { local:Element; return 17; } main :: () -> int { return size(Box(int)); }",
    );
    assert_eq!(result, 17);
}

#[test]
fn contextual_record_fields_select_the_viable_nominal_overload() {
    let (_, result) = run(
        "Number :: struct { value:u8; extra:int=6; } Flag :: struct { value:bool; } pick :: (item:Number) -> int { return cast(int)item.value+item.extra; } pick :: (item:Flag) -> int { return 42; } main :: () -> int { return pick(.{value=7}); }",
    );
    assert_eq!(result, 13);
    let fixture = Fixture::new(
        "Number :: struct { value:u8; } pick :: (item:Number) {} pick :: (item:bool) {} main :: () { pick(.{unknown=7}); }",
    );
    assert!(
        resolve_graph(&fixture.graph())
            .unwrap_err()
            .message
            .contains("no overload matches")
    );
}

#[test]
fn contextual_arrays_check_each_element_and_infer_empty_counts() {
    let (_, result) = run(
        "pick :: (values:[2]u8) -> int { return cast(int)values[0]+cast(int)values[1]; } pick :: (value:bool) -> int { return 42; } count :: (values:[$N]u8) -> int { return N; } main :: () -> int { return pick(.[7,11])+count(.[]); }",
    );
    assert_eq!(result, 18);
    let fixture = Fixture::new(
        "pick :: (values:[2]u8) {} pick :: (value:bool) {} main :: () { pick(.[7,256]); }",
    );
    assert!(
        resolve_graph(&fixture.graph())
            .unwrap_err()
            .message
            .contains("no overload matches")
    );
}

#[test]
fn baked_record_literals_canonicalize_omitted_and_explicit_defaults() {
    let (program, result) = run(
        "Config :: struct { value:u8; extra:int=6; } read :: ($config:Config) -> int { return cast(int)config.value+config.extra; } main :: () -> int { return read(.{value=7})+read(Config.{extra=6,value=7}); }",
    );
    assert_eq!(result, 26);
    assert_eq!(
        program.procedures().len(),
        2,
        "both literal shapes share the same typed baked value"
    );
}

#[test]
fn universal_descriptor_literals_preserve_the_exact_overload_context() {
    let fixture = Fixture::new(
        "pick :: (value:Any) -> int { if value.type==null && value.value_pointer==null return 21; return 1; } pick :: (value:bool) -> int { return 0; } main :: () -> int { return pick(.{})+pick(Any.{}); }",
    );
    let options = jai_sema::ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..Default::default()
    };
    let mut effects = jai_vm::NoEffects;
    let program =
        jai_sema::resolve_graph_with_options(&fixture.graph(), &options, &mut effects).unwrap();
    let Outcome::Complete(values) = jai_vm::execute(&program, Limits::default()).outcome else {
        panic!("Any literal overload must execute");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one int result");
    };
    assert_eq!(value.value(), 42);
}
