//! Snapshot preflight bounds syntax-owned trees without resolving unused types.
use crate::*;
use jai_ir::{
    ConstantKind, ConstantValue, ContextDefinition, Global, GlobalInitializer, ProgramBuilder,
    ValueExpr,
};
use jai_types::{Integer, IntegerType, RecordKind, ScalarType, TypeId, TypeRegistry, TypeView};

struct Provider {
    types: TypeRegistry,
    globals: Vec<Global>,
    context: Option<ContextDefinition>,
    signatures: std::collections::HashMap<jai_ir::ProcedureId, TypeId>,
    alignments: jai_ir::StorageAlignments,
    pending_alignment: std::cell::Cell<Option<jai_ir::GlobalId>>,
}
impl Provider {
    fn new() -> Self {
        Self {
            types: TypeRegistry::new(),
            globals: vec![],
            context: None,
            signatures: Default::default(),
            alignments: Default::default(),
            pending_alignment: std::cell::Cell::new(None),
        }
    }
    fn append(&mut self, value: ConstantValue) {
        self.globals.push(Global::new(
            self.globals.len(),
            GlobalInitializer::Value(value),
            &self.types,
        ));
    }
}
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn procedure(&self, _: jai_ir::ProcedureId) -> ProcedureAvailability<'_> {
        ProcedureAvailability::Missing
    }
    fn globals(&self) -> &[Global] {
        &self.globals
    }
    fn context(&self) -> Option<&ContextDefinition> {
        self.context.as_ref()
    }
    fn signatures(&self) -> &std::collections::HashMap<jai_ir::ProcedureId, TypeId> {
        &self.signatures
    }
    fn storage_alignments(&self) -> Option<&jai_ir::StorageAlignments> {
        Some(&self.alignments)
    }
    fn global_alignment_pending(&self, id: jai_ir::GlobalId) -> bool {
        self.pending_alignment.get() == Some(id)
    }
}

#[test]
fn pending_global_shapes_still_require_exact_procedure_constant_identity() {
    let mut provider = Provider::new();
    let pending = provider.types.reserve_record(RecordKind::Struct);
    let signature = provider
        .types
        .procedure(jai_types::ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            context: jai_types::ContextMode::None,
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: jai_types::CallingConvention::Jai,
            variadic: jai_types::Variadic::None,
        })
        .unwrap();
    let id = jai_ir::ProcedureId::new(41);
    provider.append(ConstantValue {
        ty: pending,
        kind: ConstantKind::Record(vec![ConstantValue {
            ty: signature,
            kind: ConstantKind::Procedure(id),
        }]),
    });
    assert!(matches!(
        Vm::new(&provider, NoEffects, Limits::default()),
        Err(Error::IrValidation(_))
    ));
    provider.signatures.insert(id, signature);
    let state = Vm::new(&provider, NoEffects, Limits::default())
        .unwrap()
        .into_state();
    provider.signatures.remove(&id);
    assert!(matches!(
        Vm::with_state(&provider, NoEffects, Limits::default(), state),
        Err(Error::IrValidation(_))
    ));
}

#[test]
fn state_transfer_requires_compatible_existing_global_alignment() {
    let mut provider = Provider::new();
    let integer = provider.types.scalar(ScalarType::Int(IntegerType::S64));
    provider.append(zero(integer));
    let id = provider.globals[0].id();
    provider.alignments.set_global(id, 64).unwrap();
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.evaluate(&ValueExpr::Load(provider.globals[0].place()))
            .outcome,
        Outcome::Complete(_)
    ));
    let state = vm.into_state();
    provider.alignments.set_global(id, 32).unwrap();
    let state = Vm::with_state(&provider, NoEffects, Limits::default(), state)
        .unwrap()
        .into_state();
    provider.alignments.set_global(id, 128).unwrap();
    assert!(matches!(
        Vm::with_state(&provider, NoEffects, Limits::default(), state),
        Err(Error::InvalidIr(
            "VM state global storage alignment changed"
        ))
    ));
}

#[test]
fn demanded_pending_global_alignment_precedes_allocation_and_retries_with_ready_storage() {
    let mut provider = Provider::new();
    let integer = provider.types.scalar(ScalarType::Int(IntegerType::S64));
    provider.append(zero(integer));
    let global = &provider.globals[0];
    let id = global.id();
    provider.alignments.set_global(id, 128).unwrap();
    provider.pending_alignment.set(Some(id));
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.evaluate(&ValueExpr::Load(global.place())).outcome,
        Outcome::Pending(vec![Dependency::GlobalAlignment(id)])
    );
    assert_eq!(vm.memory().allocation_count(), 0);
    assert_eq!(vm.memory().value_cells(), 0);
    assert!(matches!(
        vm.evaluate(&ValueExpr::Int(jai_ir::IntExpr::constant(
            Integer::wrapping(IntegerType::S64, 1)
        )))
        .outcome,
        Outcome::Complete(_)
    ));
    provider.pending_alignment.set(None);
    assert_eq!(
        vm.evaluate(&ValueExpr::Load(global.place())).outcome,
        Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::S64, 0))])
    );
    assert_eq!(vm.memory().allocation_count(), 1);
}
impl Drop for Provider {
    fn drop(&mut self) {
        // ProgramBuilder owns iterative disposal even for rejected staging trees.
        // Its empty frozen registry is sufficient because teardown does not query types.
        let mut disposal = ProgramBuilder::new(TypeRegistry::new().freeze().unwrap())
            .globals(std::mem::take(&mut self.globals));
        if let Some(context) = self.context.take() {
            disposal = disposal.context(context);
        }
        drop(disposal);
    }
}
fn zero(ty: TypeId) -> ConstantValue {
    ConstantValue {
        ty,
        kind: ConstantKind::Zero,
    }
}
fn deep_tree(ty: TypeId, depth: usize) -> ConstantValue {
    let mut value = zero(ty);
    for _ in 0..depth {
        value = ConstantValue {
            ty,
            kind: ConstantKind::Distinct(Box::new(value)),
        };
    }
    value
}
fn limits(cells: usize) -> Limits {
    Limits {
        value_cells: cells,
        ..Limits::default()
    }
}

