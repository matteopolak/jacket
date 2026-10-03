//! Independently authored source cases for canonical pointer and void callback aliases.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;

fn graph(files: &[(&str, &str)]) -> ModuleGraph {
    let mut source = SourceOverlay::new();
    for (name, text) in files {
        source
            .insert(
                &Path::new("/jai-source-aliases").join(name),
                text.as_bytes().to_vec(),
            )
            .unwrap();
    }
    ModuleGraph::load_with_provider(
        Path::new("/jai-source-aliases/main.jai"),
        GraphOptions::default(),
        &source,
    )
    .unwrap()
}

fn check(files: &[(&str, &str)]) {
    let program = jai_sema::resolve_graph(&graph(files)).unwrap();
    for procedure in program.procedures() {
        let signature = program
            .types()
            .procedure_definition(procedure.signature)
            .unwrap();
        assert!(
            signature
                .results
                .iter()
                .all(|&ty| !matches!(program.types().kind(ty), Ok(jai_types::TypeKind::Void)))
        );
    }
    assert!(matches!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Complete(values)
            if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)
    ));
}

#[test]
fn builtin_pointer_aliases_and_c_void_callbacks_share_canonical_signatures() {
    check(&[(
        "main.jai",
        r#"
        marg_list :: *void;
        PointerPointer :: **void;
        IMP :: #type () -> void #c_call;
        EmptyCallback :: #type () #c_call;
        NoResult :: void;
        answer:int;
        notify :: () -> NoResult #c_call { answer=42; }
        Holder :: struct { callback:IMP=notify; raw:marg_list; }
        main :: () -> int {
            holder:Holder;
            nested:PointerPointer;
            if holder.raw != null return 1;
            if nested != null return 2;
            callback:EmptyCallback=holder.callback;
            callback();
            return answer;
        }
        "#,
    )]);
}

#[test]
fn local_pointer_and_void_callback_aliases_keep_storage_address_semantics() {
    check(&[(
        "main.jai",
        r#"
        main :: () -> int {
            Number :: int;
            Pointer :: *Number;
            Callback :: #type (value:Pointer) -> void #c_call;
            notify :: (value:Pointer) -> void #c_call { value.*=42; }
            answer:int;
            pointer:Pointer=*answer;
            callback:Callback=notify;
            callback(pointer);
            return pointer.*;
        }
        "#,
    )]);
}

#[test]
fn generic_void_result_patterns_produce_no_runtime_result_slot() {
    check(&[(
        "main.jai",
        r#"
        write :: (target:*$T, value:T) -> void { target.*=value; }
        main :: () -> int { answer:int; write(*answer,42); return answer; }
        "#,
    )]);
}

#[test]
fn imported_aliases_and_module_callback_arguments_keep_zero_results() {
    check(&[
        (
            "main.jai",
            r#"
            Types :: #import,file "types.jai";
            Calls :: #import,file "calls.jai"(T=Types.Callback);
            notify :: (target:*int) -> void #c_call { target.*=42; }
            main :: () -> int { answer:int; pointer:Types.Pointer=*answer;
                Calls.invoke(notify,pointer); return answer; }
            "#,
        ),
        (
            "types.jai",
            "Pointer :: *int; Callback :: #type (target:Pointer) -> void #c_call;",
        ),
        (
            "calls.jai",
            "#module_parameters(T:Type); invoke :: (callback:T,target:*int) -> void { callback(target); }",
        ),
    ]);
}

#[test]
fn void_results_do_not_make_runtime_void_storage_or_other_callback_shapes_valid() {
    for (source, expected) in [
        (
            "Bad :: #type () -> (void,int); main :: () {}",
            "cannot occupy runtime storage",
        ),
        (
            "Bad :: #type (value:void); main :: () {}",
            "cannot occupy runtime storage",
        ),
        (
            "C :: #type () -> void #c_call; value :: () {} main :: () { callback:C=value; }",
            "signature",
        ),
        (
            "C :: #type () -> void #c_call; value :: () -> int #c_call { return 42; } main :: () { callback:C=value; }",
            "signature",
        ),
        (
            "main :: () { value:int; Address :: *value; }",
            "runtime local 'value' is unavailable before its declaration",
        ),
    ] {
        let graph = graph(&[("main.jai", source)]);
        let error = jai_sema::resolve_graph(&graph).unwrap_err();
        assert!(error.message.contains(expected), "{source}\n{error:?}");
        assert_eq!(
            graph.sources().get(error.location.source).unwrap().path(),
            Path::new("/jai-source-aliases/main.jai")
        );
    }
}
