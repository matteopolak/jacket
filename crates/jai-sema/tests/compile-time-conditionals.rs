//! Independent fixtures exercise source selection and the checked Rust VM.
use jai_modules::{GraphOptions, ModuleGraph, PreludeSource, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph, resolve_graph_with_options};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::Path;

const MAIN: &str = "/jai-static-if/main.jai";

fn graph(source: &str) -> ModuleGraph {
    let mut overlay = SourceOverlay::new();
    overlay
        .insert(Path::new(MAIN), source.as_bytes().to_vec())
        .unwrap();
    ModuleGraph::load_with_provider(Path::new(MAIN), GraphOptions::default(), &overlay).unwrap()
}

fn value(program: &jai_ir::Program) -> i128 {
    let execution = jai_vm::execute(program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("VM did not complete: {execution:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected integer result: {values:?}");
    };
    value.value()
}

#[test]
fn selected_declarations_join_the_enclosing_scope_and_inactive_names_never_bind() {
    let program = resolve_graph(&graph(
        r#"
        main :: () -> int {
            #if ENABLED {
                Choice :: struct { value: int = 17; }
                answer :: () -> int { choice: Choice; return choice.value; }
                value := 25;
            } else {
                Choice :: struct { broken: MissingType; }
                answer :: () -> int { return absent(); }
                value := unavailable;
            }
            ENABLED :: Later == 3;
            Later :: 3;
            return answer() + value;
        }
    "#,
    ))
    .unwrap();
    assert_eq!(value(&program), 42);
    assert!(!format!("{:?}", program.procedures()).contains("MissingType"));
}

#[test]
fn nested_single_statement_selection_preserves_termination_and_loop_control() {
    for source in [
        "main::()->int{ #if false return missing; else #if true return 42; else return absent; }",
        "main::()->int{ result:=0; for i:1..5 { #if true { if i==3 continue; result+=i; } else { unavailable(); } } return result+30; }",
    ] {
        assert_eq!(value(&resolve_graph(&graph(source)).unwrap()), 42);
    }
}

#[test]
fn selected_defer_runs_at_the_enclosing_scope_exit() {
    let source = "counter:int=0; main::()->int{ { #if true { defer counter += 5; } counter += 2; if counter != 2 return 1; } return counter+35; }";
    assert_eq!(value(&resolve_graph(&graph(source)).unwrap()), 42);
}

#[test]
fn typed_constant_fields_indices_and_type_queries_are_legitimate_guards() {
    let source = r#"
        main :: () -> int {
            text :: "Hello";
            values :: int.[3, 7, 9];
            runtime: s32 = 5;
            storage: [3] int;
            procedure :: () -> int { return 7; }
            #if text == "Hello" && text != "hello" && text.count == 5 && text[1] == #char "e" && values[1] == 7 {
                #if type_of(runtime) == s32 && storage.count == 3 && procedure { return 42; }
                else { unknown_type(); }
            } else { return missing; }
        }
    "#;
    assert_eq!(value(&resolve_graph(&graph(source)).unwrap()), 42);
}

#[test]
fn baked_procedure_parameters_select_independent_specializations() {
    let source = "choose::($Enabled:bool)->int{ #if Enabled { result:=20; } else { result:=22; } return result; } main::()->int{return choose(true)+choose(false);}";
    assert_eq!(value(&resolve_graph(&graph(source)).unwrap()), 42);
}

#[test]
fn a_later_independent_selection_can_supply_an_earlier_guard_constant() {
    let source = "main::()->int{ #if Choice { answer:=42; } else { answer:=missing; } #if true { Choice::true; } return answer; }";
    assert_eq!(value(&resolve_graph(&graph(source)).unwrap()), 42);
}

#[test]
fn global_addresses_and_constant_index_projections_are_constant_without_runtime_loads() {
    let source = "Cell::struct{value:int;} global:Cell; values:[3]int; main::()->int{ #if *global.value != null && *values[1] != null { return 42; } else { return missing; } }";
    assert_eq!(value(&resolve_graph(&graph(source)).unwrap()), 42);
}

#[test]
fn run_condition_waits_for_a_forward_checked_procedure_and_discards_unselected_runs() {
    let source = "main::()->int{ #if #run factorial(5)>100 { return 42; } else { #run fail(); return unavailable; } } factorial::(n:int)->int{ if n<=1 return 1; return n*factorial(n-1); } fail::(){ n:=1/0; }";
    let program = resolve_graph(&graph(source)).unwrap();
    assert_eq!(value(&program), 42);
    let jai_ir::EntryPoint::Int(entry) = program.entry() else {
        panic!("integer entry")
    };
    let main = program
        .procedures()
        .iter()
        .find(|procedure| procedure.id == entry)
        .unwrap();
    assert!(!format!("{:?}", main.body).contains("If("));
    assert!(!format!("{:?}", main.body).contains("Call"));
}

