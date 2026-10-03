//! Procedure user notes retain their actual source snapshot independently of debug emission.
use super::{DebugSourceLocation, DebugSourceLocationError, DebugSources};
use crate::{IrError, ProcedureId};
use jai_source::{SourceRecord, SourceSpan};
use std::{collections::HashMap, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcedureNote {
    location: DebugSourceLocation,
    text: Box<[u8]>,
}
impl ProcedureNote {
    /// Exact source bytes following `@`, including argument spelling and spacing.
    pub fn text(&self) -> &[u8] {
        &self.text
    }
    pub fn location(&self) -> &DebugSourceLocation {
        &self.location
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcedureNoteError {
    Location(DebugSourceLocationError),
    ExpectedNote,
    UnorderedSpans,
}
impl fmt::Display for ProcedureNoteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid procedure note source: {self:?}")
    }
}
impl std::error::Error for ProcedureNoteError {
}
impl From<DebugSourceLocationError> for ProcedureNoteError {
    fn from(error: DebugSourceLocationError) -> Self {
        Self::Location(error)
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct ProcedureNotes(HashMap<ProcedureId, Box<[ProcedureNote]>>);

impl DebugSources {
    /// Capture parser-owned note spans before source capture or specialization changes scope.
    pub fn set_procedure_notes(
        &mut self,
        procedure: ProcedureId,
        source: &SourceRecord,
        spans: &[SourceSpan],
    ) -> Result<(), ProcedureNoteError> {
        let mut notes = Vec::with_capacity(spans.len());
        let mut previous_end = None;
        for &span in spans {
            let location = self.source_location(source, span)?;
            if previous_end.is_some_and(|end| span.span.start < end) {
                return Err(ProcedureNoteError::UnorderedSpans);
            }
            let text = source.text()[span.span.start..span.span.end]
                .strip_prefix('@')
                .filter(|text| !text.is_empty())
                .ok_or(ProcedureNoteError::ExpectedNote)?;
            notes.push(ProcedureNote {
                location,
                text: text.as_bytes().into(),
            });
            previous_end = Some(span.span.end);
        }
        self.procedure_notes.0.insert(procedure, notes.into());
        Ok(())
    }
    pub fn procedure_notes(&self, procedure: ProcedureId) -> &[ProcedureNote] {
        self.procedure_notes
            .0
            .get(&procedure)
            .map_or(&[], AsRef::as_ref)
    }
    pub(super) fn clear_procedure_notes(&mut self, procedure: ProcedureId) {
        self.procedure_notes.0.remove(&procedure);
    }
    pub(super) fn validate_procedure_notes(
        &self,
        procedure_exists: impl Fn(ProcedureId) -> bool,
    ) -> Result<(), IrError> {
        for (&procedure, notes) in &self.procedure_notes.0 {
            self.validate_owner(procedure, &procedure_exists)?;
            for note in notes {
                self.validate_location(&note.location)?;
                let span = note.location.span().span;
                let expected = &note.location.source_text[span.start + 1..span.end];
                if expected.as_bytes() != note.text() {
                    return Err(IrError::UnknownIdentity {
                        kind: "procedure note source",
                        index: procedure.index(),
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, Span};

    #[test]
    fn notes_retain_source_bytes_order_and_snapshot_after_the_source_map_is_dropped() {
        let mut sources = SourceMap::default();
        let text = "@PrintLike @Reason(\"é\\x00\", context)";
        let id = sources.insert("notes.jai".into(), text.to_owned());
        let source = sources.get(id).unwrap();
        let mut metadata = DebugSources::default();
        let procedure = ProcedureId::new(3);
        metadata
            .set_procedure_notes(
                procedure,
                source,
                &[
                    SourceSpan {
                        source: id,
                        span: Span::new(0, 10),
                    },
                    SourceSpan {
                        source: id,
                        span: Span::new(11, text.len()),
                    },
                ],
            )
            .unwrap();
        drop(sources);
        assert_eq!(metadata.procedure_notes(procedure)[0].text(), b"PrintLike");
        assert_eq!(
            metadata.procedure_notes(procedure)[1].text(),
            "Reason(\"é\\x00\", context)".as_bytes()
        );
        assert_eq!(
            metadata.procedure_notes(procedure)[1]
                .location()
                .span()
                .source,
            id
        );
        metadata
            .validate_procedure_notes(|owner| owner == procedure)
            .unwrap();
    }

    #[test]
    fn invalid_source_ranges_and_note_order_do_not_replace_existing_notes() {
        let mut sources = SourceMap::default();
        let id = sources.insert("notes.jai".into(), "@A @B".to_owned());
        let other = sources.insert("other.jai".into(), "@C".to_owned());
        let source = sources.get(id).unwrap();
        let mut metadata = DebugSources::default();
        let procedure = ProcedureId::new(0);
        let first = SourceSpan {
            source: id,
            span: Span::new(0, 2),
        };
        let second = SourceSpan {
            source: id,
            span: Span::new(3, 5),
        };
        metadata
            .set_procedure_notes(procedure, source, &[first])
            .unwrap();
        assert_eq!(
            metadata.set_procedure_notes(
                procedure,
                source,
                &[SourceSpan {
                    source: other,
                    ..first
                }]
            ),
            Err(ProcedureNoteError::Location(
                DebugSourceLocationError::WrongSource
            ))
        );
        assert_eq!(
            metadata.set_procedure_notes(
                procedure,
                source,
                &[SourceSpan {
                    span: Span::new(0, 6),
                    ..first
                }]
            ),
            Err(ProcedureNoteError::Location(
                DebugSourceLocationError::InvalidSpan
            ))
        );
        assert_eq!(
            metadata.set_procedure_notes(procedure, source, &[second, first]),
            Err(ProcedureNoteError::UnorderedSpans)
        );
        assert_eq!(
            metadata.set_procedure_notes(
                procedure,
                source,
                &[SourceSpan {
                    span: Span::new(1, 2),
                    ..first
                }]
            ),
            Err(ProcedureNoteError::ExpectedNote)
        );
        assert_eq!(metadata.procedure_notes(procedure)[0].text(), b"A");
    }

    #[test]
    fn another_source_map_cannot_replace_an_existing_same_numbered_snapshot() {
        let mut original = SourceMap::default();
        let id = original.insert("original.jai".into(), "@A".to_owned());
        let mut replacement = SourceMap::default();
        let replacement_id = replacement.insert("replacement.jai".into(), "@B".to_owned());
        assert_eq!(id, replacement_id);
        let procedure = ProcedureId::new(0);
        let span = SourceSpan {
            source: id,
            span: Span::new(0, 2),
        };
        let mut metadata = DebugSources::default();
        metadata
            .set_procedure_notes(procedure, original.get(id).unwrap(), &[span])
            .unwrap();
        assert_eq!(
            metadata.set_procedure_notes(
                procedure,
                replacement.get(replacement_id).unwrap(),
                &[span]
            ),
            Err(ProcedureNoteError::Location(
                DebugSourceLocationError::WrongSource
            ))
        );
        assert_eq!(metadata.procedure_notes(procedure)[0].text(), b"A");
    }

    #[test]
    fn unknown_owners_fail_and_debug_suppression_keeps_user_notes() {
        let mut sources = SourceMap::default();
        let id = sources.insert("quiet.jai".into(), "@Quiet".to_owned());
        let procedure = ProcedureId::new(7);
        let mut metadata = DebugSources::default();
        metadata
            .set_procedure_notes(
                procedure,
                sources.get(id).unwrap(),
                &[SourceSpan {
                    source: id,
                    span: Span::new(0, 6),
                }],
            )
            .unwrap();
        assert!(matches!(
            metadata.validate_procedure_notes(|_| false),
            Err(IrError::UnknownIdentity {
                index: 7,
                ..
            })
        ));
        metadata.set_procedure_policy(procedure, crate::DebugPolicy::Suppress);
        assert_eq!(metadata.procedure_notes(procedure)[0].text(), b"Quiet");
        metadata.clear_procedure_notes(procedure);
        assert!(metadata.procedure_notes(procedure).is_empty());
    }
}
