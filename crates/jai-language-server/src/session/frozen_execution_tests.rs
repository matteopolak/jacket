//! Lifetime execution survives the actual editor code-release boundary.
use super::*;
use std::path::Path;

#[test]
fn frozen_execution_lints_use_retained_topology_after_editor_release() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root
        .join("tests/compiler/regressions/run-result-frozen-values.jai")
        .canonicalize()
        .unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let environment = test_support::lsp::environment();
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: environment.fs,
            services: None,
            options: environment.options,
        },
    );
    let config = DocumentUri::from_path(&path.parent().unwrap().join("jacket.toml")).unwrap();
    session
        .open(
            config,
            1,
            "[project]\nentry=[\"run-result-frozen-values.jai\"]\n".into(),
        )
        .unwrap();
    let uri = DocumentUri::from_path(&path).unwrap();
    session.open(uri.clone(), 1, text.clone()).unwrap();
    let diagnostics = session.diagnostics(&uri).unwrap();
    assert!(
        !diagnostics
            .iter()
            .any(|d| d.code == crate::DiagnosticCode::Lint("returned_local_address")),
        "{diagnostics:?}"
    );
    let usage = session
        .with_semantic(&uri, &text, |a, _| {
            Some(a.compiler.lifetime_execution_usage())
        })
        .unwrap();
    assert!(
        usage.0 > 0,
        "the actual adapter supplies complete-program retention at release"
    );
}
