//! Coordinate independently scoped sources and checked compilation programs.
mod compiler_effects;
pub mod host_io;

mod discovery_worklists;
mod effect_replay;
mod graph_discovery_session;
mod graph_job;
mod runtime_support;
mod source_discovery;
pub use discovery_worklists::{SourceDiscoveryPending, SourceDiscoveryRequest};
pub use graph_discovery_session::{DiscoveryQuery, PreparedGraphDiscoverySession};
pub use graph_job::{GraphJobProgress, GraphJobResult, PreparedGraphJob};
pub use source_discovery::{
    DiscoveryEffectPolicy, SemanticDiscoveryOptions, discover_graph_with_session,
};
mod workspace_job;
mod workspace_scheduler;
pub use compiler_effects::*;
pub use effect_replay::{EffectReplayCache, ReplayEffects, ReplayLimits};
pub use jai_modules as modules;
use jai_modules::{DependencyKind, GraphError, GraphOptions, ModuleGraph};
use jai_source::{Diagnostic, LocatedDiagnostic};
pub use jai_source::{ModuleId, SourceId, SourceRecord as Source, UnitId};
use std::{
    fmt,
    path::{Path, PathBuf},
};
pub use workspace_job::{
    PreparedWorkspaceJob, WorkspaceJobProgress, WorkspaceJobResult, WorkspaceSourceRebuild,
};
pub use workspace_scheduler::*;

