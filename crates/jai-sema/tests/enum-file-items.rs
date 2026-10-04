//! Closed source enum selection and literal module imports execute with NoEffects.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_vm::{ByteTarget, ExecutionPhase, Limits, NoEffects, Outcome, Value, Vm};
use std::path::Path;

struct ClosedBase;
impl jai_source::SourceProvider for ClosedBase {
    fn normalize(&self, path: &Path) -> std::io::Result<std::path::PathBuf> {
        jai_source::normalize_virtual_path("/", path)
    }
    fn canonicalize(&self, _path: &Path) -> std::io::Result<std::path::PathBuf> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "closed fixture VFS",
        ))
    }
    fn read(&self, _path: &Path) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "closed fixture VFS",
        ))
    }
    fn is_file(&self, _path: &Path) -> bool {
        false
    }
}
fn closed_sources() -> SourceOverlay {
    SourceOverlay::with_base(std::sync::Arc::new(ClosedBase))
}


fn program(source: &str) -> jai_ir::Program {
    let path = Path::new("/jai-enum-items/main.jai");
    let mut sources = closed_sources();
    sources.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &sources).unwrap();
    let options = ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..Default::default()
    };
    resolve_graph_with_options(&graph, &options, &mut NoEffects).unwrap()
}

fn execute(source: &str, limits: Limits) -> jai_vm::Execution {
    let program = program(source);
    let entry = match program.entry() {
        jai_ir::EntryPoint::Void(id) | jai_ir::EntryPoint::Int(id) => id,
    };
    let mut vm = Vm::new_with_execution_phase(
        &program,
        NoEffects,
        limits,
        ByteTarget::default(),
        ExecutionPhase::Runtime,
    )
    .unwrap();
    vm.execute(entry, vec![])
}

fn answer(source: &str) -> i128 {
    let execution = execute(source, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("{execution:?}")
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("{values:?}")
    };
    value.value()
}

#[test]
fn selected_enum_branch_preserves_progress_and_ignores_inactive_duplicates() {
    assert_eq!(
        answer(
            "E::enum u8{A::7;#if A==7{B;}else{B::99;}#if false{B::88;}C;}main::()->int{return cast(int)E.B+34;}"
        ),
        42
    );
    assert_eq!(
        answer(
            "main::()->int{E::enum_flags u8{READ;#if true{WRITE;}else{WRITE::128;}NEXT;}return cast(int)E.NEXT+38;}"
        ),
        42
    );
}
#[test]
fn inline_enum_layout_and_defaults_share_selected_member_identity() {
    assert_eq!(
        answer(
            "State::struct{tag:enum u8{START::7;#if false{START::99;}else{SELECTED;}END;}=.SELECTED;}main::()->int{state:State;return cast(int)state.tag+34;}"
        ),
        42
    );
}
#[test]
fn here_string_module_source_has_real_closed_import_execution() {
    assert_eq!(
        answer(
            "#import,string #string END\nanswer::()->int{return 42;}\nEND\n; main::()->int{return answer();}"
        ),
        42
    );
}
#[test]
fn inactive_file_and_record_operations_are_never_executed_or_loaded() {
    assert_eq!(
        answer(
            "#if false{#library,link_always \"missing-native\";#poke_name Missing absent; compiler_report(\"unsupported\");}State::struct{#if false{#import \"missing-module\";}value:int=42;}main::()->int{state:State;return state.value;}"
        ),
        42
    );
}
#[test]
fn selected_enum_generator_has_precise_missing_producer_diagnostic() {
    let source = "E::enum{A;#insert -> string{return \"B;\";}}main::()->int{return 42;}";
    let path = Path::new("/jai-enum-items/main.jai");
    let mut sources = closed_sources();
    sources.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &sources).unwrap();
    let error =
        resolve_graph_with_options(&graph, &ResolveOptions::default(), &mut NoEffects).unwrap_err();
    assert!(
        error.message.contains("enum-member expansion producer"),
        "{error:?}"
    );
    let origin = graph.sources().get(error.location.source).unwrap();
    assert_eq!(origin.path(), path);
    assert_eq!(origin.text(), source);
    assert_eq!(
        error.location.span.text(source),
        "#insert -> string{return \"B;\";}"
    );
}
#[test]
fn active_file_operations_have_located_producer_refusals() {
    for (source, message, span) in [
        (
            "#library,link_always \"missing\";",
            "unnamed source-library metadata producer",
            "#library,link_always \"missing\";",
        ),
        (
            "#poke_name Missing name;",
            "cross-module name publication producer",
            "#poke_name Missing name;",
        ),
        (
            "#if true{report(42);}",
            "file-scope execution producer",
            "report(42);",
        ),
        (
            "State::struct{#import \"missing\";value:int;}",
            "record-source namespace producer",
            "#import \"missing\";",
        ),
    ] {
        let path = Path::new("/jai-enum-items/main.jai");
        let mut sources = closed_sources();
        sources.insert(path, source.as_bytes().to_vec()).unwrap();
        let error =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &sources).unwrap_err();
        let jai_modules::GraphError::Located {
            diagnostic, ..
        } = error
        else {
            panic!("{error:?}")
        };
        assert!(diagnostic.message.contains(message), "{diagnostic:?}");
        assert_eq!(diagnostic.location.span.text(source), span);
    }
}
