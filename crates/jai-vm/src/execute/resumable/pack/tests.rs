use super::*;
use jai_types::{Integer, IntegerType, RecordKind, ScalarType, TypeRegistry, TypeView};
use std::cell::Cell;
use std::collections::HashMap;

struct Provider<T = TypeRegistry> {
    types: T,
    signatures: HashMap<ProcedureId, TypeId>,
}
impl<T: TypeView> ProcedureProvider for Provider<T> {
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
fn integer(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S64, value))
}

struct CountedTypes<'a> {
    types: &'a TypeRegistry,
    records: Cell<usize>,
}
impl TypeView for CountedTypes<'_> {
    fn kind(&self, id: TypeId) -> std::result::Result<&TypeKind, jai_types::TypeError> {
        self.types.kind(id)
    }
    fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        self.types.lookup(kind)
    }
    fn scalar(&self, ty: ScalarType) -> TypeId {
        self.types.scalar(ty)
    }
    fn float(&self, ty: jai_types::FloatType) -> TypeId {
        self.types.float(ty)
    }
    fn any_type(&self) -> Option<TypeId> {
        self.types.any_type()
    }
    fn record(
        &self,
        id: jai_types::RecordId,
    ) -> std::result::Result<&jai_types::RecordDefinition, jai_types::TypeError> {
        self.records.set(self.records.get() + 1);
        self.types.record(id)
    }
    fn enumeration(
        &self,
        id: jai_types::EnumId,
    ) -> std::result::Result<&jai_types::EnumDefinition, jai_types::TypeError> {
        self.types.enumeration(id)
    }
    fn distinct(
        &self,
        id: jai_types::DistinctId,
    ) -> std::result::Result<&jai_types::DistinctDefinition, jai_types::TypeError> {
        self.types.distinct(id)
    }
    fn procedure_type(
        &self,
        id: jai_types::ProcedureTypeId,
    ) -> std::result::Result<&jai_types::ProcedureType, jai_types::TypeError> {
        self.types.procedure_type(id)
    }
    fn record_type(
        &self,
        id: jai_types::RecordId,
    ) -> std::result::Result<TypeId, jai_types::TypeError> {
        self.types.record_type(id)
    }
}

