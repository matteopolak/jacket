//! One source run owns staged policy updates until its publication commits.
use jai_ir::SourceProcedureIdentity;
use jai_types::{RecordReflectionPolicy, RecordReflectionTransaction, TypeId, TypeView};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
mod policy_overlay;
pub(crate) use policy_overlay::{ReflectionPolicyOverlay, ReflectionPolicyRevision};

const MAX_POLICY_CALLS: usize = 1_048_576;
const MAX_POLICY_TARGETS: usize = 16_384;

/// This typed source journal does not serialize arena-local TypeId values into
/// driver requests or replay records. Suspension retains this exact instance.
pub(crate) struct ReflectionPolicyCall {
    pub(crate) record: TypeId,
    pub(crate) flags: RecordReflectionPolicy,
    pub(crate) source: SourceProcedureIdentity,
}

pub(crate) struct ReflectionPolicyJournal {
    source: SourceProcedureIdentity,
    owner: Arc<()>,
    registry: Option<TypeId>,
    policies: HashMap<TypeId, RecordReflectionPolicy>,
    transaction: RecordReflectionTransaction,
    calls: usize,
    retained_cells: usize,
    targets: HashSet<TypeId>,
    provenance: Vec<ReflectionPolicyCall>,
}
impl ReflectionPolicyJournal {
    pub(crate) fn new(source: SourceProcedureIdentity) -> Self {
        Self {
            source,
            owner: Arc::new(()),
            registry: None,
            policies: HashMap::new(),
            transaction: RecordReflectionTransaction::default(),
            calls: 0,
            retained_cells: 0,
            targets: HashSet::new(),
            provenance: Vec::new(),
        }
    }

    pub(crate) fn source(&self) -> &SourceProcedureIdentity {
        &self.source
    }

    #[cfg(test)]
    pub(crate) fn stage(
        &mut self,
        types: &dyn TypeView,
        record: TypeId,
        flags: RecordReflectionPolicy,
    ) -> Result<(), jai_vm::Error> {
        self.stage_at(types, record, flags, self.source.clone())
    }

