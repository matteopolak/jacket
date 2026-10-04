//! A requested source frontier is sealed before descriptor storage is interned.
use super::*;
use jai_ir::RuntimeInfoSnapshot;
use jai_source::SourceSpan;
use jai_types::{LayoutPolicy, RuntimeInfoField, RuntimeInfoSchema, TypeView};
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
mod requested_policies;
use crate::compile_time::reflection_journal::{ReflectionPolicyOverlay, ReflectionPolicyRevision};
use requested_policies::RequestedPolicies;

/// Issued on the first executed request, including while a definition or the
/// target is pending. The same original frontier is retained until retirement.
#[derive(Clone)]
pub(crate) struct RuntimeInfoFrontier {
    catalog: catalog::CatalogCheckpoint,
    schema: RuntimeInfoSchema,
    policy: Option<LayoutPolicy>,
    policy_epoch: u64,
    policy_revision: Option<ReflectionPolicyRevision>,
    overlay: Option<ReflectionPolicyOverlay>,
    record_policies: Arc<RefCell<HashMap<TypeId, jai_types::RecordReflectionPolicy>>>,
}

#[cfg(test)]
mod tests;
impl RuntimeInfoFrontier {
    pub(crate) fn schema(&self) -> RuntimeInfoSchema {
        self.schema
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeInfoCheckpoint {
    catalog: catalog::CatalogCheckpoint,
    schema: RuntimeInfoSchema,
    policy: LayoutPolicy,
    policy_epoch: u64,
    policy_revision: Option<ReflectionPolicyRevision>,
    graph: Arc<jai_types::ReflectionGraph>,
    record_policies: Box<[(TypeId, jai_types::RecordReflectionPolicy)]>,
}

impl RuntimeInfoCheckpoint {
    pub(crate) fn represented_types(&self) -> &[TypeId] {
        self.catalog.types()
    }
    pub(crate) fn frontier(&self) -> RuntimeInfoFrontier {
        RuntimeInfoFrontier {
            catalog: self.catalog.clone(),
            schema: self.schema,
            policy: Some(self.policy),
            policy_epoch: self.policy_epoch,
            policy_revision: self.policy_revision.clone(),
            overlay: None,
            record_policies: Arc::new(RefCell::new(self.record_policies.iter().copied().collect())),
        }
    }
    pub(crate) fn generation(&self) -> u64 {
        self.catalog.generation()
    }
    pub(crate) fn schema(&self) -> RuntimeInfoSchema {
        self.schema
    }
    pub(crate) fn policy(&self) -> LayoutPolicy {
        self.policy
    }
    pub(crate) fn validate_snapshot(
        &self,
        snapshot: &RuntimeInfoSnapshot,
    ) -> Result<(), jai_ir::RuntimeInfoSnapshotError> {
        if snapshot.schema() != self.schema {
            return Err(jai_ir::RuntimeInfoSnapshotError::DescriptorMismatch);
        }
        if snapshot.policy() != self.policy {
            return Err(jai_ir::RuntimeInfoSnapshotError::TargetMismatch);
        }
        if !snapshot
            .represented_types()
            .eq(self.catalog.types().iter().copied())
        {
            return Err(jai_ir::RuntimeInfoSnapshotError::DescriptorMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct RuntimeInfoKey {
    checkpoint: catalog::CatalogCheckpointId,
    graph: GraphIdentity,
    policy_epoch: u64,
    policy_revision: Option<ReflectionPolicyRevision>,
    schema: RuntimeInfoSchema,
    policy: LayoutPolicy,
}

#[derive(Clone)]
struct GraphIdentity(Arc<jai_types::ReflectionGraph>);
impl PartialEq for GraphIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for GraphIdentity {
}
impl Hash for GraphIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl MetaContext {
    pub(crate) fn validate_runtime_info_checkpoint(
        &self,
        types: &TypeRegistry,
        checkpoint: &RuntimeInfoCheckpoint,
        location: SourceSpan,
    ) -> Result<(), Diagnostic> {
        self.reflection_catalog
            .validate_checkpoint(types, &checkpoint.catalog)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        checkpoint
            .schema
            .revalidate(types)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))
    }

    pub(crate) fn register_reflection_source_type(
        &mut self,
        types: &TypeRegistry,
        ty: TypeId,
        source: &jai_source::SourceRecord,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.reflection_catalog
            .register_source(types, ty, source, span)
            .map_err(|error| {
                Diagnostic::at_source(
                    SourceSpan {
                        source: source.id(),
                        span,
                    },
                    error.to_string(),
                )
            })
    }

    /// Capture source membership before reporting ordinary readiness. Creating
    /// descriptor backing objects later cannot enlarge this receipt.
    pub(crate) fn runtime_info_frontier(
        &mut self,
        types: &TypeRegistry,
        schema: RuntimeInfoSchema,
        policy: Option<LayoutPolicy>,
        location: SourceSpan,
    ) -> Result<RuntimeInfoFrontier, Diagnostic> {
        match schema.revalidate(types) {
            Ok(())
            | Err(jai_types::RuntimeInfoError::Type(jai_types::TypeError::Incomplete(_))) => {}
            Err(error) => return Err(Diagnostic::at_source(location, error.to_string())),
        }
        let catalog = self
            .reflection_catalog
            .checkpoint(types)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        self.synchronize_reflection_policy(types, location.span)?;
        let record_policies = catalog
            .types()
            .iter()
            .copied()
            .filter_map(|ty| {
                matches!(types.kind(ty), Ok(jai_types::TypeKind::Record(_))).then_some(ty)
            })
            .map(|ty| {
                types
                    .record_reflection_policy(ty)
                    .map(|policy| (ty, policy))
            })
            .collect::<Result<HashMap<_, _>, _>>()
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        self.reflection_catalog
            .charge_retained_policies(record_policies.len())
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        Ok(RuntimeInfoFrontier {
            catalog,
            schema,
            policy,
            policy_epoch: self.descriptor_policy_epoch,
            policy_revision: None,
            overlay: None,
            record_policies: Arc::new(RefCell::new(record_policies)),
        })
    }

    /// A reached read observes exactly the accepted setter revision from its
    /// actual Run. The canonical registry remains unchanged until publication.
    pub(crate) fn runtime_info_frontier_with_overlay(
        &mut self,
        types: &TypeRegistry,
        schema: RuntimeInfoSchema,
        policy: Option<LayoutPolicy>,
        overlay: ReflectionPolicyOverlay,
        location: SourceSpan,
    ) -> Result<RuntimeInfoFrontier, Diagnostic> {
        let observed = overlay
            .view(types)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        let mut frontier = self.runtime_info_frontier(types, schema, policy, location)?;
        for (record, policy) in frontier.record_policies.borrow_mut().iter_mut() {
            *policy = observed
                .record_reflection_policy(*record)
                .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        }
        frontier.policy_revision = Some(overlay.revision().clone());
        frontier.overlay = Some(overlay);
        Ok(frontier)
    }

    /// Complete only this request's original roots. A ready checkpoint also
    /// retains its descriptor graph, so a later policy revision cannot reseal it.
    pub(crate) fn complete_runtime_info_frontier(
        &mut self,
        types: &TypeRegistry,
        frontier: &RuntimeInfoFrontier,
        metadata: &jai_types::ReflectionMetadata,
        location: SourceSpan,
    ) -> Result<ReflectionReadiness<RuntimeInfoCheckpoint>, Diagnostic> {
        self.reflection_catalog
            .validate_checkpoint(types, &frontier.catalog)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        let Some(policy) = frontier.policy else {
            return Ok(ReflectionReadiness::Pending(Box::new([
                jai_types::ReflectionDependency::TargetLayout,
            ])));
        };
        match frontier.schema.revalidate(types) {
            Ok(()) => {}
            Err(jai_types::RuntimeInfoError::Type(jai_types::TypeError::Incomplete(ty))) => {
                return Ok(ReflectionReadiness::Pending(Box::new([
                    jai_types::ReflectionDependency::Definition(ty),
                ])));
            }
            Err(error) => return Err(Diagnostic::at_source(location, error.to_string())),
        }
        let catalog = self
            .reflection_catalog
            .complete_checkpoint(types, &frontier.catalog)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        // Retain a newly revealed dependency's policy at its first admission;
        // never replace a policy already observed by this source request.
        let mut policies = frontier.record_policies.borrow_mut();
        let mut additions = Vec::new();
        for &ty in catalog.types() {
            if matches!(types.kind(ty), Ok(jai_types::TypeKind::Record(_))) {
                let current = match &frontier.overlay {
                    Some(overlay) => overlay
                        .view(types)
                        .and_then(|view| view.record_reflection_policy(ty)),
                    None => types.record_reflection_policy(ty),
                }
                .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
                if !policies.contains_key(&ty) {
                    additions.push((ty, current));
                }
            }
        }
        self.reflection_catalog
            .charge_retained_policies(additions.len())
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        policies.extend(additions);
        drop(policies);
        if !catalog.incomplete_definitions().is_empty() {
            return Ok(ReflectionReadiness::Pending(
                catalog
                    .incomplete_definitions()
                    .iter()
                    .copied()
                    .map(jai_types::ReflectionDependency::Definition)
                    .collect(),
            ));
        }
        let policies = frontier.record_policies.borrow();
        let requested_types = RequestedPolicies::new(types, &policies);
        let Some((&root, additional)) = catalog.types().split_first() else {
            return Err(Diagnostic::at_source(
                location,
                "Runtime_Info source checkpoint has no language types",
            ));
        };
        let graph = match jai_types::ReflectionGraph::build_with_roots(
            &requested_types,
            root,
            additional,
            Some(policy),
            metadata,
        )
        .map_err(|error| Diagnostic::at_source(location, error.to_string()))?
        {
            ReflectionReadiness::Ready(graph) => graph,
            ReflectionReadiness::Pending(dependencies) => {
                return Ok(ReflectionReadiness::Pending(dependencies));
            }
        };
        let record_policies = graph
            .descriptors()
            .iter()
            .filter_map(|descriptor| {
                matches!(descriptor.kind, jai_types::DescriptorKind::Record { .. })
                    .then_some(descriptor.id.represented_type())
            })
            .map(|ty| {
                requested_types
                    .record_reflection_policy(ty)
                    .map(|policy| (ty, policy))
            })
            .collect::<Result<Box<[_]>, _>>()
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        Ok(ReflectionReadiness::Ready(RuntimeInfoCheckpoint {
            catalog,
            schema: frontier.schema,
            policy,
            policy_epoch: frontier.policy_epoch,
            policy_revision: frontier.policy_revision.clone(),
            graph: Arc::new(graph),
            record_policies,
        }))
    }

    /// Convenience for immediate source publication. Suspended VM drives retain
    /// `runtime_info_frontier` before calling the completion method separately.
    pub(crate) fn runtime_info_checkpoint(
        &mut self,
        types: &TypeRegistry,
        schema: RuntimeInfoSchema,
        policy: Option<LayoutPolicy>,
        metadata: &jai_types::ReflectionMetadata,
        location: SourceSpan,
    ) -> Result<ReflectionReadiness<RuntimeInfoCheckpoint>, Diagnostic> {
        let frontier = self.runtime_info_frontier(types, schema, policy, location)?;
        self.complete_runtime_info_frontier(types, &frontier, metadata, location)
    }

    /// Consume the retained frontier; never reseal a later source type set in
    /// response to an earlier VM demand or admit this table's backing types.
    pub(crate) fn runtime_info_snapshot(
        &mut self,
        types: &mut TypeRegistry,
        checkpoint: &RuntimeInfoCheckpoint,
        location: SourceSpan,
    ) -> Result<ReflectionReadiness<Arc<RuntimeInfoSnapshot>>, Diagnostic> {
        self.validate_runtime_info_checkpoint(types, checkpoint, location)?;
        let key = RuntimeInfoKey {
            checkpoint: checkpoint.catalog.identity(),
            graph: GraphIdentity(Arc::clone(&checkpoint.graph)),
            policy_epoch: checkpoint.policy_epoch,
            policy_revision: checkpoint.policy_revision.clone(),
            schema: checkpoint.schema,
            policy: checkpoint.policy,
        };
        if let Some(snapshot) = self.runtime_info_snapshots.get(&key) {
            checkpoint
                .validate_snapshot(snapshot)
                .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
            snapshot
                .validate_owner(types, checkpoint.policy)
                .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
            return Ok(ReflectionReadiness::Ready(Arc::clone(snapshot)));
        }
        let schema = self.schema.as_ref().ok_or_else(|| {
            Diagnostic::at_source(
                location,
                "Runtime_Info descriptor schema is not adopted at this source checkpoint",
            )
        })?;
        if schema.header != checkpoint.schema.runtime_type_schema().header_type() {
            return Err(Diagnostic::at_source(
                location,
                "Runtime_Info and descriptor storage use different canonical source headers",
            ));
        }
        let represented = checkpoint.catalog.types();
        let current_revision = checkpoint.policy_revision.is_none()
            && checkpoint.policy_epoch == self.descriptor_policy_epoch
            && checkpoint
                .record_policies
                .iter()
                .all(|&(record, expected)| {
                    types
                        .record_reflection_policy(record)
                        .is_ok_and(|current| current == expected)
                });
        // An older sealed graph owns fresh physical objects in the same static
        // arena. Reusing or updating the current policy cache would mix graphs.
        let mut publication_storage = if current_revision {
            self.storage.clone()
        } else {
            HashMap::new()
        };
        let materialized = storage::materialize(
            types,
            &checkpoint.graph,
            schema,
            &publication_storage,
            &mut self.storage_builder,
        );
        let (data, additions) = match materialized {
            Ok(result) => result,
            Err(error) => {
                self.storage_builder.discard_unpublished();
                return Err(Diagnostic::at_source(location, error.to_string()));
            }
        };
        for (represented, pointer, address) in additions {
            publication_storage.insert(represented, (pointer, Arc::clone(&data), address));
        }
        if current_revision {
            self.storage = publication_storage.clone();
            self.storage_policies
                .extend(checkpoint.record_policies.iter().copied());
        }
        let publication = (|| -> Result<Arc<RuntimeInfoSnapshot>, Box<dyn std::error::Error>> {
            let header_pointer = checkpoint.schema.runtime_type_schema().descriptor_type();
            let mut rows = Vec::with_capacity(represented.len());
            for &ty in represented {
                let (_, _, address) = publication_storage
                    .get(&ty)
                    .ok_or("source descriptor was not materialized")?;
                let descriptor = data.object(address.object())?;
                let header = descriptor
                    .descriptor_header()
                    .ok_or("source descriptor has no checked header")?;
                rows.push(StaticValue {
                    ty: header_pointer,
                    kind: StaticValueKind::Address(header.clone()),
                });
            }
            let table = storage::view(
                types,
                &mut self.storage_builder,
                checkpoint.schema.field(RuntimeInfoField::TypeTable).ty,
                rows,
            )?;
            let object = self
                .storage_builder
                .reserve(checkpoint.schema.ty(), types)?;
            self.storage_builder.define(
                object,
                storage::record(
                    checkpoint.schema.ty(),
                    vec![
                        table,
                        storage::zero(checkpoint.schema.field(RuntimeInfoField::GlobalDataInfo).ty),
                    ],
                ),
            )?;
            let data = Arc::new(
                self.storage_builder
                    .publish(types, StaticDataLimits::default())?,
            );
            Ok(Arc::new(RuntimeInfoSnapshot::new_compile_time(
                data,
                object,
                checkpoint.schema,
                types,
                checkpoint.policy,
                represented,
            )?))
        })();
        let snapshot = match publication {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.storage_builder.discard_unpublished();
                return Err(Diagnostic::at_source(location, error.to_string()));
            }
        };
        self.runtime_info_snapshots
            .insert(key, Arc::clone(&snapshot));
        Ok(ReflectionReadiness::Ready(snapshot))
    }
}
