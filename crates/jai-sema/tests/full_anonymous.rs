use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;
fn check(source: &str) -> Result<jai_ir::Program, jai_source::LocatedDiagnostic> {
    let path = Path::new("/own-full-anonymous/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    jai_sema::resolve_graph(&graph)
}
#[test]
fn standalone_return_and_typed_assignment_have_real_source_bodies() {
    for text in [
        "main::()->int{f:=(value:s32)->s32{return value+2;};return f(40);}",
        "Callback::()->s32;make::()->Callback{return ()->s32{return 42;};}main::()->int{return make()();}",
        "Callback::(value:s32)->s32 #c_call;main::()->int{f:Callback=(value:s32)->s32 #c_call{return value+2;};return f(40);}",
        "main::()->int{return ((value:s32)->s32{return value+2;})(40);}",
        "Callback::()->void;invoke::(f:Callback){f();}main::()->int{invoke(()->void{return;});return 42;}",
        "answer::(()->s32{return 42;});main::()->int{return answer();}",
        "Callback::(value:s32)->s32;apply::(f:Callback)->s32{return f(40);}main::()->int{return apply((value:s32)->s32{return value+2;});}",
    ] {
        let program = check(text).unwrap();
        let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
        let jai_vm::Outcome::Complete(values) = outcome else {
            panic!("{outcome:?}")
        };
        assert_eq!(values[0].integer().unwrap().value(), 42);
    }
}
#[test]
fn runtime_outer_local_is_not_a_capture() {
    let err =
        check("main::()->int{value:s32=42;f:=()->s32{return value;};return f();}").unwrap_err();
    assert!(err.message.contains("capture"), "{err:?}");
}
#[test]
fn mismatched_actual_convention_is_rejected() {
    let err = check(
        "Callback::()->s32 #c_call;main::()->int{f:Callback=()->s32{return 42;};return f();}",
    )
    .unwrap_err();
    assert!(
        err.message.contains("type") || err.message.contains("signature"),
        "{err:?}"
    );
}

#[test]
fn defaults_named_results_and_discard_execute_once() {
    for text in [
        "calls:s32=0;next::()->s32{calls+=1;return 9;}main::()->int{f:=(#discard ignored:s32,value:s32,extra:s32=2)->s32{return value+extra;};return f(next(),value=40)+calls;}",
        "main::()->int{f:=()->(answer:s32=42){return;};return f();}",
    ] {
        let program = check(text).unwrap();
        let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
        let jai_vm::Outcome::Complete(values) = outcome else {
            panic!("{outcome:?}")
        };
        assert_eq!(values[0].integer().unwrap().value(), 42);
    }
}
#[test]
fn source_obligations_and_discarded_bindings_are_retained() {
    for (text, expected) in [
        (
            "main::()->int{f:=()->s32 #must{return 42;};f();return 0;}",
            "#must",
        ),
        (
            "main::()->int{f:=(#discard ignored:s32)->s32{return ignored;};return f(42);}",
            "discard",
        ),
    ] {
        let err = check(text).unwrap_err();
        assert!(err.message.contains(expected), "{err:?}");
    }
}
#[test]
fn invalid_callback_body_is_checked_before_earlier_run_argument() {
    let error=check("fail::()->s32{divisor:s32=0;return 1/divisor;}consume::(first:s32,f:()->s32)->s32{return first+f();}main::()->int{local:s32=42;return consume(#run fail(),()->s32{return local;});}").unwrap_err();
    assert!(error.message.contains("capture"), "{error:?}");
}

#[test]
fn discarded_anonymous_default_is_previewed_without_running_its_body() {
    let program = check("calls:s32=0;touch::()->s32{calls+=1;return 99;}main::()->int{f:=(#discard callback:()->s32=()->s32{return touch();})->s32{return 42;};return f()+calls;}").unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn actual_nominal_global_and_constant_callback_initializers_execute() {
    for source in [
        include_str!("fixtures/full-anonymous/nominal-global-callback.jai"),
        include_str!("fixtures/full-anonymous/nominal-constant-callback.jai"),
        include_str!("fixtures/full-anonymous/direct-global-procedure.jai"),
        include_str!("fixtures/full-anonymous/global-storage-from-anonymous.jai"),
    ] {
        let program = check(source).unwrap_or_else(|error| panic!("{source}\n{error:?}"));
        let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
        let jai_vm::Outcome::Complete(values) = outcome else {
            panic!("{outcome:?}")
        };
        assert_eq!(values[0].integer().unwrap().value(), 42);
    }
}

#[test]
fn full_procedures_reject_runtime_parameter_and_aggregate_local_captures() {
    for source in [
        "main::()->int{outer:=(value:s32)->s32{inner:=()->s32{return value;};return inner();};return outer(42);}",
        "Event::struct{value:s32;}main::()->int{event:Event=.{value=42};callback:=()->s32{return event.value;};return callback();}",
    ] {
        let error = check(source).unwrap_err();
        assert!(error.message.contains("capture"), "{error:?}");
    }
}

#[test]
fn callback_initializers_reject_different_nominal_parameter_identity() {
    for initializer in [": Listener =", "::"] {
        let source = format!(
            "Event::struct{{value:s32;}}Other::struct{{value:s32;}}Listener::struct{{callback:(event:*Event)->s32 #c_call;}}listener {initializer} Listener.{{callback=(event:*Other)->s32 #c_call{{return event.value;}}}};main::()->int{{return 42;}}"
        );
        let error = check(&source).unwrap_err();
        assert!(
            error.message.contains("contextual type")
                || error.message.contains("type")
                || error.message.contains("signature"),
            "{error:?}"
        );
    }
}
