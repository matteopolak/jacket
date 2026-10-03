use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::sync::atomic::{AtomicUsize, Ordering};

fn run(source: &str) -> Result<i128, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "jai-code-values-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("main.jai");
    std::fs::write(&path, source).unwrap();
    let result = (|| {
        let graph =
            ModuleGraph::load(&path, GraphOptions::default()).map_err(|error| error.to_string())?;
        let options = ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..ResolveOptions::default()
        };
        let program = resolve_graph_with_options(&graph, &options, &mut NoEffects)
            .map_err(|error| error.to_string())?;
        match jai_vm::execute(&program, Limits::default()).outcome {
            Outcome::Complete(values) => match values.as_slice() {
                [Value::Int(value)] => Ok(value.value()),
                values => Err(format!("expected integer result: {values:?}")),
            },
            outcome => Err(format!("code fixture execution: {outcome:?}")),
        }
    })();
    std::fs::remove_dir_all(directory).unwrap();
    result
}

#[test]
fn expression_insertion_retains_definition_binding_under_shadowing() {
    assert_eq!(
        run("main::()->int { x:=3; quoted::#code x+2; { x:=100; return #insert quoted; } }")
            .unwrap(),
        5
    );
}

#[test]
fn file_code_constants_capture_without_resolving_their_quoted_declarations() {
    assert_eq!(
        run("deferred :: #code #add_context base: Unavailable_Until_Insertion; main :: () -> int { return 42; }").unwrap(),
        42,
    );
}

#[test]
fn file_code_aliases_preserve_the_defining_scope_at_insertion() {
    assert_eq!(
        run("file_value :: 40; quoted :: #code file_value + 2; alias :: quoted; main :: () -> int { file_value :: 99; return #insert alias; }").unwrap(),
        42,
    );
}

#[test]
fn explicit_scope_insertion_rebinds_at_the_current_scope() {
    assert_eq!(
        run(
            "main::()->int { x:=3; quoted::#code x+2; { x:=100; return #insert,scope() quoted; } }"
        )
        .unwrap(),
        102
    );
}

#[test]
fn assignment_insertion_denotes_the_original_storage() {
    assert_eq!(
        run("main::()->int { x:=3; slot::#code x; { x:=100; (#insert slot)=9; } return x; }")
            .unwrap(),
        9
    );
}

#[test]
fn block_insertion_updates_captured_storage_and_keeps_its_locals_scoped() {
    assert_eq!(run("main::()->int { x:=3; body::#code { temp:=7; x+=temp; }; { x:=100; #insert body; } return x; }").unwrap(), 10);
}

#[test]
fn code_capture_keeps_specialization_bindings_and_lexical_frame_depths() {
    let source = "measure::($T:Type)->int { body::#code { value:T; return size_of(type_of(value)); }; #insert body; } main::()->int { return measure(s32)*10+measure(u8)*2; }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn quoted_runs_do_not_reuse_results_from_a_different_baked_capture() {
    let source = "emit::($N:int,target:Code)#expand{quoted::#code #run N; (#insert target)=#insert quoted;} main::()->int{left:=0;right:=0;emit(2,left);emit(40,right);return left+right;}";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn one_quote_rebound_at_two_macro_invocations_keeps_each_runs_current_constants() {
    let source = "quoted::#code #run N; emit::($N:int,target:Code)#expand{ (#insert target)=#insert,scope() quoted;} main::()->int{left:=0;right:=0;emit(2,left);emit(40,right);return left+right;}";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn statement_insertion_round_trips_the_original_target_under_shadowing() {
    assert_eq!(
        run("main::()->int { x:=3; increment::#code x+=7; { x:=100; #insert increment; } return x; }")
            .unwrap(),
        10
    );
}

#[test]
fn current_scope_statement_insertion_round_trips_the_retained_statement_kind() {
    assert_eq!(
        run("main::()->int { x:=3; increment::#code x+=7; { x:=100; #insert,scope() increment; return x; } }")
            .unwrap(),
        107
    );
}

#[test]
fn expression_insertion_rejects_a_quoted_statement() {
    let error = run("main::()->int { body::#code x:=7; return #insert body; }").unwrap_err();
    assert!(
        error.contains("expression insertion requires exactly one quoted expression"),
        "{error}"
    );
}

#[test]
fn code_values_cannot_enter_runtime_storage() {
    let error = run("main::()->int { body::#code 7; value:=body; return 0; }").unwrap_err();
    assert!(
        error.contains("runtime storage")
            || error.contains("runtime representation")
            || error.contains("coercion"),
        "{error}"
    );
}

#[test]
fn null_code_keeps_a_compiler_metatype_without_creating_a_scope() {
    assert_eq!(run("main::()->int { absent::#code,null; info:=type_info(type_of(absent)); return cast(int)info.type; }").unwrap(), 14);
}

#[test]
fn null_scope_code_cannot_supply_inserted_syntax() {
    let error = run("main::()->int { absent::#code,null; #insert absent; return 0; }").unwrap_err();
    assert!(error.contains("#code,null has no body or scope"), "{error}");
}
