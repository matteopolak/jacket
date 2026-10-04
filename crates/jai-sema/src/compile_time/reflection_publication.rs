//! Source publication acquires both prepared owners after VM/provider detachment.
use super::reflection_journal::ReflectionPolicyJournal;
use jai_source::{Diagnostic, SourceRecord, SourceSpan};
use jai_types::{RecordReflectionCommit, TypeRegistry};
use jai_vm::{CompilerEffects, PreparedPublication, PublicationOutcome, SourceOrigin, VmState};

pub(crate) struct ReflectionPublicationSource<'a> {
    pub(crate) source: &'a SourceRecord,
    pub(crate) location: SourceSpan,
}

pub(crate) struct ReflectionPublicationInput<'a> {
    pub(crate) types: &'a mut TypeRegistry,
    pub(crate) meta: &'a mut crate::reflection::MetaContext,
    pub(crate) journal: ReflectionPolicyJournal,
    pub(crate) owner: ReflectionPublicationSource<'a>,
    pub(crate) run: ReflectionPublicationSource<'a>,
    pub(crate) origin: &'a SourceOrigin,
}

pub(crate) struct RejectedReflectionPublication<E> {
    pub(crate) state: VmState,
    pub(crate) effects: E,
    pub(crate) diagnostic: Diagnostic,
    pub(crate) cancellation_error: Option<jai_vm::Error>,
}

/// The caller has released every ReadyProcedures/type borrow before entering.
/// The effect service must not query the same type/meta owners during `finish`.
/// The source payload publisher consumes only prevalidated, preallocated facts.
pub(crate) fn commit_reflection_publication<E: CompilerEffects, T, R>(
    publication: PreparedPublication<E, T>,
    input: ReflectionPublicationInput<'_>,
    publish: impl FnOnce(T, RecordReflectionCommit) -> R,
) -> Result<PublicationOutcome<E, R>, RejectedReflectionPublication<E>> {
    let ReflectionPublicationInput {
        types,
        meta,
        journal,
        owner,
        run,
        origin,
    } = input;
    let ReflectionPublicationSource { source, location } = run;
    let valid_origin = publication.source_origin() == Some(origin)
        && source.id() == location.source
        && source.path() == origin.path
        && location.span.start == origin.start
        && location.span.end == origin.end
        && source
            .text()
            .get(origin.start..origin.end)
            .map(str::as_bytes)
            == Some(origin.body.as_slice());
    if !valid_origin
        || !journal
            .source()
            .matches_source(owner.source, owner.location)
    {
        return Err(reject(
            publication,
            Diagnostic::at_source(
                location,
                "reflection publication changed its retained source run",
            ),
        ));
    }
    let (transaction, _) = journal.into_transaction();
    let prepared = match meta.prepare_reflection_policy_transaction(types, transaction, location) {
        Ok(prepared) => prepared,
        Err(error) => return Err(reject(publication, error)),
    };
    // Host finish is the only remaining fallible operation. Both prepared owners
    // remain exclusive until its result either drops the guards or applies them.
    Ok(publication.commit(move |payload| publish(payload, prepared.apply())))
}

fn reject<E: CompilerEffects, T>(
    publication: PreparedPublication<E, T>,
    diagnostic: Diagnostic,
) -> RejectedReflectionPublication<E> {
    let (state, effects, canceled) = publication.cancel();
    RejectedReflectionPublication {
        state,
        effects,
        diagnostic,
        cancellation_error: canceled.err(),
    }
}

#[cfg(test)]
mod tests;
