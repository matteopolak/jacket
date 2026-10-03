use super::*;
use jai_types::{
    AllocatorField, AllocatorMode, AllocatorSchema, CallingConvention, ContextMode, RecordKind,
    ScalarType, StorageBitcast, StorageBitcastStrength, TypeRegistry, Variadic,
};
use std::collections::HashMap;

struct Provider {
    types: TypeRegistry,
    signatures: HashMap<ProcedureId, TypeId>,
}
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        &self.signatures
    }
    fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
        ProcedureAvailability::Missing
    }
}
fn provider(bind: bool) -> (Provider, TypeId, AllocatorSchema) {
    let mut types = TypeRegistry::new();
    let mode = types.reserve_enum(jai_types::IntegerType::S64);
    types
        .define_enum(mode, AllocatorMode::ALL.map(AllocatorMode::value))
        .unwrap();
    let data = types.pointer(types.void()).unwrap();
    let word = types.scalar(ScalarType::Int(jai_types::IntegerType::S64));
    let procedure = types
        .procedure(jai_types::ProcedureType {
            parameters: Box::new([mode, word, word, data, data]),
            results: Box::new([data]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let allocator = types.reserve_record(RecordKind::Struct);
    types.define_record(allocator, [procedure, data]).unwrap();
    let schema = AllocatorSchema::validate(&types, allocator, mode).unwrap();
    if bind {
        types.bind_allocator(allocator, mode).unwrap();
    }
    let dynamic = types.dynamic_array(word).unwrap();
    (
        Provider {
            types,
            signatures: HashMap::from([(ProcedureId::new(7), procedure)]),
        },
        dynamic,
        schema,
    )
}
fn descriptor(types: &TypeRegistry, ty: TypeId, allocator: Option<Box<Value>>) -> Value {
    let TypeKind::DynamicArray(element) = types.kind(ty).unwrap() else {
        unreachable!()
    };
    Value::DynamicArray {
        ty,
        pointer: Pointer::null(*element),
        count: 0,
        allocated: 0,
        allocator,
    }
}

#[test]
fn dynamic_defaults_use_a_genuine_typed_allocator_and_reject_missing_or_unbound_payloads() {
    let (provider, dynamic, schema) = provider(true);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let value = vm.zero_value(dynamic).unwrap();
    value.validate(&provider.types, dynamic, 16).unwrap();
    assert_eq!(value.cells(16).unwrap(), 4);
    let allocator = vm.sequence_allocator(value).unwrap();
    let Value::Record {
        ty,
        fields,
    } = allocator
    else {
        panic!("expected the declared allocator record")
    };
    assert_eq!(ty, schema.ty());
    assert_eq!(
        fields,
        [
            Value::Procedure {
                signature: schema.field(AllocatorField::Procedure).ty,
                procedure: None,
            },
            Value::Pointer(Pointer::null(provider.types.void())),
        ]
    );
    assert_eq!(
        descriptor(&provider.types, dynamic, None).validate(&provider.types, dynamic, 16),
        Err(Error::TypeMismatch {
            expected: dynamic
        })
    );
    assert!(vm.statistics.steps >= 4);

    let (unbound, dynamic, schema) = self::provider(false);
    let mut vm = Vm::new(&unbound, crate::NoEffects, Limits::default()).unwrap();
    let value = vm.zero_value(dynamic).unwrap();
    assert_eq!(value.cells(16).unwrap(), 1);
    let payload = vm.zero_value(schema.ty()).unwrap();
    assert_eq!(
        descriptor(&unbound.types, dynamic, Some(Box::new(payload))).validate(
            &unbound.types,
            dynamic,
            16
        ),
        Err(Error::TypeMismatch {
            expected: dynamic
        })
    );
    assert!(vm.sequence_allocator(value).is_err());
}

#[test]
fn dynamic_allocator_zero_work_is_admitted_before_payload_construction() {
    let (bound, dynamic, _) = provider(true);
    let mut vm = Vm::new(
        &bound,
        crate::NoEffects,
        Limits {
            fuel: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(vm.memory.zero_value_work_cost(&bound.types, dynamic), Ok(4));
    assert!(matches!(
        vm.zero_value(dynamic),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(vm.statistics.steps, 0);
    assert_eq!(vm.memory.allocation_count(), 0);
    assert_eq!(vm.memory.value_cells(), 0);

    let (unbound, dynamic, _) = provider(false);
    let mut vm = Vm::new(
        &unbound,
        crate::NoEffects,
        Limits {
            fuel: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        vm.memory.zero_value_work_cost(&unbound.types, dynamic),
        Ok(1)
    );
    assert_eq!(vm.zero_value(dynamic).unwrap().cells(16).unwrap(), 1);
    assert_eq!(vm.statistics.steps, 1);
}

#[test]
fn allocator_carriers_are_preserved_and_participate_in_dynamic_value_cells() {
    let (provider, dynamic, schema) = provider(true);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let word = provider
        .types
        .scalar(ScalarType::Int(jai_types::IntegerType::S64));
    let root = vm
        .memory
        .allocate(
            &provider.types,
            word,
            Some(Value::Int(Integer::wrapping(
                jai_types::IntegerType::S64,
                42,
            ))),
        )
        .unwrap();
    let data = vm
        .memory
        .cast_pointer(
            &provider.types,
            &root,
            provider.types.void(),
            CastMode::Checked,
        )
        .unwrap();
    let payload = Value::Record {
        ty: schema.ty(),
        fields: vec![
            Value::Procedure {
                signature: schema.field(AllocatorField::Procedure).ty,
                procedure: Some(ProcedureId::new(7)),
            },
            Value::Pointer(data.clone()),
        ],
    };
    let proof = StorageBitcast::prove(
        &provider.types,
        vm.memory.target().policy,
        schema.ty(),
        schema.ty(),
        StorageBitcastStrength::EqualSize,
    )
    .unwrap();
    let carrier = vm.storage_bitcast_value(payload.clone(), proof, 0).unwrap();
    assert!(matches!(carrier, Value::StoredAggregate(_)));
    let allocator = vm.adopt_sequence_allocator(carrier.clone()).unwrap();
    let value = descriptor(&provider.types, dynamic, Some(allocator));
    value.validate(&provider.types, dynamic, 16).unwrap();
    assert!(value.cells_and_storage(1024).unwrap().1);
    assert_eq!(value.cells(1024).unwrap(), 1 + carrier.cells(1024).unwrap());
    assert_eq!(vm.sequence_allocator(value.clone()).unwrap(), carrier);
    let materialized = vm.materialize_value(&value).unwrap();
    let Value::DynamicArray {
        allocator: Some(allocator),
        ..
    } = materialized
    else {
        panic!("expected a typed allocator")
    };
    assert_eq!(*allocator, payload);
    assert!(vm.adopt_sequence_allocator(Value::Pointer(data)).is_err());
}

#[test]
fn caller_pack_addresses_cannot_escape_inside_allocator_data() {
    let (provider, dynamic, schema) = provider(true);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let word = provider
        .types
        .scalar(ScalarType::Int(jai_types::IntegerType::S64));
    let root = vm
        .memory
        .allocate(
            &provider.types,
            word,
            Some(Value::Int(Integer::wrapping(
                jai_types::IntegerType::S64,
                42,
            ))),
        )
        .unwrap();
    let data = vm
        .memory
        .cast_pointer(
            &provider.types,
            &root,
            provider.types.void(),
            CastMode::Checked,
        )
        .unwrap();
    let value = descriptor(
        &provider.types,
        dynamic,
        Some(Box::new(Value::Record {
            ty: schema.ty(),
            fields: vec![
                Value::Procedure {
                    signature: schema.field(AllocatorField::Procedure).ty,
                    procedure: None,
                },
                Value::Pointer(data),
            ],
        })),
    );
    assert_eq!(
        vm.validate_sequence_escape(&[value], &[root]),
        Err(Error::SequenceTemporaryEscape)
    );
}

#[test]
fn allocator_place_reads_skip_uninitialized_descriptor_headers_and_preserve_partial_payloads() {
    let (provider, dynamic, schema) = provider(true);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let root = vm.memory.allocate(&provider.types, dynamic, None).unwrap();
    let allocator = vm
        .memory
        .sequence_allocator(&provider.types, &root)
        .unwrap();
    let data = vm.memory.field(&provider.types, &allocator, 1).unwrap();
    vm.memory
        .store(
            &provider.types,
            &data,
            Value::Pointer(Pointer::null(provider.types.void())),
        )
        .unwrap();
    let value = vm.load_sequence_allocator(&root).unwrap();
    let Value::StoredAggregate(snapshot) = &value else {
        panic!("partial allocator payload must retain its mask")
    };
    assert_eq!(snapshot.ty(), schema.ty());
    assert!(snapshot.decoded_semantic().is_none());
    assert!(vm.materialize_value(&value).is_err());
    assert_eq!(
        snapshot.field(&provider.types, 1, 1024).unwrap(),
        Value::Pointer(Pointer::null(provider.types.void()))
    );
    assert_eq!(
        snapshot.field(&provider.types, 0, 1024),
        Err(Error::Uninitialized)
    );
}
