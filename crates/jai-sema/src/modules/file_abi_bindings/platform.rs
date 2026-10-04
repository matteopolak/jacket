//! Explicit platform-owned source selection preserves actual immutable graph receipts.
use super::*;
use jai_source::SourceProvider;

impl FileAbiBindingContext {
    pub fn from_graph_with_provider(
        graph: &ModuleGraph,
        import_dirs: &[PathBuf],
        target: BuildTarget,
        provider: &dyn SourceProvider,
    ) -> Option<Self> {
        if target.layout != jai_types::LayoutPolicy::lp64() {
            return None;
        }
        let relative = match (&target.operating_system, &target.architecture) {
            (OperatingSystem::MacOS, Architecture::Arm64) => "macos/arm64/stdio.jai",
            (OperatingSystem::MacOS, Architecture::X86_64) => "macos/x64/stdio.jai",
            (OperatingSystem::Linux, Architecture::Arm64 | Architecture::X86_64) => {
                "linux/stdio.jai"
            }
            _ => return None,
        };
        for directory in import_dirs {
            let (Ok(entry_path), Ok(stdio_path)) = (
                provider.canonicalize(&directory.join("POSIX/module.jai")),
                provider.canonicalize(&directory.join("POSIX/bindings").join(relative)),
            ) else {
                continue;
            };
            let allocator_path = provider
                .canonicalize(&directory.join("Default_Allocator/module.jai"))
                .ok();
            let allocator = graph
                .modules()
                .iter()
                .filter_map(|module| receipt(graph, module.entry()))
                .find(|receipt| Some(&receipt.path) == allocator_path.as_ref());
            for module in graph.modules() {
                let entry = receipt(graph, module.entry())?;
                if entry.path != entry_path {
                    continue;
                }
                for file in module.files() {
                    let stdio = receipt(graph, *file)?;
                    if stdio.path == stdio_path {
                        return Self::from_selected_sources(
                            graph,
                            entry.file,
                            stdio.file,
                            allocator.as_ref().map(|receipt| receipt.file),
                            target,
                        );
                    }
                }
            }
        }
        None
    }
    /// Trusted embedding selection for an independently authored virtual stdio module.
    /// No host library is opened. The real declarations, nominal FILE, signatures,
    /// library identity and selected target are validated by the ordinary binder.
    pub fn from_selected_sources(
        graph: &ModuleGraph,
        entry: FileInstanceId,
        stdio: FileInstanceId,
        allocator: Option<FileInstanceId>,
        target: BuildTarget,
    ) -> Option<Self> {
        if graph.target().is_some_and(|actual| actual != &target) {
            return None;
        }
        let module = graph
            .modules()
            .iter()
            .find(|module| module.entry() == entry && module.files().contains(&stdio))?;
        if graph.file(stdio)?.module() != module.id() {
            return None;
        }
        let allocator = match allocator {
            Some(file) => Some(receipt(graph, file)?),
            None => None,
        };
        Some(Self {
            unit: graph.unit(),
            target,
            entry: receipt(graph, entry)?,
            stdio: receipt(graph, stdio)?,
            allocator,
        })
    }
}
