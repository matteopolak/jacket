//! Typed source producers reach a fixed point before a source wait is terminal.
use crate::{DiscoveryQuery, Error};
use jai_modules::{DiscoveryStatus, GraphDiscovery, ModuleGraph};
use jai_sema::{LibraryPending, PreparedDiscoveryOutcome};
use jai_source::{LocatedDiagnostic, SourceSpan, Span};

/// Exact source request that can require a new graph frontier. This is not a VM
/// dependency and cannot be scheduled as a suspended execution token.
#[derive(Clone, Copy, Debug)]
pub enum SourceDiscoveryRequest {
    Preparation {
        query: DiscoveryQuery,
        cause: jai_sema::SourcePreparationPending,
    },
    LibraryPreparation(jai_sema::SourcePreparationPending),
    Parameter(jai_modules::ParameterRequestId),
    Insertion(jai_modules::InsertionRequestId),
    Using(jai_modules::UsingRequestId),
    Case(jai_modules::CaseRequestId),
    Condition(jai_modules::ConditionRequestId),
    Module(jai_source::ModuleId),
}

#[derive(Clone, Debug)]
pub struct SourceDiscoveryPending {
    pub request: SourceDiscoveryRequest,
    pub diagnostic: LocatedDiagnostic,
}

/// A round owns only original source request identities. Semantic arenas and VM
/// tokens stay inside each prepared query until it completes or is cancelled.
pub(crate) struct DiscoveryWorklists {
    pub parameters: Vec<jai_modules::DeferredParameter>,
    pub queries: Vec<DiscoveryQuery>,
    pub status: DiscoveryStatus,
    location: SourceSpan,
    failures: Vec<LocatedDiagnostic>,
    source_waits: Vec<SourceDiscoveryPending>,
}

impl DiscoveryWorklists {
    pub fn new(discovery: &GraphDiscovery<'_>, status: DiscoveryStatus) -> Self {
        let parameters = discovery.pending_parameter_requests();
        let insertions: Vec<_> = discovery.pending_insertion_requests().collect();
        let using = discovery.pending_using_requests();
        let cases: Vec<_> = discovery.pending_cases().collect();
        let conditions: Vec<_> = discovery.pending_conditions().collect();
        let queries = [
            (DiscoveryQuery::Insertions, !insertions.is_empty()),
            (DiscoveryQuery::Using, !using.is_empty()),
            (DiscoveryQuery::Cases, !cases.is_empty()),
            (
                DiscoveryQuery::Conditions,
                !conditions.is_empty() || discovery.has_dependency_templates(),
            ),
        ]
        .into_iter()
        .filter_map(|(query, enabled)| enabled.then_some(query))
        .collect();
        let location = insertions
            .first()
            .map(|request| request.location)
            .or_else(|| conditions.first().map(|request| request.location))
            .or_else(|| using.first().map(|request| request.location))
            .or_else(|| cases.first().map(|request| request.location))
            .or_else(|| parameters.first().map(|request| request.location))
            .or_else(|| {
                discovery
                    .graph()
                    .dependency_templates()
                    .first()
                    .map(|template| template.location())
            })
            .unwrap_or_else(|| root_location(discovery.graph()));
        Self {
            parameters,
            queries,
            status,
            location,
            failures: vec![],
            source_waits: vec![],
        }
    }

    pub fn failure(&mut self, diagnostic: LocatedDiagnostic) {
        self.failures.push(diagnostic);
    }

    /// This ticket has no VM dependency. Retire its old arena before trying a
    /// producer that needs to publish into the same graph. The journal owner is
    /// retained by the graph job; an actual VM wait never takes this path.
    pub fn source_wait(&mut self, query: DiscoveryQuery, pending: LibraryPending) {
        debug_assert!(pending.source.is_some() && pending.dependencies.is_empty());
        self.source_waits.push(SourceDiscoveryPending {
            request: SourceDiscoveryRequest::Preparation {
                query,
                cause: pending
                    .source
                    .expect("source ticket retains its actual preparation cause"),
            },
            diagnostic: pending.diagnostic,
        });
    }

    pub fn deferred(&mut self, pending: SourceDiscoveryPending) {
        self.source_waits.push(pending);
    }

    pub fn is_complete(&self) -> bool {
        self.status.is_complete() && self.failures.is_empty() && self.source_waits.is_empty()
    }

    /// Called only after every available typed producer has been attempted and
    /// none published. A declaration marker by itself cannot wake a VM job.
    pub fn stalled(self, graph: &ModuleGraph) -> Result<SourceDiscoveryPending, LocatedDiagnostic> {
        if let Some(failure) = self.failures.into_iter().next() {
            return Err(failure);
        }
        if let Some(mut pending) = self.source_waits.into_iter().next() {
            if let SourceDiscoveryRequest::Preparation {
                cause, ..
            } = pending.request
            {
                pending.diagnostic = cause.diagnostic(graph);
            }
            return Ok(pending);
        }
        if let DiscoveryStatus::Awaiting {
            dependencies, ..
        } = self.status
            && let Some(dependency) = dependencies.into_iter().next()
        {
            return Ok(SourceDiscoveryPending {
                request: SourceDiscoveryRequest::Module(dependency.module),
                diagnostic: dependency.diagnostic,
            });
        }
        Err(LocatedDiagnostic {
            location: self.location,
            message: "source discovery has no ready semantic producer".into(),
        })
    }
}

pub(crate) fn source_only_wait(pending: &LibraryPending) -> bool {
    pending.source.is_some() && pending.dependencies.is_empty()
}

