//! A requested source frontier is sealed before descriptor storage is interned.
use super::*;
use jai_ir::RuntimeInfoSnapshot;
use jai_source::SourceSpan;
use jai_types::{LayoutPolicy, RuntimeInfoField, RuntimeInfoSchema};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct RuntimeInfoCheckpoint {
    catalog: catalog::CatalogCheckpoint,
    schema: RuntimeInfoSchema,
    policy: LayoutPolicy,
}

impl RuntimeInfoCheckpoint {
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct RuntimeInfoKey {
    generation: u64,
    policy_epoch: u64,
    schema: RuntimeInfoSchema,
    policy: LayoutPolicy,
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

    /// This creates a source receipt, not descriptor storage. It is safe to
    /// retain while a VM drive waits for an actually requested Runtime_Info.
    pub(crate) fn runtime_info_checkpoint(
        &mut self,
        types: &TypeRegistry,
        schema: RuntimeInfoSchema,
        policy: Option<LayoutPolicy>,
        location: SourceSpan,
    ) -> Result<ReflectionReadiness<RuntimeInfoCheckpoint>, Diagnostic> {
        let Some(policy) = policy else {
            return Ok(ReflectionReadiness::Pending(Box::new([
                jai_types::ReflectionDependency::TargetLayout,
            ])));
        };
        match schema.revalidate(types) {
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
            .checkpoint(types)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
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
        Ok(ReflectionReadiness::Ready(RuntimeInfoCheckpoint {
            catalog,
            schema,
            policy,
        }))
    }

    /// Consume the retained frontier; never reseal a later source type set in
    /// response to an earlier VM demand or admit this table's backing types.
    pub(crate) fn runtime_info_snapshot(
        &mut self,
        types: &mut TypeRegistry,
        checkpoint: &RuntimeInfoCheckpoint,
        metadata: &jai_types::ReflectionMetadata,
        location: SourceSpan,
    ) -> Result<ReflectionReadiness<Arc<RuntimeInfoSnapshot>>, Diagnostic> {
        self.validate_runtime_info_checkpoint(types, checkpoint, location)?;
        self.synchronize_reflection_policy(types, location.span)?;
        let key = RuntimeInfoKey {
            generation: checkpoint.generation(),
            policy_epoch: self.descriptor_policy_epoch,
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
        let Some((&root, additional)) = represented.split_first() else {
            return Err(Diagnostic::at_source(
                location,
                "Runtime_Info source checkpoint has no language types",
            ));
        };
        let graph = match jai_types::ReflectionGraph::build_with_roots(
            types,
            root,
            additional,
            Some(checkpoint.policy),
            metadata,
        )
        .map_err(|error| Diagnostic::at_source(location, error.to_string()))?
        {
            ReflectionReadiness::Ready(graph) => graph,
            ReflectionReadiness::Pending(dependencies) => {
                return Ok(ReflectionReadiness::Pending(dependencies));
            }
        };
        let mut policies = Vec::new();
        for descriptor in graph.descriptors() {
            let ty = descriptor.id.represented_type();
            if matches!(descriptor.kind, jai_types::DescriptorKind::Record { .. }) {
                policies.push((
                    ty,
                    types
                        .record_reflection_policy(ty)
                        .map_err(|error| Diagnostic::at_source(location, error.to_string()))?,
                ));
            }
        }
        let materialized = storage::materialize(
            types,
            &graph,
            schema,
            &self.storage,
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
            self.storage
                .insert(represented, (pointer, Arc::clone(&data), address));
        }
        self.storage_policies.extend(policies);
        let publication = (|| -> Result<Arc<RuntimeInfoSnapshot>, Box<dyn std::error::Error>> {
            let header_pointer = checkpoint.schema.runtime_type_schema().descriptor_type();
            let mut rows = Vec::with_capacity(represented.len());
            for &ty in represented {
                let (_, _, address) = self
                    .storage
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
