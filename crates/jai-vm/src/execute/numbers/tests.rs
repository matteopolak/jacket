use super::*;
use jai_types::{CallingConvention, ContextMode, IntegerType, TypeRegistry, Variadic};

#[test]
fn code_address_truth_and_incoming_values_revalidate_the_owning_ledger() {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let opaque_pointer = types.pointer(types.void()).unwrap();
    let id = ProcedureId::new(0);
    let library = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            body: Block {
                statements: vec![],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        }])
        .finish_library()
        .unwrap();
    let mut vm = Vm::new(&library, crate::NoEffects, Limits::default()).unwrap();
    let before = vm.memory.snapshot();
    let callable = Value::Procedure {
        signature,
        procedure: Some(id),
    };
    let cell = vm
        .memory
        .allocate(library.types(), signature, Some(callable))
        .unwrap();
    let view = vm
        .memory
        .cast_pointer(library.types(), &cell, opaque_pointer, CastMode::Checked)
        .unwrap();
    let pointer = vm
        .memory
        .load(library.types(), &view)
        .unwrap()
        .pointer()
        .unwrap()
        .clone();
    vm.memory.release(&cell).unwrap();
    let number = vm
        .memory
        .pointer_to_integer(
            library.types(),
            &pointer,
            IntegerType::U64,
            CastMode::Checked,
        )
        .unwrap();
    assert!(vm.pointer_truth(&pointer).unwrap());
    assert!(vm.number_truth(&number).unwrap());
    assert_eq!(vm.memory.allocation_count(), 0);
    vm.memory.restore(before);
    assert!(matches!(
        vm.pointer_truth(&pointer),
        Err(Halt::Failed(Error::DanglingPointer))
    ));
    assert!(matches!(
        vm.number_truth(&number),
        Err(Halt::Failed(Error::DanglingPointer))
    ));
    for value in [Value::Pointer(pointer), number.into_value()] {
        assert_eq!(
            vm.memory
                .validate_runtime_type_values(library.types(), &value),
            Err(Error::DanglingPointer)
        );
    }
}

fn publication_vm_library() -> Library {
    ProgramBuilder::new(TypeRegistry::new().freeze().unwrap())
        .finish_library()
        .unwrap()
}

#[test]
fn selected_materialization_uses_one_output_quota_for_all_captures() {
    let library = publication_vm_library();
    let vm = Vm::new(&library, crate::NoEffects, Limits::default()).unwrap();
    let first = Value::String(b"abcd".to_vec());
    let second = Value::String(b"efgh".to_vec());
    let narrowed = Limits {
        value_cells: 10,
        ..Limits::default()
    };
    assert!(vm.materialize_borrowed_values(&[&first], narrowed).is_ok());
    assert!(vm.materialize_borrowed_values(&[&second], narrowed).is_ok());
    assert_eq!(
        vm.materialize_borrowed_values(&[&first, &second], narrowed),
        Err(Error::Limit(LimitKind::ValueCells))
    );
}

#[test]
fn immutable_materialization_success_and_failure_share_mutable_vm_fuel() {
    let library = publication_vm_library();
    let mut vm = Vm::new(
        &library,
        crate::NoEffects,
        Limits {
            fuel: 4,
            ..Limits::default()
        },
    )
    .unwrap();
    let value = Value::Bool(true);
    let narrowed = Limits {
        fuel: 2,
        ..Limits::default()
    };
    assert_eq!(
        vm.materialize_borrowed_values(&[&value], narrowed),
        Ok(vec![value.clone()])
    );
    assert_eq!(vm.publication_remaining_fuel(), 2);
    // The outer output slot is admitted, then the value exhausts the narrowed fuel.
    assert_eq!(
        vm.materialize_borrowed_values(
            &[&value],
            Limits {
                fuel: 1,
                ..Limits::default()
            }
        ),
        Err(Error::Limit(LimitKind::Fuel))
    );
    assert_eq!(vm.publication_remaining_fuel(), 1);
    vm.charge_work(1).unwrap();
    assert_eq!(vm.statistics.steps, 4);
    assert_eq!(vm.publication_work.get(), 0);
    assert_eq!(
        vm.materialize_value(&value),
        Err(Error::Limit(LimitKind::Fuel))
    );
    assert!(matches!(
        vm.step(0),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(vm.statistics.steps, 4);
}

#[test]
fn failed_nested_materialization_retains_work_and_obeys_narrow_depth() {
    let library = publication_vm_library();
    let vm = Vm::new(&library, crate::NoEffects, Limits::default()).unwrap();
    let value = Value::Array {
        ty: library.types().scalar(jai_types::ScalarType::Bool),
        elements: vec![Value::Bool(true)],
    };
    let before = vm.publication_remaining_fuel();
    assert_eq!(
        vm.materialize_borrowed_values(
            &[&value],
            Limits {
                evaluation_depth: 0,
                ..Limits::default()
            }
        ),
        Err(Error::Limit(LimitKind::EvaluationDepth))
    );
    assert_eq!(before - vm.publication_remaining_fuel(), 2);
    assert_eq!(
        vm.publication_retained_cells(),
        Err(Error::InvalidIr(
            "publication requires an admitted transaction"
        ))
    );
}

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    /// Tests distinguish actual rollback/inspection overhead from source operation work.
    /// This diagnostic performs borrowed admission only; it copies no VM owner.
    pub(crate) fn test_snapshot_costs(&mut self) -> (usize, u64, u64) {
        assert!(self.continuation.is_none() && self.transaction_value_cell_limit.is_none());
        assert_eq!(self.publication_work.get(), 0);
        let statistics = self.statistics;
        let limits = self.limits;
        self.statistics = Statistics::default();
        self.limits.fuel = u64::MAX;
        self.limits.value_cells = usize::MAX;
        let admission = resumable::publication_transaction_admission(self);
        let entry_work = self.statistics.steps;
        self.statistics = Statistics::default();
        let inspection = resumable::publication_transaction_residual(self);
        let inspection_work = self.statistics.steps;
        self.statistics = statistics;
        self.limits = limits;
        inspection.unwrap();
        (admission.unwrap().0, entry_work, inspection_work)
    }

    pub(crate) fn test_budget_source_work(
        &mut self,
        source: u64,
        validated: bool,
        new_inspection: u64,
    ) -> u64 {
        let (_, entry, inspection) = self.test_snapshot_costs();
        self.limits.fuel = source
            .checked_add(entry)
            .and_then(|total| total.checked_add(if validated { inspection } else { 0 }))
            .and_then(|total| total.checked_add(new_inspection))
            .unwrap();
        entry
    }

    pub(crate) fn test_literal_insert_inspection_work(&self, bytes: usize) -> u64 {
        let capacity = self.literal_backing.capacity();
        let next = if self.literal_backing.len() < capacity {
            capacity
        } else {
            std::collections::HashMap::<usize, usize>::with_capacity(self.literal_backing.len() + 1)
                .capacity()
        };
        u64::try_from(next - capacity + bytes + 1).unwrap()
    }
}
