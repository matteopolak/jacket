//! File input and generated source overlays share the same graph loading rules.
use std::{
    collections::HashMap,
    fs, io,
    path::{Component, Path, PathBuf},
};

pub trait SourceProvider {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn is_file(&self, path: &Path) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Filesystem;
impl SourceProvider for Filesystem {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        path.canonicalize()
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }
}

/// Virtual files override explicitly selected paths; other paths use the filesystem.
#[derive(Debug, Default)]
pub struct SourceOverlay {
    files: HashMap<PathBuf, Vec<u8>>,
}
impl SourceOverlay {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, path: &Path, bytes: Vec<u8>) -> io::Result<PathBuf> {
        let path = virtual_path(path)?;
        self.files.insert(path.clone(), bytes);
        Ok(path)
    }
}
impl SourceProvider for SourceOverlay {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let virtual_path = virtual_path(path)?;
        if self.files.contains_key(&virtual_path) {
            Ok(virtual_path)
        } else {
            Filesystem.canonicalize(path)
        }
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let key = virtual_path(path)?;
        match self.files.get(&key) {
            Some(bytes) => Ok(bytes.clone()),
            None => Filesystem.read(path),
        }
    }
    fn is_file(&self, path: &Path) -> bool {
        virtual_path(path).is_ok_and(|key| self.files.contains_key(&key))
            || Filesystem.is_file(path)
    }
}

fn virtual_path(path: &Path) -> io::Result<PathBuf> {
    if let Ok(existing) = path.canonicalize() {
        return Ok(existing);
    }
    if let (Some(parent), Some(name)) = (path.parent(), path.file_name())
        && let Ok(parent) = parent.canonicalize()
    {
        return Ok(parent.join(name));
    }
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

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