#[test]
fn defining_module_parameters_and_private_constants_choose_the_branch() {
    let mut overlay = SourceOverlay::new();
    for (path, source) in [
        (
            MAIN,
            "selected::#import \"Configured\"(Enabled=true); Enabled::false; main::()->int{return selected.answer();}",
        ),
        (
            "/jai-static-if/modules/Configured/module.jai",
            "#module_parameters(Enabled:bool); #scope_file SECRET::42; #scope_export answer::()->int{ #if Enabled { return SECRET; } else { return private_missing; } }",
        ),
    ] {
        overlay
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        Path::new(MAIN),
        GraphOptions {
            import_dirs: vec!["/jai-static-if/modules".into()],
        },
        &overlay,
    )
    .unwrap();
    assert_eq!(value(&resolve_graph(&graph).unwrap()), 42);
}

#[test]
fn scoped_import_prefixes_supply_guards_and_selected_declarations() {
    for source in [
        "main::()->int{ flags::#import \"Flags\"; #if flags.ENABLED { answer:=flags.ANSWER; } else { answer:=missing; } return answer; }",
        "main::()->int{ ENABLED:=false; { #import \"Flags\"; #if ENABLED { return ANSWER; } else { return missing; } } }",
        "main::()->int{ #if true { flags::#import \"Flags\"; } #if flags.ENABLED { answer::flags.ANSWER; } else { answer::missing; } return answer; }",
    ] {
        let mut overlay = SourceOverlay::new();
        for (path, source) in [
            (MAIN, source),
            (
                "/jai-static-if/modules/Flags/module.jai",
                "ENABLED::true; ANSWER::42;",
            ),
        ] {
            overlay
                .insert(Path::new(path), source.as_bytes().to_vec())
                .unwrap();
        }
        let graph = ModuleGraph::load_with_provider(
            Path::new(MAIN),
            GraphOptions {
                import_dirs: vec!["/jai-static-if/modules".into()],
            },
            &overlay,
        )
        .unwrap();
        assert_eq!(value(&resolve_graph(&graph).unwrap()), 42);
    }
}

#[test]
fn explicit_targets_select_source_nominal_tags_from_compiler_prelude() {
    for (operating_system, expected) in [
        (OperatingSystem::Windows, 11),
        (OperatingSystem::Linux, 22),
        (OperatingSystem::MacOS, 33),
    ] {
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                Path::new(MAIN),
            br#"
            Operating_System_Tag :: enum u32 #specified { WINDOWS::90; LINUX::91; MACOS::92; }
            IS_LINUX :: OS == .LINUX;
            main :: () -> int {
                #if type_of(OS) == Operating_System_Tag { application_schema_must_not_replace_preload(); }
                Local_Target :: OS;
                IS_WINDOWS :: Local_Target == .WINDOWS;
                #if IS_WINDOWS { result := 11; }
                else #if IS_LINUX { result := 22; }
                else #if OS == .MACOS { result := 33; }
                else { result := missing_target; }
                #if CPU != .X64 { unknown_architecture(); }
                return result;
            }
        "#
                .to_vec(),
            )
            .unwrap();
        overlay
            .insert(
                Path::new("/jai-static-if/modules/module.jai"),
                jai_modules::compiler_prelude_source().as_bytes().to_vec(),
            )
            .unwrap();
        let target = BuildTarget {
            operating_system,
            architecture: Architecture::X86_64,
            layout: LayoutPolicy::lp64(),
            byte_order: ByteOrder::Little,
        };
        let roots = vec!["/jai-static-if/modules".into()];
        let graph = ModuleGraph::load_with_bootstrap(
            Path::new(MAIN),
            GraphOptions {
                import_dirs: roots.clone(),
            },
            PreludeSource::Search,
            &overlay,
            Some(target.clone()),
        )
        .unwrap();
        let compiler = jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &roots,
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        );
        let program = resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                target: Some(target),
                compiler: Some(compiler),
                ..Default::default()
            },
            &mut NoEffects,
        )
        .unwrap();
        assert_eq!(value(&program), expected);
    }
}

