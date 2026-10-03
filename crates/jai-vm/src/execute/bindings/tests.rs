use super::*;
use crate::{
    AddressProvenance, ByteImage, ByteTarget, Limits, Memory, Number, Pointer, StoredAggregate,
};
use jai_ir::ProcedureId;
use jai_types::{
    CallingConvention, ContextMode, FloatValue, Integer, IntegerType, ProcedureType, RecordKind,
    ScalarType, TypeRegistry, Variadic,
};

fn id(index: usize) -> ExpressionBindingId {
    ExpressionBindingId::new(ProcedureId::new(3), index)
}
fn integer(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S64, value))
}

#[test]
fn written_order_allows_later_producers_to_read_earlier_captures() {
    let mut environment = BindingEnvironment::default();
    let scope = environment.begin();
    assert_eq!(environment.depth(), 1);
    assert_eq!(environment.cells(), 1);
    assert_eq!(environment.insert(id(0), integer(7), 2), Ok(2));
    let earlier = environment.lookup(id(0)).unwrap().integer().unwrap();
    assert_eq!(
        environment.insert(id(1), integer(earlier.value() + 1), 2),
        Ok(2)
    );
    assert_eq!(environment.lookup(id(1)), Ok(&integer(8)));
    assert_eq!(environment.cells(), 5);
    environment.end(scope).unwrap();
    assert_eq!(environment.cells(), 0);
    assert!(environment.lookup(id(0)).is_err());
}

#[test]
fn recursive_same_id_shadows_in_nested_scopes_then_restores_outer_value() {
    let mut environment = BindingEnvironment::new();
    let outer = environment.begin();
    environment.insert(id(0), integer(3), 100).unwrap();
    let inner = environment.begin();
    assert_eq!(environment.depth(), 2);
    environment
        .insert(id(0), Value::String(vec![1, 2, 3]), 100)
        .unwrap();
    assert_eq!(environment.lookup(id(0)), Ok(&Value::String(vec![1, 2, 3])));
    assert_eq!(environment.clone_charge(id(0)), Ok(4));
    assert_eq!(environment.cells(), 9);
    environment.end(inner).unwrap();
    assert_eq!(environment.depth(), 1);
    assert_eq!(environment.lookup(id(0)), Ok(&integer(3)));
    assert_eq!(environment.clone_charge(id(0)), Ok(1));
    assert_eq!(environment.cells(), 3);
    environment.end(outer).unwrap();
    assert_eq!(environment.cells(), 0);
}