pub(crate) struct PublishedDiscovery {
    pub progressed: bool,
    pub pending: Option<SourceDiscoveryPending>,
}

pub(crate) struct PublicationFailure {
    pub error: Error,
    pub discard_journal: bool,
}

/// Publication follows semantic guard retirement. The graph validates original
/// quote provenance and consumes a live receipt for each declaration insertion.
pub(crate) fn publish(
    discovery: &mut GraphDiscovery<'_>,
    outcome: PreparedDiscoveryOutcome,
) -> Result<PublishedDiscovery, PublicationFailure> {
    let insertion = matches!(&outcome, PreparedDiscoveryOutcome::Insertions(_));
    publish_inner(discovery, outcome).map_err(|error| PublicationFailure {
        error,
        discard_journal: insertion,
    })
}

fn publish_inner(
    discovery: &mut GraphDiscovery<'_>,
    outcome: PreparedDiscoveryOutcome,
) -> Result<PublishedDiscovery, Error> {
    let mut progressed = false;
    let pending = match outcome {
        PreparedDiscoveryOutcome::Insertions(outcome) => {
            if outcome.decisions.len() > 1 {
                return Err(crate::source_discovery::located(
                    discovery.graph(),
                    LocatedDiagnostic {
                        location: root_location(discovery.graph()),
                        message:
                            "declaration insertion outcome exceeds one admitted graph frontier"
                                .into(),
                    },
                ));
            }
            for decision in outcome.decisions {
                let location = discovery
                    .insertion_requests()
                    .iter()
                    .find(|request| request.id == decision.request)
                    .map(|request| request.location)
                    .unwrap_or_else(|| root_location(discovery.graph()));
                let transaction = discovery
                    .prepare_insertion_admitted(decision.request, decision.admission)
                    .map_err(|error| insertion_error(discovery.graph(), location, error))?;
                if let Err(error) = discovery.commit_insertion(transaction) {
                    // Graph append failures already retire their receipt. A
                    // response failure still needs explicit cancellation.
                    let cancellation = discovery.cancel_insertion(transaction);
                    let error = match error {
                        jai_modules::InsertionPublicationError::Graph(error) => Error::Graph(error),
                        jai_modules::InsertionPublicationError::Response(error) => {
                            insertion_error(discovery.graph(), location, error)
                        }
                    };
                    if let Err(cause) = cancellation
                        && cause != jai_modules::InsertionResponseError::StaleTransaction
                    {
                        return Err(crate::source_discovery::located(
                            discovery.graph(),
                            LocatedDiagnostic {
                                location,
                                message: format!("{error}; insertion cancellation failed: {cause}"),
                            },
                        ));
                    }
                    return Err(error);
                }
                progressed = true;
            }
            // A specialization may advance the namespace frontier. Publish
            // the code admitted against this exact snapshot before scanning it.
            for key in outcome.specializations {
                progressed |= discovery.discover_specialization(key)?;
            }
            outcome
                .pending
                .into_iter()
                .next()
                .map(|pending| SourceDiscoveryPending {
                    request: SourceDiscoveryRequest::Insertion(pending.request),
                    diagnostic: pending.diagnostic,
                })
        }
        PreparedDiscoveryOutcome::Using(outcome) => {
            for key in outcome.specializations {
                progressed |= discovery.discover_specialization(key)?;
            }
            for (request, decision) in outcome.decisions {
                discovery.resolve_using(request, decision)?;
                progressed = true;
            }
            outcome
                .pending
                .into_iter()
                .next()
                .map(|pending| SourceDiscoveryPending {
                    request: SourceDiscoveryRequest::Using(pending.request),
                    diagnostic: pending.diagnostic,
                })
        }
        PreparedDiscoveryOutcome::Cases(outcome) => {
            for key in outcome.specializations {
                progressed |= discovery.discover_specialization(key)?;
            }
            for (request, choice) in outcome.decisions {
                discovery
                    .select_case(request, choice)
                    .expect("typed case decisions retain request identity");
                progressed = true;
            }
            outcome
                .pending
                .into_iter()
                .next()
                .map(|pending| SourceDiscoveryPending {
                    request: SourceDiscoveryRequest::Case(pending.request),
                    diagnostic: pending.diagnostic,
                })
        }
        PreparedDiscoveryOutcome::Conditions(outcome) => {
            for key in outcome.specializations {
                progressed |= discovery.discover_specialization(key)?;
            }
            for (request, selected) in outcome.decisions {
                discovery
                    .select_condition(request, selected)
                    .expect("typed condition decisions retain request identity");
                progressed = true;
            }
            outcome
                .pending
                .into_iter()
                .next()
                .map(|pending| SourceDiscoveryPending {
                    request: SourceDiscoveryRequest::Condition(pending.request),
                    diagnostic: pending.diagnostic,
                })
        }
    };
    Ok(PublishedDiscovery {
        progressed,
        pending,
    })
}

fn insertion_error(
    graph: &ModuleGraph,
    location: SourceSpan,
    error: jai_modules::InsertionResponseError,
) -> Error {
    crate::source_discovery::located(
        graph,
        LocatedDiagnostic {
            location,
            message: format!("declaration insertion response: {error}"),
        },
    )
}

pub(crate) fn root_location(graph: &ModuleGraph) -> SourceSpan {
    SourceSpan {
        source: graph
            .file(graph.module(graph.root()).unwrap().entry())
            .unwrap()
            .source(),
        span: Span::default(),
    }
}
