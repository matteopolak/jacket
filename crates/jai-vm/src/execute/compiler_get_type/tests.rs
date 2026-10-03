use super::*;
use jai_types::{
    Integer, IntegerType, LayoutPolicy, RecordKind, ReflectionGraph, ReflectionMetadata,
    ReflectionReadiness, ScalarType, TypeRegistry,
};
use std::sync::Arc;

struct Provider {
    types: TypeRegistry,
}
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
        ProcedureAvailability::Missing
    }
}
fn fixture() -> (Provider, Arc<StaticData>, StaticAddress, RuntimeTypeSchema) {
    let mut types = TypeRegistry::new();
    let tag = types.reserve_enum(IntegerType::U32);
    types
        .define_enum(tag, [Integer::checked(IntegerType::U32, 2).unwrap()])
        .unwrap();
    let size = types.scalar(ScalarType::Int(IntegerType::S64));
    let header = types.reserve_record(RecordKind::Struct);
    types.define_record(header, [tag, size]).unwrap();
    types.bind_runtime_type_header(header).unwrap();
    types.pointer(header).unwrap();
    let schema = RuntimeTypeSchema::from_view(&types).unwrap();
    let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
        &types,
        types.scalar(ScalarType::Bool),
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    )
    .unwrap() else {
        panic!("ready boolean")
    };
    let mut data = StaticDataBuilder::new();
    let descriptor = data.reserve(header, &types).unwrap();
    data.define_type_descriptor(
        descriptor,
        jai_ir::StaticValue {
            ty: header,
            kind: jai_ir::StaticValueKind::Record(vec![
                jai_ir::StaticValue::constant(jai_ir::ConstantValue {
                    ty: tag,
                    kind: jai_ir::ConstantKind::Enum(
                        Integer::checked(IntegerType::U32, 2).unwrap(),
                    ),
                }),
                jai_ir::StaticValue::constant(jai_ir::ConstantValue {
                    ty: size,
                    kind: jai_ir::ConstantKind::Int(Integer::checked(IntegerType::S64, 1).unwrap()),
                }),
            ]),
        },
        &graph,
        graph.root(),
        &types,
    )
    .unwrap();
    let data = Arc::new(
        data.publish(&types, jai_ir::StaticDataLimits::default())
            .unwrap(),
    );
    (
        Provider {
            types,
        },
        data,
        StaticAddress::new(descriptor),
        schema,
    )
}

#[test]
fn inverse_lookup_returns_only_the_registered_descriptor_identity() {
    let (provider, data, address, schema) = fixture();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let pointer = vm.static_address(&data, &address, 0).unwrap();
    let values = vm
        .compiler_get_type(schema, &[Value::Pointer(pointer)])
        .unwrap();
    assert_eq!(
        vm.runtime_type_identity(&values[0]).unwrap().ty(),
        provider.types.scalar(ScalarType::Bool)
    );
    assert!(matches!(
        values.as_slice(),
        [Value::Type {
            descriptor: Some(_)
        }]
    ));
}

#[test]
fn null_descriptor_is_not_an_invented_null_type() {
    let (provider, _, _, schema) = fixture();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.compiler_get_type(
            schema,
            &[Value::Pointer(Pointer::null(schema.header_type()))]
        ),
        Err(Halt::Failed(Error::NullPointer))
    ));
}

#[test]
fn plausible_header_bytes_do_not_grant_a_descriptor_receipt() {
    let (provider, _, _, schema) = fixture();
    let fields = &provider
        .types
        .record_definition(schema.header_type())
        .unwrap()
        .fields;
    let mut data = StaticDataBuilder::new();
    let object = data.reserve(schema.header_type(), &provider.types).unwrap();
    data.define(
        object,
        jai_ir::StaticValue {
            ty: schema.header_type(),
            kind: jai_ir::StaticValueKind::Record(vec![
                jai_ir::StaticValue::constant(jai_ir::ConstantValue {
                    ty: fields[0],
                    kind: jai_ir::ConstantKind::Enum(
                        Integer::checked(IntegerType::U32, 2).unwrap(),
                    ),
                }),
                jai_ir::StaticValue::constant(jai_ir::ConstantValue {
                    ty: fields[1],
                    kind: jai_ir::ConstantKind::Int(Integer::checked(IntegerType::S64, 1).unwrap()),
                }),
            ]),
        },
    )
    .unwrap();
    let data = Arc::new(
        data.publish(&provider.types, jai_ir::StaticDataLimits::default())
            .unwrap(),
    );
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let pointer = vm
        .static_address(&data, &StaticAddress::new(object), 0)
        .unwrap();
    assert!(matches!(
        vm.compiler_get_type(schema, &[Value::Pointer(pointer)]),
        Err(Halt::Failed(Error::InvalidIr(
            "runtime Type value does not name a canonical descriptor"
        )))
    ));
}

#[test]
fn another_canonical_arena_cannot_authorize_a_descriptor() {
    let (provider, data, address, schema) = fixture();
    let (foreign, _, _, foreign_schema) = fixture();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let pointer = vm.static_address(&data, &address, 0).unwrap();
    assert!(
        vm.compiler_get_type(foreign_schema, &[Value::Pointer(pointer)])
            .is_err()
    );
    assert!(schema.validate(&foreign.types).is_err());
}

#[test]
fn metering_failure_occurs_before_descriptor_lookup_or_result_copy() {
    let (provider, _, _, schema) = fixture();
    let mut vm = Vm::new(
        &provider,
        crate::NoEffects,
        Limits {
            fuel: 0,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        vm.compiler_get_type(
            schema,
            &[Value::Pointer(Pointer::null(schema.header_type()))]
        ),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
}
