//! Compilation identities and immutable source records.
use crate::{Diagnostic, Span};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(usize);
        impl $name {
            pub fn index(self) -> usize {
                self.0
            }
        }
    };
}
identity!(SourceId);
identity!(UnitId);
identity!(ModuleId);
identity!(ScopeId);
identity!(DeclarationId);

/// Allocate identities within one compilation session; IDs never identify spellings.
#[derive(Clone, Debug, Default)]
pub struct Identities {
    units: usize,
    modules: usize,
    scopes: usize,
    declarations: usize,
}
impl Identities {
    pub fn unit(&mut self) -> UnitId {
        let id = UnitId(self.units);
        self.units += 1;
        id
    }
    pub fn module(&mut self) -> ModuleId {
        let id = ModuleId(self.modules);
        self.modules += 1;
        id
    }
    pub fn scope(&mut self) -> ScopeId {
        let id = ScopeId(self.scopes);
        self.scopes += 1;
        id
    }
    pub fn declaration(&mut self) -> DeclarationId {
        let id = DeclarationId(self.declarations);
        self.declarations += 1;
        id
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    pub source: SourceId,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedDiagnostic {
    pub location: SourceSpan,
    pub message: String,
}
impl LocatedDiagnostic {
    pub fn new(source: SourceId, diagnostic: Diagnostic) -> Self {
        Self {
            location: SourceSpan {
                source: diagnostic.source.unwrap_or(source),
                span: diagnostic.span,
            },
            message: diagnostic.message,
        }
    }
    pub fn render(&self, sources: &SourceMap) -> String {
        let diagnostic = Diagnostic::at_source(self.location, &self.message);
        match sources.get(self.location.source) {
            Some(source) => diagnostic.render(&source.path.to_string_lossy(), &source.text),
            None => format!(
                "source {}: error: {}",
                self.location.source.index(),
                self.message
            ),
        }
    }
}
impl std::fmt::Display for LocatedDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for LocatedDiagnostic {
}
/// Unforgeable allocation witness: equal paths, source IDs and bytes in another map differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceAllocationId(u64);
static NEXT_SOURCE_ALLOCATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn source_allocation() -> SourceAllocationId {
    use std::sync::atomic::Ordering;
    SourceAllocationId(
        NEXT_SOURCE_ALLOCATION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("source allocation identities exhausted"),
    )
}
/// Actual retained text owner. Appending generated source preserves the original prefix owner;
/// constructing an equal replacement never preserves its allocation witness.
#[derive(Clone, Debug)]
pub struct SourceTextSnapshot {
    allocation: SourceAllocationId,
    text: Arc<str>,
    prefix: Option<(Arc<SourceTextSnapshot>, usize)>,
}
impl PartialEq for SourceTextSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.allocation == other.allocation
    }
}
impl Eq for SourceTextSnapshot {
}
impl SourceTextSnapshot {
    pub fn new(text: String) -> Self {
        Self {
            allocation: source_allocation(),
            text: text.into(),
            prefix: None,
        }
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn append(&self, suffix: &str) -> Self {
        if suffix.is_empty() {
            return self.clone();
        }
        let mut text = self.text().to_owned();
        text.push_str(suffix);
        Self {
            allocation: source_allocation(),
            text: text.into(),
            prefix: Some((Arc::new(self.clone()), self.text.len())),
        }
    }
    fn owner_at(&self, span: Span) -> Option<(SourceAllocationId, Arc<str>)> {
        let owner = self
            .owner_ref_at_with_work(span, &mut |_| Ok::<_, std::convert::Infallible>(()))
            .unwrap();
        owner.map(|(allocation, text)| (allocation, Arc::clone(text)))
    }
    fn owner_ref_at_with_work<E>(
        &self,
        span: Span,
        admit: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<(SourceAllocationId, &Arc<str>)>, E> {
        admit(1)?;
        if span.start > span.end || self.text.get(span.start..span.end).is_none() {
            return Ok(None);
        }
        let mut owner = self;
        loop {
            // Charge before inspecting each genuine retained prefix, including
            // the terminal owner. No path/text substitution can skip this walk.
            admit(1)?;
            let Some((prefix, length)) = &owner.prefix else {
                break;
            };
            if span.end > *length {
                break;
            }
            owner = prefix;
        }
        admit(1)?;
        Ok(Some((owner.allocation, &owner.text)))
    }
}
/// Display labels never imply filesystem ownership for embedded source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceRecordKind {
    File,
    Embedded {
        importing: SourceSpan,
        resolution_path: PathBuf,
    },
}
#[derive(Clone, Debug)]
pub struct SourceRecord {
    snapshot: SourceTextSnapshot,
    id: SourceId,
    path: PathBuf,
    text: Arc<str>,
    kind: SourceRecordKind,
}
impl SourceRecord {
    /// Borrow every genuine retained prefix text; no Arc clone or equal-text substitution.
    /// Text control/payload storage belongs to the text callback's actual-owner census.
    pub fn visit_retained_metadata<E>(
        &self,
        admit: &mut impl FnMut(usize, usize) -> Result<(), E>,
        text: &mut impl FnMut(SourceAllocationId, &Arc<str>) -> Result<(), E>,
    ) -> Result<(), E> {
        admit(1, self.path.capacity())?;
        if let SourceRecordKind::Embedded {
            resolution_path, ..
        } = &self.kind
        {
            admit(1, resolution_path.capacity())?;
        }
        let mut owner = &self.snapshot;
        loop {
            admit(1, 0)?;
            text(owner.allocation, &owner.text)?;
            admit(1, 0)?;
            let Some((prefix, _)) = &owner.prefix else {
                break;
            };
            admit(
                1,
                std::mem::size_of::<SourceTextSnapshot>() + 3 * std::mem::size_of::<usize>(),
            )?;
            owner = prefix;
        }
        Ok(())
    }

