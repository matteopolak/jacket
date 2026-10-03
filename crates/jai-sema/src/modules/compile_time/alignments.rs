//! Resolve requested global allocation policy before any virtual allocation.
use super::*;

pub(super) struct Progress {
    pub completed: usize,
    pub error: Option<LocatedDiagnostic>,
    pub stalled: Option<(FileInstanceId, Span, String)>,
}
pub(super) struct Requests<'a> {
    pub jobs: &'a mut Vec<super::super::storage_alignment::Job>,
    pub owners: &'a HashMap<DeclarationId, ProcedureId>,
}

pub(super) fn bind(
    context: &Context<'_>,
    requests: Requests<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
    options: &crate::ResolveOptions,
) -> Progress {
    let mut progress = Progress {
        completed: 0,
        error: None,
        stalled: None,
    };
    let mut retry = Vec::new();
    for job in requests.jobs.drain(..) {
        let child = context.for_source(
            requests.owners[&job.declaration],
            job.file,
            job.location(declarations.graph).source,
        );
        let result = super::super::storage_alignment::declaration(
            &job,
            declarations,
            types,
            meta,
            options,
            &child,
            places,
        );
        let dependencies = child.pending.borrow().clone();
        let constants = child.pending_constants.borrow().clone();
        let fields = child.pending_field_defaults.borrow().clone();
        context.merge_pending_from(&child);
        match result {
            Ok(alignment) => {
                let published = alignment.map_or(Ok(()), |alignment| {
                    meta.storage_alignments
                        .set_global(job.global, alignment)
                        .map_err(|error| {
                            Diagnostic::at_source(
                                job.location(declarations.graph),
                                error.to_string(),
                            )
                        })
                });
                match published {
                    Ok(()) => progress.completed += 1,
                    Err(error) => {
                        progress
                            .error
                            .get_or_insert_with(|| located(declarations.graph, job.file, error));
                        retry.push(job);
                    }
                }
            }
            Err(error) => {
                if !dependencies.is_empty() || !constants.is_empty() || !fields.is_empty() {
                    progress.stalled = Some((
                        job.file,
                        job.annotation_span(),
                        format!(
                            "storage alignment {dependencies:?}; constants {constants:?}; fields {fields:?}"
                        ),
                    ));
                } else {
                    progress
                        .error
                        .get_or_insert_with(|| located(declarations.graph, job.file, error));
                }
                retry.push(job);
            }
        }
    }
    *requests.jobs = retry;
    progress
}
