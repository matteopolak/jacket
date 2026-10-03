use super::{RecordReflectionFlag, RecordReflectionPolicy};
use crate::{TypeError, TypeId, TypeRegistry, TypeView};
use std::{collections::HashMap, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordReflectionChange {
    record: TypeId,
    before: RecordReflectionPolicy,
    after: RecordReflectionPolicy,
}

impl RecordReflectionChange {
    pub fn record(self) -> TypeId {
        self.record
    }
    pub fn before(self) -> RecordReflectionPolicy {
        self.before
    }
    pub fn after(self) -> RecordReflectionPolicy {
        self.after
    }
}

#[derive(Debug)]
pub struct RecordReflectionCommit {
    changes: Box<[RecordReflectionChange]>,
}

impl RecordReflectionCommit {
    pub fn changes(&self) -> &[RecordReflectionChange] {
        &self.changes
    }
    pub fn len(&self) -> usize {
        self.changes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

pub struct PreparedRecordReflectionTransaction<'a> {
    types: &'a mut TypeRegistry,
    changes: Box<[RecordReflectionChange]>,
}

impl fmt::Debug for PreparedRecordReflectionTransaction<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedRecordReflectionTransaction")
            .field("changes", &self.changes)
            .finish_non_exhaustive()
    }
}

impl PreparedRecordReflectionTransaction<'_> {
    pub fn changes(&self) -> &[RecordReflectionChange] {
        &self.changes
    }

    pub fn apply(self) -> RecordReflectionCommit {
        for change in &self.changes {
            self.types
                .set_record_reflection_policy(change.record, change.after);
        }
        RecordReflectionCommit {
            changes: self.changes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordReflectionTransactionError {
    Type(TypeError),
    Stale {
        record: TypeId,
        expected: RecordReflectionPolicy,
        actual: RecordReflectionPolicy,
    },
}

impl fmt::Display for RecordReflectionTransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => error.fmt(f),
            Self::Stale { .. } => {
                f.write_str("record reflection policy changed while an update was staged")
            }
        }
    }
}

impl std::error::Error for RecordReflectionTransactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Type(error) => Some(error),
            Self::Stale { .. } => None,
        }
    }
}

impl From<TypeError> for RecordReflectionTransactionError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}

#[derive(Debug, Default)]
pub struct RecordReflectionTransaction {
    owner: Option<TypeId>,
    changes: Vec<RecordReflectionChange>,
    positions: HashMap<TypeId, usize>,
}

impl RecordReflectionTransaction {
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn changes(&self) -> &[RecordReflectionChange] {
        &self.changes
    }

    pub fn stage(
        &mut self,
        types: &dyn TypeView,
        record: TypeId,
        flags: RecordReflectionPolicy,
    ) -> Result<RecordReflectionPolicy, RecordReflectionTransactionError> {
        let current = types.record_reflection_policy(record)?;
        if let Some(owner) = self.owner {
            types.kind(owner)?;
        }
        if let Some(&position) = self.positions.get(&record) {
            let change = &mut self.changes[position];
            Self::check_policy(*change, current)?;
            change.after = change.after.union(flags);
            return Ok(change.after);
        }
        let after = current.union(flags);
        if after != current {
            self.positions.insert(record, self.changes.len());
            self.changes.push(RecordReflectionChange {
                record,
                before: current,
                after,
            });
        }
        self.owner.get_or_insert(record);
        Ok(after)
    }

    pub fn validate(&self, types: &dyn TypeView) -> Result<(), RecordReflectionTransactionError> {
        if let Some(owner) = self.owner {
            types.kind(owner)?;
        }
        for &change in &self.changes {
            Self::check_policy(change, types.record_reflection_policy(change.record)?)?;
        }
        Ok(())
    }

    pub fn commit(
        self,
        types: &mut TypeRegistry,
    ) -> Result<RecordReflectionCommit, RecordReflectionTransactionError> {
        Ok(self.prepare(types)?.apply())
    }

