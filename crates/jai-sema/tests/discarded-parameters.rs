use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::resolve_graph;
use jai_vm::{Limits, Outcome};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

fn check(source: &str, files: &[(&str, &str)]) -> Result<i128, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-discard-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&fixture.0).unwrap();
    let entry = fixture.0.join("main.jai");
    std::fs::write(&entry, source).unwrap();
    for (path, source) in files {
        let path = fixture.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    }
    let graph = ModuleGraph::load(
        &entry,
        GraphOptions {
            import_dirs: vec![fixture.0.clone()],
        },
    )
    .map_err(|error| format!("{error:?}"))?;
    let program = resolve_graph(&graph).map_err(|error| error.message)?;
    let execution = jai_vm::execute(&program, Limits::default());
    match execution.outcome {
        Outcome::Complete(values) => Ok(values[0].integer().unwrap().value()),
        other => Err(format!("{other:?}")),
    }
}

#[test]
fn direct_discarded_defaults_and_arguments_have_no_runtime_or_run_effects() {
    assert_eq!(check("counter:int;tick::()->int{counter+=1;return 9;}read::(#discard ignored:int=tick(),value:int=14)->int{return value;}main::()->int{a:=read();b:=read(tick());c:=read(#run tick());return a+b+c+counter;}",&[]).unwrap(),42);
}

#[test]
fn inferred_discarded_defaults_and_anonymous_runs_are_checked_without_effects() {
    assert_eq!(check("counter:int;tick::()->int{counter+=1;return 7;}ignore::(#discard value:=tick())->int{return 14;}main::()->int{return ignore()+ignore(#run ->int{counter+=1;value:=7;return value;})+ignore()+counter;}",&[]).unwrap(),42);
    let error=check("ignore::(#discard value:int)->int{return 42;}main::()->int{return ignore(#run ->int{return missing;});}",&[]).unwrap_err();
    assert!(
        error.contains("missing") || error.contains("unknown"),
        "{error}"
    );
}

#[test]
fn discarded_anonymous_runs_keep_capture_context_and_defer_rules() {
    let error=check("ignore::(#discard value:int)->int{return 42;}main::()->int{local:=7;return ignore(#run ->int{return local;});}",&[]).unwrap_err();
    assert!(error.contains("capture"), "{error}");
    let error=check("ignore::(#discard value:int)->int{return 42;}main::()->int{return ignore(#run ->int{defer{return;}return 7;});}",&[]).unwrap_err();
    assert!(error.contains("deferred"), "{error}");
    assert_eq!(check("#add_context number:int=7;ignore::(#discard value:int)->int #c_call{return 42;}main::()->int #c_call{return ignore(#run ->int{return context.number;});}",&[]).unwrap(),42);
}

#[test]
fn discarded_builtin_expressions_reject_invalid_operators_conditions_indices_and_casts() {
    for value in [
        "yes+yes",
        "ifx record then 1 else 2",
        "items[yes]",
        "cast(int) record",
    ] {
        let source = format!(
            "R::struct{{value:int;}}ignore::(#discard value:int)->int{{return 42;}}main::()->int{{yes:bool=true;record:R;items:[1]int;return ignore({value});}}"
        );
        assert!(
            check(&source, &[]).is_err(),
            "accepted invalid discarded expression {value}"
        );
    }
    let error = check(
        "ignore::(#discard value:*int)->int{return 42;}main::()->int{return ignore(*42);}",
        &[],
    )
    .unwrap_err();
    assert!(error.contains("storage"), "{error}");
}

#[test]
fn named_callback_discard_policy_survives_aliases_and_variadic_source_slots() {
    assert_eq!(check("counter:int;tick::()->int{counter+=1;return 9;}Callback::#type(#discard ignored:int,value:int)->int;Alias::#type Callback;read::(#discard ignored:int,value:int)->int{return value;}pack::(#discard values:..int,value:int=21)->int{return value;}main::()->int{callback:Alias=read;a:=callback(value=21,ignored=tick());b:=pack(tick(),#run tick(),value=21);return a+b+counter;}",&[]).unwrap(),42);
}

