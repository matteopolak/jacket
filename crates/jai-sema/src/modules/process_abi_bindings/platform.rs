use super::*;
use jai_source::SourceProvider;
impl ProcessAbiBindingContext {
    pub fn from_graph_with_provider(
        graph: &ModuleGraph,
        import_dirs: &[PathBuf],
        target: BuildTarget,
        provider: &dyn SourceProvider,
    ) -> Option<Self> {
        if target.layout != LayoutPolicy::lp64() || target.byte_order != ByteOrder::Little {
            return None;
        }
        let (bindings, generated) = match (&target.operating_system, &target.architecture) {
            (OperatingSystem::MacOS, Architecture::Arm64) => ("macos/arm64", "generated_macos.jai"),
            (OperatingSystem::MacOS, Architecture::X86_64) => ("macos/x64", "generated_macos.jai"),
            (OperatingSystem::Linux, Architecture::Arm64 | Architecture::X86_64) => {
                ("linux", "generated_linux.jai")
            }
            _ => return None,
        };
        for directory in import_dirs {
            let Some(entry) =
                selected_platform_entry(graph, directory.join("POSIX/module.jai"), provider)
            else {
                continue;
            };
            let Some(base) = selected_platform_file(
                graph,
                &entry,
                directory
                    .join("POSIX/bindings")
                    .join(bindings)
                    .join("base.jai"),
                provider,
            ) else {
                continue;
            };
            let stdio = selected_platform_file(
                graph,
                &entry,
                directory
                    .join("POSIX/bindings")
                    .join(bindings)
                    .join("stdio.jai"),
                provider,
            );
            let socket =
                selected_platform_entry(graph, directory.join("Socket/module.jai"), provider)
                    .and_then(|entry| {
                        selected_platform_file(
                            graph,
                            &entry,
                            directory.join("Socket").join(generated),
                            provider,
                        )
                        .map(|generated| SocketReceipts { entry, generated })
                    });
            return Some(Self {
                unit: graph.unit(),
                target,
                entry,
                base,
                stdio,
                socket,
            });
        }
        None
    }
}
fn selected_platform_entry(
    graph: &ModuleGraph,
    path: PathBuf,
    provider: &dyn SourceProvider,
) -> Option<SourceReceipt> {
    let path = provider.canonicalize(&path).ok()?;
    graph
        .modules()
        .iter()
        .filter_map(|module| receipt(graph, module.entry()))
        .find(|source| source.path == path)
}
fn selected_platform_file(
    graph: &ModuleGraph,
    entry: &SourceReceipt,
    path: PathBuf,
    provider: &dyn SourceProvider,
) -> Option<SourceReceipt> {
    let path = provider.canonicalize(&path).ok()?;
    graph
        .modules()
        .iter()
        .filter(|module| module.entry() == entry.file)
        .flat_map(|module| module.files())
        .filter_map(|file| receipt(graph, *file))
        .find(|source| source.path == path)
}
