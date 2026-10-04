//! Independently authored source runs through real graph resolution and checked VM IR.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let mut sources = SourceOverlay::new();
    sources
        .insert(
            Path::new("/jai-implicit-ifx/main.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    ModuleGraph::load_with_provider(
        Path::new("/jai-implicit-ifx/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap()
}
fn run(source: &str) -> i128 {
    let program = jai_sema::resolve_graph(&graph(source)).unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("VM did not complete: {execution:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected integer result: {values:?}");
    };
    value.value()
}
#[test]
fn scalar_subjects_and_omitted_defaults_keep_their_actual_values() {
    for source in [
        "main::()->int{value:int=42;return ifx value;}",
        "main::()->int{value:int=0;return (ifx value)+42;}",
        "main::()->int{value:int=42;return ifx value>5 else 7;}",
        "main::()->int{value:int=0;return ifx !value else 42;}",
        "main::()->int{value:float64=20.5;chosen:float64=ifx value;return cast(int)(chosen*2)+1;}",
        "main::()->int{value:float64=0.0;chosen:float64=ifx value else 21.0;return cast(int)(chosen*2);}",
    ] {
        let expected = if source.contains("!value") {
            0
        } else {
            42
        };
        assert_eq!(run(source), expected, "{source}");
    }
}
#[test]
fn effectful_subject_is_evaluated_once_and_fallback_stays_lazy() {
    let source = "counter:int=0;next::()->int{counter+=1;return 20;}fallback::()->int{counter+=100;return 0;}main::()->int{value:=ifx next()>5 else fallback();if counter!=1 return -1;return value+22;}";
    assert_eq!(run(source), 42);
    let program = jai_sema::resolve_graph(&graph(source)).unwrap();
    assert!(format!("{:?}", program.procedures()).contains("Bind"));
    assert_eq!(
        run(
            "counter:int=0;zero::()->int{counter+=1;return 0;}fallback::()->int{counter+=10;return 31;}main::()->int{value:=ifx zero() else fallback();return value+counter;}"
        ),
        42
    );
}
#[test]
fn nested_direct_first_arguments_preserve_order_and_return_the_leaf_subject() {
    let source = "counter:int=0;next::()->int{counter+=1;return 20;}later::()->int{if counter!=1 return -100;counter+=1;return 22;}sum::(a:int,b:int)->int{return a+b;}accept::(value:int)->bool{return value==42;}main::()->int{value:=ifx accept(sum(next(),later())) else 0;if counter!=2 return -1;return value+22;}";
    assert_eq!(run(source), 42);
}
#[test]
fn one_boolean_unwrap_and_nested_captures_preserve_checked_types() {
    assert_eq!(
        run(
            "counter:int=0;next::()->int{counter+=1;return 20;}wrong::(value:int)->bool{return value!=20;}main::()->int{value:=ifx !wrong(next()) else 0;if counter!=1 return -1;return value+22;}"
        ),
        42
    );
    assert_eq!(
        run(
            "counter:int=0;next::()->int{counter+=1;return 20;}main::()->int{value:=ifx (ifx next()>5 else 0)>10 else 0;if counter!=1 return -1;return value+22;}"
        ),
        42
    );
    assert_eq!(
        run(
            "main::()->int{left:int=1;right:bool=true;value:=ifx left>0&&right else false;if value return 42;return 1;}"
        ),
        42
    );
}
#[test]
fn false_conditions_skip_right_operands_and_evaluate_fallback_once() {
    let source = "counter:int=0;zero::()->int{counter+=1;return 0;}fail::()->bool{denominator:int=0;return 1/denominator==0;}fallback::()->int{counter+=10;return 31;}main::()->int{value:=ifx zero()>0&&fail() then 99 else fallback();return value+counter;}";
    assert_eq!(run(source), 42);
    assert_eq!(
        run(
            "counter:int=0;zero::()->int{counter+=1;return 0;}fail::()->bool{denominator:int=0;return 1/denominator==0;}fallback::()->bool{counter+=10;return false;}main::()->int{value:=ifx zero()>0&&fail() else fallback();if value return -1;return counter+31;}"
        ),
        42
    );
}
#[test]
fn pointer_capture_retains_provenance_and_null_defaults() {
    assert_eq!(
        run(
            "value:int=42;counter:int=0;address::()->*int{counter+=1;return *value;}main::()->int{pointer:=ifx address();if counter!=1 return -1;return pointer.*;}"
        ),
        42
    );
    assert_eq!(
        run(
            "main::()->int{missing:*int;chosen:*int=ifx missing;if chosen==null return 42;return 1;}"
        ),
        42
    );
}
#[test]
fn sequence_truth_uses_actual_count_and_preserves_descriptors() {
    for source in [
        r#"main::()->int{text:string="hello";chosen:string=ifx text else "fallback";if chosen=="hello" return 42;return 1;}"#,
        r#"main::()->int{text:string="";chosen:string=ifx text else "fallback";if chosen=="fallback" return 42;return 1;}"#,
        "main::()->int{values:[2]int=.[20,22];view:[]int=values;chosen:[]int=ifx view;return chosen[0]+chosen[1];}",
        "main::()->int{view:[]int;chosen:[]int=ifx view;if chosen.count==0 return 42;return 1;}",
    ] {
        assert_eq!(run(source), 42, "{source}");
    }
}
#[test]
fn constants_parameters_and_module_domains_use_the_same_actual_subject() {
    let source = "ANSWER::ifx 42>5 else 0;main::()->int{return ANSWER;}";
    assert_eq!(run(source), 42);
    assert_eq!(
        run("pick::(value:int=ifx 42>5 else 0)->int{return value;}main::()->int{return pick();}"),
        42
    );
    assert_eq!(
        run("main::()->int{value:=#run (ifx 42>5 else 0);return value;}"),
        42
    );
}
#[test]
fn unsupported_subject_orders_and_branch_blocks_keep_precise_diagnostics() {
    for (source, message) in [
        (
            "f::(value:int)->bool{return true;}main::()->int{callback:=f;value:=ifx callback(42) else 0;return value;}",
            "indirect callback",
        ),
        (
            "main::()->int{value:=ifx is_constant(42) else 0;return value;}",
            "explicit then arm",
        ),
    ] {
        let error = jai_sema::resolve_graph(&graph(source)).unwrap_err();
        assert!(error.message.contains(message), "{error}");
    }
    let mut sources = SourceOverlay::new();
    sources
        .insert(
            Path::new("/jai-implicit-ifx/main.jai"),
            b"main::()->int{return ifx true {42;} else {0;}}".to_vec(),
        )
        .unwrap();
    assert!(
        ModuleGraph::load_with_provider(
            Path::new("/jai-implicit-ifx/main.jai"),
            GraphOptions::default(),
            &sources
        )
        .is_err()
    );
}
