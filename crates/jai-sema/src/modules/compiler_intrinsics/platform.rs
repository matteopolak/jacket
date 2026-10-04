use super::*;
use jai_source::SourceProvider;
impl CompilerModuleOrigins {
    pub fn from_graph_with_provider(
        graph: &ModuleGraph,
        import_dirs: &[PathBuf],
        provider: &dyn SourceProvider,
    ) -> Self {
        let mut origins = Self::default();
        origins.register(graph.root(), CompilerModuleOrigin::Application);
        for module in graph.modules() {
            let Some(file) = graph.file(module.entry()) else {
                continue;
            };
            let Some(source) = graph.sources().get(file.source()) else {
                continue;
            };
            for directory in import_dirs {
                for (name, origin) in [
                    ("Compiler", CompilerModuleOrigin::Compiler),
                    ("Preload", CompilerModuleOrigin::Preload),
                ] {
                    let candidates = [
                        directory.join(format!("{name}.jai")),
                        directory.join(name).join("module.jai"),
                    ];
                    if candidates.iter().any(|candidate| {
                        provider
                            .canonicalize(candidate)
                            .is_ok_and(|path| path == source.path())
                    }) {
                        origins.register(module.id(), origin);
                    }
                }
            }
        }
        if let Some(module) = graph.prelude() {
            origins.register(module, CompilerModuleOrigin::Preload);
        }
        if let Some(module) = graph.runtime_support() {
            origins.register(module, CompilerModuleOrigin::RuntimeSupport);
        }
        origins
    }
}
impl CompilerBindingContext {
    pub fn from_graph_with_provider(
        graph: &ModuleGraph,
        import_dirs: &[PathBuf],
        current_workspace: WorkspaceId,
        provider: &dyn SourceProvider,
    ) -> Self {
        Self {
            current_workspace,
            origins: CompilerModuleOrigins::from_graph_with_provider(graph, import_dirs, provider),
        }
    }
}
