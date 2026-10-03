//! Resume the retained source graph with genuine typed compiler decisions.
use crate::{CompilationUnit, CompilerSession, EffectReplayCache, Error, ReplayEffects};
use jai_modules::{BootstrapOptions, GraphDiscovery, GraphOptions, ModuleGraph, SourceProvider};
use jai_source::LocatedDiagnostic;
use jai_types::BuildTarget;
use jai_vm::{Limits, WorkspaceId};
use std::path::Path;

/// The caller retains the same compiler session and replay cache for final binding.
pub struct SemanticDiscoveryOptions {
    pub graph: GraphOptions,
    pub bootstrap: BootstrapOptions,
    pub target: BuildTarget,
    pub workspace: WorkspaceId,
    pub limits: Limits,
    pub effect_policy: DiscoveryEffectPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryEffectPolicy {
    Disabled,
    CompilerSession,
}

impl CompilationUnit {
    pub fn load_with_bootstrap_session(
        path: &Path,
        options: SemanticDiscoveryOptions,
        session: &mut CompilerSession,
        replay: &mut EffectReplayCache,
    ) -> Result<Self, Error> {
        let graph_options = options.graph.clone();
        let compile_time_limits = options.limits;
        let graph =
            discover_graph_with_session(path, options, &jai_modules::Filesystem, session, replay)?;
        Ok(Self {
            graph,
            options: graph_options,
            compile_time_limits,
        })
    }

    pub fn resolve_discovered_with_session(
        &self,
        layout: jai_types::LayoutPolicy,
        session: &mut CompilerSession,
        replay: &mut EffectReplayCache,
        policy: DiscoveryEffectPolicy,
    ) -> Result<jai_sema::Program, Error> {
        let options = self.resolve_options(layout, session);
        let workspace = session.root();
        let program = match policy {
            DiscoveryEffectPolicy::Disabled => {
                jai_sema::resolve_graph_with_options(&self.graph, &options, &mut jai_vm::NoEffects)
            }
            DiscoveryEffectPolicy::CompilerSession => {
                return private_session(session, replay, workspace, |effects| {
                    jai_sema::resolve_graph_with_options(&self.graph, &options, effects)
                        .map_err(|diagnostic| self.located(diagnostic))
                });
            }
        }
        .map_err(|diagnostic| self.located(diagnostic))?;
        if let Some(error) = session.error() {
            return Err(Error::CompilerReport(error.clone()));
        }
        Ok(program)
    }

