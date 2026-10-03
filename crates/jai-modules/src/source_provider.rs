//! Graph discovery consumes the selected pure source contract; native adapters live separately.
pub use jai_native_source::{Filesystem, SourceOverlay};
pub use jai_source::SourceProvider;
#[cfg(test)]
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Binding, GraphOptions, ModuleGraph};
    use jai_syntax::NamePath;
    fn overlay(files: &[(&str, &str)]) -> SourceOverlay {
        let mut provider = SourceOverlay::new();
        for (path, source) in files {
            provider
                .insert(Path::new(path), source.as_bytes().to_vec())
                .unwrap();
        }
        provider
    }
    #[test]
    fn generated_files_keep_separate_scopes_and_normal_dependency_resolution() {
        let provider = overlay(&[
            (
                "/jai-virtual/main.jai",
                "#load \"extra.jai\"; A :: #import,file \"a.jai\"; main :: () -> int { return A.value + extra; }",
            ),
            ("/jai-virtual/extra.jai", "extra :: 4;"),
            ("/jai-virtual/a.jai", "value :: 7;"),
        ]);
        let graph = ModuleGraph::load_with_provider(
            Path::new("/jai-virtual/main.jai"),
            GraphOptions::default(),
            &provider,
        )
        .unwrap();
        assert_eq!(graph.files().len(), 3);
        assert_eq!(graph.modules().len(), 2);
        let root = graph.module(graph.root()).unwrap().entry();
        let path = NamePath {
            root: graph.symbols().find("extra").unwrap(),
            members: vec![],
        };
        assert!(matches!(
            graph.lookup(root, &path),
            Ok(Binding::Declaration(_))
        ));
    }
    #[test]
    fn virtual_search_imports_and_canonical_load_cycles_are_real_graph_edges() {
        let provider = overlay(&[
            ("/jai-virtual/main.jai", "A :: #import \"A\"; main :: () {}"),
            ("/jai-virtual/modules/A/module.jai", "value :: 1;"),
        ]);
        let options = GraphOptions {
            import_dirs: vec!["/jai-virtual/modules".into()],
        };
        assert_eq!(
            ModuleGraph::load_with_provider(Path::new("/jai-virtual/main.jai"), options, &provider)
                .unwrap()
                .modules()
                .len(),
            2
        );
        let provider = overlay(&[("/jai-virtual/main.jai", "#load \"./other/../main.jai\";")]);
        assert!(matches!(
            ModuleGraph::load_with_provider(
                Path::new("/jai-virtual/main.jai"),
                GraphOptions::default(),
                &provider
            ),
            Err(crate::GraphError::Cycle { .. })
        ));
    }
    #[test]
    fn generated_parse_errors_retain_their_file_and_decode_rules() {
        let mut provider = overlay(&[
            ("/jai-virtual/main.jai", "#load \"broken.jai\";"),
            ("/jai-virtual/broken.jai", "\nbroken :: () { return +; }"),
        ]);
        let error = ModuleGraph::load_with_provider(
            Path::new("/jai-virtual/main.jai"),
            GraphOptions::default(),
            &provider,
        )
        .unwrap_err();
        assert!(error.to_string().contains("broken.jai:2:"), "{error}");
        provider
            .insert(Path::new("/jai-virtual/main.jai"), vec![0xff])
            .unwrap();
        assert!(matches!(
            ModuleGraph::load_with_provider(
                Path::new("/jai-virtual/main.jai"),
                GraphOptions::default(),
                &provider
            ),
            Err(crate::GraphError::Decode { .. })
        ));
    }
}
