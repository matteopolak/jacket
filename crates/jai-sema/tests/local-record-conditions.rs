//! Self-written record controls use the typed local definition environment.
use jai_modules::{ModuleGraph, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph, resolve_graph_with_options};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    graph_result(source).unwrap()
}

fn graph_result(source: &str) -> Result<ModuleGraph, jai_modules::GraphError> {
    let path = Path::new("/jai-local-record-controls/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, Default::default(), &overlay)
}

fn run(source: &str) -> i128 {
    let program = resolve_graph_with_options(
        &graph(source),
        &ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..ResolveOptions::default()
        },
        &mut NoEffects,
    )
    .unwrap();
    let outcome = jai_vm::execute(&program, Limits::default()).outcome;
    let Outcome::Complete(values) = outcome else {
        panic!("record fixture did not complete: {outcome:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    value.value()
}

#[test]
fn selected_fields_keep_source_order_and_inactive_fields_never_bind() {
    assert_eq!(
        run(r#"
        main :: () -> int {
            Record :: struct {
                before: u8 = 1;
                #if ENABLED { middle: u16 = 40; }
                else { middle: MissingType; #assert false; }
                after: u8 = 1;
                ENABLED :: Later;
                Later :: true;
            }
            record: Record;
            return cast(int)record.before + cast(int)record.middle + cast(int)record.after;
        }
    "#),
        42
    );
}

#[test]
fn a_later_selection_supplies_an_earlier_record_guard() {
    assert_eq!(
        run(r#"
        main :: () -> int {
            Record :: struct {
                #if CHOICE { value: int = 42; } else { value: MissingType; }
                #if true { CHOICE :: true; }
                #assert CHOICE;
            }
            record: Record;
            return record.value;
        }
    "#),
        42
    );
}

#[test]
fn source_cases_select_original_members_and_retry_forward_constants() {
    assert_eq!(
        run(r#"
            main::()->int {
                Record::struct {
                    before:u8=1;
                    #if SELECT == {
                        case false; value:MissingType; #assert false;
                        case true;
                            Inner::struct { value:int=40; }
                            value:Inner;
                            #through;
                        case; after:u8=1;
                    }
                    #if true { SELECT::true; }
                }
                record:Record;
                return cast(int)record.before+record.value.value+cast(int)record.after;
            }
        "#),
        42
    );
    assert_eq!(
        run(r#"
            main::()->int {
                Record::struct {
                    #if #complete false == {
                        case true; value:MissingType; #assert false;
                        case false; value:int=42;
                    }
                }
                record:Record;
                return record.value;
            }
        "#),
        42
    );
}

#[test]
fn source_cases_reject_invalid_labels_and_runtime_selectors() {
    for (source, expected) in [
        (
            "main::(){R::struct {#if true == {case true; a:int; case true; b:int;}}}",
            "duplicate compile-time case label",
        ),
        (
            "main::(){R::struct {#if #complete true == {case true; a:int;}}}",
            "do not cover the canonical value domain",
        ),
        (
            "main::(){selector::true; R::struct {selector:bool; #if selector == {case true; a:int;}}}",
            "runtime",
        ),
        (
            "main::(){R::struct {#if true == {case true; a:int; #through;}}}",
            "no valid fallthrough body",
        ),
    ] {
        let error = resolve_graph(&graph(source)).unwrap_err();
        assert!(error.message.contains(expected), "{error}");
    }
}

#[test]
fn anonymous_children_retry_parent_constants_before_promoting_runtime_names() {
    assert_eq!(
        run(r#"
            main::()->int {
                Record::struct {
                    struct {
                        #if CHOICE { value:int=ANSWER; }
                        else { value:MissingType; }
                    }
                    #if true { CHOICE::true; ANSWER::42; }
                    #if CHOICE { marker:u8; }
                }
                record:Record;
                return record.value;
            }
        "#),
        42
    );
    let source = "main::(){enabled::true; Child::struct{enabled:bool;} R::struct{using child:Child; #if enabled{value:int;}}}";
    let error = resolve_graph(&graph(source)).unwrap_err();
    assert!(error.message.contains("runtime"), "{error}");
    let source = "main::(){enabled::true; R::struct{struct{#if MISSING{enabled:bool;}} #if enabled{value:int;}}}";
    let error = resolve_graph(&graph(source)).unwrap_err();
    assert!(error.message.contains("MISSING"), "{error}");
}

#[test]
fn active_assertions_wait_for_the_selected_record_shape_and_defaults() {
    assert_eq!(
        run(r#"
        main :: () -> int {
            Record :: struct {
                #assert size_of(Record) == 8;
                #if true { value: int = ANSWER; }
                else { value: MissingType; }
                ANSWER :: 42;
                #assert ANSWER == 42;
            }
            record: Record;
            return record.value;
        }
    "#),
        42
    );
}

#[test]
fn chosen_nested_types_methods_and_constants_join_the_record_namespace() {
    assert_eq!(
        run(r#"
        main :: () -> int {
            Record :: struct {
                #if true {
                    LIMIT :: 21;
                    Inner :: struct { value: int = LIMIT; }
                    read :: (value: *Inner) -> int { return value.value; }
                } else {
                    LIMIT :: absent;
                    Inner :: struct { value: MissingType; }
                    read :: () -> int { return missing(); }
                }
                value: Inner;
                #assert LIMIT == 21;
            }
            record: Record;
            return Record.read(*record.value) + Record.LIMIT;
        }
    "#),
        42
    );
}

#[test]
fn run_guards_wait_for_real_checked_procedure_readiness() {
    assert_eq!(
        run(r#"
        main :: () -> int {
            Record :: struct {
                #if #run enabled() { value: int = 42; }
                else { value: MissingType; }
            }
            record: Record;
            return record.value;
        }
        enabled :: () -> bool { return true; }
    "#),
        42
    );
    assert_eq!(
        run(r#"
        main :: () -> int {
            Record :: struct {
                #if #run enabled() { value: int = 42; }
                else { value: MissingType; }
                enabled :: () -> bool { return true; }
                #assert #run enabled();
            }
            record: Record;
            return record.value;
        }
    "#),
        42,
    );
}

#[test]
fn baked_procedures_and_inline_records_select_separate_local_shapes() {
    assert_eq!(
        run(r#"
        choose :: ($Enabled: bool) -> int {
            record: struct {
                #if Enabled { value: int = 17; }
                else { value: int = 25; }
                #assert size_of(int) == 8;
            };
            return record.value;
        }
        main :: () -> int { return choose(true) + choose(false); }
    "#),
        42
    );
}

#[test]
fn inactive_assertions_runs_and_declarations_have_no_execution_effect() {
    let source = r#"
        main :: () -> int {
            Record :: struct {
                #if false {
                    #assert false;
                    Broken :: #run -> int { while true {} return 0; };
                    Hidden :: struct { value: MissingType; }
                    value: Hidden;
                } else { value: int = 42; }
            }
            record: Record;
            return record.value;
        }
    "#;
    let program = resolve_graph_with_options(
        &graph(source),
        &ResolveOptions {
            compile_time_limits: Limits {
                fuel: 100,
                ..Limits::default()
            },
            ..ResolveOptions::default()
        },
        &mut NoEffects,
    )
    .unwrap();
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42))
    );
}

#[test]
fn guards_and_assertions_reject_runtime_storage_and_active_field_shadows() {
    for source in [
        "main::()->int { enabled:bool=true; Record::struct { #if enabled { value:int; } } return 0; }",
        "main::()->int { enabled::true; Record::struct { enabled:bool=true; #if enabled { value:int; } } return 0; }",
        "main::()->int { enabled::true; Record::struct { enabled:bool=true; #assert enabled; } return 0; }",
    ] {
        let error = resolve_graph(&graph(source)).unwrap_err().to_string();
        assert!(error.contains("runtime"), "{error}");
    }
}

#[test]
fn a_false_active_assertion_retains_its_original_condition_location() {
    let source = "main::()->int { Record::struct { #assert false; value:int; } return 0; }";
    let error = resolve_graph(&graph(source)).unwrap_err();
    let start = source.find("false").unwrap();
    assert_eq!(error.location.span, jai_source::Span::new(start, start + 5));
    assert!(error.message.contains("compile-time assertion failed"));
}

#[test]
fn record_assertion_messages_use_typed_constants_and_selected_members() {
    assert_eq!(
        run(r#"
            main::()->int {
                Record::struct {
                    #assert(size_of(Record)==8, REASON);
                    #assert true "shape is ready";
                    #if false { #assert(false, 17); }
                    REASON::"the selected shape must hold";
                    value:int=42;
                }
                record:Record;
                return record.value;
            }
        "#),
        42
    );
    let source = "main::()->int { Record::struct { #if true { #assert(false, REASON); } REASON::\"the selected shape must hold\"; value:int; } return 0; }";
    let error = resolve_graph(&graph(source)).unwrap_err();
    assert!(
        error.message.contains("the selected shape must hold"),
        "{error}"
    );
    let start = source.find("false").unwrap();
    assert_eq!(error.location.span, jai_source::Span::new(start, start + 5));
    let source = "main::()->int { Record::struct { #assert(true, 17); } return 0; }";
    let error = resolve_graph(&graph(source)).unwrap_err().to_string();
    assert!(
        error.contains("message requires a compile-time string"),
        "{error}"
    );
    let source = "main::()->int { message:string=\"runtime\"; Record::struct { #assert(true, message); } return 0; }";
    assert!(resolve_graph(&graph(source)).is_err());
}

#[test]
fn selected_duplicate_names_and_missing_guard_dependencies_are_diagnostics() {
    for source in [
        "main::()->int { Record::struct { VALUE::1; #if true { VALUE::2; } } return 0; }",
        "main::()->int { Record::struct { #if MISSING { value:int; } } return 0; }",
        "main::()->int { Record::struct { #if CHOICE { CHOICE::true; value:int; } } return 0; }",
    ] {
        assert!(resolve_graph(&graph(source)).is_err(), "{source}");
    }
}

#[test]
fn record_controls_retain_the_lexical_arithmetic_check_policy() {
    assert_eq!(
        run(r#"
        main :: () -> int #no_aoc {
            Record :: struct {
                #if cast(u8)255+1 == 0 { value: int = 42; }
                else { value: MissingType; }
                #assert cast(u8)255+1 == 0;
            }
            record: Record;
            return record.value;
        }
    "#),
        42
    );
    assert_eq!(
        run(r#"
            main::()->int #no_aoc {
                Record::struct {
                    #if cast(u8)255+1 == {
                        case 0; value:int=42;
                        case; value:MissingType;
                    }
                }
                record:Record;
                return record.value;
            }
        "#),
        42
    );
    let source =
        "main::()->int { Record::struct { #if cast(u8)255+1==0 { value:int; } } return 0; }";
    let error = resolve_graph(&graph(source)).unwrap_err().to_string();
    assert!(error.to_ascii_lowercase().contains("overflow"), "{error}");
    let source = "main::(){Record::struct{#if cast(u8)255+1 == {case 0; value:int;}}}";
    let error = resolve_graph(&graph(source)).unwrap_err().to_string();
    assert!(error.to_ascii_lowercase().contains("overflow"), "{error}");
}

#[test]
fn record_control_depth_has_a_diagnostic_and_inactive_branches_do_not_bind() {
    let boundary = format!("{}value:int=42;{}", "#if true {".repeat(64), "}".repeat(64));
    let source = format!(
        "main::()->int {{ Record::struct {{{boundary}}} record:Record; return record.value; }}"
    );
    assert_eq!(run(&source), 42);
    let nested = format!(
        "{}value:int=42;{}",
        "#if true {".repeat(130),
        "}".repeat(130)
    );
    let source = format!("main::()->int {{ Record::struct {{{nested}}} return 0; }}");
    let error = graph_result(&source).unwrap_err().to_string();
    assert!(
        error.contains("record conditional nesting exceeds"),
        "{error}"
    );
    let nested = format!(
        "{}value:MissingType;#assert false;{}",
        "#if true {".repeat(30),
        "}".repeat(30)
    );
    let source = format!(
        "main::()->int {{ Record::struct {{ #if false {{{nested}}} else {{value:int=42;}} }} record:Record; return record.value; }}"
    );
    assert_eq!(run(&source), 42);
}
