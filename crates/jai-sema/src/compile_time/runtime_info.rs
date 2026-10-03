//! Private staging for execution-demanded, drive-owned reflection checkpoints.
//!
//! This module is deliberately not registered while the shared VM dependency
//! carrier is frozen. No source name or replay request grants this capability.
use crate::reflection::RuntimeInfoCheckpoint;
use jai_ir::RuntimeInfoSnapshot;
use jai_types::{ReflectionReadiness, TypeView};
use jai_vm::WorkspaceId;
use std::collections::HashMap;
use std::fmt;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_ACTIVE_DEMANDS: usize = 16_384;
static NEXT_DEMAND: AtomicU64 = AtomicU64::new(0);

/// A transient source-service key. It cannot be converted into a replay origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RuntimeInfoDemandId(NonZeroU64);

struct Demand {
    workspace: WorkspaceId,
    checkpoint: Arc<RuntimeInfoCheckpoint>,
    snapshot: Option<Arc<RuntimeInfoSnapshot>>,
}

#[derive(Default)]
pub(crate) struct RuntimeInfoDemands {
    active: HashMap<RuntimeInfoDemandId, Demand>,
}

pub(crate) struct RuntimeInfoService<'a> {
    pub(crate) meta: &'a mut crate::reflection::MetaContext,
    pub(crate) types: &'a mut jai_types::TypeRegistry,
    pub(crate) metadata: &'a jai_types::ReflectionMetadata,
    pub(crate) location: jai_source::SourceSpan,
}

pub(crate) enum RuntimeInfoDemandAvailability<'a> {
    Requested(RuntimeInfoDemandId),
    Ready {
        workspace: WorkspaceId,
        snapshot: &'a RuntimeInfoSnapshot,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RuntimeInfoDemandError {
    Limit,
    IdentityExhausted,
    Retired,
    ForeignWorkspace,
    ForeignSchema,
    ChangedCheckpoint,
    ChangedSnapshot,
    InvalidSnapshot(String),
}
impl fmt::Display for RuntimeInfoDemandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit => f.write_str("runtime-info active demand limit exceeded"),
            Self::IdentityExhausted => f.write_str("runtime-info demand identities are exhausted"),
            Self::Retired => f.write_str("runtime-info demand is unknown or retired"),
            Self::ForeignWorkspace => {
                f.write_str("runtime-info demand belongs to another workspace")
            }
            Self::ForeignSchema => f.write_str("runtime-info demand uses another source schema"),
            Self::ChangedCheckpoint => f.write_str("runtime-info demand checkpoint changed"),
            Self::ChangedSnapshot => f.write_str("runtime-info demand snapshot changed"),
            Self::InvalidSnapshot(error) => {
                write!(f, "invalid runtime-info demand snapshot: {error}")
            }
        }
    }
}

impl RuntimeInfoDemands {
    /// Resolve an actually observed preparation request before issuing a token.
    /// An incomplete frontier stays in the normal definition readiness domain.
    pub(crate) fn prepare_requested(
        &mut self,
        workspace: WorkspaceId,
        schema: jai_types::RuntimeInfoSchema,
        policy: Option<jai_types::LayoutPolicy>,
        service: RuntimeInfoService<'_>,
    ) -> Result<ReflectionReadiness<RuntimeInfoDemandId>, crate::Diagnostic> {
        let RuntimeInfoService {
            meta,
            types,
            location,
            ..
        } = service;
        let checkpoint = match meta.runtime_info_checkpoint(types, schema, policy, location)? {
            ReflectionReadiness::Ready(checkpoint) => checkpoint,
            ReflectionReadiness::Pending(dependencies) => {
                return Ok(ReflectionReadiness::Pending(dependencies));
            }
        };
        let id = self
            .prepare(workspace, Arc::new(checkpoint))
            .map_err(|error| crate::Diagnostic::at_source(location, error.to_string()))?;
        Ok(ReflectionReadiness::Ready(id))
    }

    /// Called only after the VM provider borrow has ended and the typed demand
    /// was actually observed. Readiness does not reseal the source frontier.
    pub(crate) fn service(
        &mut self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
        service: RuntimeInfoService<'_>,
    ) -> Result<ReflectionReadiness<()>, crate::Diagnostic> {
        let RuntimeInfoService {
            meta,
            types,
            metadata,
            location,
        } = service;
        let checkpoint = self
            .checkpoint(id, workspace)
            .map_err(|error| crate::Diagnostic::at_source(location, error.to_string()))?;
        let snapshot = match meta.runtime_info_snapshot(types, &checkpoint, metadata, location)? {
            ReflectionReadiness::Ready(snapshot) => snapshot,
            ReflectionReadiness::Pending(dependencies) => {
                return Ok(ReflectionReadiness::Pending(dependencies));
            }
        };
        self.publish(id, workspace, &checkpoint, snapshot, types)
            .map_err(|error| crate::Diagnostic::at_source(location, error.to_string()))?;
        Ok(ReflectionReadiness::Ready(()))
    }

