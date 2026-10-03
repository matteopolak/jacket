use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, RecordKind,
    RuntimeTypeSchema, ScalarType, TypeError, TypeId, TypeKind, TypeRegistry, Variadic,
};

fn signature(types: &mut TypeRegistry) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}

fn publish(mut types: TypeRegistry, values: Vec<ValueExpr>) -> Result<Library, IrError> {
    let signature = signature(&mut types);
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![Procedure {
            id: ProcedureId::new(17),
            signature,
            parameters: vec![],
            locals: vec![],
            body: Block {
                statements: values.into_iter().map(Statement::DiscardValue).collect(),
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        }])
        .finish_library()
}

fn integer(ty: IntegerType, value: i128) -> IntExpr {
    IntExpr::constant(Integer::checked(ty, value).unwrap())
}

fn from_pointer(ty: IntegerType, value: ValueExpr, mode: CastMode) -> ValueExpr {
    ValueExpr::Int(IntExpr::new(
        ty,
        IntExprKind::FromPointer {
            value: Box::new(value),
            mode,
        },
    ))
}

#[test]
fn pointer_integer_conversions_accept_both_modes_without_host_address_rules() {
    for mode in [CastMode::Checked, CastMode::Unchecked] {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let pointer = types.pointer(byte).unwrap();
        let void = types.void();
        let void_pointer = types.pointer(void).unwrap();
        let mut values = vec![];
        for ty in [pointer, void_pointer] {
            for target in [
                IntegerType::S8,
                IntegerType::U8,
                IntegerType::S64,
                IntegerType::U64,
            ] {
                values.push(from_pointer(target, ValueExpr::Zero(ty), mode));
            }
            for value in [
                integer(IntegerType::S64, -1),
                integer(IntegerType::U64, u64::MAX.into()),
                integer(IntegerType::U8, 255),
            ] {
                values.push(ValueExpr::PointerFromInteger {
                    value,
                    ty,
                    mode,
                });
            }
        }
        publish(types, values).unwrap();
    }
}

#[test]
fn pointer_to_integer_rejects_scalars_and_procedure_addresses() {
    for mode in [CastMode::Checked, CastMode::Unchecked] {
        for procedure in [false, true] {
            let mut types = TypeRegistry::new();
            let source = if procedure {
                signature(&mut types)
            } else {
                types.scalar(ScalarType::Int(IntegerType::S64))
            };
            assert!(matches!(
                publish(types, vec![from_pointer(IntegerType::U64, ValueExpr::Zero(source), mode)]),
                Err(IrError::InvalidValue(actual)) if actual == source
            ));
        }
    }
}

#[test]
fn integer_to_pointer_requires_a_data_pointer_target() {
    for mode in [CastMode::Checked, CastMode::Unchecked] {
        for procedure in [false, true] {
            let mut types = TypeRegistry::new();
            let target = if procedure {
                signature(&mut types)
            } else {
                types.scalar(ScalarType::Int(IntegerType::U64))
            };
            assert!(matches!(
                publish(types, vec![ValueExpr::PointerFromInteger {
                    value: integer(IntegerType::U64, 0), ty: target, mode,
                }]),
                Err(IrError::InvalidValue(actual)) if actual == target
            ));
        }
    }
}

#[test]
fn integer_to_pointer_validates_the_integer_expression_width() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer = types.pointer(byte).unwrap();
    let invalid = IntExpr::new(
        IntegerType::U64,
        IntExprKind::Constant(Integer::checked(IntegerType::U8, 0).unwrap()),
    );
    assert!(matches!(
        publish(
            types,
            vec![ValueExpr::PointerFromInteger {
                value: invalid,
                ty: pointer,
                mode: CastMode::Checked,
            }]
        ),
        Err(IrError::IntegerMismatch {
            expected: IntegerType::U64,
            actual: IntegerType::U8
        })
    ));
}

#[test]
fn foreign_registry_pointer_types_are_rejected_for_all_conversion_directions() {
    for direction in 0..3 {
        let types = TypeRegistry::new();
        let mut foreign = TypeRegistry::new();
        let byte = foreign.scalar(ScalarType::Int(IntegerType::U8));
        let pointer = foreign.pointer(byte).unwrap();
        let value = match direction {
            0 => from_pointer(
                IntegerType::U64,
                ValueExpr::Zero(pointer),
                CastMode::Checked,
            ),
            1 => ValueExpr::PointerFromInteger {
                value: integer(IntegerType::U64, 0),
                ty: pointer,
                mode: CastMode::Unchecked,
            },
            2 => ValueExpr::PointerOffsetLeft {
                offset: integer(IntegerType::S64, 1),
                pointer: Box::new(ValueExpr::Zero(pointer)),
                ty: pointer,
            },
            _ => unreachable!(),
        };
        assert!(matches!(publish(types, vec![value]), Err(IrError::Type(_))));
    }
}

