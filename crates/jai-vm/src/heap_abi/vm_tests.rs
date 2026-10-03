use super::*;
use crate::{
    CompilerEffects, CompilerRequest, EffectOutcome, Limits, NoEffects, Outcome,
    ProcedureAvailability, ProcedureProvider, Vm,
};
use jai_ir::{IntExpr, ParameterId, ValueExpr};
use jai_types::{Integer, ProcedureType, ScalarType, TypeRegistry};
fn malloc() -> ProcedureId {
    ProcedureId::new(1)
}
fn realloc() -> ProcedureId {
    ProcedureId::new(2)
}
fn free() -> ProcedureId {
    ProcedureId::new(3)
}
struct Provider {
    types: TypeRegistry,
    signatures: HashMap<ProcedureId, TypeId>,
    bound: HashMap<ProcedureId, HeapAbiProcedure>,
}
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        &self.signatures
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        self.bound.get(&id).copied().map_or(
            ProcedureAvailability::Missing,
            ProcedureAvailability::HeapAbi,
        )
    }
}
fn provider() -> Provider {
    let mut types = TypeRegistry::new();
    let pointer = types.pointer(types.void()).unwrap();
    let size = types.scalar(ScalarType::Int(IntegerType::U64));
    let library = ForeignLibrary {
        id: ForeignLibraryId::new(jai_source::Identities::default().declaration()),
        kind: ForeignLibraryKind::System {
            name: "libc".into(),
        },
        options: Default::default(),
    };
    let operations = [
        (malloc(), HeapAbiOperation::Malloc),
        (realloc(), HeapAbiOperation::Realloc),
        (free(), HeapAbiOperation::Free),
    ];
    let authority = HeapAuthority::from_verified_source(library.clone(), operations).unwrap();
    let mut signatures = HashMap::new();
    let mut bound = HashMap::new();
    for (id, operation) in operations {
        let (parameters, results) = match operation {
            HeapAbiOperation::Malloc => (vec![size], vec![pointer]),
            HeapAbiOperation::Realloc => (vec![pointer, size], vec![pointer]),
            HeapAbiOperation::Free => (vec![pointer], vec![]),
        };
        let signature = types
            .procedure(ProcedureType {
                parameters: parameters.into(),
                results: results.into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let prototype = ProcedurePrototype {
            id,
            signature,
            origin: PrototypeOrigin::Foreign {
                symbol: operation.symbol().into(),
                library: Some(library.clone()),
            },
        };
        signatures.insert(id, signature);
        bound.insert(id, authority.bind(&prototype, &types).unwrap());
    }
    Provider {
        types,
        signatures,
        bound,
    }
}
fn size(bytes: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::U64, bytes))
}
fn complete(outcome: Outcome) -> Vec<Value> {
    match outcome {
        Outcome::Complete(values) => values,
        other => panic!("expected complete, got {other:?}"),
    }
}
fn byte_view(provider: &Provider, memory: &Memory, value: &Value) -> crate::Pointer {
    memory
        .cast_pointer(
            &provider.types,
            value.pointer().unwrap(),
            provider.types.scalar(ScalarType::Int(IntegerType::U8)),
            jai_types::CastMode::Checked,
        )
        .unwrap()
}
#[test]
fn root_direct_indirect_and_transferred_state_share_real_heap_ownership() {
    let provider = provider();
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    let first = complete(vm.execute(malloc(), vec![size(8)]).outcome).remove(0);
    let byte = byte_view(&provider, vm.memory(), &first);
    vm.memory_mut()
        .byte_set(&provider.types, &byte, 42, 8)
        .unwrap();
    let pointer_type = provider
        .types
        .procedure_definition(provider.signatures[&malloc()])
        .unwrap()
        .results[0];
    let direct = ValueExpr::Call {
        ty: pointer_type,
        call: jai_ir::Call::new(
            malloc(),
            vec![(
                ParameterId::new(0),
                ValueExpr::Int(IntExpr::constant(Integer::wrapping(IntegerType::U64, 4))),
            )],
        ),
    };
    let direct = complete(vm.evaluate(&direct).outcome).remove(0);
    let indirect = ValueExpr::IndirectCall {
        inline_hint: jai_types::InlineHint::Automatic,
        callee: Box::new(ValueExpr::ProcedureValue {
            procedure: malloc(),
            ty: provider.signatures[&malloc()],
        }),
        arguments: vec![(
            ParameterId::new(0),
            ValueExpr::Int(IntExpr::constant(Integer::wrapping(IntegerType::U64, 6))),
        )],
        ty: pointer_type,
    };
    let indirect = complete(vm.evaluate(&indirect).outcome).remove(0);
    assert_eq!(vm.memory().allocation_count(), 3);
    let mut resumed =
        Vm::with_state(&provider, NoEffects, Limits::default(), vm.into_state()).unwrap();
    assert_eq!(
        resumed.memory().load(&provider.types, &byte).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 42))
    );
    for value in [first, direct, indirect] {
        assert!(complete(resumed.execute(free(), vec![value]).outcome).is_empty());
    }
    assert_eq!(resumed.memory().allocation_count(), 0);
}
#[test]
fn exact_id_and_signature_validation_precede_heap_mutation() {
    let mut provider = provider();
    provider.bound.insert(realloc(), provider.bound[&malloc()]);
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.execute(
            realloc(),
            vec![
                Value::Pointer(crate::Pointer::null(provider.types.void())),
                size(16)
            ]
        )
        .outcome,
        Outcome::Failed(Error::InvalidIr(_))
    ));
    assert_eq!(vm.memory().allocation_count(), 0);
    let mut provider = self::provider();
    provider
        .signatures
        .insert(malloc(), provider.signatures[&realloc()]);
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.execute(malloc(), vec![size(16)]).outcome,
        Outcome::Failed(_)
    ));
    assert_eq!(vm.memory().allocation_count(), 0);
}
#[test]
fn low_fuel_and_rejected_transactions_restore_alloc_realloc_and_free() {
    let provider = provider();
    let mut vm = Vm::new(
        &provider,
        NoEffects,
        Limits {
            fuel: 8,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        vm.execute(malloc(), vec![size(128)]).outcome,
        Outcome::Failed(Error::Limit(LimitKind::Fuel))
    ));
    assert_eq!(vm.memory().allocation_count(), 0);
    let mut vm = Vm::new(&provider, RejectCommit(false), Limits::default()).unwrap();
    vm.effects_mut().0 = true;
    assert!(matches!(
        vm.execute(malloc(), vec![size(8)]).outcome,
        Outcome::Failed(Error::EffectRejected(_))
    ));
    assert_eq!(vm.memory().allocation_count(), 0);
    vm.effects_mut().0 = false;
    let root = complete(vm.execute(malloc(), vec![size(8)]).outcome).remove(0);
    let byte = byte_view(&provider, vm.memory(), &root);
    vm.memory_mut()
        .byte_set(&provider.types, &byte, 7, 8)
        .unwrap();
    vm.effects_mut().0 = true;
    assert!(matches!(
        vm.execute(realloc(), vec![root.clone(), size(16)]).outcome,
        Outcome::Failed(Error::EffectRejected(_))
    ));
    assert_eq!(vm.memory().allocation_count(), 1);
    assert_eq!(
        vm.memory().load(&provider.types, &byte).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 7))
    );
    assert!(matches!(
        vm.execute(free(), vec![root.clone()]).outcome,
        Outcome::Failed(Error::EffectRejected(_))
    ));
    assert_eq!(vm.memory().allocation_count(), 1);
    assert_eq!(
        vm.memory().load(&provider.types, &byte).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 7))
    );
    vm.effects_mut().0 = false;
    complete(vm.execute(free(), vec![root]).outcome);
    assert_eq!(vm.memory().allocation_count(), 0);
}
struct RejectCommit(bool);
impl CompilerEffects for RejectCommit {
    fn begin(&mut self) {
    }
    fn request(&mut self, _: CompilerRequest) -> EffectOutcome {
        EffectOutcome::Rejected("no requests".into())
    }
    fn finish(&mut self, commit: bool) -> Result<(), Error> {
        if commit && self.0 {
            Err(Error::EffectRejected(
                "self-authored commit rejection".into(),
            ))
        } else {
            Ok(())
        }
    }
}
