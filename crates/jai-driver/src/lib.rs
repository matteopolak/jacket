//! Coordinate independently scoped sources and checked compilation programs.
pub use jai_modules as modules;
use jai_modules::{DependencyKind, GraphError, GraphOptions, ModuleGraph};
use jai_source::{Diagnostic, LocatedDiagnostic};
pub use jai_source::{ModuleId, SourceId, SourceRecord as Source, UnitId};
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct CompilationUnit {
    graph: ModuleGraph,
}
#[derive(Debug)]
pub enum Error {
    Io {
        path: PathBuf,
        cause: std::io::Error,
    },
    Decode {
        path: PathBuf,
        diagnostic: Diagnostic,
    },
    Located {
        source: SourceId,
        path: PathBuf,
        diagnostic: Diagnostic,
        rendered: String,
    },
    LoadCycle {
        path: PathBuf,
    },
    Graph(GraphError),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, cause } => write!(f, "{}: {cause}", path.display()),
            Self::Decode { path, diagnostic } => write!(f, "{}: {diagnostic}", path.display()),
            Self::Located { rendered, .. } => f.write_str(rendered),
            Self::LoadCycle { path } => write!(f, "{}: cyclic #load", path.display()),
            Self::Graph(error) => fmt::Display::fmt(error, f),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { cause, .. } => Some(cause),
            Self::Decode { diagnostic, .. } | Self::Located { diagnostic, .. } => Some(diagnostic),
            Self::Graph(error) => Some(error),
            Self::LoadCycle { .. } => None,
        }
    }
}
impl From<GraphError> for Error {
    fn from(error: GraphError) -> Self {
        match error {
            GraphError::Io { path, cause } => Self::Io { path, cause },
            GraphError::Decode { path, diagnostic } => Self::Decode { path, diagnostic },
            GraphError::Cycle {
                kind: DependencyKind::Load,
                path,
                ..
            } => Self::LoadCycle { path },
            other => Self::Graph(other),
        }
    }
}
impl CompilationUnit {
    pub fn load(path: &Path) -> Result<Self, Error> {
        Self::load_with_options(path, GraphOptions::default())
    }
    pub fn load_with_options(path: &Path, options: GraphOptions) -> Result<Self, Error> {
        Ok(Self {
            graph: ModuleGraph::load(path, options)?,
        })
    }
    pub fn id(&self) -> UnitId {
        self.graph.unit()
    }
    pub fn module(&self) -> ModuleId {
        self.graph.root()
    }
    pub fn sources(&self) -> &[Source] {
        self.graph.sources().records()
    }
    pub fn graph(&self) -> &ModuleGraph {
        &self.graph
    }
    pub fn resolve(&self) -> Result<jai_sema::Program, Error> {
        jai_sema::resolve_graph(&self.graph).map_err(|error| self.located(error))
    }
    pub fn resolve_library(&self) -> Result<jai_sema::Library, Error> {
        jai_sema::resolve_library(&self.graph).map_err(|error| self.located(error))
    }
    fn located(&self, error: LocatedDiagnostic) -> Error {
        let record = self
            .graph
            .sources()
            .get(error.location.source)
            .expect("semantic diagnostics retain graph source identities");
        let rendered = error.render(self.graph.sources());
        Error::Located {
            source: error.location.source,
            path: record.path().to_owned(),
            diagnostic: Diagnostic::new(error.location.span, error.message),
            rendered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new(files: &[(&str, &str)]) -> Self {
            let root = std::env::temp_dir().join(format!(
                "jai-driver-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&root).unwrap();
            for (name, text) in files {
                fs::write(root.join(name), text).unwrap();
            }
            Self(root)
        }
        fn load(&self) -> Result<CompilationUnit, Error> {
            CompilationUnit::load(&self.0.join("main.jai"))
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn recursive_load_resolves_symbols_and_deduplicates() {
        let f = Fixture::new(&[
            (
                "main.jai",
                "#load \"helper.jai\"; #load \"./helper.jai\"; main :: () -> int { return answer(); }",
            ),
            (
                "helper.jai",
                "#load \"value.jai\"; answer :: () -> int { return value; }",
            ),
            ("value.jai", "value :: 42;"),
        ]);
        let unit = f.load().unwrap();
        assert_eq!(unit.sources().len(), 3);
        unit.resolve().unwrap();
    }
    #[test]
    fn diagnostic_points_into_loaded_file() {
        let f = Fixture::new(&[
            ("main.jai", "#load \"bad.jai\"; main :: () {}"),
            ("bad.jai", "\nbad :: () -> int { return absent; }"),
        ]);
        let error = f.load().unwrap().resolve().unwrap_err().to_string();
        assert!(error.contains("bad.jai:2:"), "{error}");
    }
    #[test]
    fn structured_errors_and_immutable_source_identity() {
        let fixture = Fixture::new(&[
            ("main.jai", "#load \"bad.jai\"; main :: () {}"),
            ("bad.jai", "\nbad :: () -> int { return absent; }"),
        ]);
        let unit = fixture.load().unwrap();
        assert_eq!(unit.id().index(), 0);
        assert_eq!(unit.module().index(), 0);
        let source = &unit.sources()[1];
        assert_eq!(source.id().index(), 1);
        assert_eq!(source.path().file_name().unwrap(), "bad.jai");
        assert!(source.text().starts_with('\n'));
        match unit.resolve().unwrap_err() {
            Error::Located {
                source: id,
                path,
                diagnostic,
                ..
            } => {
                assert_eq!(id, source.id());
                assert_eq!(path, source.path());
                assert!(diagnostic.span.start > 0);
            }
            other => panic!("expected located diagnostic, got {other:?}"),
        }
        assert!(matches!(
            Fixture::new(&[("main.jai", "#load \"main.jai\";")]).load(),
            Err(Error::LoadCycle { .. })
        ));
        assert!(matches!(
            Fixture::new(&[("main.jai", "#load \"missing.jai\";")]).load(),
            Err(Error::Io { .. })
        ));
    }
    #[test]
    fn cycles_missing_files_and_imports_fail() {
        for files in [
            vec![("main.jai", "#load \"main.jai\";")],
            vec![("main.jai", "#load \"missing.jai\";")],
            vec![("main.jai", "#import \"Basic\";")],
            vec![("main.jai", "main :: () { #load \"a.jai\"; }")],
            vec![("main.jai", "#load \"a.jai\"")],
        ] {
            assert!(Fixture::new(&files).load().is_err());
        }
    }
}