#[test]
fn imported_callback_proofs_retain_private_discarded_and_evaluated_nominals() {
    assert_eq!(check("Lib::#import \"Library\";counter:int;make::()->Lib.Value{counter+=1;return .{21};}main::()->int{callback:=Lib.get();value:Lib.Value=.{21};return callback(hidden=make(),value=value)+callback(hidden=#run make(),value=value)+counter;}", &[("Library.jai","#scope_file Private::struct{value:int;}Callback::#type(#discard hidden:Private,value:Private)->int;#scope_export Value::#type Private;read::(#discard hidden:Private,value:Private)->int{return value.value;}get::()->Callback{return read;}")]).unwrap(),42);
}

#[test]
fn same_abi_callbacks_keep_distinct_erased_formal_types() {
    assert_eq!(check("Left::struct{value:int;}Right::struct{value:int;}L::#type(#discard ignored:Left,value:int)->int;R::#type(#discard ignored:Right,value:int)->int;left::(#discard ignored:Left,value:int)->int{return value;}right::(#discard ignored:Right,value:int)->int{return value;}main::()->int{a:L=left;b:R=right;return a(.{1},21)+b(.{2},21);}",&[]).unwrap(),42);
    let error=check("Left::struct{value:int;}Right::struct{value:int;}L::#type(#discard ignored:Left,value:int)->int;left::(#discard ignored:Left,value:int)->int{return value;}main::()->int{a:L=left;wrong:Right;return a(wrong,42);}",&[]).unwrap_err();
    assert!(
        error.contains("convert") || error.contains("nominal"),
        "{error}"
    );
}

#[test]
fn nested_discarded_callback_calls_check_source_slots_without_effects() {
    assert_eq!(check("counter:int;tick::()->int{counter+=1;return 9;}Callback::#type(#discard ignored:int,value:int)->int;read::(#discard ignored:int,value:int)->int{counter+=100;return value;}ignore::(#discard value:int)->int{return 42;}main::()->int{callback:Callback=read;return ignore(callback(value=7,ignored=#run tick()))+counter;}",&[]).unwrap(),42);
    let error=check("Callback::#type(#discard ignored:int,value:int)->int;read::(#discard ignored:int,value:int)->int{return value;}ignore::(#discard value:int)->int{return 42;}main::()->int{callback:Callback=read;return ignore(callback(value=7,ignored=true));}",&[]).unwrap_err();
    assert!(error.contains("convert"), "{error}");
}

#[test]
fn discarded_context_calls_check_overrides_without_copying_or_evaluating_context() {
    let prefix = "#add_context number:int=7;counter:int;tick::()->int{counter+=1;return 9;}Callback::#type(#discard ignored:int,value:int)->int;read::(#discard ignored:int,value:int)->int{counter+=100;return value+context.number;}ignore::(#discard value:int)->int{return 42;}";
    assert_eq!(check(&format!("{prefix}main::()->int{{callback:Callback=read;return ignore(callback(value=7,ignored=#run tick(),,number=#run tick()))+counter+context.number-7;}}"),&[]).unwrap(),42);
    let error=check(&format!("{prefix}main::()->int{{callback:Callback=read;return ignore(callback(value=7,ignored=1,,number=true));}}"),&[]).unwrap_err();
    assert!(error.contains("convert"), "{error}");
    let error=check(&format!("{prefix}main::()->int{{callback:Callback=read;return ignore(callback(value=7,ignored=1,,number=1,number=2));}}"),&[]).unwrap_err();
    assert!(error.contains("duplicate"), "{error}");
}

