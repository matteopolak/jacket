//! Advance only after the retained semantic guard releases its graph borrow.
use super::*;
use crate::discovery_worklists::{
    DiscoveryWorklists, SourceDiscoveryPending, SourceDiscoveryRequest, publish, source_only_wait,
};

struct Guard<'session, 'graph> {
    session: &'session mut PreparedGraphDiscoverySession<'graph>,
    state: Rc<RefCell<JobState>>,
}
impl Drop for Guard<'_, '_> {
    fn drop(&mut self) {
        let mut state = self.state.borrow_mut();
        if let Some(Journal { compiler, replay }) = state.journal.as_mut()
            && let Err(error) = self
                .session
                .cancel(&mut ReplayEffects::new(compiler, replay))
        {
            state.cancellation_error.get_or_insert(error);
        }
    }
}

enum QueryResponse {
    Complete(PreparedDiscoveryOutcome),
    SourceWait(LibraryPending),
    CancellationFailed(jai_vm::Error),
}

async fn query(
    discovery: &GraphDiscovery<'_>,
    options: &SemanticDiscoveryOptions,
    kind: DiscoveryQuery,
    state: Rc<RefCell<JobState>>,
) -> Result<QueryResponse, Error> {
    let mut session = PreparedGraphDiscoverySession::new(discovery, options, kind)
        .map_err(|error| crate::source_discovery::located(discovery.graph(), error))?;
    let owner = Rc::clone(&state);
    let guard = Guard {
        session: &mut session,
        state,
    };
    let response = poll_fn(|_| {
        let mut state = guard.state.borrow_mut();
        let progress = {
            let Journal { compiler, replay } = state
                .journal
                .as_mut()
                .expect("running graph retains its private journals");
            guard
                .session
                .drive(&mut ReplayEffects::new(compiler, replay))
        };
        match progress {
            DiscoveryReadiness::Complete(outcome) => {
                state.pending = None;
                Poll::Ready(Ok(QueryResponse::Complete(outcome)))
            }
            DiscoveryReadiness::Failed(error) => {
                state.pending = None;
                Poll::Ready(Err(crate::source_discovery::located(
                    discovery.graph(),
                    error,
                )))
            }
            DiscoveryReadiness::Pending(pending) if source_only_wait(&pending) => {
                // This owned source ticket needs another graph producer. Drop
                // the guard before publishing; keep both journal owners.
                state.pending = None;
                Poll::Ready(Ok(QueryResponse::SourceWait(pending)))
            }
            DiscoveryReadiness::Pending(pending) if pending.dependencies.is_empty() => {
                // No retained dependency can wake an untyped empty wait.
                state.pending = None;
                Poll::Ready(Err(crate::source_discovery::located(
                    discovery.graph(),
                    pending.diagnostic,
                )))
            }
            DiscoveryReadiness::Pending(pending) => {
                // Source metadata may accompany this wait, but its actual VM
                // dependencies keep this checkpoint alive for its scheduler.
                state.pending = Some(pending);
                Poll::Pending
            }
        }
    })
    .await;
    drop(guard);
    if let Some(error) = owner.borrow().cancellation_error.clone() {
        return Ok(QueryResponse::CancellationFailed(error));
    }
    response
}

pub(super) async fn discover(
    path: &std::path::Path,
    options: &SemanticDiscoveryOptions,
    provider: &dyn SourceProvider,
    state: Rc<RefCell<JobState>>,
) -> Result<ModuleGraph, GraphJobFailure> {
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
            let outcome = {
                let mut state = state.borrow_mut();
                let Journal { compiler, replay } = state
                    .journal
                    .as_mut()
                    .expect("running graph retains its journals");
                match options.effect_policy {
                    DiscoveryEffectPolicy::Disabled => jai_sema::resolve_discovery_parameters(
                        discovery.graph(),
                        &worklists.parameters,
                        &resolve,
                        &mut jai_vm::NoEffects,
                    ),
                    DiscoveryEffectPolicy::CompilerSession => {
                        jai_sema::resolve_discovery_parameters(
                            discovery.graph(),
                            &worklists.parameters,
                            &resolve,
                            &mut ReplayEffects::new(compiler, replay),
                        )
                    }
                }
            };
            match outcome {
                Ok(outcome) => {
                    let progressed = !outcome.decisions.is_empty();
                    for (request, response) in outcome.decisions {
                        discovery
                            .resolve_parameter(request, response)
                            .expect("typed parameter responses retain request identity");
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
                Err(error) => worklists.failure(error),
            }
        }
        let mut advanced = false;
        for kind in worklists.queries.clone() {
            let response = match query(&discovery, options, kind, Rc::clone(&state)).await {
                Ok(response) => response,
                Err(error)
                    if matches!(
                        kind,
                        DiscoveryQuery::Conditions | DiscoveryQuery::Insertions
                    ) =>
                {
                    return Err(error.into());
                }
                Err(Error::Located {
                    source, diagnostic, ..
                }) => {
                    worklists.failure(jai_source::LocatedDiagnostic {
                        location: jai_source::SourceSpan {
                            source,
                            span: diagnostic.span,
                        },
                        message: diagnostic.message,
                    });
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            match response {
                QueryResponse::SourceWait(pending) => worklists.source_wait(kind, pending),
                QueryResponse::CancellationFailed(error) => {
                    return Err(crate::source_discovery::located(
                        discovery.graph(),
                        jai_source::LocatedDiagnostic {
                            location: crate::discovery_worklists::root_location(discovery.graph()),
                            message: format!("source query cancellation failed: {error}"),
                        },
                    )
                    .into());
                }
                QueryResponse::Complete(outcome) => {
                    let published = publish(&mut discovery, outcome).map_err(|failure| {
                        if failure.discard_journal {
                            // The graph did not admit this source publication.
                            // No failed insertion's private effects may merge.
                            state.borrow_mut().journal = None;
                        }
                        failure.error
                    })?;
                    advanced = published.progressed;
                    if let Some(pending) = published.pending {
                        worklists.deferred(pending);
                    }
                }
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
            Ok(pending) => {
                let error =
                    crate::source_discovery::located(discovery.graph(), pending.diagnostic.clone());
                Err(GraphJobFailure::Source { pending, error })
            }
            Err(diagnostic) => {
                Err(crate::source_discovery::located(discovery.graph(), diagnostic).into())
            }
        };
    }
}
