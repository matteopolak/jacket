//! Source-visible reflection roots are independent of descriptor storage types.
use jai_source::{SourceRecord, SourceSpan, Span};
use jai_types::{FloatType, IntegerType, ScalarType, TypeError, TypeId, TypeKind, TypeRegistry};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

const MAX_VISIBLE_TYPES: usize = 65_536;
const MAX_DEPENDENCIES: usize = 1_048_576;
const MAX_PUBLISHED_ROWS: usize = 1_048_576;

#[derive(Clone, Copy)]
enum Origin {
    Builtin,
    Source(SourceSpan),
}

#[derive(Clone, Default)]
pub(crate) struct SourceTypeCatalog {
    owner: Arc<()>,
    registry_anchor: Option<TypeId>,
    ordered: Vec<TypeId>,
    origins: HashMap<TypeId, Origin>,
    generation: u64,
    cached: Option<CatalogCheckpoint>,
    completions: HashMap<CatalogCheckpointId, CatalogCheckpoint>,
    published_rows: usize,
}

/// Its private constructors require checked source promotion. Interner activity
/// alone cannot add a row or change this immutable checkpoint.
#[derive(Clone)]
pub(crate) struct CatalogCheckpoint {
    owner: Arc<()>,
    identity: CatalogCheckpointId,
    generation: u64,
    types: Arc<[TypeId]>,
    origins: Arc<[Origin]>,
    incomplete: Arc<[TypeId]>,
}

/// Receipt identity distinguishes completion of one retained source frontier
/// from another frontier with the same original source generation.
#[derive(Clone)]
pub(crate) struct CatalogCheckpointId(Arc<()>);
impl PartialEq for CatalogCheckpointId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for CatalogCheckpointId {
}
impl Hash for CatalogCheckpointId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
impl CatalogCheckpoint {
    pub(crate) fn identity(&self) -> CatalogCheckpointId {
        self.identity.clone()
    }
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
    pub(crate) fn types(&self) -> &[TypeId] {
        &self.types
    }
    pub(crate) fn incomplete_definitions(&self) -> &[TypeId] {
        &self.incomplete
    }
}

#[derive(Debug)]
pub(crate) enum CatalogError {
    Type(TypeError),
    InvalidSource,
    TypeLimit,
    DependencyLimit,
    SnapshotLimit,
    GenerationExhausted,
    ForeignCheckpoint,
}
impl From<TypeError> for CatalogError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => write!(formatter, "invalid reflection catalog type: {error}"),
            Self::InvalidSource => formatter.write_str("reflection catalog source span is invalid"),
            Self::TypeLimit => formatter.write_str("reflection catalog exceeds its type limit"),
            Self::DependencyLimit => {
                formatter.write_str("reflection catalog exceeds its dependency limit")
            }
            Self::SnapshotLimit => formatter
                .write_str("reflection catalog exceeds its cumulative snapshot reference limit"),
            Self::GenerationExhausted => {
                formatter.write_str("reflection catalog generation is exhausted")
            }
            Self::ForeignCheckpoint => {
                formatter.write_str("reflection checkpoint belongs to another source catalog")
            }
        }
    }
}
impl std::error::Error for CatalogError {
}

impl SourceTypeCatalog {
    pub(crate) fn charge_retained_policies(&mut self, count: usize) -> Result<(), CatalogError> {
        let total = self
            .published_rows
            .checked_add(count)
            .filter(|total| *total <= MAX_PUBLISHED_ROWS)
            .ok_or(CatalogError::SnapshotLimit)?;
        self.published_rows = total;
        Ok(())
    }
    pub(crate) fn validate_checkpoint(
        &self,
        types: &TypeRegistry,
        checkpoint: &CatalogCheckpoint,
    ) -> Result<(), CatalogError> {
        if !Arc::ptr_eq(&self.owner, &checkpoint.owner) || checkpoint.generation > self.generation {
            return Err(CatalogError::ForeignCheckpoint);
        }
        self.validate_owner(types)?;
        Ok(())
    }

