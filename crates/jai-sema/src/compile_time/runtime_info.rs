//! Private staging for execution-demanded, drive-owned reflection checkpoints.
//!
//! This module is deliberately not registered while the shared VM dependency
//! carrier is frozen. No source name or replay request grants this capability.
use crate::reflection::{RuntimeInfoCheckpoint, RuntimeInfoFrontier};
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
    frontier: Arc<RuntimeInfoFrontier>,
    checkpoint: Option<Arc<RuntimeInfoCheckpoint>>,
    snapshot: Option<Arc<RuntimeInfoSnapshot>>,
}

/// Only an exact, already published demand can create this immutable carrier.
/// Bound providers retain it without consulting current catalog or cache state.
#[derive(Clone)]
pub(crate) struct RuntimeInfoDemandReceipt {
    workspace: WorkspaceId,
    frontier: Arc<RuntimeInfoFrontier>,
    checkpoint: Arc<RuntimeInfoCheckpoint>,
    snapshot: Arc<RuntimeInfoSnapshot>,
}
impl RuntimeInfoDemandReceipt {
    pub(crate) fn workspace(&self) -> WorkspaceId {
        self.workspace
    }
    pub(crate) fn schema(&self) -> jai_types::RuntimeInfoSchema {
        self.frontier.schema()
    }
    pub(crate) fn snapshot(&self) -> &Arc<RuntimeInfoSnapshot> {
        &self.snapshot
    }
    pub(crate) fn validate_owner(
        &self,
        types: &dyn TypeView,
        policy: jai_types::LayoutPolicy,
    ) -> Result<(), RuntimeInfoDemandError> {
        if self.frontier.schema() != self.checkpoint.schema() || policy != self.checkpoint.policy()
        {
            return Err(RuntimeInfoDemandError::ForeignSchema);
        }
        self.checkpoint
            .validate_snapshot(&self.snapshot)
            .map_err(|error| RuntimeInfoDemandError::InvalidSnapshot(error.to_string()))?;
        self.snapshot
            .validate_owner(types, policy)
            .map_err(|error| RuntimeInfoDemandError::InvalidSnapshot(error.to_string()))
    }
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
    PendingCheckpoint,
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
            Self::PendingCheckpoint => f.write_str("runtime-info checkpoint is still pending"),
            Self::ChangedCheckpoint => f.write_str("runtime-info demand checkpoint changed"),
            Self::ChangedSnapshot => f.write_str("runtime-info demand snapshot changed"),
            Self::InvalidSnapshot(error) => {
                write!(f, "invalid runtime-info demand snapshot: {error}")
            }
        }
    }
}

impl RuntimeInfoDemands {
    /// Issue a token on the first actually executed request. The original
    /// source frontier is retained even if nominal definitions are incomplete.
    pub(crate) fn prepare_requested(
        &mut self,
        workspace: WorkspaceId,
        schema: jai_types::RuntimeInfoSchema,
        policy: Option<jai_types::LayoutPolicy>,
        service: RuntimeInfoService<'_>,
    ) -> Result<RuntimeInfoDemandId, crate::Diagnostic> {
        let RuntimeInfoService {
            meta,
            types,
            location,
            ..
        } = service;
        let frontier = meta.runtime_info_frontier(types, schema, policy, location)?;
        self.prepare_frontier(workspace, Arc::new(frontier))
            .map_err(|error| crate::Diagnostic::at_source(location, error.to_string()))
    }