    pub fn span_owner(&self, span: Span) -> Option<(SourceAllocationId, Arc<str>)> {
        self.snapshot.owner_at(span)
    }
    /// Borrow the actual span owner after admitting each source/prefix lookup.
    /// The caller can measure its backing before retaining an Arc clone; this
    /// method allocates nothing and preserves the same allocation witness.
    pub fn span_owner_ref_with_work<E>(
        &self,
        span: Span,
        admit: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<(SourceAllocationId, &Arc<str>)>, E> {
        admit(1)?;
        self.snapshot.owner_ref_at_with_work(span, admit)
    }
    pub fn snapshot(&self) -> &SourceTextSnapshot {
        &self.snapshot
    }
    pub fn id(&self) -> SourceId {
        self.id
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn kind(&self) -> &SourceRecordKind {
        &self.kind
    }
    pub fn physical_path(&self) -> Option<&Path> {
        match self.kind {
            SourceRecordKind::File => Some(&self.path),
            SourceRecordKind::Embedded {
                ..
            } => None,
        }
    }
    pub fn resolution_path(&self) -> &Path {
        match &self.kind {
            SourceRecordKind::File => &self.path,
            SourceRecordKind::Embedded {
                resolution_path, ..
            } => resolution_path,
        }
    }
    pub fn importing_site(&self) -> Option<SourceSpan> {
        match self.kind {
            SourceRecordKind::File => None,
            SourceRecordKind::Embedded {
                importing, ..
            } => Some(importing),
        }
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Retain this source's immutable text and allocation provenance across phases.
    pub fn shared_text(&self) -> Arc<str> {
        Arc::clone(&self.text)
    }
}
#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    records: Vec<SourceRecord>,
}
impl SourceMap {
    /// Visit actual record capacity before lending records to a source-owner scanner.
    pub fn visit_retained_record_storage<E>(
        &self,
        admit: &mut impl FnMut(usize, usize) -> Result<(), E>,
        source: &mut impl FnMut(&SourceRecord) -> Result<(), E>,
    ) -> Result<(), E> {
        admit(
            self.records.capacity().saturating_add(1),
            self.records
                .capacity()
                .saturating_mul(std::mem::size_of::<SourceRecord>()),
        )?;
        for record in &self.records {
            admit(1, 0)?;
            source(record)?;
        }
        Ok(())
    }

    pub fn insert(&mut self, path: PathBuf, text: String) -> SourceId {
        self.insert_snapshot(path, SourceTextSnapshot::new(text))
    }
    pub fn insert_snapshot(&mut self, path: PathBuf, snapshot: SourceTextSnapshot) -> SourceId {
        let id = SourceId(self.records.len());
        let text = Arc::clone(&snapshot.text);
        self.records.push(SourceRecord {
            snapshot,
            id,
            path,
            text,
            kind: SourceRecordKind::File,
        });
        id
    }
    /// Publish literal source with an actual importing site and a separate relative-resolution anchor.
    pub fn insert_embedded(
        &mut self,
        label: PathBuf,
        text: String,
        importing: SourceSpan,
        resolution_path: PathBuf,
    ) -> SourceId {
        let id = SourceId(self.records.len());
        let snapshot = SourceTextSnapshot::new(text);
        let text = Arc::clone(&snapshot.text);
        self.records.push(SourceRecord {
            snapshot,
            id,
            path: label,
            text,
            kind: SourceRecordKind::Embedded {
                importing,
                resolution_path,
            },
        });
        id
    }
    pub fn get(&self, id: SourceId) -> Option<&SourceRecord> {
        self.records.get(id.0)
    }
    pub fn records(&self) -> &[SourceRecord] {
        &self.records
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn borrowed_owner_admission_preserves_prefix_authority_and_stops_before_each_hop() {
        let original = SourceTextSnapshot::new("ANSWER :: 42;".into());
        let span = Span::new(0, original.text().len());
        let appended = original
            .append("\na :: 1;")
            .append("\nb :: 2;")
            .append("\nc :: 3;");
        let mut sources = SourceMap::default();
        let id = sources.insert_snapshot("root.jai".into(), appended);
        let record = sources.get(id).unwrap();
        let mut steps = 0;
        let (allocation, text) = record
            .span_owner_ref_with_work(span, &mut |work| {
                steps += work;
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(allocation, original.allocation);
        assert!(Arc::ptr_eq(text, &original.text));
        assert!(steps >= 6);
        for stop in 1..=steps {
            let mut calls = 0;
            let result = record.span_owner_ref_with_work(span, &mut |_| {
                calls += 1;
                if calls == stop {
                    Err("denied")
                } else {
                    Ok(())
                }
            });
            assert_eq!(result, Err("denied"));
            assert_eq!(calls, stop);
        }
        let copied = SourceTextSnapshot::new(record.text().to_owned());
        let (other, _) = copied.owner_at(span).unwrap();
        assert_ne!(allocation, other);
        let (retained, shared) = record.span_owner(span).unwrap();
        assert_eq!(retained, allocation);
        assert!(Arc::ptr_eq(&shared, text));
    }
    #[test]
    fn invalid_span_is_checked_only_after_owner_work_admission() {
        let mut sources = SourceMap::default();
        let id = sources.insert("root.jai".into(), "é".into());
        let record = sources.get(id).unwrap();
        assert_eq!(
            record.span_owner_ref_with_work(Span::new(1, 2), &mut |_| Err("denied")),
            Err("denied")
        );
        assert!(
            record
                .span_owner_ref_with_work(Span::new(1, 2), &mut |_| Ok::<_, ()>(()))
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn registry_owns_identity_and_source_provenance() {
        let mut identities = Identities::default();
        assert_ne!(identities.module(), identities.module());
        assert_eq!(identities.scope().index(), 0);
        let mut sources = SourceMap::default();
        let id = sources.insert("a.jai".into(), "é\nbad".into());
        let diagnostic = LocatedDiagnostic::new(id, Diagnostic::new(Span::new(3, 6), "bad name"));
        assert_eq!(diagnostic.render(&sources), "a.jai:2:1: error: bad name");
        assert_eq!(sources.get(id).unwrap().id(), id);
    }
    #[test]
    fn explicit_diagnostic_origin_survives_lexical_source_fallbacks() {
        let mut sources = SourceMap::default();
        let caller = sources.insert("caller.jai".into(), "caller".into());
        let quote = sources.insert("quote.jai".into(), "é\r\nquoted".into());
        let location = SourceSpan {
            source: quote,
            span: Span::new(4, 10),
        };
        let diagnostic = Diagnostic::at_source(location, "bad quote").with_fallback_source(caller);
        let located = LocatedDiagnostic::new(caller, diagnostic);
        assert_eq!(located.location, location);
        assert_eq!(located.render(&sources), "quote.jai:2:1: error: bad quote");
        let caller_diagnostic = Diagnostic::new(Span::new(0, 6), "bad caller");
        assert_eq!(caller_diagnostic.source, None);
        assert_eq!(
            LocatedDiagnostic::new(caller, caller_diagnostic)
                .location
                .source,
            caller
        );
    }
}
