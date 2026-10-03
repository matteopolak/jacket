//! Ordinary publication reserves rollback owners and restores its temporary quota.
use super::*;
use crate::{NoEffects, SourceOrigin, WorkspaceId};
use jai_types::TypeRegistry;

struct Provider(TypeRegistry);
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.0
    }
    fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
        ProcedureAvailability::Missing
    }
}
fn origin() -> SourceOrigin {
    SourceOrigin {
        workspace: WorkspaceId::from_raw(1).unwrap(),
        path: "capture-admission.jai".into(),
        start: 0,
        end: 4,
        body_hash: 0,
        body: b"#run".to_vec(),
        specialization: vec![],
    }
}

#[test]
fn ordinary_admission_rejects_before_action_and_snapshot_retention() {
    let provider = Provider(TypeRegistry::new());
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    let byte = provider
        .0
        .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8));
    vm.prepare_layout(byte).unwrap();
    vm.memory.allocate(&provider.0, byte, None).unwrap();
    let (rollback, _) = resumable::publication_transaction_admission(&mut vm).unwrap();
    let quota = rollback.checked_mul(2).unwrap() - 1;
    vm.limits.value_cells = quota;
    vm.memory.replace_value_cell_limit(quota).unwrap();
    let allocations = vm.memory.allocation_count();
    let mut called = false;
    let result = vm.transaction(|_| {
        called = true;
        Ok(vec![])
    });
    assert_eq!(
        result.outcome,
        Outcome::Failed(Error::Limit(LimitKind::ValueCells))
    );
    assert!(!called);
    assert_eq!(vm.memory.allocation_count(), allocations);
    assert_eq!(vm.memory.value_cell_limit(), quota);
    assert_eq!(vm.transaction_value_cell_limit, None);
}

#[test]
fn ordinary_refresh_accounts_for_literal_pool_growth_before_validation() {
    let mut types = TypeRegistry::new();
    types.string();
    let provider = Provider(types);
    let limits = Limits::default();
    let mut vm = Vm::new(&provider, NoEffects, limits).unwrap();
    vm.pin_publication_source_origin(origin()).unwrap();
    let result = vm.transaction(|vm| {
        let before = vm.transaction_ancillary_cells;
        let value = vm.string_descriptor(vec![7; 128], 0)?;
        vm.refresh_publication_transaction()?;
        assert!(vm.transaction_ancillary_cells > before);
        assert!(vm.publication_retained_cells()? <= limits.value_cells);
        assert!(vm.publication_source_origin().is_some());
        Ok(vec![value])
    });
    assert!(matches!(result.outcome, Outcome::Complete(_)));
    assert_eq!(vm.memory.value_cell_limit(), limits.value_cells);
    assert_eq!(vm.limits, limits);
    assert_eq!(vm.transaction_value_cell_limit, None);
    assert!(vm.publication_source_origin().is_none());
}

#[test]
fn ordinary_failure_restores_memory_quota_and_clears_pinned_origin() {
    let mut types = TypeRegistry::new();
    types.string();
    let provider = Provider(types);
    let limits = Limits::default();
    let mut vm = Vm::new(&provider, NoEffects, limits).unwrap();
    vm.pin_publication_source_origin(origin()).unwrap();
    let allocations = vm.memory.allocation_count();
    let result = vm.transaction(|vm| {
        vm.string_descriptor(vec![9; 64], 0)?;
        vm.charge_publication_work(11)?;
        Err(Error::InvalidIr("reject selected publication").into())
    });
    assert_eq!(
        result.outcome,
        Outcome::Failed(Error::InvalidIr("reject selected publication"))
    );
    assert!(result.statistics.steps >= 11);
    assert_eq!(vm.publication_work.get(), 0);
    assert_eq!(vm.memory.allocation_count(), allocations);
    assert!(vm.literal_backing.is_empty());
    assert_eq!(vm.memory.value_cell_limit(), limits.value_cells);
    assert_eq!(vm.limits, limits);
    assert_eq!(vm.transaction_retained_cells, 0);
    assert_eq!(vm.transaction_ancillary_cells, 0);
    assert!(vm.publication_source_origin().is_none());
}

#[test]
fn pinned_source_preparation_remains_charged_through_publication_and_failure() {
    let provider = Provider(TypeRegistry::new());
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    for accepted in [true, false] {
        vm.pin_publication_source_origin(origin()).unwrap();
        let first = vm.statistics.steps;
        assert!(first > 0);
        vm.pin_publication_source_origin(origin()).unwrap();
        let preparation = vm.statistics.steps;
        assert!(preparation > first);
        let result = vm.transaction(|vm| {
            vm.charge_publication_work(13)?;
            if accepted {
                Ok(vec![])
            } else {
                Err(Error::InvalidIr("reject pinned publication").into())
            }
        });
        assert!(result.statistics.steps >= preparation + 13);
        assert_eq!(matches!(result.outcome, Outcome::Complete(_)), accepted);
        assert!(vm.publication_source_origin().is_none());
        assert_eq!(vm.publication_work.get(), 0);
    }
}
