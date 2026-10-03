//! Pure source discovery and checking retain one actual compiler session.
use crate::{Error, source_configuration::SourceConfiguration};
use jai_codegen::target::NativeTarget;
use jai_driver::{
    CompilationUnit, CompilerSession, DiscoveryEffectPolicy, EffectReplayCache, ReplayLimits,
    SemanticDiscoveryOptions,
};
use std::path::Path;

pub enum CheckKind {
    Application,
    Library,
}

pub fn run(
    path: &Path,
    sources: SourceConfiguration,
    target: &NativeTarget,
    kind: CheckKind,
) -> Result<(), Error> {
    let policy = DiscoveryEffectPolicy::Disabled;
    let layout = target
        .layout_policy()
        .map_err(|error| Error::Source(error.to_string()))?;
    let mut session = CompilerSession::new();
    let mut replay = EffectReplayCache::new(ReplayLimits::default());
    let options = SemanticDiscoveryOptions {
        graph: sources.graph,
        bootstrap: sources.bootstrap,
        target: target
            .build_target()
            .map_err(|error| Error::Source(error.to_string()))?,
        workspace: session.root(),
        limits: sources.compile_time_limits,
        effect_policy: policy,
    };
    let unit =
        CompilationUnit::load_with_bootstrap_session(path, options, &mut session, &mut replay)
            .map_err(|error| Error::Source(error.to_string()))?;
    match kind {
        CheckKind::Application => {
            let program = unit
                .resolve_discovered_with_session(layout, &mut session, &mut replay, policy)
                .map_err(|error| Error::Source(error.to_string()))?;
            crate::source_warnings::emit(program.library());
            println!("checked {}", path.display());
        }
        CheckKind::Library => {
            let library = unit
                .resolve_library_discovered_with_session(layout, &mut session, &mut replay, policy)
                .map_err(|error| Error::Source(error.to_string()))?;
            crate::source_warnings::emit(&library);
            println!("checked library {}", path.display());
        }
    }
    Ok(())
}