    /// Called only after the VM provider borrow has ended and the typed demand
    /// was actually observed. Readiness does not reseal the source frontier.
    pub(crate) fn prepare_requested_with_overlay(
        &mut self,
        workspace: WorkspaceId,
        schema: jai_types::RuntimeInfoSchema,
        policy: Option<jai_types::LayoutPolicy>,
        overlay: super::reflection_journal::ReflectionPolicyOverlay,
        service: RuntimeInfoService<'_>,
    ) -> Result<RuntimeInfoDemandId, crate::Diagnostic> {
        let RuntimeInfoService {
            meta,
            types,
            location,
            ..
        } = service;
        let frontier =
            meta.runtime_info_frontier_with_overlay(types, schema, policy, overlay, location)?;
        self.prepare_frontier(workspace, Arc::new(frontier))
            .map_err(|error| crate::Diagnostic::at_source(location, error.to_string()))
    }

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
        let demand = self
            .demand(id, workspace)
            .map_err(|error| crate::Diagnostic::at_source(location, error.to_string()))?;
        if let Some(snapshot) = &demand.snapshot {
            let checkpoint = demand
                .checkpoint
                .as_ref()
                .expect("published demand has a sealed checkpoint");
            meta.validate_runtime_info_checkpoint(types, checkpoint, location)?;
            checkpoint
                .validate_snapshot(snapshot)
                .map_err(|error| crate::Diagnostic::at_source(location, error.to_string()))?;
            snapshot
                .validate_owner(types, checkpoint.policy())
                .map_err(|error| crate::Diagnostic::at_source(location, error.to_string()))?;
            return Ok(ReflectionReadiness::Ready(()));
        }
        let checkpoint = match &demand.checkpoint {
            Some(checkpoint) => Arc::clone(checkpoint),
            None => {
                let frontier = Arc::clone(&demand.frontier);
                let checkpoint = match meta
                    .complete_runtime_info_frontier(types, &frontier, metadata, location)?
                {
                    ReflectionReadiness::Ready(checkpoint) => Arc::new(checkpoint),
                    ReflectionReadiness::Pending(dependencies) => {
                        return Ok(ReflectionReadiness::Pending(dependencies));
                    }
                };
                self.active
                    .get_mut(&id)
                    .expect("validated active demand")
                    .checkpoint = Some(Arc::clone(&checkpoint));
                checkpoint
            }
        };
        let snapshot = match meta.runtime_info_snapshot(types, &checkpoint, location)? {
            ReflectionReadiness::Ready(snapshot) => snapshot,
            ReflectionReadiness::Pending(dependencies) => {
                return Ok(ReflectionReadiness::Pending(dependencies));
            }
        };
        self.publish(id, workspace, &checkpoint, snapshot, types)
            .map_err(|error| crate::Diagnostic::at_source(location, error.to_string()))?;
        Ok(ReflectionReadiness::Ready(()))
    }

    /// Seed an already sealed checkpoint. Source execution ordinarily uses
    /// `prepare_requested` so pending frontiers are retained from the start.
    pub(crate) fn prepare(
        &mut self,
        workspace: WorkspaceId,
        checkpoint: Arc<RuntimeInfoCheckpoint>,
    ) -> Result<RuntimeInfoDemandId, RuntimeInfoDemandError> {
        let id = self.prepare_frontier(workspace, Arc::new(checkpoint.frontier()))?;
        self.active
            .get_mut(&id)
            .expect("new active demand")
            .checkpoint = Some(checkpoint);
        Ok(id)
    }

    fn prepare_frontier(
        &mut self,
        workspace: WorkspaceId,
        frontier: Arc<RuntimeInfoFrontier>,
    ) -> Result<RuntimeInfoDemandId, RuntimeInfoDemandError> {
        if self.active.len() >= MAX_ACTIVE_DEMANDS {
            return Err(RuntimeInfoDemandError::Limit);
        }
        let mut previous = NEXT_DEMAND.load(Ordering::Relaxed);
        loop {
            let next = previous
                .checked_add(1)
                .ok_or(RuntimeInfoDemandError::IdentityExhausted)?;
            match NEXT_DEMAND.compare_exchange_weak(
                previous,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current) => previous = current,
            }
        }
        let id = RuntimeInfoDemandId(NonZeroU64::new(previous + 1).expect("checked nonzero nonce"));
        self.active.insert(
            id,
            Demand {
                workspace,
                frontier,
                checkpoint: None,
                snapshot: None,
            },
        );
        Ok(id)
    }

    pub(crate) fn frontier(
        &self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
    ) -> Result<Arc<RuntimeInfoFrontier>, RuntimeInfoDemandError> {
        Ok(Arc::clone(&self.demand(id, workspace)?.frontier))
    }

    /// Clone the receipt, then drop the registry borrow before invoking the
    /// mutable source snapshot factory. Pending factory results retain the key.
    pub(crate) fn checkpoint(
        &self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
    ) -> Result<Arc<RuntimeInfoCheckpoint>, RuntimeInfoDemandError> {
        self.demand(id, workspace)?
            .checkpoint
            .as_ref()
            .map(Arc::clone)
            .ok_or(RuntimeInfoDemandError::PendingCheckpoint)
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

    /// An unreached, pending or failed request grants no ready receipt.
    pub(crate) fn published_receipt(
        &self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
    ) -> Result<Option<RuntimeInfoDemandReceipt>, RuntimeInfoDemandError> {
        let demand = self.demand(id, workspace)?;
        let Some(snapshot) = &demand.snapshot else {
            return Ok(None);
        };
        let checkpoint = demand
            .checkpoint
            .as_ref()
            .ok_or(RuntimeInfoDemandError::PendingCheckpoint)?;
        Ok(Some(RuntimeInfoDemandReceipt {
            workspace,
            frontier: Arc::clone(&demand.frontier),
            checkpoint: Arc::clone(checkpoint),
            snapshot: Arc::clone(snapshot),
        }))
    }

    pub(crate) fn availability_for(
        &self,
        id: RuntimeInfoDemandId,
        workspace: WorkspaceId,
        schema: jai_types::RuntimeInfoSchema,
    ) -> Result<RuntimeInfoDemandAvailability<'_>, RuntimeInfoDemandError> {
        if self.demand(id, workspace)?.frontier.schema() != schema {
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
        if !demand
            .checkpoint
            .as_ref()
            .is_some_and(|retained| Arc::ptr_eq(checkpoint, retained))
        {
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