    pub(crate) fn register_source(
        &mut self,
        types: &TypeRegistry,
        ty: TypeId,
        source: &SourceRecord,
        span: Span,
    ) -> Result<(), CatalogError> {
        if span.start > span.end || source.text().get(span.start..span.end).is_none() {
            return Err(CatalogError::InvalidSource);
        }
        // Check the proposed root before changing this catalog, including on
        // retries where a source record refers to another compilation arena.
        types.kind(ty)?;
        self.validate_owner(types)?;
        let origin = Origin::Source(SourceSpan {
            source: source.id(),
            span,
        });
        if self.origins.contains_key(&ty) {
            // Definition completion is revisited at the next checkpoint. A
            // repeated checked expression must not copy the whole frontier.
            return self.admit(types, ty, origin);
        }
        self.register_sources(types, [(ty, source, span)])
    }

    pub(crate) fn register_sources<'a>(
        &mut self,
        types: &TypeRegistry,
        roots: impl IntoIterator<Item = (TypeId, &'a SourceRecord, Span)>,
    ) -> Result<(), CatalogError> {
        self.validate_owner(types)?;
        let mut proposed = Vec::new();
        for (ty, source, span) in roots {
            if span.start > span.end || source.text().get(span.start..span.end).is_none() {
                return Err(CatalogError::InvalidSource);
            }
            types.kind(ty)?;
            if proposed.len() >= MAX_DEPENDENCIES {
                return Err(CatalogError::DependencyLimit);
            }
            proposed.push((
                ty,
                Origin::Source(SourceSpan {
                    source: source.id(),
                    span,
                }),
            ));
        }
        if !self.origins.is_empty() && proposed.iter().all(|(ty, _)| self.origins.contains_key(ty))
        {
            // All admission checks have completed, and no new row can fail a
            // budget or generation check. Preserve atomic source promotion
            // without copying a frontier on a repeated provider drive.
            for (ty, origin) in proposed {
                self.admit(types, ty, origin)?;
            }
            return Ok(());
        }
        // The bounded catalog contains only IDs and source facts. Plan on a
        // private copy so admission failure cannot expose a partial frontier.
        let mut staged = self.clone();
        staged.register_builtins(types)?;
        staged.close(types, proposed)?;
        *self = staged;
        Ok(())
    }

    pub(crate) fn checkpoint(
        &mut self,
        types: &TypeRegistry,
    ) -> Result<CatalogCheckpoint, CatalogError> {
        self.validate_owner(types)?;
        let mut staged = self.clone();
        staged.register_builtins(types)?;
        // A reserved source nominal can acquire fields after registration.
        // Promote its now-ready dependencies without admitting unrelated types
        // added by descriptor construction or another implementation helper.
        let roots = staged
            .ordered
            .iter()
            .map(|ty| (*ty, staged.origins[ty]))
            .collect();
        let incomplete = staged.close(types, roots)?;
        let checkpoint = if let Some(cached) = &staged.cached
            && cached.generation == staged.generation
            && cached.incomplete.as_ref() == incomplete.as_slice()
        {
            cached.clone()
        } else {
            let reuse_rows = staged
                .cached
                .as_ref()
                .is_some_and(|cached| cached.generation == staged.generation);
            let new_rows = if reuse_rows {
                0
            } else {
                staged.ordered.len()
            };
            staged.published_rows = staged
                .published_rows
                .checked_add(new_rows.saturating_mul(2))
                .and_then(|count| count.checked_add(incomplete.len()))
                .filter(|count| *count <= MAX_PUBLISHED_ROWS)
                .ok_or(CatalogError::SnapshotLimit)?;
            let rows = if reuse_rows {
                let cached = staged.cached.as_ref().expect("reusable catalog checkpoint");
                Arc::clone(&cached.types)
            } else {
                Arc::from(staged.ordered.as_slice())
            };
            let origins = if reuse_rows {
                Arc::clone(
                    &staged
                        .cached
                        .as_ref()
                        .expect("reusable catalog checkpoint")
                        .origins,
                )
            } else {
                staged.ordered.iter().map(|ty| staged.origins[ty]).collect()
            };
            let checkpoint = CatalogCheckpoint {
                owner: Arc::clone(&staged.owner),
                identity: CatalogCheckpointId(Arc::new(())),
                generation: staged.generation,
                types: rows,
                origins,
                incomplete: incomplete.into(),
            };
            staged.cached = Some(checkpoint.clone());
            checkpoint
        };
        *self = staged;
        Ok(checkpoint)
    }