#[test]
fn left_pointer_offset_accepts_signed_s64_offsets_and_sized_pointees() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer = types.pointer(byte).unwrap();
    let empty_record = types.reserve_record(RecordKind::Struct);
    types.define_record(empty_record, []).unwrap();
    let empty_pointer = types.pointer(empty_record).unwrap();
    let values = [pointer, empty_pointer]
        .into_iter()
        .flat_map(|ty| {
            [-1, 0, 1]
                .into_iter()
                .map(move |offset| ValueExpr::PointerOffsetLeft {
                    offset: integer(IntegerType::S64, offset),
                    pointer: Box::new(ValueExpr::Zero(ty)),
                    ty,
                })
        })
        .collect();
    publish(types, values).unwrap();
}

#[test]
fn left_pointer_offset_rejects_unsigned_or_narrow_offset_widths() {
    for actual in [IntegerType::U64, IntegerType::S32, IntegerType::U8] {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let pointer = types.pointer(byte).unwrap();
        assert!(matches!(
            publish(types, vec![ValueExpr::PointerOffsetLeft {
                offset: integer(actual, 1),
                pointer: Box::new(ValueExpr::Zero(pointer)), ty: pointer,
            }]),
            Err(IrError::IntegerMismatch { expected: IntegerType::S64, actual: found }) if found == actual
        ));
    }
}

#[test]
fn left_pointer_offset_requires_exact_pointer_identity() {
    for scalar_source in [false, true] {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let pointer = types.pointer(byte).unwrap();
        let actual = if scalar_source {
            byte
        } else {
            let signed = types.scalar(ScalarType::Int(IntegerType::S8));
            types.pointer(signed).unwrap()
        };
        assert!(matches!(
            publish(types, vec![ValueExpr::PointerOffsetLeft {
                offset: integer(IntegerType::S64, 1),
                pointer: Box::new(ValueExpr::Zero(actual)), ty: pointer,
            }]),
            Err(IrError::TypeMismatch { expected, actual: found }) if expected == pointer && found == actual
        ));
    }
}

#[test]
fn left_pointer_offset_requires_a_pointer_result_type() {
    let types = TypeRegistry::new();
    let scalar = types.scalar(ScalarType::Int(IntegerType::S64));
    assert!(matches!(
        publish(types, vec![ValueExpr::PointerOffsetLeft {
            offset: integer(IntegerType::S64, 1),
            pointer: Box::new(ValueExpr::Zero(scalar)), ty: scalar,
        }]),
        Err(IrError::InvalidValue(actual)) if actual == scalar
    ));
}

#[test]
fn left_pointer_offset_rejects_unsized_void_and_code_pointees() {
    for kind in 0..2 {
        let mut types = TypeRegistry::new();
        let pointee = match kind {
            0 => types.void(),
            1 => types.code_type(),
            _ => unreachable!(),
        };
        let pointer = types.pointer(pointee).unwrap();
        assert!(matches!(
            publish(types, vec![ValueExpr::PointerOffsetLeft {
                offset: integer(IntegerType::S64, 1),
                pointer: Box::new(ValueExpr::Zero(pointer)), ty: pointer,
            }]),
            Err(IrError::InvalidValue(actual)) if actual == pointee
        ));
    }
}

#[test]
fn left_pointer_offset_requires_a_bound_runtime_metatype_schema() {
    let mut types = TypeRegistry::new();
    let meta = types.meta_type();
    let pointer = types.pointer(meta).unwrap();
    assert!(matches!(
        publish(types, vec![ValueExpr::PointerOffsetLeft {
            offset: integer(IntegerType::S64, 1),
            pointer: Box::new(ValueExpr::Zero(pointer)), ty: pointer,
        }]),
        Err(IrError::Type(TypeError::Incomplete(actual))) if actual == meta
    ));
}

#[test]
fn left_pointer_offset_accepts_metatype_after_ready_header_schema_binding() {
    let mut types = TypeRegistry::new();
    let header = types.reserve_record(RecordKind::Struct);
    let tag = types.scalar(ScalarType::Int(IntegerType::U32));
    let size = types.scalar(ScalarType::Int(IntegerType::S64));
    types.define_record(header, [tag, size]).unwrap();
    types.bind_runtime_type_header(header).unwrap();
    let schema = RuntimeTypeSchema::from_view(&types).unwrap();
    assert_eq!(schema.header_type(), header);
    assert_eq!(
        types.kind(schema.descriptor_type()).unwrap(),
        &TypeKind::Pointer(header)
    );
    let pointer = types.pointer(schema.ty()).unwrap();
    let library = publish(
        types,
        vec![ValueExpr::PointerOffsetLeft {
            offset: integer(IntegerType::S64, 1),
            pointer: Box::new(ValueExpr::Zero(pointer)),
            ty: pointer,
        }],
    )
    .unwrap();
    assert_eq!(
        RuntimeTypeSchema::from_view(library.types()).unwrap(),
        schema
    );
}