#[test]
fn runtime_storage_and_implicit_calls_do_not_become_compile_time_conditions() {
    for source in [
        "main::()->int{ dynamic:=true; #if dynamic { return 42; } else { return 0; } }",
        "dynamic:bool=true; main::()->int{ #if true || dynamic { return 42; } else { return 0; } }",
        "answer::()->bool{return true;} main::()->int{ #if answer() { return 42; } else { return 0; } }",
        "answer::()->bool{return true;} main::()->int{ #if false && answer() { return 42; } else { return 0; } }",
    ] {
        let error = resolve_graph(&graph(source)).unwrap_err();
        assert!(
            error
                .message
                .contains("#if condition requires compile-time values"),
            "{error:?}"
        );
    }
}

#[test]
fn failures_preserve_condition_or_selected_statement_source_ranges() {
    for (source, message, snippet) in [
        (
            "main::()->int{ #if true { missing=2; } else { return 0; } }",
            "unknown name 'missing'",
            "missing=2;",
        ),
        (
            "main::()->int{ #if (1/0)>2 { return 42; } else { return 0; } }",
            "ZeroDivisor",
            "(1/0)>2",
        ),
        (
            "fail::()->bool{ value:=1/0; return true; } main::()->int{ #if #run fail() { return 42; } else { return 0; } }",
            "ZeroDivisor",
            "#run fail()",
        ),
        (
            "main::()->int{ #if true { return 42; } extra:=1; }",
            "unreachable statement",
            "extra:=1;",
        ),
    ] {
        let graph = graph(source);
        let error = resolve_graph(&graph).unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
        assert_eq!(
            error
                .location
                .span
                .text(graph.sources().get(error.location.source).unwrap().text()),
            snippet
        );
    }
}

#[test]
fn unselected_branch_must_still_parse() {
    let mut overlay = SourceOverlay::new();
    overlay
        .insert(
            Path::new(MAIN),
            b"main::(){ #if false { absent( ; } }".to_vec(),
        )
        .unwrap();
    assert!(
        ModuleGraph::load_with_provider(Path::new(MAIN), GraphOptions::default(), &overlay)
            .is_err()
    );
}

#[test]
fn active_file_and_body_assertions_use_typed_readiness_and_source_messages() {
    let source = r#"
        #assert choose() "forward checked source predicate";
        #assert #run choose() "explicit checked source run";
        #assert(5 == 5, "parenthesized message");
        #if false { #assert false "inactive file assertion"; }
        choose :: () -> bool { return true; }
        check :: ($Enabled: bool) -> int {
            #if Enabled { #assert Enabled; return 20; }
            else { #assert !Enabled "selected generic assertion"; return 22; }
            #if false { #assert absent(); }
        }
        main :: () -> int {
            #assert !(2 & 1) "iterator style flag assertion";
            #assert #run choose();
            return check(true) + check(false);
        }
    "#;
    let program = resolve_graph(&graph(source)).unwrap();
    assert_eq!(value(&program), 42);
    assert!(!format!("{:?}", program.procedures()).contains("Assert"));
}

#[test]
fn assertion_failures_and_runtime_operands_keep_actual_source_locations() {
    for (source, message, selected) in [
        (
            "#assert false \"unsupported target\"; main::()->int{return 42;}",
            "unsupported target",
            "#assert false \"unsupported target\";",
        ),
        (
            "main::()->int{ #assert(false, \"chosen assertion\"); return 42; }",
            "chosen assertion",
            "#assert(false, \"chosen assertion\");",
        ),
        (
            "main::()->int{ enabled:=true; #assert enabled; return 42; }",
            "requires compile-time values",
            "enabled",
        ),
        (
            "#assert false&&absent(); main::()->int{return 42;}",
            "absent",
            "absent()",
        ),
    ] {
        let error = resolve_graph(&graph(source)).unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
        assert_eq!(error.location.span.text(source), selected, "{error:?}");
    }
    assert!(
        std::panic::catch_unwind(|| graph(
            "main::()->int{ #if false { #assert(1 +); } return 42; }"
        ))
        .is_err()
    );
}

#[test]
fn baked_nominal_iterator_flags_use_contextual_members_in_assertions() {
    let declarations = "Flags::enum_flags u32{POINTER::1;REVERSE::2;} check::($flags:Flags)->int{#assert(!(flags & .REVERSE));return 42;}";
    let accepted = format!("{declarations} main::()->int{{return check(.POINTER);}}");
    assert_eq!(value(&resolve_graph(&graph(&accepted)).unwrap()), 42);
    let rejected = format!("{declarations} main::()->int{{return check(.REVERSE);}}");
    let error = resolve_graph(&graph(&rejected)).unwrap_err();
    assert!(
        error.message.contains("compile-time assertion failed"),
        "{error}"
    );
    assert_eq!(
        error.location.span.text(&rejected),
        "#assert(!(flags & .REVERSE));"
    );
}
