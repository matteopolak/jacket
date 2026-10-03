//! Checked source provenance, separate from executable expression semantics.
mod paths;
mod procedure_notes;
mod types;
use crate::{CleanupId, DebugPolicy, LocalId, Procedure, ProcedureId};
use jai_source::{SourceId, SourceRecord, SourceSpan};
pub use paths::*;
pub use procedure_notes::{ProcedureNote, ProcedureNoteError};
use std::{collections::HashMap, fmt, num::NonZeroU32, path::Path, sync::Arc};
pub use types::{FieldSource, TypeSource};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugSourceLocation {
    path: Arc<Path>,
    source_span: SourceSpan,
    source_text: Arc<str>,
    line: NonZeroU32,
    column: NonZeroU32,
}
impl DebugSourceLocation {
    pub fn from_source(
        source: &SourceRecord,
        span: SourceSpan,
    ) -> Result<Self, DebugSourceLocationError> {
        Self::with_coordinates(
            source,
            span,
            Arc::from(source.path()),
            &SourceCoordinates::new(source.text()),
        )
    }
    fn with_coordinates(
        source: &SourceRecord,
        span: SourceSpan,
        path: Arc<Path>,
        coordinates: &SourceCoordinates,
    ) -> Result<Self, DebugSourceLocationError> {
        if source.id() != span.source {
            return Err(DebugSourceLocationError::WrongSource);
        }
        let text = source.text();
        if span.span.start > span.span.end
            || span.span.end > text.len()
            || !text.is_char_boundary(span.span.start)
            || !text.is_char_boundary(span.span.end)
        {
            return Err(DebugSourceLocationError::InvalidSpan);
        }
        let (line, column) = coordinates.at(span.span.start);
        let line = u32::try_from(line)
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or(DebugSourceLocationError::CoordinateOverflow)?;
        let column = u32::try_from(column)
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or(DebugSourceLocationError::CoordinateOverflow)?;
        Ok(Self {
            path,
            source_span: span,
            source_text: source.shared_text(),
            line,
            column,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn span(&self) -> SourceSpan {
        self.source_span
    }
    pub fn line(&self) -> NonZeroU32 {
        self.line
    }
    pub fn column(&self) -> NonZeroU32 {
        self.column
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcedureSource {
    pub name: String,
    pub location: DebugSourceLocation,
}
#[derive(Clone, Debug, Default)]
pub struct DebugSources {
    primary_source: Option<(SourceId, Arc<str>)>,
    files: HashMap<SourceId, SourceFile>,
    procedures: HashMap<ProcedureId, ProcedureSource>,
    procedure_policies: HashMap<ProcedureId, DebugPolicy>,
    blocks: HashMap<BlockPath, DebugSourceLocation>,
    statements: HashMap<StatementPath, DebugSourceLocation>,
    locals: HashMap<LocalId, LocalSource>,
    cleanup_parents: HashMap<(ProcedureId, CleanupId), BlockPath>,
    type_provenance: types::TypeProvenance,
    procedure_notes: procedure_notes::ProcedureNotes,
}
#[derive(Clone, Debug)]
struct SourceFile {
    path: Arc<Path>,
    text: Arc<str>,
    coordinates: Arc<SourceCoordinates>,
}
#[derive(Debug)]
struct SourceCoordinates {
    line_starts: Vec<usize>,
    continuation_bytes: Vec<usize>,
}
impl SourceCoordinates {
    fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        let mut continuation_bytes = Vec::new();
        for (position, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(position + 1);
            }
            if byte & 0xc0 == 0x80 {
                continuation_bytes.push(position);
            }
        }
        Self {
            line_starts,
            continuation_bytes,
        }
    }
    fn at(&self, byte: usize) -> (usize, usize) {
        let line = self.line_starts.partition_point(|start| *start <= byte);
        let start = self.line_starts[line - 1];
        let continuations = self
            .continuation_bytes
            .partition_point(|position| *position < byte)
            - self
                .continuation_bytes
                .partition_point(|position| *position < start);
        (line, byte - start - continuations + 1)
    }
}

impl DebugSources {
    /// A same-identity semantic rebind must publish its new body provenance.
    /// Shared source files and nominal type/field origins remain valid.
    pub fn clear_procedure(&mut self, procedure: ProcedureId) {
        self.clear_procedure_notes(procedure);
        self.procedures.remove(&procedure);
        self.procedure_policies.remove(&procedure);
        self.blocks
            .retain(|path, _| path.root.procedure() != procedure);
        self.statements
            .retain(|path, _| path.root.procedure() != procedure);
        self.locals.retain(|id, _| id.procedure() != procedure);
        self.cleanup_parents
            .retain(|(owner, _), _| *owner != procedure);
    }
    /// Preserve the declaration's source independently of metadata emission.
    pub fn set_procedure_policy(&mut self, procedure: ProcedureId, policy: DebugPolicy) {
        self.procedure_policies.insert(procedure, policy);
    }
    pub fn procedure_policy(&self, procedure: ProcedureId) -> DebugPolicy {
        self.procedure_policies
            .get(&procedure)
            .copied()
            .unwrap_or_default()
    }
    pub fn set_primary_source(&mut self, source: &SourceRecord) {
        self.retain_source(source);
        self.primary_source = Some((source.id(), source.shared_text()));
    }
    pub fn primary_path(&self) -> Option<&Path> {
        self.primary_source
            .as_ref()
            .and_then(|(source, text)| {
                self.files
                    .get(source)
                    .filter(|file| Arc::ptr_eq(&file.text, text))
            })
            .map(|file| file.path.as_ref())
    }
    pub fn retain_source(&mut self, source: &SourceRecord) {
        self.files.entry(source.id()).or_insert_with(|| SourceFile {
            path: Arc::from(source.path()),
            text: source.shared_text(),
            coordinates: Arc::new(SourceCoordinates::new(source.text())),
        });
    }
    /// Reuse this inventory's indexed coordinates while retaining exact source ownership.
    pub fn source_location(
        &mut self,
        source: &SourceRecord,
        span: SourceSpan,
    ) -> Result<DebugSourceLocation, DebugSourceLocationError> {
        self.retain_source(source);
        let file = self.files.get(&source.id()).expect("retained source");
        if !Arc::ptr_eq(&file.text, &source.shared_text()) {
            return Err(DebugSourceLocationError::WrongSource);
        }
        DebugSourceLocation::with_coordinates(source, span, file.path.clone(), &file.coordinates)
    }
    pub(crate) fn validate(
        &self,
        procedure_exists: impl Fn(ProcedureId) -> bool,
    ) -> Result<(), crate::IrError> {
        self.validate_procedure_notes(&procedure_exists)?;
        if let Some((source, text)) = &self.primary_source
            && !self
                .files
                .get(source)
                .is_some_and(|file| Arc::ptr_eq(&file.text, text))
        {
            return Err(crate::IrError::UnknownIdentity {
                kind: "debug primary source",
                index: source.index(),
            });
        }
        for (id, source) in &self.procedures {
            if !procedure_exists(*id) {
                return Err(crate::IrError::UnknownIdentity {
                    kind: "debug procedure",
                    index: id.index(),
                });
            }
            self.validate_location(&source.location)?;
        }
        for id in self.procedure_policies.keys() {
            self.validate_owner(*id, &procedure_exists)?;
        }
        for (path, location) in &self.blocks {
            self.validate_owner(path.root.procedure(), &procedure_exists)?;
            self.validate_location(location)?;
        }
        for (path, location) in &self.statements {
            self.validate_owner(path.root.procedure(), &procedure_exists)?;
            self.validate_location(location)?;
        }
        for (id, source) in &self.locals {
            self.validate_owner(id.procedure(), &procedure_exists)?;
            self.validate_location(&source.location)?;
            if source.name.is_empty() || source.name.as_bytes().contains(&0) {
                return Err(crate::IrError::UnknownIdentity {
                    kind: "debug local name",
                    index: id.index(),
                });
            }
        }
        for (&(procedure, _), parent) in &self.cleanup_parents {
            self.validate_owner(procedure, &procedure_exists)?;
            self.validate_owner(parent.root.procedure(), &procedure_exists)?;
        }
        Ok(())
    }
    fn validate_owner(
        &self,
        procedure: ProcedureId,
        exists: impl Fn(ProcedureId) -> bool,
    ) -> Result<(), crate::IrError> {
        if exists(procedure) {
            Ok(())
        } else {
            Err(crate::IrError::UnknownIdentity {
                kind: "debug procedure",
                index: procedure.index(),
            })
        }
    }
    fn validate_location(&self, location: &DebugSourceLocation) -> Result<(), crate::IrError> {
        let Some(file) = self.files.get(&location.source_span.source) else {
            return Err(crate::IrError::UnknownIdentity {
                kind: "debug source",
                index: location.source_span.source.index(),
            });
        };
        let span = location.source_span.span;
        if !Arc::ptr_eq(&file.text, &location.source_text)
            || file.path != location.path
            || span.start > span.end
            || span.end > file.text.len()
            || !file.text.is_char_boundary(span.start)
            || !file.text.is_char_boundary(span.end)
        {
            return Err(crate::IrError::UnknownIdentity {
                kind: "debug source span",
                index: location.source_span.source.index(),
            });
        }
        let (line, column) = file.coordinates.at(span.start);
        if line != location.line.get() as usize || column != location.column.get() as usize {
            return Err(crate::IrError::UnknownIdentity {
                kind: "debug source coordinate",
                index: location.source_span.source.index(),
            });
        }
        Ok(())
    }
    pub(crate) fn validate_ir<'a>(
        &self,
        lookup: impl Fn(ProcedureId) -> Option<&'a Procedure>,
    ) -> Result<(), crate::IrError> {
        let get = |id: ProcedureId| {
            lookup(id).ok_or(crate::IrError::UnknownIdentity {
                kind: "debug procedure body",
                index: id.index(),
            })
        };
        for path in self.blocks.keys() {
            paths::block(path, get(path.root.procedure())?)?;
        }
        for path in self.statements.keys() {
            paths::statement(path, get(path.root.procedure())?)?;
        }
        for (&(procedure, cleanup), parent) in &self.cleanup_parents {
            let body = get(procedure)?;
            if parent.root.procedure() != procedure {
                return Err(crate::IrError::LocalOwner {
                    expected: procedure,
                    actual: parent.root.procedure(),
                });
            }
            paths::block(&BlockPath::cleanup(procedure, cleanup), body)?;
            paths::block(parent, body)?;
        }
        self.validate_cleanup_parents()?;
        for (id, source) in &self.locals {
            let procedure = get(id.procedure())?;
            if source.scope.root.procedure() != id.procedure() {
                return Err(crate::IrError::LocalOwner {
                    expected: id.procedure(),
                    actual: source.scope.root.procedure(),
                });
            }
            paths::block(&source.scope, procedure)?;
            if !procedure
                .parameters
                .iter()
                .chain(&procedure.locals)
                .any(|local| local.id() == *id)
            {
                return Err(crate::IrError::UnknownIdentity {
                    kind: "debug local",
                    index: id.index(),
                });
            }
            match &source.declaration {
                LocalDeclaration::Parameter(ordinal) => {
                    if procedure.parameters.get(*ordinal).map(|local| local.id()) != Some(*id)
                        || source.scope != BlockPath::procedure(id.procedure())
                    {
                        return Err(crate::IrError::UnknownIdentity {
                            kind: "debug parameter",
                            index: *ordinal,
                        });
                    }
                }
                LocalDeclaration::Statement(path) => {
                    paths::statement(path, procedure)?;
                    // Case subjects are statements nested directly in another
                    // statement, so their path can end in Child(CaseSubject).
                    let statement_index = path
                        .steps
                        .iter()
                        .rposition(|step| matches!(step, DebugPathStep::Statement(_)))
                        .expect("validated statement path has a containing block");
                    let declaration_block = &path.steps[..statement_index];
                    let loop_body = source
                        .scope
                        .steps
                        .strip_prefix(path.steps.as_ref())
                        .is_some_and(|tail| {
                            matches!(
                                tail,
                                [DebugPathStep::Child(
                                    DebugBranch::While | DebugBranch::Range
                                )]
                            )
                        });
                    let in_scope = path.root == source.scope.root
                        && (declaration_block == source.scope.steps.as_ref() || loop_body);
                    if !in_scope {
                        return Err(crate::IrError::UnknownIdentity {
                            kind: "debug declaration scope",
                            index: id.index(),
                        });
                    }
                }
            }
        }
        Ok(())
    }
    fn validate_cleanup_parents(&self) -> Result<(), crate::IrError> {
        let mut complete = HashMap::new();
        for &root in self.cleanup_parents.keys() {
            if complete.contains_key(&root) {
                continue;
            }
            let mut active = std::collections::HashSet::new();
            let mut chain = Vec::new();
            let mut current = Some(root);
            let mut height = 0usize;
            while let Some(key) = current {
                if let Some(&known) = complete.get(&key) {
                    height = known;
                    break;
                }
                if !active.insert(key) {
                    return Err(crate::IrError::UnknownIdentity {
                        kind: "debug cleanup lexical cycle",
                        index: key.1.index(),
                    });
                }
                chain.push(key);
                if chain.len() > 128 {
                    return Err(crate::IrError::VerificationDepth);
                }
                current = self
                    .cleanup_parents
                    .get(&key)
                    .and_then(|parent| match parent.root {
                        BlockRoot::ProcedureBody(_) => None,
                        BlockRoot::Cleanup {
                            procedure,
                            cleanup,
                        } => Some((procedure, cleanup)),
                    });
            }
            while let Some(key) = chain.pop() {
                height += 1;
                if height > 128 {
                    return Err(crate::IrError::VerificationDepth);
                }
                complete.insert(key, height);
            }
        }
        Ok(())
    }
    /// Retain the lexical declaration scope for a separately emitted cleanup body.
    pub fn insert_cleanup_parent(
        &mut self,
        procedure: ProcedureId,
        cleanup: CleanupId,
        parent: BlockPath,
    ) -> Option<BlockPath> {
        self.cleanup_parents.insert((procedure, cleanup), parent)
    }
    pub fn cleanup_parent(&self, procedure: ProcedureId, cleanup: CleanupId) -> Option<&BlockPath> {
        self.cleanup_parents.get(&(procedure, cleanup))
    }
    pub fn insert_block(
        &mut self,
        path: BlockPath,
        location: DebugSourceLocation,
    ) -> Option<DebugSourceLocation> {
        self.blocks.insert(path, location)
    }
    pub fn insert_statement(
        &mut self,
        path: StatementPath,
        location: DebugSourceLocation,
    ) -> Option<DebugSourceLocation> {
        self.statements.insert(path, location)
    }
    pub fn insert_local(&mut self, id: LocalId, source: LocalSource) -> Option<LocalSource> {
        self.locals.insert(id, source)
    }
    pub fn block(&self, path: &BlockPath) -> Option<&DebugSourceLocation> {
        self.blocks.get(path)
    }
    pub fn statement(&self, path: &StatementPath) -> Option<&DebugSourceLocation> {
        self.statements.get(path)
    }
    pub fn local(&self, id: LocalId) -> Option<&LocalSource> {
        self.locals.get(&id)
    }
    pub fn blocks(&self) -> impl Iterator<Item = (&BlockPath, &DebugSourceLocation)> {
        self.blocks.iter()
    }
    pub fn statements(&self) -> impl Iterator<Item = (&StatementPath, &DebugSourceLocation)> {
        self.statements.iter()
    }
    pub fn locals(&self) -> impl Iterator<Item = (LocalId, &LocalSource)> {
        self.locals.iter().map(|(id, source)| (*id, source))
    }
    pub fn insert(&mut self, id: ProcedureId, source: ProcedureSource) -> Option<ProcedureSource> {
        self.procedures.insert(id, source)
    }
    pub fn procedure(&self, id: ProcedureId) -> Option<&ProcedureSource> {
        self.procedures.get(&id)
    }
    pub fn procedures(&self) -> impl Iterator<Item = (ProcedureId, &ProcedureSource)> {
        self.procedures.iter().map(|(id, source)| (*id, source))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugSourceLocationError {
    WrongSource,
    InvalidSpan,
    CoordinateOverflow,
}
impl fmt::Display for DebugSourceLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid debug source location: {self:?}")
    }
}
impl std::error::Error for DebugSourceLocationError {
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, Span};
    #[test]
    fn checked_utf8_span_keeps_real_source_coordinates() {
        let mut sources = SourceMap::default();
        let id = sources.insert("module/input.jai".into(), "// é\r\nmain :: () {}".into());
        let source = sources.get(id).unwrap();
        let at = source.text().find("main").unwrap();
        let location = DebugSourceLocation::from_source(
            source,
            SourceSpan {
                source: id,
                span: Span::new(at, source.text().len()),
            },
        )
        .unwrap();
        assert_eq!(location.path(), Path::new("module/input.jai"));
        assert_eq!(location.line().get(), 2);
        assert_eq!(location.column().get(), 1);
        assert_eq!(location.span().span.start, at);
        assert_eq!(
            DebugSourceLocation::from_source(
                source,
                SourceSpan {
                    source: id,
                    span: Span::new(4, 5)
                }
            ),
            Err(DebugSourceLocationError::InvalidSpan)
        );
        assert_eq!(
            DebugSourceLocation::from_source(
                source,
                SourceSpan {
                    source: id,
                    span: Span::new(0, source.text().len() + 1)
                }
            ),
            Err(DebugSourceLocationError::InvalidSpan)
        );
    }
    fn provenance() -> (SourceMap, DebugSources) {
        let mut map = SourceMap::default();
        let id = map.insert("own/input.jai".into(), "main :: () {}".into());
        let record = map.get(id).unwrap();
        let location = DebugSourceLocation::from_source(
            record,
            SourceSpan {
                source: id,
                span: Span::new(0, record.text().len()),
            },
        )
        .unwrap();
        let mut sources = DebugSources::default();
        sources.retain_source(record);
        for procedure in [27, 105] {
            sources.insert(
                ProcedureId::new(procedure),
                ProcedureSource {
                    name: "main".into(),
                    location: location.clone(),
                },
            );
        }
        (map, sources)
    }
    #[test]
    fn sparse_specializations_can_share_a_real_source_span() {
        let (_, sources) = provenance();
        assert!(
            sources
                .validate(|id| matches!(id.index(), 27 | 105))
                .is_ok()
        );
    }
    #[test]
    fn publication_rejects_unknown_procedure_debug_identity() {
        let (_, sources) = provenance();
        let types = jai_types::TypeRegistry::new().freeze().unwrap();
        let error = crate::ProgramBuilder::new(types)
            .debug_sources(sources)
            .finish_library()
            .unwrap_err();
        assert!(matches!(
            error,
            crate::IrError::UnknownIdentity {
                kind: "debug procedure",
                ..
            }
        ));
    }
    #[test]
    fn finalization_checks_retained_inventory_and_original_byte_bounds() {
        let (_, mut sources) = provenance();
        sources.files.clear();
        assert!(matches!(
            sources.validate(|_| true),
            Err(crate::IrError::UnknownIdentity {
                kind: "debug source",
                ..
            })
        ));
        let (_, mut sources) = provenance();
        sources
            .procedures
            .get_mut(&ProcedureId::new(27))
            .unwrap()
            .location
            .source_span
            .span
            .end += 1;
        assert!(matches!(
            sources.validate(|_| true),
            Err(crate::IrError::UnknownIdentity {
                kind: "debug source span",
                ..
            })
        ));
    }
    #[test]
    fn primary_source_uses_the_same_exact_record_provenance() {
        let (map, mut sources) = provenance();
        let mut other = SourceMap::default();
        let id = other.insert("own/input.jai".into(), "main :: () {}".into());
        assert_eq!(id, map.records()[0].id());
        sources.set_primary_source(other.get(id).unwrap());
        assert!(matches!(
            sources.validate(|_| true),
            Err(crate::IrError::UnknownIdentity {
                kind: "debug primary source",
                ..
            })
        ));
    }
}