#[test]
fn discarded_structural_cast_callbacks_keep_their_checked_source_names() {
    assert_eq!(check("counter:int;read::(input:int)->int{counter+=1;return input;}ignore::(#discard value:int)->int{return 42;}main::()->int{return ignore((cast((value:int)->int)read)(value=7))+counter;}",&[]).unwrap(),42);
    assert_eq!(check("counter:int;read::(input:int)->int{counter+=1;return input;}ignore::(#discard value:int)->int{return 42;}main::()->int{return ignore((cast((#discard hidden:bool,value:int)->int)read)(hidden=true,value=7))+counter;}",&[]).unwrap(),42);
    let error=check("read::(input:int)->int{return input;}ignore::(#discard value:int)->int{return 42;}main::()->int{return ignore((cast((#discard hidden:bool,value:int)->int)read)(hidden=1,value=7));}",&[]).unwrap_err();
    assert!(
        error.contains("convert") || error.contains("numeric parameter"),
        "{error}"
    );
}

#[test]
fn discarded_annotations_use_ready_counts_without_running_fresh_count_sources() {
    let prefix = "counter:int;count::()->int{counter+=1;return 1;}ignore::(#discard value:[1]int)->int{return 42;}";
    for body in [
        "source:[1]int;return ignore(cast([#run count()]int)source);",
        "return ignore(#run ->[#run count()]int{return .[7];});",
        "return ignore(#run ->[1]int{source:[#run count()]int;return source;});",
    ] {
        let error = check(&format!("{prefix}main::()->int{{{body}}}"), &[]).unwrap_err();
        assert!(
            error.contains("requires typed compile-time evaluation"),
            "{body}: {error}"
        );
    }
    assert_eq!(check("Count::1;ignore::(#discard value:[1]int)->int{return 42;}main::()->int{return ignore(#run ->[Count]int{source:[Count]int=.[7];return source;});}",&[]).unwrap(),42);
}

#[test]
fn discarded_callback_slots_are_checked_before_evaluated_run_arguments() {
    let error=check("Callback::#type(value:int,#discard hidden:bool)->int;read::(input:int)->int{return input;}trap::(zero:int)->int{return 1/zero;}main::()->int{callback:Callback=read;return callback(value=#run trap(0),hidden=1);}",&[]).unwrap_err();
    assert!(error.contains("numeric parameter"), "{error}");
    assert_eq!(check("Callback::#type(value:int,#discard hidden:bool)->int;read::(input:int)->int{return input;}main::()->int{callback:Callback=read;return callback(value=#run ->int{value:=0;for index:0..2{value+=14;}return value;},hidden=true);}",&[]).unwrap(),42);
}

#[test]
fn inferred_callback_branches_require_a_consistent_source_policy() {
    let prefix = "Left::struct{value:int;}Right::struct{value:int;}left::(#discard ignored:Left,value:int)->int{return value;}right::(#discard ignored:Right,value:int)->int{return value;}";
    let error=check(&format!("{prefix}main::()->int{{callback:=ifx true then left else right;return callback(.{{1}},42);}}"),&[]).unwrap_err();
    assert!(error.contains("polic"), "{error}");
    assert_eq!(check(&format!("{prefix}Callback::#type(#discard ignored:Left,value:int)->int;main::()->int{{callback:Callback=ifx true then left else right;return callback(.{{1}},42);}}"),&[]).unwrap(),42);
}

#[test]
fn discarded_formals_reject_reads_addresses_assignments_and_type_queries() {
    for body in [
        "return ignored;",
        "pointer:=*ignored;return 0;",
        "ignored=7;return 0;",
        "T::#type type_of(ignored);return 0;",
    ] {
        let source =
            format!("read::(#discard ignored:int)->int{{{body}}}main::()->int{{return read(7);}}");
        let error = check(&source, &[]).unwrap_err();
        assert!(error.contains("discard"), "{body}: {error}");
    }
    for marker in ["$", "$$"] {
        for body in [
            "return ignored;",
            "pointer:=*ignored;return 0;",
            "ignored=7;return 0;",
            "T::#type type_of(ignored);return 0;",
        ] {
            let source = format!(
                "read::(#discard {marker}ignored:int)->int{{{body}}}main::()->int{{return read(7);}}"
            );
            let error = check(&source, &[]).unwrap_err();
            assert!(error.contains("discard"), "{marker} {body}: {error}");
        }
    }
}