    pub fn resolve_library_discovered_with_session(
        &self,
        layout: jai_types::LayoutPolicy,
        session: &mut CompilerSession,
        replay: &mut EffectReplayCache,
        policy: DiscoveryEffectPolicy,
    ) -> Result<jai_sema::Library, Error> {
        let options = self.resolve_options(layout, session);
        let workspace = session.root();
        let library = match policy {
            DiscoveryEffectPolicy::Disabled => jai_sema::resolve_library_with_options(
                &self.graph,
                &options,
                &mut jai_vm::NoEffects,
            ),
            DiscoveryEffectPolicy::CompilerSession => {
                return private_session(session, replay, workspace, |effects| {
                    jai_sema::resolve_library_with_options(&self.graph, &options, effects)
                        .map_err(|diagnostic| self.located(diagnostic))
                });
            }
        }
        .map_err(|diagnostic| self.located(diagnostic))?;
        if let Some(error) = session.error() {
            return Err(Error::CompilerReport(error.clone()));
        }
        Ok(library)
    }
}

pub fn discover_graph_with_session(
    path: &Path,
    options: SemanticDiscoveryOptions,
    provider: &dyn SourceProvider,
    session: &mut CompilerSession,
    replay: &mut EffectReplayCache,
) -> Result<ModuleGraph, Error> {
    let workspace = options.workspace;
    match options.effect_policy {
        DiscoveryEffectPolicy::Disabled => {
            discover_graph_with_effects(path, options, provider, &mut jai_vm::NoEffects)
        }
        DiscoveryEffectPolicy::CompilerSession => {
            private_session(session, replay, workspace, |effects| {
                discover_graph_with_effects(path, options, provider, effects)
            })
        }
    }
}

/// Publish only a completed source operation's private compiler journal.
fn private_session<T>(
    session: &mut CompilerSession,
    replay: &mut EffectReplayCache,
    workspace: WorkspaceId,
    operation: impl FnOnce(&mut dyn jai_vm::CompilerEffects) -> Result<T, Error>,
) -> Result<T, Error> {
    require_idle(session)?;
    let mut compiler = session.clone();
    let mut traces = replay.fork_committed();
    let value = operation(&mut ReplayEffects::new(&mut compiler, &mut traces))
        .map_err(|error| record_failure(session, workspace, error))?;
    if let Some(error) = compiler.error() {
        return Err(record_failure(
            session,
            workspace,
            Error::CompilerReport(error.clone()),
        ));
    }
    *session = compiler;
    *replay = traces;
    Ok(value)
}

pub(crate) fn require_idle(session: &CompilerSession) -> Result<(), Error> {
    session.require_source_idle().map_err(|error| {
        Error::CompilerReport(crate::CompilerMessage {
            level: jai_vm::MessageLevel::Error,
            text: error.to_string(),
            location: None,
        })
    })
}

pub(crate) fn record_failure(
    session: &mut CompilerSession,
    workspace: WorkspaceId,
    error: Error,
) -> Error {
    match session.record_source_failure(workspace, &error) {
        Ok(()) => error,
        Err(cause) => Error::CompilerReport(crate::CompilerMessage {
            level: jai_vm::MessageLevel::Error,
            text: format!("{error}; could not publish the workspace failure: {cause}"),
            location: None,
        }),
    }
}

/// Keep the caller's retained handler across graph decisions, including a
/// scheduler that can service the exact suspended VM job without root replay.
pub(crate) fn discover_graph_with_effects(
    path: &Path,
    options: SemanticDiscoveryOptions,
    provider: &dyn SourceProvider,
    effects: &mut dyn jai_vm::CompilerEffects,
) -> Result<ModuleGraph, Error> {
    use crate::discovery_worklists::{
        DiscoveryWorklists, SourceDiscoveryPending, SourceDiscoveryRequest, publish,
        source_only_wait,
    };
    let mut discovery = GraphDiscovery::with_bootstrap(
        path,
        options.graph.clone(),
        options.bootstrap.clone(),
        provider,
        Some(options.target.clone()),
    )?;
    loop {
        let status = discovery.advance()?;
        let mut worklists = DiscoveryWorklists::new(&discovery, status);
        if worklists.status.is_complete()
            && worklists.queries.is_empty()
            && worklists.parameters.is_empty()
        {
            return Ok(discovery
                .into_graph()
                .ok()
                .expect("complete discovery retains its graph"));
        }
        if !worklists.parameters.is_empty() {
            let resolve = jai_sema::ResolveOptions {
                file_abi: jai_sema::FileAbiBindingContext::allocator_from_graph(
                    discovery.graph(),
                    &options.graph.import_dirs,
                    options.target.clone(),
                ),
                target: Some(options.target.clone()),
                compile_time_limits: options.limits,
                compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                    discovery.graph(),
                    &options.graph.import_dirs,
                    options.workspace,
                )),
                ..Default::default()
            };
            match jai_sema::resolve_discovery_parameters(
                discovery.graph(),
                &worklists.parameters,
                &resolve,
                effects,
            ) {
                Ok(outcome) => {
                    let progressed = !outcome.decisions.is_empty();
                    for (request, response) in outcome.decisions {
                        discovery
                            .resolve_parameter(request, response)
                            .expect("typed parameter responses retain source request identity");
                    }
                    if progressed {
                        continue;
                    }
                    if let Some(pending) = outcome.pending.into_iter().next() {
                        worklists.deferred(SourceDiscoveryPending {
                            request: SourceDiscoveryRequest::Parameter(pending.request),
                            diagnostic: pending.diagnostic,
                        });
                    }
                }
                Err(diagnostic) => worklists.failure(diagnostic),
            }
        }
        let mut advanced = false;
        for kind in worklists.queries.clone() {
            let mut query =
                match crate::PreparedGraphDiscoverySession::new(&discovery, &options, kind) {
                    Ok(query) => query,
                    Err(diagnostic)
                        if matches!(
                            kind,
                            crate::DiscoveryQuery::Conditions | crate::DiscoveryQuery::Insertions,
                        ) =>
                    {
                        return Err(located(discovery.graph(), diagnostic));
                    }
                    Err(diagnostic) => {
                        worklists.failure(diagnostic);
                        continue;
                    }
                };
            let readiness = query.drive(effects);
            let cancellation = query.cancel(effects);
            drop(query);
            if let Err(error) = cancellation {
                return Err(located(
                    discovery.graph(),
                    LocatedDiagnostic {
                        location: crate::discovery_worklists::root_location(discovery.graph()),
                        message: format!("source query cancellation failed: {error}"),
                    },
                ));
            }
            match readiness {
                jai_sema::DiscoveryReadiness::Complete(outcome) => {
                    let published =
                        publish(&mut discovery, outcome).map_err(|failure| failure.error)?;
                    advanced = published.progressed;
                    if let Some(pending) = published.pending {
                        worklists.deferred(pending);
                    }
                }
                jai_sema::DiscoveryReadiness::Pending(pending) if source_only_wait(&pending) => {
                    worklists.source_wait(kind, pending);
                }
                jai_sema::DiscoveryReadiness::Pending(pending) => {
                    // The synchronous entry point cannot retain a live external
                    // checkpoint. Its original diagnostic remains authoritative.
                    return Err(located(discovery.graph(), pending.diagnostic));
                }
                jai_sema::DiscoveryReadiness::Failed(diagnostic)
                    if matches!(
                        kind,
                        crate::DiscoveryQuery::Conditions | crate::DiscoveryQuery::Insertions
                    ) =>
                {
                    return Err(located(discovery.graph(), diagnostic));
                }
                jai_sema::DiscoveryReadiness::Failed(diagnostic) => worklists.failure(diagnostic),
            }
            if advanced {
                break;
            }
        }
        if advanced {
            continue;
        }
        if worklists.is_complete() {
            return Ok(discovery
                .into_graph()
                .ok()
                .expect("complete discovery retains its graph"));
        }
        return match worklists.stalled(discovery.graph()) {
            Ok(pending) => Err(located(discovery.graph(), pending.diagnostic)),
            Err(diagnostic) => Err(located(discovery.graph(), diagnostic)),
        };
    }
}

pub(crate) fn located(graph: &ModuleGraph, diagnostic: LocatedDiagnostic) -> Error {
    let record = graph
        .sources()
        .get(diagnostic.location.source)
        .expect("discovery diagnostics retain defining sources");
    Error::Located {
        source: diagnostic.location.source,
        path: record.path().to_owned(),
        diagnostic: jai_source::Diagnostic::at_source(
            diagnostic.location,
            diagnostic.message.clone(),
        ),
        rendered: diagnostic.render(graph.sources()),
    }
}

#[cfg(test)]
#[path = "source_discovery/using_tests.rs"]
mod using_tests;