#[test]
fn wide_zero_stride_pack_layouts_preflight_cold_work_and_do_not_rescan_warm_fields() {
    let mut types = TypeRegistry::new();
    let empty = types.reserve_record(RecordKind::Struct);
    types.define_record(empty, []).unwrap();
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![empty; 512]).unwrap();
    let slice = types.slice(wide).unwrap();
    let provider = Provider {
        types: CountedTypes {
            types: &types,
            records: Cell::new(0),
        },
        signatures: HashMap::new(),
    };
    let mut exhausted = Vm::new(
        &provider,
        crate::NoEffects,
        Limits {
            fuel: 16,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        PackState::new(&mut exhausted, slice),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert!(exhausted.statistics.steps > 0);
    assert_eq!(exhausted.memory.value_cells(), 0);
    assert!(exhausted.root_temporaries.is_empty());

    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    PackState::new(&mut vm, slice).unwrap();
    let cold_steps = vm.statistics.steps;
    assert!(cold_steps > 512);
    let retained = vm.memory.value_cells();
    provider.types.records.set(0);
    for _ in 0..16 {
        let Value::Slice {
            pointer,
            count,
            ..
        } = PackState::new(&mut vm, slice)
            .unwrap()
            .finish(&mut vm)
            .unwrap()
        else {
            panic!("expected empty slice")
        };
        assert!(pointer.is_null());
        assert_eq!(count, 0);
    }
    assert_eq!(provider.types.records.get(), 0);
    assert_eq!(vm.statistics.steps - cold_steps, 16);
    for _ in 0..16 {
        let Value::Slice {
            pointer,
            count,
            ..
        } = vm.sequence_concat(slice, &[], 0).unwrap()
        else {
            panic!("expected empty slice")
        };
        assert!(pointer.is_null());
        assert_eq!(count, 0);
    }
    assert_eq!(provider.types.records.get(), 0);
    // Each synchronous pack also admits its expression node before the cache hit.
    assert_eq!(vm.statistics.steps - cold_steps, 48);
    assert_eq!(vm.memory.value_cells(), retained);
    assert!(vm.root_temporaries.is_empty());
}

#[test]
fn retained_pack_snapshots_survive_source_mutation_before_later_capture() {
    let mut provider = Provider {
        types: TypeRegistry::new(),
        signatures: HashMap::new(),
    };
    let element = provider.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = provider.types.fixed_array(element, 2).unwrap();
    let slice = provider.types.slice(element).unwrap();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let root = vm
        .memory
        .allocate(
            &provider.types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![integer(10), integer(20)],
            }),
        )
        .unwrap();
    let pointer = vm.memory.sequence_data(&provider.types, &root).unwrap();
    let mut state = PackState::new(&mut vm, slice).unwrap();
    state
        .capture(
            &mut vm,
            PackPartMode::Spread,
            Operand::Value(Value::Slice {
                ty: slice,
                pointer: pointer.clone(),
                count: 2,
            }),
            0,
        )
        .unwrap();
    vm.memory
        .store(&provider.types, &pointer, integer(99))
        .unwrap();
    state
        .capture(
            &mut vm,
            PackPartMode::ElementValue,
            Operand::Value(integer(7)),
            0,
        )
        .unwrap();
    let Value::Slice {
        pointer,
        count,
        ..
    } = state.finish(&mut vm).unwrap()
    else {
        panic!("expected slice")
    };
    assert_eq!(count, 3);
    assert_eq!(vm.root_temporaries.len(), 1);
    assert_eq!(vm.root_sequence_temp_bytes, 198);
    let values = (0..3)
        .map(|index| {
            let pointer = vm.memory.offset(&provider.types, &pointer, index).unwrap();
            vm.memory.load(&provider.types, &pointer).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(values, [integer(10), integer(20), integer(7)]);
}

#[test]
fn stored_inactive_union_bytes_cannot_hide_a_pack_allocation_origin() {
    let mut provider = Provider {
        types: TypeRegistry::new(),
        signatures: HashMap::new(),
    };
    let element = provider.types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = provider.types.slice(element).unwrap();
    let pointer_type = provider.types.pointer(element).unwrap();
    let byte = provider.types.scalar(ScalarType::Int(IntegerType::U8));
    let union = provider.types.reserve_record(RecordKind::Union);
    provider
        .types
        .define_record(union, [pointer_type, byte])
        .unwrap();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let mut state = PackState::new(&mut vm, slice).unwrap();
    state
        .capture(
            &mut vm,
            PackPartMode::ElementValue,
            Operand::Value(integer(42)),
            0,
        )
        .unwrap();
    let Value::Slice {
        pointer, ..
    } = state.finish(&mut vm).unwrap()
    else {
        panic!("expected slice")
    };
    let union_root = vm
        .memory
        .allocate(
            &provider.types,
            union,
            Some(Value::Union {
                ty: union,
                field: 0,
                value: Box::new(Value::Pointer(pointer)),
            }),
        )
        .unwrap();
    let low = vm.memory.field(&provider.types, &union_root, 1).unwrap();
    vm.memory
        .store(
            &provider.types,
            &low,
            Value::Int(Integer::wrapping(IntegerType::U8, 0)),
        )
        .unwrap();
    let snapshot = vm.memory.load(&provider.types, &union_root).unwrap();
    assert!(matches!(snapshot, Value::StoredAggregate(_)));
    assert_eq!(
        vm.validate_sequence_escape(&[snapshot], &vm.root_temporaries),
        Err(Error::SequenceTemporaryEscape)
    );
}

#[test]
fn empty_pack_has_no_allocation_and_nonempty_zero_stride_pack_has_a_sentinel() {
    let mut provider = Provider {
        types: TypeRegistry::new(),
        signatures: HashMap::new(),
    };
    let empty = provider.types.reserve_record(RecordKind::Struct);
    provider.types.define_record(empty, []).unwrap();
    let slice = provider.types.slice(empty).unwrap();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let Value::Slice {
        pointer,
        count,
        ..
    } = PackState::new(&mut vm, slice)
        .unwrap()
        .finish(&mut vm)
        .unwrap()
    else {
        panic!("expected slice")
    };
    assert_eq!(count, 0);
    assert!(pointer.is_null());
    assert!(vm.root_temporaries.is_empty());
    let mut state = PackState::new(&mut vm, slice).unwrap();
    state
        .capture(
            &mut vm,
            PackPartMode::ElementValue,
            Operand::Value(Value::Record {
                ty: empty,
                fields: vec![],
            }),
            0,
        )
        .unwrap();
    let Value::Slice {
        pointer,
        count,
        ..
    } = state.finish(&mut vm).unwrap()
    else {
        panic!("expected slice")
    };
    assert_eq!(count, 1);
    assert!(!pointer.is_null());
    assert_eq!(vm.root_sequence_temp_bytes, 80);
    assert_eq!(
        vm.memory.load(&provider.types, &pointer).unwrap(),
        Value::Record {
            ty: empty,
            fields: vec![]
        }
    );
}
