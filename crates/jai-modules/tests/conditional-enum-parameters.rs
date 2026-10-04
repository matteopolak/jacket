use jai_modules::{Binding, GraphOptions, ModuleGraph, ParameterValue, SourceOverlay};
use jai_syntax::NamePath;
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

#[test]
fn selected_enum_default_keeps_member_bits_and_same_instance_identity() {
    let mut source = closed_sources();
    for (name, text) in [
        (
            "main.jai",
            "A::#import,file \"renderer.jai\";B::#import,file \"renderer.jai\";",
        ),
        (
            "renderer.jai",
            "#module_parameters(Tag:TagEnum=TagEnum.NEXT){TagEnum::enum u8{A::7;#if A==7{NEXT;}else{NEXT::99;}}} #if Tag==.NEXT{#load \"selected.jai\";}else{#load \"missing.jai\";}",
        ),
        ("selected.jai", "answer::42;"),
    ] {
        source
            .insert(
                &Path::new("/jai-enum-parameters").join(name),
                text.as_bytes().to_vec(),
            )
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        Path::new("/jai-enum-parameters/main.jai"),
        GraphOptions::default(),
        &source,
    )
    .unwrap();
    let entry = graph.module(graph.root()).unwrap().entry();
    let binding = |name| {
        graph
            .lookup(
                entry,
                &NamePath {
                    root: graph.symbols().find(name).unwrap(),
                    members: vec![],
                },
            )
            .unwrap()
    };
    assert!(matches!(binding("A"), Binding::Module(_)));
    assert_eq!(binding("A"), binding("B"));
    let parameter = graph
        .parameters()
        .iter()
        .find(|parameter| graph.symbols().name(parameter.name) == "Tag")
        .unwrap();
    let ParameterValue::Enumeration(value) = &parameter.value else {
        panic!()
    };
    assert_eq!(value.value.value(), 8);
    assert!(graph.files().iter().all(|file| {
        !graph
            .sources()
            .get(file.source())
            .unwrap()
            .path()
            .ends_with("missing.jai")
    }));
}
