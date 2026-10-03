use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, IntegerType, ProcedureType, ScalarType, TypeRegistry, Variadic,
};

#[test]
fn simd_address_children_keep_the_common_expression_depth_budget() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut types = TypeRegistry::new();
            let byte = types.scalar(ScalarType::Int(IntegerType::U8));
            let pointer = types.pointer(byte).unwrap();
            let signature = types
                .procedure(ProcedureType {
                    parameters: Box::new([]),
                    results: Box::new([]),
                    convention: CallingConvention::Jai,
                    context: ContextMode::None,
                    variadic: Variadic::None,
                })
                .unwrap();
            let mut address = ValueExpr::Zero(pointer);
            for _ in 0..20_000 {
                address = ValueExpr::PointerCast {
                    value: Box::new(address),
                    ty: pointer,
                    mode: CastMode::Checked,
                };
            }
            let mut simd = SimdBuilder::new(SimdFeatures::default()).unwrap();
            let destination = simd.register(SimdWidth::X128).unwrap();
            simd.instruction(
                SimdInstruction::Load {
                    destination,
                    interpretation: SimdInterpretation::U8,
                    address,
                },
                &types,
            )
            .unwrap();
            let result = ProgramBuilder::new(types.freeze().unwrap())
                .procedures(vec![Procedure {
                    id: ProcedureId::new(9),
                    signature,
                    parameters: vec![],
                    locals: vec![],
                    body: Block {
                        statements: vec![Statement::Simd(simd.finish())],
                        flow: Flow::FallsThrough,
                    },
                    cleanups: vec![],
                }])
                .finish_library();
            assert!(matches!(result, Err(IrError::VerificationDepth)));
        })
        .unwrap()
        .join()
        .unwrap();
}
