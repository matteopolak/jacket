//! Expanded calls share checked IR while retaining caller code and private locals.
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::sync::atomic::{AtomicUsize, Ordering};

fn run(source: &str) -> Result<i128, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-macros-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&scratch.0).unwrap();
    let path = scratch.0.join("main.jai");
    std::fs::write(&path, source).unwrap();
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
        outcome => Err(format!("macro execution: {outcome:?}")),
    }
}

#[test]
fn implicit_code_arguments_preserve_places_and_private_macro_temporaries() {
    let source = "swap::(a:Code,b:Code)#expand { t:=(#insert a); (#insert a)=(#insert b); (#insert b)=t; } main::()->int { a:=4; b:=2; t:=100; swap(a,b); swap(a,b); return a*10+b+(t-100); }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn discarded_runtime_and_run_inputs_are_checked_without_execution_or_stores() {
    assert_eq!(
        run(include_str!("fixtures/discarded-macro-inputs.jai")).unwrap(),
        42
    );
}

#[test]
fn discarded_defaults_and_variadic_inputs_do_not_evaluate_source_arguments() {
    assert_eq!(run("calls:int; tick::()->int{calls+=1;return 9;} ignore::(#discard message:=\"\",#discard arguments:..Any)#expand{} main::()->int{ignore();ignore(\"message\",tick(),#run tick());return 42+calls*100;}").unwrap(), 42);
}

#[test]
fn annotated_discarded_defaults_are_checked_without_execution() {
    assert_eq!(run("calls:int; tick::()->int{calls+=1;return 9;} ignore::(#discard value:int=tick())#expand{} main::()->int{ignore();return 42+calls*100;}").unwrap(),42);
    let error = run("ignore::(#discard value:int=true)#expand{} main::()->int{ignore();return 0;}")
        .unwrap_err();
    assert!(
        error.contains("argument type cannot be implicitly converted"),
        "{error}"
    );
}

#[test]
fn discarded_defaults_use_the_macros_captured_definition_environment() {
    assert_eq!(run("main::()->int{DEFAULT::42; ignore::(#discard value:int=DEFAULT)#expand{} {DEFAULT::true;ignore();}return 42;}").unwrap(),42);
}

#[test]
fn discarded_inputs_still_require_valid_argument_types() {
    let error = run("ignore::(#discard value:int)#expand{} main::()->int{ignore(true);return 0;}")
        .unwrap_err();
    assert!(
        error.contains("argument type cannot be implicitly converted"),
        "{error}"
    );
    let error =
        run("ignore::(#discard value:int)#expand{} main::()->int{ignore(missing());return 0;}")
            .unwrap_err();
    assert!(
        error.contains("unknown") || error.contains("procedure"),
        "{error}"
    );
}

#[test]
fn discarded_formals_cannot_be_read_by_the_expanded_body() {
    let error = run("total:int; read::(#discard value:int)#expand{total=value;} main::()->int{read(42);return total;}").unwrap_err();
    assert!(error.contains("discard"), "{error}");
}

#[test]
fn declarations_in_inserted_code_can_shadow_unreadable_discarded_formals() {
    assert_eq!(run("total:int; ignore::(#discard value:int)#expand{body::#code{value:int=42;total=value;};#insert body;} main::()->int{ignore(1);return total;}").unwrap(),42);
}

#[test]
fn a_named_inferred_lambda_is_classified_without_eager_macro_value_binding() {
    assert_eq!(run("count:int;total:int;tick::(value:int)->int{count+=1;total=value+1;return total;} main::()->int{sink::(value)=>tick(value);sink(41);return total+count-1;}").unwrap(),42);
}

#[test]
fn runtime_named_arguments_are_evaluated_once_in_source_order() {
    let source = "order:int; total:int; tick::(digit:int)->int { order=order*10+digit; return digit; } combine::(first:int,second:int)#expand { total=first+first+second+second; } main::()->int { combine(second=tick(2),first=tick(1)); if total!=6 return 0; return order; }";
    assert_eq!(run(source).unwrap(), 21);
}

#[test]
fn implicit_code_argument_runs_only_at_each_insertion() {
    let source = "count:int; tick::()->int { count+=1; return count; } twice::(body:Code)#expand { #insert body; #insert body; } main::()->int { twice(tick()); return count; }";
    assert_eq!(run(source).unwrap(), 2);
}