#[test]
fn duplicate_current_scope_and_failed_capacity_admission_are_atomic() {
    let mut environment = BindingEnvironment::new();
    let scope = environment.begin();
    environment.insert(id(0), integer(3), 2).unwrap();
    assert_eq!(
        environment.insert(id(0), integer(4), 2),
        Err(Error::InvalidIr(
            "duplicate expression binding in one scope"
        ))
    );
    assert_eq!(
        environment.insert(id(1), Value::String(vec![0; 4]), 4),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(environment.cells(), 3);
    assert_eq!(environment.lookup(id(0)), Ok(&integer(3)));
    assert!(environment.lookup(id(1)).is_err());
    environment.end(scope).unwrap();
}

#[test]
fn out_of_order_stale_and_foreign_tokens_cannot_end_a_scope() {
    let mut environment = BindingEnvironment::new();
    let outer = environment.begin();
    environment.insert(id(0), integer(3), 2).unwrap();
    let inner = environment.begin();
    assert!(environment.end(outer).is_err());
    assert_eq!(environment.lookup(id(0)), Ok(&integer(3)));
    environment.end(inner).unwrap();
    environment.end(outer).unwrap();
    let replacement = environment.begin();
    assert!(environment.end(outer).is_err());
    let foreign = BindingEnvironment::new().begin();
    assert!(environment.end(foreign).is_err());
    environment.end(replacement).unwrap();
}

#[test]
fn clear_retires_all_scopes_and_preserves_token_nonreuse() {
    let mut environment = BindingEnvironment::new();
    let stale = environment.begin();
    environment.insert(id(0), integer(3), 2).unwrap();
    environment.clear();
    assert_eq!(environment.cells(), 0);
    assert!(environment.lookup(id(0)).is_err());
    assert!(environment.insert(id(0), integer(4), 2).is_err());
    let scope = environment.begin();
    assert!(environment.end(stale).is_err());
    environment.insert(id(0), integer(4), 2).unwrap();
    environment.end(scope).unwrap();
}

#[test]
fn captures_retain_pointer_procedure_address_origin_and_float_bits() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [byte]).unwrap();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory.allocate(&types, record, None).unwrap();
    let pointer = memory.field(&types, &root, 0).unwrap();
    let number = Number::address(
        Integer::wrapping(IntegerType::U64, 42),
        AddressProvenance::Pointer(pointer.clone()),
    );
    let values = [
        Value::Pointer(pointer),
        Value::Procedure {
            signature,
            procedure: Some(ProcedureId::new(7)),
        },
        number.into_value(),
        Value::Float(FloatValue::F64(0x8000_0000_0000_0000)),
        Value::Float(FloatValue::F32(0x7fa1_2345)),
    ];
    let mut environment = BindingEnvironment::new();
    let scope = environment.begin();
    let mut total = 0;
    for (index, value) in values.into_iter().enumerate() {
        let cells = value.cells(100).unwrap();
        let expected = value.clone();
        assert_eq!(environment.insert(id(index), value, 100), Ok(cells + 1));
        assert_eq!(environment.lookup(id(index)), Ok(&expected));
        assert_eq!(environment.clone_charge(id(index)), Ok(cells));
        total += cells + 1;
    }
    assert_eq!(environment.cells(), total + 1);
    environment.end(scope).unwrap();
}

#[test]
fn stored_aggregate_capture_keeps_inactive_bytes_and_complete_cached_charge() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [byte, word]).unwrap();
    let target = ByteTarget::default();
    let image = ByteImage::from_bytes(target, vec![1, 2, 3, 4, 5, 6, 7, 8], 100).unwrap();
    let semantic = Value::Union {
        ty: union,
        field: 0,
        value: Box::new(Value::Int(Integer::wrapping(IntegerType::U8, 1))),
    };
    let snapshot = StoredAggregate::new(union, semantic, image, 100).unwrap();
    let value = Value::StoredAggregate(snapshot);
    let expected = value.clone();
    let cells = value.cells(100).unwrap();
    let mut environment = BindingEnvironment::new();
    let scope = environment.begin();
    assert_eq!(environment.insert(id(0), value, cells + 1), Ok(cells + 1));
    assert_eq!(environment.lookup(id(0)), Ok(&expected));
    assert_eq!(environment.clone_charge(id(0)), Ok(cells));
    assert_eq!(environment.cells(), cells + 2);
    let Value::StoredAggregate(snapshot) = environment.lookup(id(0)).unwrap() else {
        panic!("capture lost its storage carrier")
    };
    assert_eq!(
        snapshot
            .field(&types, 1, 100)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        0x0807_0605_0403_0201
    );
    environment.end(scope).unwrap();
}

#[test]
fn failed_nested_insert_keeps_the_outer_shadow_and_other_procedure_id_distinct() {
    let mut environment = BindingEnvironment::new();
    let outer = environment.begin();
    environment.insert(id(0), integer(3), 2).unwrap();
    let inner = environment.begin();
    assert!(
        environment
            .insert(
                id(0),
                Value::Pointer(Pointer::null(TypeRegistry::new().void())),
                0
            )
            .is_err()
    );
    assert_eq!(environment.lookup(id(0)), Ok(&integer(3)));
    let foreign_id = ExpressionBindingId::new(ProcedureId::new(8), 0);
    environment.insert(foreign_id, integer(8), 2).unwrap();
    assert_eq!(environment.lookup(foreign_id), Ok(&integer(8)));
    assert_eq!(environment.lookup(id(0)), Ok(&integer(3)));
    environment.end(inner).unwrap();
    environment.end(outer).unwrap();
}