    pub fn prepare(
        self,
        types: &mut TypeRegistry,
    ) -> Result<PreparedRecordReflectionTransaction<'_>, RecordReflectionTransactionError> {
        self.validate(types)?;
        if self.changes.iter().any(|change| {
            change
                .after
                .contains(RecordReflectionFlag::ProceduresAreVoidPointers)
        }) {
            types.pointer(types.void())?;
        }
        types.reserve_record_reflection_policy_capacity(self.changes.len());
        Ok(PreparedRecordReflectionTransaction {
            types,
            changes: self.changes.into_boxed_slice(),
        })
    }

    fn check_policy(
        change: RecordReflectionChange,
        actual: RecordReflectionPolicy,
    ) -> Result<(), RecordReflectionTransactionError> {
        if actual != change.before {
            return Err(RecordReflectionTransactionError::Stale {
                record: change.record,
                expected: change.before,
                actual,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RecordKind, TypeKind};

    fn flags(flag: RecordReflectionFlag) -> RecordReflectionPolicy {
        RecordReflectionPolicy::from_flags([flag])
    }

    #[test]
    fn prepared_updates_apply_without_a_second_validation_or_allocation_phase() {
        let mut types = TypeRegistry::new();
        let first = types.reserve_record(RecordKind::Struct);
        let second = types.reserve_record(RecordKind::Union);
        let hidden = flags(RecordReflectionFlag::NoTypeInfo);
        let pointers = flags(RecordReflectionFlag::ProceduresAreVoidPointers);
        let mut transaction = RecordReflectionTransaction::default();
        transaction.stage(&types, first, hidden).unwrap();
        transaction.stage(&types, second, pointers).unwrap();
        let prepared = transaction.prepare(&mut types).unwrap();
        assert_eq!(prepared.changes().len(), 2);
        let receipt: RecordReflectionCommit = prepared.apply();
        assert_eq!(receipt.changes()[0].record(), first);
        assert_eq!(receipt.changes()[1].record(), second);
        assert_eq!(types.record_reflection_policy(first).unwrap(), hidden);
        assert_eq!(types.record_reflection_policy(second).unwrap(), pointers);
        assert!(types.lookup(&TypeKind::Pointer(types.void())).is_some());
    }

    #[test]
    fn cancelling_prepared_updates_keeps_policies_and_source_schema_unchanged() {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        let flags = flags(RecordReflectionFlag::ProceduresAreVoidPointers);
        let mut transaction = RecordReflectionTransaction::default();
        transaction.stage(&types, record, flags).unwrap();
        let prepared = transaction.prepare(&mut types).unwrap();
        assert_eq!(prepared.changes()[0].after(), flags);
        drop(prepared);
        assert_eq!(
            types.record_reflection_policy(record).unwrap(),
            RecordReflectionPolicy::default()
        );
        assert!(types.runtime_type_header().is_none());
    }

    #[test]
    fn stale_preparation_rejects_every_policy_before_auxiliary_interning() {
        let mut types = TypeRegistry::new();
        let first = types.reserve_record(RecordKind::Struct);
        let second = types.reserve_record(RecordKind::Union);
        let pointers = flags(RecordReflectionFlag::ProceduresAreVoidPointers);
        let mut transaction = RecordReflectionTransaction::default();
        transaction.stage(&types, first, pointers).unwrap();
        transaction.stage(&types, second, pointers).unwrap();
        types
            .add_record_reflection_flags(second, flags(RecordReflectionFlag::NoSizeComplaint))
            .unwrap();
        assert!(matches!(
            transaction.prepare(&mut types),
            Err(RecordReflectionTransactionError::Stale { record, .. }) if record == second
        ));
        assert_eq!(
            types.record_reflection_policy(first).unwrap(),
            RecordReflectionPolicy::default()
        );
        assert!(types.lookup(&TypeKind::Pointer(types.void())).is_none());
    }

    #[test]
    fn accumulated_updates_commit_once_and_preserve_first_target_order() {
        let mut types = TypeRegistry::new();
        let first = types.reserve_record(RecordKind::Struct);
        let second = types.reserve_record(RecordKind::Union);
        let hidden = flags(RecordReflectionFlag::NoTypeInfo);
        let pointers = flags(RecordReflectionFlag::ProceduresAreVoidPointers);
        let mut transaction = RecordReflectionTransaction::default();
        transaction.stage(&types, first, hidden).unwrap();
        transaction.stage(&types, second, pointers).unwrap();
        assert_eq!(
            transaction.stage(&types, first, pointers).unwrap(),
            hidden.union(pointers)
        );
        assert_eq!(transaction.len(), 2);
        assert_eq!(
            types.record_reflection_policy(first).unwrap(),
            RecordReflectionPolicy::default()
        );
        assert!(types.lookup(&TypeKind::Pointer(types.void())).is_none());
        let receipts = transaction.commit(&mut types).unwrap();
        let receipts = receipts.changes();
        assert_eq!(receipts[0].record(), first);
        assert_eq!(receipts[1].record(), second);
        assert_eq!(receipts[0].before(), RecordReflectionPolicy::default());
        assert_eq!(receipts[0].after(), hidden.union(pointers));
        assert_eq!(
            types.record_reflection_policy(first).unwrap(),
            receipts[0].after()
        );
        assert_eq!(types.record_reflection_policy(second).unwrap(), pointers);
        assert!(types.lookup(&TypeKind::Pointer(types.void())).is_some());
        types.define_record(first, vec![]).unwrap();
        types.define_record(second, vec![]).unwrap();
        let frozen = types.freeze().unwrap();
        assert_eq!(
            frozen.record_reflection_policy(first).unwrap(),
            receipts[0].after()
        );
    }

    #[test]
    fn stale_later_target_rejects_all_changes_before_pointer_interning() {
        let mut types = TypeRegistry::new();
        let first = types.reserve_record(RecordKind::Struct);
        let second = types.reserve_record(RecordKind::Struct);
        let hidden = flags(RecordReflectionFlag::NoTypeInfo);
        let mut transaction = RecordReflectionTransaction::default();
        transaction
            .stage(
                &types,
                first,
                flags(RecordReflectionFlag::ProceduresAreVoidPointers),
            )
            .unwrap();
        transaction.stage(&types, second, hidden).unwrap();
        types
            .add_record_reflection_flags(second, flags(RecordReflectionFlag::NoSizeComplaint))
            .unwrap();
        assert!(
            matches!(transaction.commit(&mut types), Err(RecordReflectionTransactionError::Stale { record, .. }) if record == second)
        );
        assert_eq!(
            types.record_reflection_policy(first).unwrap(),
            RecordReflectionPolicy::default()
        );
        assert!(types.lookup(&TypeKind::Pointer(types.void())).is_none());
        assert_eq!(
            types.record_reflection_policy(second).unwrap(),
            flags(RecordReflectionFlag::NoSizeComplaint)
        );
    }

    #[test]
    fn foreign_commit_and_invalid_stages_leave_both_arenas_unchanged() {
        let mut owner = TypeRegistry::new();
        let record = owner.reserve_record(RecordKind::Struct);
        let mut other = TypeRegistry::new();
        let foreign = other.reserve_record(RecordKind::Struct);
        let hidden = flags(RecordReflectionFlag::NoTypeInfo);
        let mut transaction = RecordReflectionTransaction::default();
        transaction.stage(&owner, record, hidden).unwrap();
        assert!(matches!(
            transaction.stage(&owner, foreign, hidden),
            Err(RecordReflectionTransactionError::Type(
                TypeError::ForeignType(_)
            ))
        ));
        assert!(matches!(
            transaction.stage(&owner, owner.void(), hidden),
            Err(RecordReflectionTransactionError::Type(
                TypeError::WrongKind(_)
            ))
        ));
        assert_eq!(transaction.len(), 1);
        assert!(matches!(
            transaction.commit(&mut other),
            Err(RecordReflectionTransactionError::Type(
                TypeError::ForeignType(_)
            ))
        ));
        assert_eq!(
            owner.record_reflection_policy(record).unwrap(),
            RecordReflectionPolicy::default()
        );
        assert_eq!(
            other.record_reflection_policy(foreign).unwrap(),
            RecordReflectionPolicy::default()
        );
    }

    #[test]
    fn stale_restage_is_atomic_and_cancellation_has_no_registry_effect() {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        let hidden = flags(RecordReflectionFlag::NoTypeInfo);
        let mut transaction = RecordReflectionTransaction::default();
        transaction.stage(&types, record, hidden).unwrap();
        types
            .add_record_reflection_flags(record, flags(RecordReflectionFlag::NoSizeComplaint))
            .unwrap();
        assert!(matches!(
            transaction.stage(
                &types,
                record,
                flags(RecordReflectionFlag::ProceduresAreVoidPointers)
            ),
            Err(RecordReflectionTransactionError::Stale { .. })
        ));
        assert_eq!(transaction.changes()[0].after(), hidden);
        drop(transaction);
        assert_eq!(
            types.record_reflection_policy(record).unwrap(),
            flags(RecordReflectionFlag::NoSizeComplaint)
        );
        assert!(types.lookup(&TypeKind::Pointer(types.void())).is_none());
    }

    #[test]
    fn no_op_updates_produce_no_receipt_but_still_validate_the_target() {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        let hidden = flags(RecordReflectionFlag::NoTypeInfo);
        types.add_record_reflection_flags(record, hidden).unwrap();
        let mut transaction = RecordReflectionTransaction::default();
        assert_eq!(transaction.stage(&types, record, hidden).unwrap(), hidden);
        assert!(
            transaction
                .stage(&types, types.void(), RecordReflectionPolicy::default())
                .is_err()
        );
        assert!(transaction.is_empty());
        assert!(transaction.commit(&mut types).unwrap().is_empty());
        assert_eq!(types.record_reflection_policy(record).unwrap(), hidden);
    }

    #[test]
    fn locally_valid_targets_cannot_mix_compilation_arenas() {
        let mut owner = TypeRegistry::new();
        let record = owner.reserve_record(RecordKind::Struct);
        let mut other = TypeRegistry::new();
        let foreign = other.reserve_record(RecordKind::Struct);
        let hidden = flags(RecordReflectionFlag::NoTypeInfo);
        let mut transaction = RecordReflectionTransaction::default();
        transaction.stage(&owner, record, hidden).unwrap();
        assert!(matches!(
            transaction.stage(&other, foreign, hidden),
            Err(RecordReflectionTransactionError::Type(TypeError::ForeignType(id))) if id == record
        ));
        assert_eq!(transaction.len(), 1);
        transaction.commit(&mut owner).unwrap();
        assert_eq!(owner.record_reflection_policy(record).unwrap(), hidden);
        assert_eq!(
            other.record_reflection_policy(foreign).unwrap(),
            RecordReflectionPolicy::default()
        );
    }

    #[test]
    fn no_op_transaction_retains_its_canonical_owner() {
        let mut owner = TypeRegistry::new();
        let record = owner.reserve_record(RecordKind::Struct);
        let mut other = TypeRegistry::new();
        let mut transaction = RecordReflectionTransaction::default();
        transaction
            .stage(&owner, record, RecordReflectionPolicy::default())
            .unwrap();
        assert!(transaction.is_empty());
        assert!(
            matches!(transaction.commit(&mut other), Err(RecordReflectionTransactionError::Type(TypeError::ForeignType(id))) if id == record)
        );
        assert_eq!(
            owner.record_reflection_policy(record).unwrap(),
            RecordReflectionPolicy::default()
        );
    }
}