#[test]
fn a_file_code_argument_keeps_its_quoted_statement_body() {
    let source = "total:int; BODY::#code { total+=42; }; apply::(body:Code)#expand { #insert body; } main::()->int { apply(BODY); return total; }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn a_local_macro_captures_lexical_constants_and_storage_under_shadowing() {
    let source = "main::()->int { total:=0; OFFSET::40; apply::(amount:int)#expand { total=OFFSET+amount; } { OFFSET::100; apply(2); } return total; }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn a_local_macro_resolves_its_formal_nominal_type_in_its_defining_scope() {
    let source = "main::()->int { Pair::struct { value:s32; } total:=0; apply::(value:Pair)#expand { total=cast(int)value.value; } pair:Pair; pair.value=42; { Pair::struct { other:int; } apply(pair); } return total; }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn local_macro_recursion_and_runtime_values_have_precise_boundaries() {
    let error =
        run("main::()->int { again::()#expand { again(); } again(); return 0; }").unwrap_err();
    assert!(error.contains("cyclic #expand"), "{error}");
    let error = run("main::()->int { apply::()#expand {} value:=apply; return 0; }").unwrap_err();
    assert!(error.contains("source templates"), "{error}");
}

#[test]
fn a_return_in_inserted_caller_code_uses_the_callers_result_contract() {
    let source = "apply::(body:Code)#expand { #insert body; } main::()->int { apply(#code { return 42; }); }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn baked_scalar_arguments_acquire_the_formal_type() {
    let source = "total:int; add::($N:u8)#expand { total=cast(int)N+size_of(type_of(N)); } main::()->int { add(41); return total; }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn type_arguments_stay_semantic_values_in_macro_body_annotations() {
    let source = "total:int; apply::(T:Type,body:Code)#expand { value:T; total=size_of(type_of(value)); #insert body; } main::()->int { apply(s32,#code { total+=38; }); return total; }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn exported_storage_shadows_a_captured_caller_name_without_changing_that_storage() {
    let source = "total:int; apply::(body:Code)#expand { `x:=41; #insert body; } main::()->int { x:=100; { apply(#code { x+=1; total=x; }); } return total+(x-100); }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn exported_declarations_remain_visible_after_the_macro_statement() {
    assert_eq!(
        run(include_str!("fixtures/caller-export-persistence.jai")).unwrap(),
        42
    );
}

#[test]
fn caller_exports_reject_same_block_collisions() {
    let error = run("apply::()#expand{`value:=42;} main::()->int{value:=1;apply();return value;}")
        .unwrap_err();
    assert!(
        error.contains("caller export duplicates local declaration 'value'"),
        "{error}"
    );
}

#[test]
fn exported_constants_remain_typed_immutable_bindings() {
    let source = "total:int; apply::(body:Code)#expand { `answer::cast(u8)42; #insert body; } main::()->int { apply(#code { total=cast(int)answer; }); return total; }";
    assert_eq!(run(source).unwrap(), 42);
}

#[test]
fn a_caller_export_requires_an_active_macro_invocation() {
    let error = run("main::()->int { `x:=42; return x; }").unwrap_err();
    assert!(
        error.contains("caller exports require an active #expand invocation"),
        "{error}"
    );
}

#[test]
fn recursive_expansion_and_value_results_have_explicit_diagnostics() {
    let error =
        run("again::()#expand { again(); } main::()->int { again(); return 0; }").unwrap_err();
    assert!(error.contains("cyclic #expand"), "{error}");
    let error =
        run("value::()->int #expand { return 42; } main::()->int { return value(); }").unwrap_err();
    assert!(
        error.contains("statement position") || error.contains("expansion result binding"),
        "{error}"
    );
}

#[test]
fn only_the_selected_compile_time_branch_rejects_macro_body_returns() {
    let source = "total:int; apply::($active:bool)#expand { #if active { return; } else { total=42; } } main::()->int { apply(false); return total; }";
    assert_eq!(run(source).unwrap(), 42);
    let error = run("apply::($active:bool)#expand { #if active { return; } } main::()->int { apply(true); return 0; }").unwrap_err();
    assert!(
        error.contains("expanded procedure require expansion result binding"),
        "{error}"
    );
}

#[test]
fn runtime_effects_cannot_be_baked_as_a_scalar_constant() {
    let error = run("count:int; tick::()->int { count+=1; return count; } apply::($N:int)#expand {} main::()->int { apply(tick()); return count; }").unwrap_err();
    assert!(
        error.contains("constant") || error.contains("compile-time"),
        "{error}"
    );
}
