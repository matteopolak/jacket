//! A typed policy revision belongs to one actual source journal.
use super::*;
use jai_types::*;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct ReflectionPolicyRevision {
    owner: Arc<()>,
    revision: u64,
}
impl PartialEq for ReflectionPolicyRevision {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner) && self.revision == other.revision
    }
}
impl Eq for ReflectionPolicyRevision {
}
impl Hash for ReflectionPolicyRevision {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.owner).hash(state);
        self.revision.hash(state);
    }
}

/// Owns the exact accepted writes visible at a reached reflection read. Later
/// setters cannot change this retained map or reseal an earlier pending demand.
#[derive(Clone)]
pub(crate) struct ReflectionPolicyOverlay {
    revision: ReflectionPolicyRevision,
    registry: Option<TypeId>,
    policies: Arc<HashMap<TypeId, RecordReflectionPolicy>>,
}
impl ReflectionPolicyOverlay {
    pub(crate) fn revision(&self) -> &ReflectionPolicyRevision {
        &self.revision
    }
    pub(crate) fn view<'a>(
        &'a self,
        types: &'a dyn TypeView,
    ) -> Result<OverlayTypes<'a>, TypeError> {
        if self
            .registry
            .is_some_and(|owner| owner != types.scalar(ScalarType::Bool))
        {
            return Err(TypeError::WrongKind(self.registry.unwrap()));
        }
        Ok(OverlayTypes {
            types,
            policies: &self.policies,
        })
    }
}

pub(crate) struct OverlayTypes<'a> {
    types: &'a dyn TypeView,
    policies: &'a HashMap<TypeId, RecordReflectionPolicy>,
}
impl TypeView for OverlayTypes<'_> {
    fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        self.types.kind(id)
    }
    fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        self.types.lookup(kind)
    }
    fn lookup_procedure(&self, signature: &ProcedureType) -> Option<TypeId> {
        self.types.lookup_procedure(signature)
    }
    fn scalar(&self, ty: ScalarType) -> TypeId {
        self.types.scalar(ty)
    }
    fn float(&self, ty: FloatType) -> TypeId {
        self.types.float(ty)
    }
    fn any_type(&self) -> Option<TypeId> {
        self.types.any_type()
    }
    fn runtime_type_header(&self) -> Option<TypeId> {
        self.types.runtime_type_header()
    }
    fn allocator_schema(&self) -> Option<AllocatorSchema> {
        self.types.allocator_schema()
    }
    fn record_reflection_policy(
        &self,
        record: TypeId,
    ) -> Result<RecordReflectionPolicy, TypeError> {
        let current = self.types.record_reflection_policy(record)?;
        Ok(self.policies.get(&record).copied().unwrap_or(current))
    }
    fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError> {
        self.types.record(id)
    }
    fn enumeration(&self, id: EnumId) -> Result<&EnumDefinition, TypeError> {
        self.types.enumeration(id)
    }
    fn distinct(&self, id: DistinctId) -> Result<&DistinctDefinition, TypeError> {
        self.types.distinct(id)
    }
    fn procedure_type(&self, id: ProcedureTypeId) -> Result<&ProcedureType, TypeError> {
        self.types.procedure_type(id)
    }
    fn record_type(&self, id: RecordId) -> Result<TypeId, TypeError> {
        self.types.record_type(id)
    }
}

impl ReflectionPolicyJournal {
    pub(crate) fn snapshot_policies(
        &self,
        available_cells: usize,
        charge: &mut impl FnMut(usize) -> Result<(), jai_vm::Error>,
    ) -> Result<ReflectionPolicyOverlay, jai_vm::Error> {
        let cells = self
            .policies
            .len()
            .checked_mul(4)
            .and_then(|n| n.checked_add(4))
            .filter(|n| *n <= available_cells)
            .ok_or(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells))?;
        // One immutable map and its revision are admitted before any copy.
        charge(cells)?;
        Ok(ReflectionPolicyOverlay {
            revision: ReflectionPolicyRevision {
                owner: Arc::clone(&self.owner),
                revision: self.calls as u64,
            },
            registry: self.registry,
            policies: Arc::new(self.policies.clone()),
        })
    }
}