#[test]
fn borrowed_deep_unused_global_is_rejected_before_snapshot_clone() {
    let mut provider = Provider::new();
    let integer = provider.types.scalar(ScalarType::Int(IntegerType::S64));
    provider.append(deep_tree(integer, 20_000));
    let limits = Limits {
        evaluation_depth: usize::MAX,
        ..Limits::default()
    };
    assert_eq!(
        crate::constants::preflight_provider_constants(&provider.globals, None, limits),
        Err(Error::Limit(LimitKind::EvaluationDepth)),
    );
    assert!(matches!(
        Vm::new(&provider, NoEffects, limits),
        Err(Error::Limit(LimitKind::EvaluationDepth))
    ));
}

#[test]
fn resumed_provider_is_preflighted_before_recursive_definition_comparison() {
    let mut provider = Provider::new();
    let state = Vm::new(&provider, NoEffects, Limits::default())
        .unwrap()
        .into_state();
    let integer = provider.types.scalar(ScalarType::Int(IntegerType::S64));
    provider.append(deep_tree(integer, 20_000));
    assert!(matches!(
        Vm::with_state(&provider, NoEffects, Limits::default(), state),
        Err(Error::Limit(LimitKind::EvaluationDepth)),
    ));
}

#[test]
fn unused_pending_nominal_global_survives_state_transfer_until_read() {
    let mut provider = Provider::new();
    let pending = provider.types.reserve_record(RecordKind::Struct);
    provider.append(zero(pending));
    let place = provider.globals[0].place();
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    let one = ValueExpr::Int(jai_ir::IntExpr::constant(Integer::wrapping(
        IntegerType::S64,
        1,
    )));
    assert_eq!(
        vm.evaluate(&one).outcome,
        Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::S64, 1))])
    );
    let state = vm.into_state();
    let mut resumed = Vm::with_state(&provider, NoEffects, Limits::default(), state).unwrap();
    assert_eq!(
        resumed.evaluate(&one).outcome,
        Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::S64, 1))])
    );
    assert_eq!(
        resumed.evaluate(&ValueExpr::Load(place)).outcome,
        Outcome::Pending(vec![Dependency::Type(pending)])
    );
}

#[test]
fn string_bytes_and_global_roots_share_one_snapshot_budget() {
    let mut provider = Provider::new();
    provider.append(ConstantValue {
        ty: provider.types.string(),
        kind: ConstantKind::StringBytes(b"abc".to_vec()),
    });
    provider.globals.push(Global::new(
        1,
        GlobalInitializer::Bool(false),
        &provider.types,
    ));
    assert_eq!(
        crate::constants::preflight_provider_constants(&provider.globals, None, limits(5)),
        Ok(())
    );
    assert_eq!(
        crate::constants::preflight_provider_constants(&provider.globals, None, limits(4)),
        Err(Error::Limit(LimitKind::ValueCells))
    );
}

#[test]
fn context_default_is_preflighted_with_globals_without_resolving_types() {
    let mut provider = Provider::new();
    let record = provider.types.reserve_record(RecordKind::Struct);
    let pointer = provider.types.pointer(record).unwrap();
    provider.context = Some(ContextDefinition {
        record_type: record,
        pointer_type: pointer,
        default: zero(record),
    });
    provider.globals.push(Global::new(
        0,
        GlobalInitializer::Bool(false),
        &provider.types,
    ));
    assert_eq!(
        crate::constants::preflight_provider_constants(
            &provider.globals,
            provider.context.as_ref(),
            limits(2)
        ),
        Ok(())
    );
    assert_eq!(
        crate::constants::preflight_provider_constants(
            &provider.globals,
            provider.context.as_ref(),
            limits(1)
        ),
        Err(Error::Limit(LimitKind::ValueCells))
    );
}

#[test]
fn wide_children_are_reserved_before_work_queue_growth() {
    let mut provider = Provider::new();
    let integer = provider.types.scalar(ScalarType::Int(IntegerType::S64));
    provider.append(ConstantValue {
        ty: integer,
        kind: ConstantKind::Array((0..1000).map(|_| zero(integer)).collect()),
    });
    assert_eq!(
        crate::constants::preflight_provider_constants(&provider.globals, None, limits(1000)),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(
        crate::constants::preflight_provider_constants(&provider.globals, None, limits(1001)),
        Ok(())
    );
}
