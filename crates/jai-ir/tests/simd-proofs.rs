use jai_ir::*;
use jai_types::{
    CallingConvention, CastMode, ContextMode, Integer, IntegerType, ScalarType, TypeId,
    TypeRegistry, Types, Variadic,
};

struct Fixture {
    types: Types,
    pointer: TypeId,
    signature: TypeId,
    pointer_call_signature: TypeId,
}

impl Fixture {
    fn new() -> Self {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let boolean = types.scalar(ScalarType::Bool);
        let pointer = types.pointer(byte).unwrap();
        let signature = types
            .procedure(jai_types::ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: Variadic::None,
            })
            .unwrap();
        let pointer_call_signature = types
            .procedure(jai_types::ProcedureType {
                parameters: Box::new([boolean]),
                results: Box::new([pointer]),
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        Self {
            types: types.freeze().unwrap(),
            pointer,
            signature,
            pointer_call_signature,
        }
    }

    fn procedure(&self, simd: SimdBlock, locals: Vec<Local>) -> Procedure {
        Procedure {
            id: ProcedureId::new(7),
            signature: self.signature,
            parameters: vec![],
            locals,
            body: Block {
                statements: vec![Statement::Simd(simd)],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        }
    }

    fn outer_checked_simd(&self, address: ValueExpr, store: bool) -> SimdBlock {
        let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
        let register = builder.register(SimdWidth::X128).unwrap();
        builder
            .instruction(
                SimdInstruction::Load {
                    destination: register,
                    interpretation: SimdInterpretation::U8,
                    address: if store {
                        ValueExpr::Zero(self.pointer)
                    } else {
                        address.clone()
                    },
                },
                &self.types,
            )
            .unwrap();
        if store {
            builder
                .instruction(
                    SimdInstruction::Store {
                        source: register,
                        interpretation: SimdInterpretation::U8,
                        address,
                    },
                    &self.types,
                )
                .unwrap();
        }
        let simd = builder.finish();
        simd.validate(&self.types).unwrap();
        simd
    }

    fn unknown_nested_call(&self) -> ValueExpr {
        ValueExpr::PointerCast {
            value: Box::new(ValueExpr::Call {
                call: Call::new(ProcedureId::new(999), vec![]),
                ty: self.pointer,
            }),
            ty: self.pointer,
            mode: CastMode::Checked,
        }
    }
}

#[test]
fn typed_pointer_load_and_store_publish_with_checked_registers() {
    let fixture = Fixture::new();
    let local = Local::new_typed(ProcedureId::new(7), 0, fixture.pointer, &fixture.types).unwrap();
    let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
    let register = builder.register(SimdWidth::X128).unwrap();
    for instruction in [
        SimdInstruction::Load {
            destination: register,
            interpretation: SimdInterpretation::U8,
            address: ValueExpr::Load(local.place()),
        },
        SimdInstruction::Store {
            source: register,
            interpretation: SimdInterpretation::U8,
            address: ValueExpr::Load(local.place()),
        },
    ] {
        builder.instruction(instruction, &fixture.types).unwrap();
    }
    let simd = builder.finish();
    assert_eq!(simd.registers(), &[SimdWidth::X128]);
    assert_eq!(simd.width(register).unwrap(), SimdWidth::X128);
    assert_eq!(simd.instructions().len(), 2);
    let procedure = fixture.procedure(simd, vec![local]);
    let library = ProgramBuilder::new(fixture.types)
        .procedures(vec![procedure])
        .finish_library()
        .unwrap();
    assert!(library.checked_procedure(ProcedureId::new(7)).is_some());
}

#[test]
fn publication_checks_unknown_procedure_inside_simd_load_address() {
    let fixture = Fixture::new();
    let simd = fixture.outer_checked_simd(fixture.unknown_nested_call(), false);
    let procedure = fixture.procedure(simd, vec![]);
    assert!(matches!(
        ProgramBuilder::new(fixture.types)
            .procedures(vec![procedure])
            .finish_library(),
        Err(IrError::UnknownIdentity {
            kind: "called procedure",
            index: 999
        })
    ));
}

#[test]
fn publication_checks_unknown_procedure_inside_simd_store_address() {
    let fixture = Fixture::new();
    let simd = fixture.outer_checked_simd(fixture.unknown_nested_call(), true);
    let procedure = fixture.procedure(simd, vec![]);
    assert!(matches!(
        ProgramBuilder::new(fixture.types)
            .procedures(vec![procedure])
            .finish_library(),
        Err(IrError::UnknownIdentity {
            kind: "called procedure",
            index: 999
        })
    ));
}

#[test]
fn publication_checks_local_owner_inside_simd_pointer_load() {
    let fixture = Fixture::new();
    let foreign =
        Local::new_typed(ProcedureId::new(99), 0, fixture.pointer, &fixture.types).unwrap();
    let simd = fixture.outer_checked_simd(ValueExpr::Load(foreign.place()), false);
    let procedure = fixture.procedure(simd, vec![]);
    assert!(
        matches!(ProgramBuilder::new(fixture.types).procedures(vec![procedure]).finish_library(),
        Err(IrError::LocalOwner { expected, actual }) if expected == ProcedureId::new(7) && actual == ProcedureId::new(99))
    );
}

#[test]
fn publication_checks_argument_type_inside_simd_pointer_call() {
    let fixture = Fixture::new();
    let address = ValueExpr::Call {
        call: Call::new(
            ProcedureId::new(41),
            vec![(
                ParameterId::new(0),
                ValueExpr::Int(IntExpr::constant(
                    Integer::checked(IntegerType::S64, 1).unwrap(),
                )),
            )],
        ),
        ty: fixture.pointer,
    };
    let simd = fixture.outer_checked_simd(address, false);
    let procedure = fixture.procedure(simd, vec![]);
    let prototype = ProcedurePrototype {
        id: ProcedureId::new(41),
        signature: fixture.pointer_call_signature,
        origin: PrototypeOrigin::Compiler,
    };
    assert!(matches!(
        ProgramBuilder::new(fixture.types)
            .procedures(vec![procedure])
            .prototypes(vec![prototype])
            .finish_library(),
        Err(IrError::TypeMismatch { .. })
    ));
}