    /// Complete only the retained frontier's real nominal dependencies. Later
    /// source admissions and descriptor backing types do not enter this table.
    /// The source generation remains the request's generation, while the opaque
    /// identity seals the resulting ordered rows independently of that number.
    pub(crate) fn complete_checkpoint(
        &mut self,
        types: &TypeRegistry,
        original: &CatalogCheckpoint,
    ) -> Result<CatalogCheckpoint, CatalogError> {
        self.validate_checkpoint(types, original)?;
        let retained_frontier = self.completions.get(&original.identity).unwrap_or(original);
        if retained_frontier.incomplete.is_empty() {
            return Ok(retained_frontier.clone());
        }
        let mut retained = SourceTypeCatalog {
            owner: Arc::clone(&self.owner),
            registry_anchor: self.registry_anchor,
            ordered: retained_frontier.types.to_vec(),
            origins: retained_frontier
                .types
                .iter()
                .copied()
                .zip(retained_frontier.origins.iter().copied())
                .collect(),
            generation: original.generation,
            cached: None,
            completions: HashMap::new(),
            published_rows: 0,
        };
        let roots = retained_frontier
            .types
            .iter()
            .copied()
            .zip(retained_frontier.origins.iter().copied())
            .collect();
        let incomplete = retained.close(types, roots)?;
        if retained.ordered.as_slice() == retained_frontier.types.as_ref()
            && incomplete.as_slice() == retained_frontier.incomplete.as_ref()
        {
            return Ok(retained_frontier.clone());
        }
        let published_rows = self
            .published_rows
            .checked_add(retained.ordered.len().saturating_mul(2))
            .and_then(|count| count.checked_add(incomplete.len()))
            .filter(|count| *count <= MAX_PUBLISHED_ROWS)
            .ok_or(CatalogError::SnapshotLimit)?;
        let checkpoint = CatalogCheckpoint {
            owner: Arc::clone(&self.owner),
            identity: CatalogCheckpointId(Arc::new(())),
            generation: original.generation,
            types: retained.ordered.as_slice().into(),
            origins: retained
                .ordered
                .iter()
                .map(|ty| retained.origins[ty])
                .collect(),
            incomplete: incomplete.into(),
        };
        self.published_rows = published_rows;
        self.completions
            .insert(original.identity(), checkpoint.clone());
        Ok(checkpoint)
    }

    pub(crate) fn source_origin(&self, ty: TypeId) -> Option<SourceSpan> {
        match self.origins.get(&ty)? {
            Origin::Source(location) => Some(*location),
            Origin::Builtin => None,
        }
    }

    fn validate_owner(&self, types: &TypeRegistry) -> Result<(), CatalogError> {
        if let Some(anchor) = self.registry_anchor {
            // Every admitted root and dependency is validated before mutation.
            // This real builtin identity pins that same registry on retries.
            types.kind(anchor)?;
        }
        Ok(())
    }

    fn register_builtins(&mut self, types: &TypeRegistry) -> Result<(), CatalogError> {
        self.validate_owner(types)?;
        self.registry_anchor = Some(types.void());
        let mut builtins = vec![types.void(), types.scalar(ScalarType::Bool), types.string()];
        builtins.extend(
            [
                IntegerType::S8,
                IntegerType::S16,
                IntegerType::S32,
                IntegerType::S64,
                IntegerType::U8,
                IntegerType::U16,
                IntegerType::U32,
                IntegerType::U64,
            ]
            .map(|ty| types.scalar(ScalarType::Int(ty))),
        );
        builtins.extend([types.float(FloatType::F32), types.float(FloatType::F64)]);
        builtins.extend([types.meta_type(), types.code_type()]);
        if let Some(any) = types.any_type() {
            builtins.push(any);
        }
        for ty in builtins {
            self.admit(types, ty, Origin::Builtin)?;
        }
        Ok(())
    }