    /// Allocate once per drive. Callers retain this key and exact Arc through
    /// suspension; concurrent drives sharing one checkpoint remain independent.
    pub(crate) fn prepare(
        &mut self,
        workspace: WorkspaceId,
        checkpoint: Arc<RuntimeInfoCheckpoint>,
    ) -> Result<RuntimeInfoDemandId, RuntimeInfoDemandError> {
        if self.active.len() >= MAX_ACTIVE_DEMANDS {
            return Err(RuntimeInfoDemandError::Limit);
        }
        let previous = NEXT_DEMAND
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| RuntimeInfoDemandError::IdentityExhausted)?;
        let id = RuntimeInfoDemandId(NonZeroU64::new(previous + 1).expect("checked nonzero nonce"));
        self.active.insert(
            id,
            Demand {
                workspace,
                checkpoint,
                snapshot: None,
            },
        );
        Ok(id)
    }

    /// Clone the receipt, then drop the registry borrow before invoking the
    /// mutable source snapshot factory. Pending factory results retain the key.
    pub(crate) fn checkpoint(
        &self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
    ) -> Result<Arc<RuntimeInfoCheckpoint>, RuntimeInfoDemandError> {
        Ok(Arc::clone(&self.demand(id, workspace)?.checkpoint))
    }

    pub(crate) fn availability(
        &self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
    ) -> Result<RuntimeInfoDemandAvailability<'_>, RuntimeInfoDemandError> {
        let demand = self.demand(id, workspace)?;
        Ok(match &demand.snapshot {
            Some(snapshot) => RuntimeInfoDemandAvailability::Ready {
                workspace,
                snapshot,
            },
            None => RuntimeInfoDemandAvailability::Requested(id),
        })
    }

    pub(crate) fn availability_for(
        &self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
        schema: jai_types::RuntimeInfoSchema,
    ) -> Result<RuntimeInfoDemandAvailability<'_>, RuntimeInfoDemandError> {
        if self.demand(id, workspace)?.checkpoint.schema() != schema {
            return Err(RuntimeInfoDemandError::ForeignSchema);
        }
        self.availability(id, workspace)
    }

    /// Publication verifies the retained receipt after the factory borrow ends;
    /// it never substitutes the latest source catalog for an earlier request.
    pub(crate) fn publish(
        &mut self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
        checkpoint: &Arc<RuntimeInfoCheckpoint>,
        snapshot: Arc<RuntimeInfoSnapshot>,
        types: &dyn TypeView,
    ) -> Result<(), RuntimeInfoDemandError> {
        let demand = self.demand(id, workspace)?;
        if !Arc::ptr_eq(checkpoint, &demand.checkpoint) {
            return Err(RuntimeInfoDemandError::ChangedCheckpoint);
        }
        checkpoint
            .validate_snapshot(&snapshot)
            .map_err(|error| RuntimeInfoDemandError::InvalidSnapshot(error.to_string()))?;
        snapshot
            .validate_owner(types, checkpoint.policy())
            .map_err(|error| RuntimeInfoDemandError::InvalidSnapshot(error.to_string()))?;
        if let Some(previous) = &demand.snapshot {
            return if Arc::ptr_eq(previous, &snapshot) {
                Ok(())
            } else {
                Err(RuntimeInfoDemandError::ChangedSnapshot)
            };
        }
        self.active
            .get_mut(&id)
            .expect("validated active demand")
            .snapshot = Some(snapshot);
        Ok(())
    }

    pub(crate) fn release(
        &mut self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
    ) -> Result<(), RuntimeInfoDemandError> {
        self.demand(id, workspace)?;
        self.active.remove(&id);
        Ok(())
    }

    fn demand(
        &self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
    ) -> Result<&Demand, RuntimeInfoDemandError> {
        let demand = self
            .active
            .get(&id)
            .ok_or(RuntimeInfoDemandError::Retired)?;
        if demand.workspace != workspace {
            return Err(RuntimeInfoDemandError::ForeignWorkspace);
        }
        Ok(demand)
    }
}

#[cfg(test)]
mod tests;
