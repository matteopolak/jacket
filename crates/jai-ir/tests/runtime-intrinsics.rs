use jai_ir::{
    IrError, ProcedureId, ProcedurePrototype, ProgramBuilder, PrototypeOrigin, RuntimeIntrinsic,
};
use jai_types::{
    CallingConvention, ContextMode, IntegerType, ProcedureType, ScalarType, TypeRegistry, Variadic,
};

fn trap_builder(intrinsic: RuntimeIntrinsic, context: ContextMode) -> ProgramBuilder {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context,
            variadic: Variadic::None,
        })
        .unwrap();
    ProgramBuilder::new(types.freeze().unwrap()).prototypes(vec![ProcedurePrototype {
        id: ProcedureId::new(0),
        signature,
        origin: PrototypeOrigin::Intrinsic(intrinsic),
    }])
}

#[test]
fn checked_publication_preserves_runtime_prototype_identity_without_a_body() {
    let library = trap_builder(RuntimeIntrinsic::DebugTrap, ContextMode::None)
        .finish_library()
        .unwrap();
    assert!(library.procedures().is_empty());
    assert!(matches!(
        library.prototypes()[0].origin,
        PrototypeOrigin::Intrinsic(RuntimeIntrinsic::DebugTrap)
    ));
}

#[test]
fn checked_publication_rejects_a_forged_intrinsic_operation_or_context() {
    for builder in [
        trap_builder(RuntimeIntrinsic::MemoryCopy, ContextMode::None),
        trap_builder(RuntimeIntrinsic::DebugTrap, ContextMode::Implicit),
    ] {
        assert!(matches!(
            builder.finish_library(),
            Err(IrError::RuntimeIntrinsic(_))
        ));
    }
}

#[test]
fn checked_publication_rejects_a_foreign_nominal_cas_value_identity() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = types.pointer(integer).unwrap();
    let boolean = types.scalar(ScalarType::Bool);
    let signature = types
        .procedure(ProcedureType {
            parameters: vec![pointer, integer, integer].into(),
            results: vec![boolean, integer].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let foreign = TypeRegistry::new().scalar(ScalarType::Int(IntegerType::S64));
    let result = ProgramBuilder::new(types.freeze().unwrap())
        .prototypes(vec![ProcedurePrototype {
            id: ProcedureId::new(0),
            signature,
            origin: PrototypeOrigin::Intrinsic(RuntimeIntrinsic::CompareAndSwap {
                value: foreign,
            }),
        }])
        .finish_library();
    assert!(matches!(result, Err(IrError::RuntimeIntrinsic(_))));
}