    fn admit(
        &mut self,
        types: &TypeRegistry,
        ty: TypeId,
        origin: Origin,
    ) -> Result<(), CatalogError> {
        types.kind(ty)?;
        if let Some(previous) = self.origins.get_mut(&ty) {
            // Canonical structural storage may later be named by real source.
            // Its identity does not change; eligibility is monotone.
            if matches!(previous, Origin::Builtin) && matches!(origin, Origin::Source(_)) {
                *previous = origin;
                self.cached = None;
            }
            return Ok(());
        }
        if self.ordered.len() >= MAX_VISIBLE_TYPES {
            return Err(CatalogError::TypeLimit);
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(CatalogError::GenerationExhausted)?;
        self.origins.insert(ty, origin);
        self.ordered.push(ty);
        self.generation = generation;
        Ok(())
    }

    fn close(
        &mut self,
        types: &TypeRegistry,
        mut pending: Vec<(TypeId, Origin)>,
    ) -> Result<Vec<TypeId>, CatalogError> {
        pending.reverse();
        let mut seen = HashSet::new();
        let mut incomplete = Vec::new();
        let mut edges = 0usize;
        while let Some((ty, origin)) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            self.admit(types, ty, origin)?;
            let borrowed = match types.kind(ty)? {
                TypeKind::Pointer(element)
                | TypeKind::Slice(element)
                | TypeKind::DynamicArray(element)
                | TypeKind::FixedArray {
                    element, ..
                } => std::slice::from_ref(element),
                TypeKind::Record(_) => match types.record_definition(ty) {
                    Ok(record) => record.fields.as_ref(),
                    Err(TypeError::Incomplete(_)) => {
                        incomplete.push(ty);
                        continue;
                    }
                    Err(error) => return Err(error.into()),
                },
                TypeKind::Procedure(_) => {
                    let signature = types.procedure_definition(ty)?;
                    edges = edges.saturating_add(signature.results.len());
                    if edges > MAX_DEPENDENCIES
                        || pending.len().saturating_add(signature.results.len()) > MAX_DEPENDENCIES
                    {
                        return Err(CatalogError::DependencyLimit);
                    }
                    pending.extend(signature.results.iter().rev().map(|&ty| (ty, origin)));
                    signature.parameters.as_ref()
                }
                TypeKind::Distinct(_) => match types.distinct_definition(ty) {
                    Ok(definition) => std::slice::from_ref(&definition.representation),
                    Err(TypeError::Incomplete(_)) => {
                        incomplete.push(ty);
                        continue;
                    }
                    Err(error) => return Err(error.into()),
                },
                TypeKind::Enum(_) => {
                    match types.enum_definition(ty) {
                        Ok(definition) => {
                            if edges >= MAX_DEPENDENCIES || pending.len() >= MAX_DEPENDENCIES {
                                return Err(CatalogError::DependencyLimit);
                            }
                            pending.push((
                                types.scalar(ScalarType::Int(definition.representation)),
                                origin,
                            ));
                        }
                        Err(TypeError::Incomplete(_)) => {
                            incomplete.push(ty);
                            continue;
                        }
                        Err(error) => return Err(error.into()),
                    }
                    edges = edges.saturating_add(1);
                    &[]
                }
                _ => &[],
            };
            edges = edges.saturating_add(borrowed.len());
            if edges > MAX_DEPENDENCIES
                || pending.len().saturating_add(borrowed.len()) > MAX_DEPENDENCIES
            {
                return Err(CatalogError::DependencyLimit);
            }
            pending.extend(borrowed.iter().rev().map(|&ty| (ty, origin)));
        }
        Ok(incomplete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;
    use jai_types::RecordKind;

    #[test]
    fn pending_checkpoint_completes_only_its_original_source_dependencies() {
        let mut types = TypeRegistry::new();
        let mut catalog = SourceTypeCatalog::default();
        let root = types.reserve_record(RecordKind::Struct);
        let child = types.reserve_record(RecordKind::Struct);
        let unrelated = types.reserve_record(RecordKind::Struct);
        types.define_record(unrelated, Vec::new()).unwrap();
        let (sources, id) = source("Root::struct{}");
        let source = sources.get(id).unwrap();
        catalog
            .register_source(&types, root, source, Span::new(0, 14))
            .unwrap();
        let original = catalog.checkpoint(&types).unwrap();
        assert!(original.incomplete_definitions().contains(&root));
        let waiting = catalog.complete_checkpoint(&types, &original).unwrap();
        assert!(waiting.identity() == original.identity());
        catalog
            .register_source(&types, unrelated, source, Span::new(0, 14))
            .unwrap();
        types
            .define_record(child, [types.scalar(ScalarType::Bool)])
            .unwrap();
        let pointer = types.pointer(child).unwrap();
        types.define_record(root, [pointer]).unwrap();
        let complete = catalog.complete_checkpoint(&types, &original).unwrap();
        assert_eq!(complete.generation(), original.generation());
        assert!(complete.identity() != original.identity());
        assert!(complete.incomplete_definitions().is_empty());
        assert!(complete.types().contains(&root));
        assert!(complete.types().contains(&child));
        assert!(complete.types().contains(&pointer));
        assert!(!complete.types().contains(&unrelated));
        assert!(!original.types().contains(&child));
        assert!(
            catalog
                .checkpoint(&types)
                .unwrap()
                .types()
                .contains(&unrelated)
        );
        assert!(matches!(
            SourceTypeCatalog::default().complete_checkpoint(&types, &original),
            Err(CatalogError::ForeignCheckpoint)
        ));
    }

    #[test]
    fn partial_completion_reuses_its_receipt_until_a_definition_changes() {
        let mut types = TypeRegistry::new();
        let mut catalog = SourceTypeCatalog::default();
        let root = types.reserve_record(RecordKind::Struct);
        let child = types.reserve_record(RecordKind::Struct);
        let (sources, id) = source("Root::struct{}");
        catalog
            .register_source(&types, root, sources.get(id).unwrap(), Span::new(0, 14))
            .unwrap();
        let original = catalog.checkpoint(&types).unwrap();
        let pointer = types.pointer(child).unwrap();
        types.define_record(root, [pointer]).unwrap();
        let partial = catalog.complete_checkpoint(&types, &original).unwrap();
        assert_eq!(partial.incomplete_definitions(), &[child]);
        let charged = catalog.published_rows;
        for _ in 0..100 {
            let retry = catalog.complete_checkpoint(&types, &original).unwrap();
            assert!(retry.identity() == partial.identity());
            assert!(Arc::ptr_eq(&retry.types, &partial.types));
            assert_eq!(catalog.published_rows, charged);
        }
        types.define_record(child, []).unwrap();
        let complete = catalog.complete_checkpoint(&types, &original).unwrap();
        assert!(complete.incomplete_definitions().is_empty());
        assert!(complete.identity() != partial.identity());
        assert_eq!(complete.generation(), original.generation());
    }

    #[test]
    fn retained_checkpoint_is_catalog_owned_and_keeps_its_original_frontier() {
        let mut types = TypeRegistry::new();
        let mut catalog = SourceTypeCatalog::default();
        let original = catalog.checkpoint(&types).unwrap();
        let root = types.reserve_record(RecordKind::Struct);
        types.define_record(root, Vec::new()).unwrap();
        let (sources, id) = source("Root::struct{}");
        catalog
            .register_source(&types, root, sources.get(id).unwrap(), Span::new(0, 14))
            .unwrap();
        let latest = catalog.checkpoint(&types).unwrap();
        catalog.validate_checkpoint(&types, &original).unwrap();
        assert!(!original.types().contains(&root));
        assert!(latest.types().contains(&root));
        assert!(matches!(
            SourceTypeCatalog::default().validate_checkpoint(&types, &original),
            Err(CatalogError::ForeignCheckpoint)
        ));
        assert!(matches!(
            catalog.validate_checkpoint(&TypeRegistry::new(), &original),
            Err(CatalogError::Type(TypeError::ForeignType(_)))
        ));
    }

    fn source(text: &str) -> (SourceMap, jai_source::SourceId) {
        let mut sources = SourceMap::default();
        let id = sources.insert("catalog.jai".into(), text.into());
        (sources, id)
    }

    #[test]
    fn generated_storage_does_not_add_source_visible_rows() {
        let mut types = TypeRegistry::new();
        let mut catalog = SourceTypeCatalog::default();
        let before = catalog.checkpoint(&types).unwrap();
        let header = types.reserve_record(RecordKind::Struct);
        let tag = types.scalar(ScalarType::Int(IntegerType::U64));
        types.define_record(header, vec![tag, tag]).unwrap();
        let header_pointer = types.pointer(header).unwrap();
        let backing = types.fixed_array(header_pointer, 17).unwrap();
        let after = catalog.checkpoint(&types).unwrap();
        assert_eq!(before.generation(), after.generation());
        assert!(Arc::ptr_eq(&before.types, &after.types));
        assert!(!after.types().contains(&header));
        assert!(!after.types().contains(&backing));
    }

    #[test]
    fn real_source_promotes_existing_canonical_storage_once() {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let backing = types.fixed_array(byte, 3).unwrap();
        let mut catalog = SourceTypeCatalog::default();
        let before = catalog.checkpoint(&types).unwrap();
        let (sources, id) = source("[3]u8");
        let span = Span {
            start: 0,
            end: 5,
        };
        catalog
            .register_source(&types, backing, sources.get(id).unwrap(), span)
            .unwrap();
        let after = catalog.checkpoint(&types).unwrap();
        catalog
            .register_source(&types, backing, sources.get(id).unwrap(), span)
            .unwrap();
        let retry = catalog.checkpoint(&types).unwrap();
        assert_eq!(after.generation(), before.generation() + 1);
        assert_eq!(after.types().iter().filter(|&&ty| ty == backing).count(), 1);
        assert!(Arc::ptr_eq(&after.types, &retry.types));
        assert!(!before.types().contains(&backing));
        assert_eq!(
            catalog.source_origin(backing),
            Some(SourceSpan {
                source: id,
                span
            })
        );
    }

    #[test]
    fn ready_nominal_fields_extend_only_the_existing_source_frontier() {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        let unrelated = types.reserve_record(RecordKind::Struct);
        let pointer = types.pointer(record).unwrap();
        let mut catalog = SourceTypeCatalog::default();
        let (sources, id) = source("Node");
        catalog
            .register_source(
                &types,
                record,
                sources.get(id).unwrap(),
                Span {
                    start: 0,
                    end: 4,
                },
            )
            .unwrap();
        let reserved = catalog.checkpoint(&types).unwrap();
        assert_eq!(reserved.incomplete_definitions(), &[record]);
        assert!(!reserved.types().contains(&pointer));
        types.define_record(record, vec![pointer]).unwrap();
        let ready = catalog.checkpoint(&types).unwrap();
        assert!(ready.incomplete_definitions().is_empty());
        assert!(ready.types().contains(&record));
        assert!(ready.types().contains(&pointer));
        assert!(!ready.types().contains(&unrelated));
        assert!(!reserved.types().contains(&pointer));
    }

    #[test]
    fn invalid_source_or_registry_cannot_mutate_the_catalog() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let pointer = types.pointer(int).unwrap();
        let mut catalog = SourceTypeCatalog::default();
        let before = catalog.checkpoint(&types).unwrap();
        let (sources, id) = source("é");
        assert!(matches!(
            catalog.register_source(
                &types,
                pointer,
                sources.get(id).unwrap(),
                Span {
                    start: 1,
                    end: 2
                }
            ),
            Err(CatalogError::InvalidSource)
        ));
        let other = TypeRegistry::new();
        assert!(matches!(
            catalog.checkpoint(&other),
            Err(CatalogError::Type(_))
        ));
        let after = catalog.checkpoint(&types).unwrap();
        assert_eq!(before.types(), after.types());
        assert!(Arc::ptr_eq(&before.types, &after.types));
    }

    #[test]
    fn failed_generation_admission_keeps_existing_rows_and_origins() {
        let mut types = TypeRegistry::new();
        let mut catalog = SourceTypeCatalog::default();
        let before = catalog.checkpoint(&types).unwrap();
        catalog.generation = u64::MAX;
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let pointer = types.pointer(int).unwrap();
        let (sources, id) = source("*int");
        assert!(matches!(
            catalog.register_source(
                &types,
                pointer,
                sources.get(id).unwrap(),
                Span {
                    start: 0,
                    end: 4
                }
            ),
            Err(CatalogError::GenerationExhausted)
        ));
        assert_eq!(catalog.ordered.as_slice(), before.types());
        assert_eq!(catalog.generation, u64::MAX);
        assert!(catalog.source_origin(pointer).is_none());
    }

    #[test]
    fn cumulative_publication_limit_does_not_charge_reused_checkpoints() {
        let mut types = TypeRegistry::new();
        let mut catalog = SourceTypeCatalog::default();
        let before = catalog.checkpoint(&types).unwrap();
        catalog.published_rows = MAX_PUBLISHED_ROWS;
        let retry = catalog.checkpoint(&types).unwrap();
        assert!(Arc::ptr_eq(&before.types, &retry.types));
        let record = types.reserve_record(RecordKind::Struct);
        let (sources, id) = source("Node");
        catalog
            .register_source(
                &types,
                record,
                sources.get(id).unwrap(),
                Span {
                    start: 0,
                    end: 4,
                },
            )
            .unwrap();
        let pointer = types.pointer(record).unwrap();
        types.define_record(record, vec![pointer]).unwrap();
        let rows = catalog.ordered.clone();
        let generation = catalog.generation;
        assert!(matches!(
            catalog.checkpoint(&types),
            Err(CatalogError::SnapshotLimit)
        ));
        assert_eq!(catalog.ordered, rows);
        assert_eq!(catalog.generation, generation);
        assert!(!catalog.ordered.contains(&pointer));
    }

    #[test]
    fn batch_admission_validates_all_original_sources_before_publication() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let pointer = types.pointer(int).unwrap();
        let array = types.fixed_array(pointer, 2).unwrap();
        let mut catalog = SourceTypeCatalog::default();
        let before = catalog.checkpoint(&types).unwrap();
        let (sources, id) = source("*int [2]*int");
        let source = sources.get(id).unwrap();
        assert!(matches!(
            catalog.register_sources(
                &types,
                [
                    (pointer, source, Span::new(0, 4)),
                    (array, source, Span::new(5, 99)),
                ],
            ),
            Err(CatalogError::InvalidSource)
        ));
        assert_eq!(catalog.ordered, before.types());
        assert!(catalog.source_origin(pointer).is_none());
        catalog
            .register_sources(
                &types,
                [
                    (pointer, source, Span::new(0, 4)),
                    (array, source, Span::new(5, 12)),
                ],
            )
            .unwrap();
        let after = catalog.checkpoint(&types).unwrap();
        assert_eq!(&after.types()[before.types().len()..], &[pointer, array]);
        catalog
            .register_source(&types, pointer, source, Span::new(0, 4))
            .unwrap();
        let retry = catalog.checkpoint(&types).unwrap();
        assert!(Arc::ptr_eq(&after.types, &retry.types));
    }