#[derive(Debug)]
pub struct CompilationUnit {
    graph: ModuleGraph,
    options: GraphOptions,
    compile_time_limits: jai_vm::Limits,
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
    CompilerReport(CompilerMessage),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, cause } => write!(f, "{}: {cause}", path.display()),
            Self::Decode { path, diagnostic } => write!(f, "{}: {diagnostic}", path.display()),
            Self::Located { rendered, .. } => f.write_str(rendered),
            Self::LoadCycle { path } => write!(f, "{}: cyclic #load", path.display()),
            Self::Graph(error) => fmt::Display::fmt(error, f),
            Self::CompilerReport(message) => {
                if let Some(location) = &message.location {
                    write!(
                        f,
                        "{}:{}:{}: error: {}",
                        location.path.display(),
                        location.line,
                        location.column,
                        message.text
                    )
                } else {
                    write!(f, "error: {}", message.text)
                }
            }
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { cause, .. } => Some(cause),
            Self::Decode { diagnostic, .. } | Self::Located { diagnostic, .. } => Some(diagnostic),
            Self::Graph(error) => Some(error),
            Self::LoadCycle { .. } | Self::CompilerReport(_) => None,
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
    /// Load configured actual automatic modules using explicit runtime build policy.
    pub fn load_with_bootstrap(
        path: &Path,
        options: GraphOptions,
        bootstrap: modules::BootstrapOptions,
        target: Option<jai_types::BuildTarget>,
    ) -> Result<Self, Error> {
        Ok(Self {
            graph: ModuleGraph::load_with_bootstrap_options(
                path,
                options.clone(),
                bootstrap,
                &modules::Filesystem,
                target,
            )?,
            options,
            compile_time_limits: jai_vm::Limits::default(),
        })
    }
    pub fn load(path: &Path) -> Result<Self, Error> {
        Self::load_with_options(path, GraphOptions::default())
    }
    pub fn load_with_options(path: &Path, options: GraphOptions) -> Result<Self, Error> {
        Ok(Self {
            graph: ModuleGraph::load(path, options.clone())?,
            options,
            compile_time_limits: jai_vm::Limits::default(),
        })
    }
    pub fn load_with_target(
        path: &Path,
        options: GraphOptions,
        target: jai_types::BuildTarget,
    ) -> Result<Self, Error> {
        Ok(Self {
            graph: ModuleGraph::load_with_target(
                path,
                options.clone(),
                &modules::Filesystem,
                target,
            )?,
            options,
            compile_time_limits: jai_vm::Limits::default(),
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
    pub fn resolve_with_session(
        &self,
        layout: jai_types::LayoutPolicy,
        session: &mut CompilerSession,
    ) -> Result<jai_sema::Program, Error> {
        let options = self.resolve_options(layout, session);
        let workspace = session.root();
        source_discovery::require_idle(session)?;
        let mut compiler = session.clone();
        let program = jai_sema::resolve_graph_with_options(&self.graph, &options, &mut compiler)
            .map_err(|error| {
                source_discovery::record_failure(session, workspace, self.located(error))
            })?;
        if let Some(error) = compiler.error() {
            return Err(source_discovery::record_failure(
                session,
                workspace,
                Error::CompilerReport(error.clone()),
            ));
        }
        *session = compiler;
        Ok(program)
    }
    pub fn resolve_library_with_session(
        &self,
        layout: jai_types::LayoutPolicy,
        session: &mut CompilerSession,
    ) -> Result<jai_sema::Library, Error> {
        let options = self.resolve_options(layout, session);
        let workspace = session.root();
        source_discovery::require_idle(session)?;
        let mut compiler = session.clone();
        let library = jai_sema::resolve_library_with_options(&self.graph, &options, &mut compiler)
            .map_err(|error| {
                source_discovery::record_failure(session, workspace, self.located(error))
            })?;
        if let Some(error) = compiler.error() {
            return Err(source_discovery::record_failure(
                session,
                workspace,
                Error::CompilerReport(error.clone()),
            ));
        }
        *session = compiler;
        Ok(library)
    }
    fn resolve_options(
        &self,
        layout: jai_types::LayoutPolicy,
        session: &CompilerSession,
    ) -> jai_sema::ResolveOptions {
        jai_sema::ResolveOptions {
            compile_time_limits: self.compile_time_limits,
            file_abi: self.graph.target().and_then(|target| {
                jai_sema::FileAbiBindingContext::allocator_from_graph(
                    &self.graph,
                    &self.options.import_dirs,
                    target.clone(),
                )
            }),
            target: self.graph.target().cloned(),
            layout: Some(layout),
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &self.graph,
                &self.options.import_dirs,
                session.root(),
            )),
            ..Default::default()
        }
    }
    pub fn resolve_with_target(
        &self,
        target: jai_types::BuildTarget,
    ) -> Result<jai_sema::Program, Error> {
        let session = CompilerSession::new();
        let options = jai_sema::ResolveOptions {
            compile_time_limits: self.compile_time_limits,
            file_abi: jai_sema::FileAbiBindingContext::allocator_from_graph(
                &self.graph,
                &self.options.import_dirs,
                target.clone(),
            ),
            target: Some(target),
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &self.graph,
                &self.options.import_dirs,
                session.root(),
            )),
            ..Default::default()
        };
        jai_sema::resolve_graph_with_options(&self.graph, &options, &mut jai_vm::NoEffects)
            .map_err(|error| self.located(error))
    }
    pub fn resolve_library_with_target(
        &self,
        target: jai_types::BuildTarget,
    ) -> Result<jai_sema::Library, Error> {
        let session = CompilerSession::new();
        let options = jai_sema::ResolveOptions {
            compile_time_limits: self.compile_time_limits,
            file_abi: jai_sema::FileAbiBindingContext::allocator_from_graph(
                &self.graph,
                &self.options.import_dirs,
                target.clone(),
            ),
            target: Some(target),
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &self.graph,
                &self.options.import_dirs,
                session.root(),
            )),
            ..Default::default()
        };
        jai_sema::resolve_library_with_options(&self.graph, &options, &mut jai_vm::NoEffects)
            .map_err(|error| self.located(error))
    }
    pub fn resolve_with_layout(
        &self,
        layout: jai_types::LayoutPolicy,
    ) -> Result<jai_sema::Program, Error> {
        let options = jai_sema::ResolveOptions {
            compile_time_limits: self.compile_time_limits,
            layout: Some(layout),
            ..Default::default()
        };
        jai_sema::resolve_graph_with_options(&self.graph, &options, &mut jai_vm::NoEffects)
            .map_err(|error| self.located(error))
    }
    pub fn resolve_library_with_layout(
        &self,
        layout: jai_types::LayoutPolicy,
    ) -> Result<jai_sema::Library, Error> {
        let options = jai_sema::ResolveOptions {
            compile_time_limits: self.compile_time_limits,
            layout: Some(layout),
            ..Default::default()
        };
        jai_sema::resolve_library_with_options(&self.graph, &options, &mut jai_vm::NoEffects)
            .map_err(|error| self.located(error))
    }
    pub fn resolve(&self) -> Result<jai_sema::Program, Error> {
        let options = jai_sema::ResolveOptions {
            compile_time_limits: self.compile_time_limits,
            ..Default::default()
        };
        jai_sema::resolve_graph_with_options(&self.graph, &options, &mut jai_vm::NoEffects)
            .map_err(|error| self.located(error))
    }
    pub fn resolve_library(&self) -> Result<jai_sema::Library, Error> {
        let options = jai_sema::ResolveOptions {
            compile_time_limits: self.compile_time_limits,
            ..Default::default()
        };
        jai_sema::resolve_library_with_options(&self.graph, &options, &mut jai_vm::NoEffects)
            .map_err(|error| self.located(error))
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
            diagnostic: Diagnostic::at_source(error.location, error.message),
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
    fn configured_compile_time_fuel_reaches_final_resolution() {
        let fixture = Fixture::new(&[(
            "main.jai",
            "measure::()->int{sum:=0;for 0..100 {sum+=it;}return sum;} measured::#run measure();main::()->int{return measured;}",
        )]);
        let mut unit = fixture.load().unwrap();
        unit.compile_time_limits.fuel = 1;
        let error = unit.resolve().unwrap_err();
        assert!(error.to_string().contains("Fuel"), "{error}");
        unit.compile_time_limits.fuel = 20_000_000;
        let program = unit.resolve().unwrap();
        let jai_sema::EntryPoint::Int(entry) = program.entry() else {
            panic!("integer entry");
        };
        let mut vm =
            jai_vm::Vm::new(&program, jai_vm::NoEffects, jai_vm::Limits::default()).unwrap();
        assert_eq!(
            vm.execute(entry, vec![]).outcome,
            jai_vm::Outcome::Complete(vec![jai_vm::Value::Int(
                jai_types::Integer::checked(jai_types::IntegerType::S64, 5_050).unwrap()
            )])
        );
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
    fn selected_target_controls_compile_time_layout_and_byte_order() {
        use jai_types::{
            Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem, ScalarLayout,
        };
        let fixture = Fixture::new(&[(
            "main.jai",
            "probe :: () -> int { bits:u32 = 0x01020304; data := cast(*u8) *bits; return cast(int) data[0]; } main :: () -> int { return size_of(*int) + #run probe(); }",
        )]);
        for width in [4, 8] {
            for order in [ByteOrder::Little, ByteOrder::Big] {
                let layout = LayoutPolicy::new(
                    ScalarLayout::new(width, width as u32),
                    [
                        ScalarLayout::new(1, 1),
                        ScalarLayout::new(2, 2),
                        ScalarLayout::new(4, 4),
                        ScalarLayout::new(8, 8),
                    ],
                    [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
                    ScalarLayout::new(1, 1),
                )
                .unwrap();
                let target = BuildTarget {
                    operating_system: OperatingSystem::Linux,
                    architecture: Architecture::Arm,
                    layout,
                    byte_order: order,
                };
                let unit = CompilationUnit::load_with_target(
                    &fixture.0.join("main.jai"),
                    GraphOptions::default(),
                    target.clone(),
                )
                .unwrap();
                assert_eq!(unit.graph().target(), Some(&target));
                let program = unit.resolve_with_target(target.clone()).unwrap();
                let jai_sema::EntryPoint::Int(entry) = program.entry() else {
                    panic!("integer entry");
                };
                let mut vm = jai_vm::Vm::new_with_target(
                    &program,
                    jai_vm::NoEffects,
                    jai_vm::Limits::default(),
                    jai_vm::ByteTarget::from(&target),
                )
                .unwrap();
                let byte = if order == ByteOrder::Big { 1 } else { 4 };
                assert_eq!(
                    vm.execute(entry, vec![]).outcome,
                    jai_vm::Outcome::Complete(vec![jai_vm::Value::Int(
                        jai_types::Integer::checked(
                            jai_types::IntegerType::S64,
                            (width + byte) as i128
                        )
                        .unwrap()
                    )])
                );
            }
        }
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
