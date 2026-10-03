use jai_modules::{PreludeError, PreludeSource, SourceOverlay};
use std::path::{Path, PathBuf};

fn provider(paths: &[&str]) -> SourceOverlay {
    let mut provider = SourceOverlay::new();
    for path in paths {
        provider
            .insert(Path::new(path), b"value :: 42;".to_vec())
            .unwrap();
    }
    provider
}

#[test]
fn search_preserves_root_order_and_file_before_directory_form() {
    let provider = provider(&[
        "/jai-bootstrap/first/module.jai",
        "/jai-bootstrap/first/Preload/module.jai",
        "/jai-bootstrap/second/module.jai",
    ]);
    let roots = [
        "/jai-bootstrap/first".into(),
        "/jai-bootstrap/second".into(),
    ];
    assert_eq!(
        PreludeSource::default().resolve(&roots, &provider).unwrap(),
        Some(PathBuf::from("/jai-bootstrap/first/module.jai"))
    );
    let roots = [roots[1].clone(), roots[0].clone()];
    assert_eq!(
        PreludeSource::Search.resolve(&roots, &provider).unwrap(),
        Some(PathBuf::from("/jai-bootstrap/second/module.jai"))
    );
}

#[test]
fn directory_form_and_exact_override_use_canonical_provider_paths() {
    let provider = provider(&[
        "/jai-bootstrap/first/Preload/module.jai",
        "/jai-bootstrap/custom.jai",
    ]);
    assert_eq!(
        PreludeSource::Search
            .resolve(&["/jai-bootstrap/first".into()], &provider)
            .unwrap(),
        Some(PathBuf::from("/jai-bootstrap/first/Preload/module.jai"))
    );
    assert_eq!(
        PreludeSource::File("/jai-bootstrap/a/../custom.jai".into())
            .resolve(&[], &provider)
            .unwrap(),
        Some(PathBuf::from("/jai-bootstrap/custom.jai"))
    );
}

#[test]
fn missing_required_source_fails_and_disabled_is_explicit() {
    let provider = provider(&[]);
    let roots = vec![PathBuf::from("/jai-bootstrap/absent")];
    assert!(
        matches!(PreludeSource::Search.resolve(&roots, &provider), Err(PreludeError::Missing { roots: searched }) if searched == roots)
    );
    assert!(matches!(
        PreludeSource::File("/jai-bootstrap/absent.jai".into()).resolve(&roots, &provider),
        Err(PreludeError::Io { .. })
    ));
    assert_eq!(
        PreludeSource::Disabled.resolve(&roots, &provider).unwrap(),
        None
    );
}