    #[test]
    fn readiness_only_checkpoints_charge_new_pending_definition_references() {
        let mut types = TypeRegistry::new();
        let first = types.reserve_record(RecordKind::Struct);
        let second = types.reserve_record(RecordKind::Struct);
        let (sources, id) = source("One Two");
        let source = sources.get(id).unwrap();
        let mut catalog = SourceTypeCatalog::default();
        catalog
            .register_sources(
                &types,
                [
                    (first, source, Span::new(0, 3)),
                    (second, source, Span::new(4, 7)),
                ],
            )
            .unwrap();
        let before = catalog.checkpoint(&types).unwrap();
        assert_eq!(before.incomplete_definitions(), &[first, second]);
        catalog.published_rows = MAX_PUBLISHED_ROWS;
        types.define_record(first, Vec::new()).unwrap();
        assert!(matches!(
            catalog.checkpoint(&types),
            Err(CatalogError::SnapshotLimit)
        ));
        assert_eq!(
            catalog.cached.as_ref().unwrap().incomplete_definitions(),
            &[first, second]
        );
        catalog.published_rows = MAX_PUBLISHED_ROWS - 1;
        let partial = catalog.checkpoint(&types).unwrap();
        assert_eq!(partial.incomplete_definitions(), &[second]);
        assert!(Arc::ptr_eq(&before.types, &partial.types));
        assert_eq!(catalog.published_rows, MAX_PUBLISHED_ROWS);
        types.define_record(second, Vec::new()).unwrap();
        let ready = catalog.checkpoint(&types).unwrap();
        assert!(ready.incomplete_definitions().is_empty());
        assert!(Arc::ptr_eq(&partial.types, &ready.types));
        assert_eq!(catalog.published_rows, MAX_PUBLISHED_ROWS);
        assert_eq!(before.incomplete_definitions(), &[first, second]);
    }
}
