use jai_modules::{GraphDiscovery, GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;

fn graph(source: &str) -> Result<ModuleGraph, jai_source::LocatedDiagnostic> {
    let mut sources = SourceOverlay::new();
    sources
        .insert(
            Path::new("/own-source-case/main.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    let mut discovery = GraphDiscovery::new(
        Path::new("/own-source-case/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap();
    for _ in 0..32 {
        if discovery.advance().unwrap().is_complete() {
            return Ok(discovery.into_graph().ok().unwrap());
        }
        let cases = discovery.pending_cases().cloned().collect::<Vec<_>>();
        assert!(
            !cases.is_empty(),
            "fixture only requires typed source-case discovery"
        );
        let outcome = crate::resolve_discovery_cases(
            discovery.graph(),
            &cases,
            &crate::ResolveOptions::default(),
            &mut jai_vm::NoEffects,
        )?;
        if outcome.decisions.is_empty() {
            return Err(outcome.pending.into_iter().next().unwrap().diagnostic);
        }
        for (request, choice) in outcome.decisions {
            discovery.select_case(request, choice).unwrap();
        }
    }
    panic!("source case discovery did not converge")
}
fn result(source: &str) -> i128 {
    let graph = graph(source).unwrap();
    let program = crate::resolve_graph(&graph).unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}")
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("{values:?}")
    };
    value.value()
}
#[test]
fn procedure_enum_cases_select_original_declarations_and_through() {
    assert_eq!(
        result(
            "Tag::enum{OTHER;CHOSEN;LAST;} selected::Tag.CHOSEN; main::()->int{#if selected == {case .OTHER; value:=missing();case .CHOSEN; value:=40;#through;case .LAST;value+=2;}return value;}"
        ),
        42
    );
}
#[test]
fn source_type_cases_use_canonical_type_identity() {
    assert_eq!(
        result(
            "Alias::u8; main::()->int{#if Alias == {case s32; ANSWER::missing;case u8;ANSWER::42;case;ANSWER::missing_too;}return ANSWER;}"
        ),
        42
    );
}
#[test]
fn typed_file_enum_cases_do_not_load_inactive_dependencies() {
    assert_eq!(
        result(
            "Tag::enum{OTHER;CHOSEN;}selected::Tag.CHOSEN;#if selected == {case .OTHER;#load \"missing.jai\";case .CHOSEN;ANSWER::42;}main::()->int{return ANSWER;}"
        ),
        42
    );
}
#[test]
fn file_cases_preserve_nominal_module_parameter_aliases() {
    for source in [
        "#module_parameters(K:Kind=.B){Kind::enum{A;B;}};alias::K;#if alias == {case .A;#load \"missing.jai\";case .B;ANSWER::42;}main::()->int{return ANSWER;}",
        "#module_parameters(T:Type=u8);alias::T;#if alias == {case s32;#load \"missing.jai\";case u8;ANSWER::42;}main::()->int{return ANSWER;}",
        "#module_parameters(K:string=\"B\");alias::K;#if alias == {case \"A\";#load \"missing.jai\";case \"B\";ANSWER::42;}main::()->int{return ANSWER;}",
    ] {
        assert_eq!(result(source), 42);
    }
}
#[test]
fn active_file_case_assertions_run_and_inactive_assertions_stay_unbound() {
    assert_eq!(
        result(
            "Tag::enum{A;B;}#if Tag.B == {case .A;#assert missing_value;case .B;#assert true;ANSWER::42;}main::()->int{return ANSWER;}"
        ),
        42
    );
    for source in [
        "#if 1 == {case 1;#assert false \"active case\";case;#assert true;}main::()->int{return 42;}",
        "Tag::enum{A;B;}#if Tag.B == {case .A;#assert true;case .B;#assert false \"active case\";}main::()->int{return 42;}",
        "#if 1 == {case 1;#through;case 2;#assert false \"active case\";}main::()->int{return 42;}",
    ] {
        let graph = graph(source).unwrap();
        let error = crate::resolve_graph(&graph).unwrap_err();
        assert!(error.message.contains("active case"), "{error:?}");
    }
}
#[test]
fn canonical_duplicate_and_incompatible_labels_fail() {
    for (source, message) in [
        (
            "Tag::enum{A;B;} alias::Tag.A; main::(){#if Tag.A == {case .A;case alias;}}",
            "duplicate",
        ),
        (
            "Left::enum{A;}Right::enum{A;}main::(){#if Left.A == {case Right.A;}}",
            "enum",
        ),
        ("main::(){#if #complete true == {case true;}}", "cover"),
        ("main::(value:int){#if value == {case 1;}}", "compile-time"),
    ] {
        let graph = graph(source).unwrap();
        let error = crate::resolve_graph(&graph).unwrap_err();
        assert!(error.message.contains(message), "{}", error.message);
    }
}
#[test]
fn complete_enum_tables_and_default_not_equal_cases_select_correctly() {
    assert_eq!(
        result(
            "Tag::enum{A;B;}main::()->int{#if #complete Tag.B == {case .A;ANSWER::1;case .B;ANSWER::42;}return ANSWER;}"
        ),
        42
    );
    assert_eq!(
        result(
            "main::()->int{#if 2 != {case 2;ANSWER::missing;case 1;ANSWER::42;case;ANSWER::missing_too;}return ANSWER;}"
        ),
        42
    );
}
