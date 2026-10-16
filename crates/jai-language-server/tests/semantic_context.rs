//! Original passive-observation fixtures; no Jai program execution or library API oracle.
use jai_language_server::{DocumentUri, Environment, Limits, Session};
use jaic::sema::{Options, VirtualFs};
use serde_json::Value;
use std::{cell::Cell, path::PathBuf, rc::Rc};

fn fixture(configured: bool) -> (Session, DocumentUri, Rc<Cell<usize>>) {
    let mut fs = VirtualFs::default();
    if configured {
        fs.insert(
            "/observed/jai.toml",
            "build_files = [\"entry.jai\"]\n".as_bytes(),
        );
        fs.insert(
            "/observed/entry.jai",
            "#load \"leaf.jai\";\nmain :: () { value := answer; }".as_bytes(),
        );
    }
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    let mut session = Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(fs),
            options: Box::new(move |root| {
                observed.set(observed.get() + 1);
                let mut options = Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options.import_paths =
                    vec![PathBuf::from("/actual-options").join(root.file_name().unwrap())];
                options
            }),
        },
    );
    let leaf = DocumentUri::parse("file:///observed/leaf.jai").unwrap();
    session
        .open(leaf.clone(), 1, "answer :: 42;".into())
        .unwrap();
    (session, leaf, calls)
}

#[test]
fn actual_root_observation_distinguishes_configured_entry_from_standalone_leaf() {
    let (configured, leaf, _) = fixture(true);
    let (standalone, _, _) = fixture(false);
    let under_entry = configured.semantic_coverage(&leaf).unwrap().unwrap();
    let alone = standalone.semantic_coverage(&leaf).unwrap().unwrap();
    assert!(under_entry.source_loaded && alone.source_loaded);
    assert_eq!(
        under_entry.analysis.root.as_ref().unwrap().path,
        PathBuf::from("/observed/entry.jai")
    );
    assert_eq!(
        under_entry.analysis.entry.as_ref().unwrap().path,
        PathBuf::from("/observed/entry.jai")
    );
    assert_eq!(
        alone.analysis.entry.as_ref().unwrap().path,
        PathBuf::from("/observed/leaf.jai")
    );
    assert_eq!(
        under_entry.analysis.import_dirs.as_ref().unwrap()[0].path,
        PathBuf::from("/actual-options/entry.jai")
    );
    assert_eq!(
        alone.analysis.import_dirs.as_ref().unwrap()[0].path,
        PathBuf::from("/actual-options/leaf.jai")
    );
    let entry = serde_json::to_value(&under_entry).unwrap();
    let leaf = serde_json::to_value(&alone).unwrap();
    assert!(entry["analysis"]["root"].is_object(), "{entry}");
    assert_eq!(entry["analysis"]["root"], entry["analysis"]["entry"]);
    assert_eq!(leaf["analysis"]["root"], leaf["analysis"]["entry"]);
    assert_ne!(entry["analysis"]["root"], leaf["analysis"]["root"]);
    assert_ne!(under_entry.context_id, alone.context_id);
}

#[test]
fn observation_reuses_analysis_and_does_not_requery_options() {
    let (session, leaf, calls) = fixture(true);
    session.diagnostics(&leaf).unwrap();
    let before = (session.compile_counts(), calls.get());
    let first = session.semantic_coverage(&leaf).unwrap().unwrap();
    let second = session.semantic_coverage(&leaf).unwrap().unwrap();
    assert_eq!(before, (session.compile_counts(), calls.get()));
    let first = serde_json::to_value(first).unwrap();
    assert_eq!(first, serde_json::to_value(second).unwrap());
    assert!(first["analysis"]["import_dirs"].is_array(), "{first}");
    assert_eq!(
        first["analysis"]["workspace_id"],
        jaic::build::TOP_LEVEL_WORKSPACE
    );
}

#[test]
fn serialization_never_adds_raw_paths_to_existing_redacted_reports() {
    let (session, leaf, _) = fixture(true);
    let coverage = session.semantic_coverage(&leaf).unwrap().unwrap();
    let json = serde_json::to_string(&coverage).unwrap();
    assert!(
        !json.contains("/observed") && !json.contains("/actual-options"),
        "{json}"
    );
    let debug = format!("{coverage:?}");
    assert!(
        !debug.contains("/observed") && !debug.contains("/actual-options"),
        "{debug}"
    );
    let value: Value = serde_json::from_str(&json).unwrap();
    assert!(value["analysis"]["root"]["identity"].is_string(), "{value}");
}

#[test]
fn unknown_observations_remain_null_and_syntax_only_has_no_semantic_context() {
    let unknown = jai_language_server::SemanticAnalysisContext::default();
    let json = serde_json::to_value(unknown).unwrap();
    for field in ["root", "entry", "import_dirs", "workspace_id"] {
        assert!(json[field].is_null());
    }
    let mut syntax = Session::new(Limits::default());
    let uri = DocumentUri::parse("file:///observed/leaf.jai").unwrap();
    syntax.open(uri.clone(), 1, "answer :: 42;".into()).unwrap();
    assert!(syntax.semantic_coverage(&uri).unwrap().is_none());
    assert_eq!(syntax.compile_counts(), (0, 0));
}
