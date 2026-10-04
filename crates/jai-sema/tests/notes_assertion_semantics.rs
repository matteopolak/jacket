//! Independently authored source exercises existing selected compile-time and layout consumers.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::Path;

fn program(text: &str) -> Result<jai_ir::Program, jai_source::LocatedDiagnostic> {
    let path = Path::new("/notes-assertions/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, text.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut NoEffects,
    )
}
fn run(text: &str) -> i128 {
    let program = program(text).unwrap();
    let Outcome::Complete(values) = jai_vm::execute(&program, Limits::default()).outcome else {
        panic!()
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!()
    };
    value.value()
}

#[test]
fn selected_comma_assertions_evaluate_real_forward_constants_in_each_scope() {
    let text = r#"Reason::"checked message"; #assert Ready, Reason; Ready::true;
        R::struct { #assert Ready, Reason; value:int=42; }
        main::()->int{ #if false { #assert false, "inactive"; } #assert Ready, Reason; r:R;return r.value; }"#;
    assert_eq!(run(text), 42);
}

#[test]
fn false_comma_assertion_keeps_the_original_message_and_source_span() {
    let text = "Reason::\"target rejected\"; main::(){#assert false, Reason;}";
    let error = program(text).unwrap_err();
    assert!(
        error
            .message
            .contains("compile-time assertion failed: target rejected"),
        "{error:?}"
    );
    assert_eq!(error.location.span.text(text), "#assert false, Reason;");
}

#[test]
fn a_message_must_be_a_real_compile_time_string_even_when_condition_is_true() {
    let text = "main::(){#assert true, 17;}";
    let error = program(text).unwrap_err();
    assert!(
        error
            .message
            .contains("message requires a compile-time string"),
        "{error:?}"
    );
    assert_eq!(error.location.span.text(text), "17");
}

#[test]
fn postfix_alignment_uses_the_real_checked_layout_and_notes_keep_source_bytes() {
    let text = r#"R::struct @Before { first:u8; value:u64=--- #align 32; @"Display label" } @After
        main::()->int{ info:=type_info(R); if info.members[1].offset_in_bytes!=32 return 1;
            if info.notes.count!=2 return 2; if info.members[1].notes.count!=1 return 3;
            if info.members[1].notes[0].count!=15 return 4; return 42; }"#;
    assert_eq!(run(text), 42);
    let error = program("R::struct{value:int=0 #align 3;}main::(){r:R;}").unwrap_err();
    assert!(error.message.contains("power-of-two"), "{error:?}");
}

const ITERATOR: &str = r#"For_Flags::enum_flags u8{POINTER::1;REVERSE::2;}
    Container::struct{value:int;} Reason::"defining iterator restriction";
    for_expansion::(source:Container,body:Code,flags:For_Flags)#expand{
        for slot:0..0{`it:=source.value;`it_index:=slot;
            #insert(remove=#assert false "defining iterator restriction") body;}}
"#;

#[test]
fn an_unused_lazy_assertion_does_not_fail_the_iterator() {
    assert_eq!(
        run(&format!(
            "{ITERATOR}main::()->int{{source:Container;for item:source{{}}return 42;}}"
        )),
        42
    );
}

#[test]
fn a_triggered_replacement_reports_its_defining_assertion_message() {
    let text = format!("{ITERATOR}main::(){{source:Container;for item:source remove item;}}");
    let error = program(&text).unwrap_err();
    assert!(
        error.message.contains(
            "loop-control insertion replacement assertion failed: defining iterator restriction"
        ),
        "{error:?}"
    );
    assert_eq!(
        error.location.span.text(&text),
        "#assert false \"defining iterator restriction\""
    );
}
