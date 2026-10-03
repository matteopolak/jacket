use super::*;
use jai_modules::{GraphOptions, ModuleGraph};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

fn program(source: &str) -> Result<Program, jai_source::LocatedDiagnostic> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "jai-context-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("main.jai");
    fs::write(&path, source).unwrap();
    let graph = ModuleGraph::load(&path, GraphOptions::default()).unwrap();
    let result = crate::resolve_graph(&graph);
    fs::remove_dir_all(directory).unwrap();
    result
}
fn result(source: &str) -> i128 {
    let program = program(source).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("{:?}", execution.outcome)
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("{:?}", values)
    };
    value.value()
}
#[test]
fn shared_context_mutations_return_to_the_caller() {
    assert_eq!(
        result(
            "#add_context number:int=33; change::(){context.number+=9;} main::()->int{change(); return context.number;}"
        ),
        42
    );
}
#[test]
fn pushed_copy_and_default_context_restore_the_previous_record() {
    assert_eq!(
        result(
            "#add_context number:int=10; change::(){context.number+=1;} main::()->int{context.number=40; copy:=context; copy.number=1; push_context copy {change();} push_context {change();} return context.number+2;}"
        ),
        42
    );
}
#[test]
fn c_callback_establishes_default_context_before_calling_jai() {
    assert_eq!(
        result(
            "#add_context number:int=42; read::()->int{return context.number;} callback::()->int #c_call {push_context {return read();}} main::()->int{return callback();}"
        ),
        42
    );
    let error = program("read::()->int{return 42;} callback::()->int #c_call {return read();} main::()->int{return callback();}").unwrap_err();
    assert!(error.message.contains("context"));
}
#[test]
fn context_transfer_snapshots_return_values_before_cleanup() {
    assert_eq!(
        result(
            "#add_context number:int=40; callback::()->int{copy:=context; copy.number=42; push_context copy {defer context.number=0; return context.number;}} main::()->int{answer:=callback(); return answer+context.number-40;}"
        ),
        42
    );
}
#[test]
fn context_member_names_collide_across_imported_modules() {
    let error = program("#add_context number:int=1; #add_context number:bool=true; main::(){}")
        .unwrap_err();
    assert!(error.message.contains("context") || error.message.contains("conflict"));
}

#[test]
fn explicit_context_type_uses_the_canonical_schema_and_defaults() {
    assert_eq!(
        result(
            "#add_context number:int=42; read::(value:#Context)->int{return value.number;} main::()->int{value:#Context; return read(value);}"
        ),
        42
    );
}

#[test]
fn context_constants_preserve_values_and_callable_signatures() {
    assert_eq!(
        result(
            "read::()->int{return 40;} #add_context answer::2; #add_context callback::read; main::()->int{return context.callback()+context.answer;}"
        ),
        42
    );
}

#[test]
fn context_call_overrides_check_fields_and_context_availability() {
    for source in [
        "#add_context number:int=1; read::()->int{return context.number;} main::()->int{return read(,,number=40,number=2);}",
        "#add_context number:int=1; read::()->int{return context.number;} main::()->int{return read(,,missing=42);}",
        "#add_context number:int=1; read::()->int{return context.number;} detached::()->int #no_context{return read(,,number=42);} main::()->int{return detached();}",
    ] {
        let error = program(source).unwrap_err();
        assert!(error.message.contains("context"), "{}", error.message);
    }
}

#[test]
fn embedded_context_fields_share_promoted_storage_and_override_paths() {
    assert_eq!(
        result(
            "Base::struct{number:int=40;} #add_context using base:Base; #add_context extra:int=2; read::()->int{return context.number+context.extra;} main::()->int{context.number=1; answer:=read(,,number=40); return answer+context.base.number-1;}"
        ),
        42
    );
}

#[test]
fn promoted_context_members_reject_ambiguous_or_nonrecord_embeddings() {
    for source in [
        "Base::struct{number:int=1;} #add_context using left:Base; #add_context using right:Base; main::()->int{return context.number;}",
        "#add_context using number:int=42; main::()->int{return context.number;}",
    ] {
        let error = program(source).unwrap_err();
        assert!(
            error.message.contains("using") || error.message.contains("ambiguous"),
            "{}",
            error.message
        );
    }
}

#[test]
fn context_constants_on_explicit_records_need_no_implicit_context() {
    assert_eq!(
        result(
            "#add_context answer::42; read::(value:#Context)->int #no_context{return value.answer;} main::()->int #no_context{value:#Context;return read(value);}"
        ),
        42
    );
}

#[test]
fn context_field_defaults_keep_nonnull_procedure_identity_and_binding_names() {
    assert_eq!(
        result(
            "digits::(a:int,b:int)->int{return a*10+b;} #add_context callback:(left:int,right:int)->int=digits; main::()->int{return context.callback(right=2,left=4);}"
        ),
        42
    );
}

#[test]
fn global_context_defaults_preserve_nonzero_fields_and_callback_identity() {
    assert_eq!(
        result(
            "adjust::(value:int)->int{return value+2;} #add_context number:int=40; #add_context callback:(value:int)->int=adjust; saved:#Context; main::()->int{push_context saved{return context.callback(context.number);}}"
        ),
        42
    );
    assert_eq!(
        result(
            "adjust::(value:int)->int{return value+2;} #add_context number:int=1; #add_context callback:(value:int)->int=adjust; saved:#Context=.{number=40}; main::()->int{push_context saved{return context.callback(context.number);}}"
        ),
        42
    );
}

#[test]
fn global_context_defaults_reuse_original_imported_field_values() {
    use jai_modules::SourceOverlay;
    use std::path::Path;
    let mut sources = SourceOverlay::new();
    sources.insert(Path::new("/own-context-global/main.jai"), b"ContextModule::#import \"ContextModule\"; seed::1; saved:#Context; main::()->int{push_context saved{return context.callback(context.number);}}".to_vec()).unwrap();
    sources.insert(Path::new("/own-context-global/ContextModule.jai"), b"seed::40; adjust::(value:int)->int{return value+2;} #add_context number:int=seed; #add_context callback:(value:int)->int=adjust;".to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(
        Path::new("/own-context-global/main.jai"),
        GraphOptions {
            import_dirs: vec!["/own-context-global".into()],
        },
        &sources,
    )
    .unwrap();
    let program = crate::resolve_graph(&graph).unwrap();
    let jai_ir::GlobalInitializer::Value(initializer) = program.globals()[0].initializer() else {
        panic!("global Context retains typed defaults")
    };
    assert_eq!(initializer, &program.context().unwrap().default);
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}")
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("{values:?}")
    };
    assert_eq!(value.value(), 42);
}

#[test]
fn repeated_global_context_defaults_respect_constant_expansion_budget() {
    let error =
        program("#add_context number:int=1; saved:[1048576]#Context; main::(){}").unwrap_err();
    assert!(
        error.message.contains("constant cell budget"),
        "{}",
        error.message
    );
}

#[test]
fn context_parameter_defaults_use_the_established_schema() {
    assert_eq!(
        result(
            "#add_context number:int=42; read::(value:#Context=.{})->int{return value.number;} main::()->int{return read();}"
        ),
        42
    );
}
