//! Expanded caller returns preserve real ownership, source contracts and cleanup order.
#[path = "support/supplied_caller_return.rs"]
mod supplied_caller_return;
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_source::LocatedDiagnostic;
use std::path::Path;

fn program(source: &str) -> Result<jai_ir::Program, LocatedDiagnostic> {
    program_with_files(source, &[])
}

fn program_with_files(
    source: &str,
    files: &[(&str, &str)],
) -> Result<jai_ir::Program, LocatedDiagnostic> {
    let path = Path::new("/own-caller-return/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    for &(name, contents) in files {
        overlay
            .insert(
                &Path::new("/own-caller-return").join(name),
                contents.as_bytes().to_vec(),
            )
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        path,
        GraphOptions {
            import_dirs: vec!["/own-caller-return".into()],
        },
        &overlay,
    )
    .unwrap();
    jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
}

fn check(source: &str) {
    let program = program(source).unwrap_or_else(|error| panic!("{source}\n{error}"));
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn operands_run_once_before_macro_and_caller_cleanups() {
    check(include_str!("fixtures/caller-return-lifecycle.jai"));
}

#[test]
fn named_results_and_defaults_use_the_caller_source_signature() {
    check(include_str!("fixtures/caller-return-named.jai"));
}

#[test]
fn nominal_aggregate_returns_keep_macro_storage_and_snapshot_before_cleanup() {
    check(include_str!("fixtures/caller-return-aggregate.jai"));
}

#[test]
fn nested_conditional_macros_return_from_the_actual_parent_procedure() {
    check(include_str!("fixtures/caller-return-nested.jai"));
}

#[test]
fn pushed_contexts_and_exported_cleanups_keep_their_own_contexts() {
    check(include_str!("fixtures/caller-return-context.jai"));
}

#[test]
fn macro_invocations_inside_inserted_code_keep_the_caller_return_target() {
    check(include_str!("fixtures/caller-return-quotes.jai"));
}

#[test]
fn returned_callback_retains_caller_alias_names_and_source_policy() {
    check(include_str!("fixtures/caller-return-callback.jai"));
}

#[test]
fn inferred_default_callback_returns_retain_the_original_default_arguments() {
    check(
        "target::(input:int=40)->int#must{return input+2;}finish::()#expand{`return target;}answer::()->(result:=target){finish();}main::()->int{result:=answer();return result();}",
    );
}

#[test]
fn ordinary_macro_break_and_continue_target_actual_caller_loops() {
    check(include_str!("fixtures/macro-loop-exits.jai"));
}

#[test]
fn void_bool_float_and_string_returns_are_checked_against_the_caller() {
    for source in [
        "total:int; finish::()#expand{defer total=42;`return;} answer::(){finish();} main::()->int{answer();return total;}",
        "finish::()#expand{`return true;} answer::()->bool{finish();} main::()->int{if answer() return 42;return 0;}",
        "finish::()#expand{`return 42.0;} answer::()->float64{finish();} main::()->int{return cast(int)answer();}",
        "finish::()#expand{`return \"answer\";} answer::()->string{finish();} main::()->int{return answer().count+36;}",
    ] {
        check(source);
    }
}

#[test]
fn caller_returns_require_an_active_invocation_and_matching_results() {
    for (source, message) in [
        ("main::()->int{`return 42;}", "active #expand"),
        (
            "finish::()#expand{`return true;} main::()->int{finish();}",
            "implicit integer conversion",
        ),
        (
            "finish::()#expand{`return;} main::()->int{finish();}",
            "missing required return",
        ),
        (
            "finish::()#expand{`return 1,2;} main::()->int{finish();}",
            "too many return",
        ),
        (
            "finish::()#expand{`return missing=42;} main::()->(answer:int){finish();}",
            "unknown named return",
        ),
        (
            "finish::()#expand{`return 42;} main::(){finish();}",
            "does not match procedure signature",
        ),
    ] {
        let error = program(source).unwrap_err();
        assert!(error.message.contains(message), "{source}\n{error}");
        assert_ne!(error.location.span.start, error.location.span.end);
    }
}

#[test]
fn caller_return_cannot_escape_a_deferred_body() {
    let error = program("finish::()#expand{`return 42;} main::()->int{defer{finish();}return 0;}")
        .unwrap_err();
    assert!(
        error.message.contains("deferred body cannot return"),
        "{error}"
    );
}

#[test]
fn macro_body_returns_are_distinct_from_caller_returns() {
    for source in [
        "finish::()#expand{return;} main::(){finish();}",
        "finish::()#expand{return 42;} main::()->int{finish();}",
        "finish::()->int#expand{`return 42;} main::()->int{return finish();}",
    ] {
        let error = program(source).unwrap_err();
        assert!(
            error.message.contains("expansion result binding")
                || error.message.contains("statement position"),
            "{error}"
        );
    }
}

#[test]
fn required_caller_results_and_returned_callbacks_cannot_be_discarded() {
    for source in [
        "finish::()#expand{`return 42;} answer::()->int#must{finish();} main::(){answer();}",
        "Required::#type()->int#must; callback::()->int{return 42;} finish::()#expand{`return callback;} answer::()->Required{finish();} main::(){value:=answer();value();}",
        "finish::()#expand{`return 40,2;} pair::()->(int#must,int){finish();} main::()->int{_,right:=pair();return right;}",
    ] {
        let error = program(source).unwrap_err();
        assert!(error.message.contains("#must"), "{source}\n{error}");
    }
}

#[test]
fn an_ordinary_local_procedure_does_not_inherit_its_outer_macro_return_target() {
    let source = "outer::()#expand{helper::()->int{`return 42;} value:=helper();} main::()->int{outer();return 0;}";
    let error = program(source).unwrap_err();
    assert!(
        error.message.contains("actual procedure") || error.message.contains("active #expand"),
        "{error}"
    );
}

#[test]
fn an_inner_local_macro_uses_its_own_actual_caller_procedure() {
    check(
        "outer::()#expand{helper::()->int{inner::()#expand{`return 42;}inner();}result:=helper();`return result;} main::()->int{outer();}",
    );
}

#[test]
fn unchanged_supplied_sgpu_macro_preserves_conditional_enum_caller_exit() {
    check(&supplied_caller_return::sgpu_source());
}

#[test]
fn imported_caller_return_operands_keep_the_actual_defining_file_scope() {
    let source = "Dependency::#import\"Dependency\";main::()->int{VALUE::100;Dependency.finish();}";
    let program = program_with_files(
        source,
        &[(
            "Dependency.jai",
            "#scope_file VALUE::42;#scope_export finish::()#expand{`return VALUE;}",
        )],
    )
    .unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn inserted_caller_export_quotes_use_the_live_expansion_target() {
    check(
        "wrap::(body:Code)#expand{#insert body;}main::()->int{code::#code `return 42;wrap(code);}",
    );
}
