//! Bind source user notes to real procedure identities before losing lexical origin.
use crate::*;
use jai_source::{SourceRecord, SourceSpan};

pub(crate) fn retain(
    metadata: &mut jai_ir::DebugSources,
    procedure: ProcedureId,
    source: &SourceRecord,
    notes: &[syntax::NoteSyntax],
) -> Result<(), Diagnostic> {
    let spans: Vec<_> = notes
        .iter()
        .map(|note| SourceSpan {
            source: source.id(),
            span: note.span,
        })
        .collect();
    metadata
        .set_procedure_notes(procedure, source, &spans)
        .map_err(|error| {
            Diagnostic::at_source(
                SourceSpan {
                    source: source.id(),
                    span: notes.first().map_or(Span::default(), |note| note.span),
                },
                error.to_string(),
            )
        })
}

impl Resolver<'_> {
    pub(crate) fn remember_procedure_notes(
        &mut self,
        procedure: ProcedureId,
        notes: &[syntax::NoteSyntax],
    ) -> Result<(), Diagnostic> {
        if notes.is_empty() {
            return Ok(());
        }
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(
                notes[0].span,
                "procedure notes require their defining source snapshot",
            )
        })?;
        let source = self.debug.source().unwrap_or_else(|| scope.source());
        let source = scope.source_record(source).ok_or_else(|| {
            Diagnostic::at_source(
                SourceSpan {
                    source,
                    span: notes[0].span,
                },
                "procedure note source is unavailable",
            )
        })?;
        retain(&mut self.meta.debug_sources, procedure, source, notes)
    }
}