    fn stage_at(
        &mut self,
        types: &dyn TypeView,
        record: TypeId,
        flags: RecordReflectionPolicy,
        source: SourceProcedureIdentity,
    ) -> Result<(), jai_vm::Error> {
        if self.calls >= MAX_POLICY_CALLS {
            return Err(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells));
        }
        if !self.targets.contains(&record) && self.targets.len() >= MAX_POLICY_TARGETS {
            return Err(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells));
        }
        let accepted = self
            .transaction
            .stage(types, record, flags)
            .map_err(|error| jai_vm::Error::EffectRejected(error.to_string()))?;
        self.policies.insert(record, accepted);
        self.registry = Some(types.scalar(jai_types::ScalarType::Bool));
        self.provenance.push(ReflectionPolicyCall {
            record,
            flags,
            source,
        });
        self.targets.insert(record);
        self.calls += 1;
        Ok(())
    }

    /// The caller supplies the actual VM's remaining root allowance and shared
    /// work meter; arena-local policy facts never become replay arguments.
    pub(crate) fn stage_at_in_run(
        &mut self,
        types: &jai_types::TypeRegistry,
        record: TypeId,
        flags: RecordReflectionPolicy,
        source: SourceProcedureIdentity,
        available_cells: usize,
        charge: &mut impl FnMut(usize) -> Result<(), jai_vm::Error>,
    ) -> Result<(), jai_vm::Error> {
        let cells = 12usize
            .checked_add(if self.targets.contains(&record) {
                0
            } else {
                16
            })
            .ok_or(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells))?;
        let retained = self
            .retained_cells
            .checked_add(cells)
            .filter(|n| *n <= available_cells)
            .ok_or(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells))?;
        charge(cells)?;
        self.stage_at(types, record, flags, source)?;
        self.retained_cells = retained;
        Ok(())
    }

    pub(crate) fn retained_cells(&self) -> usize {
        self.retained_cells
    }

    pub(crate) fn calls(&self) -> &[ReflectionPolicyCall] {
        &self.provenance
    }

    /// Consuming transfer is permitted only to the source publication preflight.
    /// Dropping the journal or its transaction publishes no record policies.
    pub(crate) fn into_transaction(self) -> (RecordReflectionTransaction, SourceProcedureIdentity) {
        (self.transaction, self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, SourceSpan, Span};
    use jai_types::{RecordKind, RecordReflectionFlag, TypeRegistry};

    fn journal() -> ReflectionPolicyJournal {
        let mut sources = SourceMap::default();
        let source = sources.insert("policy-run.jai".into(), "#run {}".into());
        ReflectionPolicyJournal::new(
            SourceProcedureIdentity::new(
                sources.get(source).unwrap(),
                SourceSpan {
                    source,
                    span: Span::new(0, 7),
                },
            )
            .unwrap(),
        )
    }

    #[test]
    fn independent_source_runs_do_not_share_staged_updates() {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, []).unwrap();
        let mut first = journal();
        let second = journal();
        let flags = RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]);
        first.stage(&types, record, flags).unwrap();
        assert_eq!(
            types.record_reflection_policy(record).unwrap(),
            RecordReflectionPolicy::default()
        );
        assert!(second.into_transaction().0.is_empty());
        drop(first);
        assert_eq!(
            types.record_reflection_policy(record).unwrap(),
            RecordReflectionPolicy::default()
        );
    }

    #[test]
    fn rejected_target_and_repeated_calls_preserve_the_actual_arena_and_order() {
        let mut types = TypeRegistry::new();
        let first = types.reserve_record(RecordKind::Struct);
        let second = types.reserve_record(RecordKind::Struct);
        types.define_record(first, []).unwrap();
        types.define_record(second, []).unwrap();
        let mut journal = journal();
        let hidden = RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]);
        journal.stage(&types, second, hidden).unwrap();
        journal.stage(&types, first, hidden).unwrap();
        journal
            .stage(
                &types,
                second,
                RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoSizeComplaint]),
            )
            .unwrap();
        assert!(journal.stage(&TypeRegistry::new(), second, hidden).is_err());
        assert_eq!(journal.calls, 3);
        let (transaction, source) = journal.into_transaction();
        assert_eq!(source.location().span, Span::new(0, 7));
        assert_eq!(
            transaction
                .changes()
                .iter()
                .map(|change| change.record())
                .collect::<Vec<_>>(),
            vec![second, first]
        );
        assert!(
            transaction.changes()[0]
                .after()
                .contains(RecordReflectionFlag::NoSizeComplaint)
        );
    }
    #[test]
    fn reads_observe_accepted_writes_but_prior_revisions_and_registry_remain_immutable() {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, []).unwrap();
        let mut first = journal();
        let before = first.snapshot_policies(128, &mut |_| Ok(())).unwrap();
        first
            .stage(
                &types,
                record,
                RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
            )
            .unwrap();
        let accepted = first.snapshot_policies(128, &mut |_| Ok(())).unwrap();
        assert_eq!(
            before
                .view(&types)
                .unwrap()
                .record_reflection_policy(record)
                .unwrap()
                .bits(),
            0
        );
        assert_eq!(
            accepted
                .view(&types)
                .unwrap()
                .record_reflection_policy(record)
                .unwrap()
                .bits(),
            1
        );
        assert_eq!(types.record_reflection_policy(record).unwrap().bits(), 0);
        assert!(before.revision() != accepted.revision());
        first
            .stage(
                &types,
                record,
                RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoSizeComplaint]),
            )
            .unwrap();
        assert_eq!(
            accepted
                .view(&types)
                .unwrap()
                .record_reflection_policy(record)
                .unwrap()
                .bits(),
            1
        );
        let second = journal();
        let unrelated = second.snapshot_policies(128, &mut |_| Ok(())).unwrap();
        assert!(before.revision() != unrelated.revision());
        assert!(accepted.view(&TypeRegistry::new()).is_err());
        drop(first);
        assert_eq!(types.record_reflection_policy(record).unwrap().bits(), 0);
    }
    #[test]
    fn accepted_noop_has_a_real_read_revision_without_a_policy_commit_change() {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, []).unwrap();
        let mut journal = journal();
        let old = journal.snapshot_policies(64, &mut |_| Ok(())).unwrap();
        journal
            .stage(&types, record, RecordReflectionPolicy::default())
            .unwrap();
        let current = journal.snapshot_policies(64, &mut |_| Ok(())).unwrap();
        assert!(old.revision() != current.revision());
        assert_eq!(
            current
                .view(&types)
                .unwrap()
                .record_reflection_policy(record)
                .unwrap()
                .bits(),
            0
        );
        assert!(journal.into_transaction().0.is_empty());
    }
    #[test]
    fn run_stage_retention_is_cumulative_and_rejection_preserves_the_revision() {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, []).unwrap();
        let mut journal = journal();
        let source = journal.source().clone();
        let mut work = 0usize;
        for _ in 0..2 {
            journal
                .stage_at_in_run(
                    &types,
                    record,
                    RecordReflectionPolicy::default(),
                    source.clone(),
                    40,
                    &mut |n| {
                        work += n;
                        Ok(())
                    },
                )
                .unwrap();
        }
        assert_eq!(journal.retained_cells(), 40);
        let before = journal.snapshot_policies(64, &mut |_| Ok(())).unwrap();
        assert!(
            journal
                .stage_at_in_run(
                    &types,
                    record,
                    RecordReflectionPolicy::default(),
                    source,
                    40,
                    &mut |_| Ok(())
                )
                .is_err()
        );
        let after = journal.snapshot_policies(64, &mut |_| Ok(())).unwrap();
        assert!(before.revision() == after.revision());
        assert_eq!(journal.calls().len(), 2);
        assert_eq!(work, 40);
    }
}
