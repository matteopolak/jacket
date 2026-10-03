//! Optional baking keeps runtime formals and constant specializations distinct.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn compile(source: &str) -> Result<jai_ir::Program, String> {
    let path = Path::new("/optional-baking/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay)
        .map_err(|error| error.to_string())?;
    jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..jai_sema::ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .map_err(|error| error.render(graph.sources()))
}

fn run(source: &str) -> (jai_ir::Program, i128) {
    let program = compile(source).unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("{execution:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    let result = value.value();
    (program, result)
}

#[test]
fn constants_are_baked_and_runtime_arguments_retain_their_slots() {
    let (program, result) = run("probe::(prefix:int,$$value:int)->int { \
         #if is_constant(value) {return prefix+value;} else {return 100+prefix+value;} \
         } main::()->int {value:int=5; \
         return probe(1,5)+probe(2,cast(int)5)+probe(3,value)+probe(4,6);}");
    assert_eq!(result, 131);
    let one_parameter = program
        .procedures()
        .iter()
        .filter(|procedure| {
            program
                .types()
                .procedure_definition(procedure.signature)
                .unwrap()
                .parameters
                .len()
                == 1
        })
        .count();
    assert_eq!(one_parameter, 2, "typed and weak 5 share one baked body");
    assert_eq!(
        program.procedures().len(),
        4,
        "main, two bakes, one runtime body"
    );
}

#[test]
fn optional_strings_distinguish_literal_constants_from_runtime_storage() {
    let (_, result) = run("length::($$text:string)->int { \
         #if is_constant(text) {return text.count;} else {return text.count+10;} \
         } main::()->int {text:string=\"abc\";return length(\"abc\")+length(text);}");
    assert_eq!(result, 16);
    let error = compile(
        "mutate::($$text:string) {#if is_constant(text) {#assert false \"read-only string\";}} \
         main::(){mutate(\"abc\");}",
    )
    .unwrap_err();
    assert!(error.contains("read-only string"), "{error}");
}

#[test]
fn constantness_queries_do_not_execute_ordinary_or_void_calls() {
    let (_, result) = run(
        "counter:int=0; next::()->int {counter+=1;return 7;} touch::(){counter+=1;} \
         main::()->int {value:int=7; \
         if is_constant(next()) || is_constant(touch()) || is_constant(value) return 1; \
         if !is_constant(7) || !is_constant(is_constant(next())) return 2; \
         return counter+42;}",
    );
    assert_eq!(result, 42);
}

#[test]
fn explicit_void_runs_inside_queries_execute_through_the_shared_engine() {
    let (_, result) =
        run("touch::() {} main::()->int {if is_constant(#run touch()) return 42;return 1;}");
    assert_eq!(result, 42);
    let error = compile(
        "fail::() {value:int=0;value=10/value;} main::()->int { \
         if is_constant(#run fail()) return 42;return 1;}",
    )
    .unwrap_err();
    assert!(
        error.contains("zero") || error.contains("arithmetic"),
        "{error}"
    );
}

#[test]
fn optional_baking_never_falls_back_after_a_constant_type_error() {
    let error = compile("probe::($$value:u8)->int{return value;}main::()->int{return probe(300);}")
        .unwrap_err();
    assert!(error.contains("out of range"), "{error}");
    let error =
        compile("probe::($$value:u8)->int{return value;}main::()->int{return probe(cast(u8)300);}")
            .unwrap_err();
    assert!(error.contains("constant materialization"), "{error}");
}

#[test]
fn explicit_float_enum_and_distinct_casts_retain_baked_constants() {
    let (program, result) = run("Choice::enum{ANSWER::21;} Count::#type,distinct u8; \
         probe::($$value:$T)->int{#if !is_constant(value) {return 1;} return cast(int)value;} \
         main::()->int{return probe(cast(float64)15.5)+probe(cast(Choice)21)+probe(cast(Count)6);}");
    assert_eq!(result, 42);
    assert_eq!(program.procedures().len(), 4);
    assert!(program.procedures().iter().all(|procedure| {
        program
            .types()
            .procedure_definition(procedure.signature)
            .unwrap()
            .parameters
            .is_empty()
    }));
}

#[test]
fn casts_of_bound_type_queries_share_the_same_baked_integer_body() {
    let (program, result) = run(
        "probe::($$value:int)->int{#if is_constant(value) {return value+20;} else {return 1;}} \
         main::()->int{return probe(cast(int)size_of(u8))+probe(cast(int)1);}",
    );
    assert_eq!(result, 42);
    assert_eq!(program.procedures().len(), 2);
}

#[test]
fn instance_namespace_constants_preserve_receiver_evaluation() {
    let (program, result) = run("Bag::struct($N:int){Size::N;value:int;} calls:int=0; \
         make::()->Bag(7){calls+=1;return .{value=7};} \
         probe::($$value:int)->int{#if is_constant(value) {return value+10;} else {return value;}} \
         main::()->int{bag:Bag(7)=.{};return probe(bag.Size)+probe(make().Size)+calls*18;}");
    assert_eq!(result, 42);
    assert_eq!(program.procedures().len(), 4);
}

#[test]
fn baked_discarded_parameters_remain_unreadable() {
    for marker in ["$", "$$"] {
        let error = compile(&format!(
            "probe::(#discard {marker}value:int)->int{{return value;}} \
             main::()->int{{return probe(7);}}"
        ))
        .unwrap_err();
        assert!(
            error.contains("#discard parameter cannot be read"),
            "{error}"
        );
    }
}

#[test]
fn nonscalar_distinct_casts_preserve_their_owned_constants() {
    let cases = [
        "Pair::struct{value:int;} Wrapped::#type,distinct Pair; pair::Pair.{value=42}; \
         probe::($$value:Wrapped)->int{#if is_constant(value) { \
         unwrapped:=cast(Pair)value;return unwrapped.value;} else {return 0;}} \
         main::()->int{return probe(cast(Wrapped)pair);}",
        "Wrapped::#type,distinct [2]int; \
         probe::($$value:Wrapped)->int{#if !is_constant(value) return 0; \
         view:=cast([]int)value;return view[0]+view[1];} \
         main::()->int{return probe(cast(Wrapped)int.[20,22]);}",
        "Wrapped::#type,distinct string; \
         probe::($$value:Wrapped)->int{#if !is_constant(value) return 0; \
         view:=cast([]u8)value;return view.count+39;} \
         main::()->int{return probe(cast(Wrapped)\"abc\");}",
    ];
    for source in cases {
        let (program, result) = run(source);
        assert_eq!(result, 42, "{source}");
        assert_eq!(program.procedures().len(), 2, "{source}");
    }
}

#[test]
fn exact_empty_slice_casts_and_null_wrappers_remain_baked() {
    let cases = [
        "Empty::struct{values:[]int;} empty::Empty.{}; \
         probe::($$value:[]int)->int{#if is_constant(value) {return value.count+42;} else {return 0;}} \
         main::()->int{return probe(cast([]int)empty.values);}",
        "Opaque::#type,distinct *void; \
         probe::($$value:Opaque)->int{#if !is_constant(value) return 0; \
         if cast(*void)value==null return 42;return 1;} \
         main::()->int{return probe(cast(Opaque)cast(*int)null);}",
        "answer::()->int{return 42;} \
         probe::($$callback:()->int)->int{#if is_constant(callback) {return callback();} else {return 0;}} \
         main::()->int{return probe(cast(()->int)answer);}",
    ];
    for source in cases {
        let (_, result) = run(source);
        assert_eq!(result, 42, "{source}");
    }
}
