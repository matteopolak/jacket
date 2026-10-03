//! Actual immutable IR to native generator, independent of source resolution.
#[cfg(test)]
mod tests {
    use jai_codegen::target::{Features, NativeTarget, TargetOptions, TargetSelection, Triple};
    use jai_ir::*;
    use jai_types::{
        CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, ScalarType,
        TypeRegistry, Variadic,
    };
    fn program(instruction: SimdInstruction) -> Program {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let signature = types
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: Box::new([integer]),
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let mut simd = SimdBuilder::new(SimdFeatures::default()).unwrap();
        simd.instruction(instruction, &types).unwrap();
        ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![Procedure {
                id: ProcedureId::new(0),
                signature,
                parameters: vec![],
                locals: vec![],
                cleanups: vec![],
                body: Block {
                    flow: Flow::Terminates,
                    statements: vec![
                        Statement::Simd(simd.finish()),
                        Statement::Exit(Exit {
                            cleanups: vec![],
                            transfer: Transfer::ReturnInt(IntExpr::constant(
                                Integer::checked(IntegerType::S64, 42).unwrap(),
                            )),
                        }),
                    ],
                },
            }])
            .finish(EntryPoint::Int(ProcedureId::new(0)))
            .unwrap()
    }
    #[test]
    fn real_generator_enforces_trap_isa_and_preserves_continuation() {
        for (instruction, triple, features, expected) in [
            (
                SimdInstruction::DebugTrap,
                "x86_64-unknown-linux-gnu",
                "-sse2",
                "llvm.debugtrap",
            ),
            (
                SimdInstruction::Arm64DebugTrap,
                "aarch64-unknown-linux-gnu",
                "",
                "brk #1",
            ),
        ] {
            let program = program(instruction);
            assert!(matches!(
                jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
                jai_vm::Outcome::Failed(jai_vm::Error::RuntimeTrap)
            ));
            let target = NativeTarget::select(&TargetOptions {
                selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
                features: Features::new(features).unwrap(),
                ..TargetOptions::default()
            })
            .unwrap();
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            module.verify().unwrap();
            let llvm = module.print_to_string().to_string();
            assert!(llvm.contains(expected), "{llvm}");
            assert!(llvm.contains("ret i64 42"), "{llvm}");
            let object = std::env::temp_dir().join(format!(
                "jai-assembly-ir-{}-{}.o",
                std::process::id(),
                triple
            ));
            target.write_object(&module, &object).unwrap();
            assert!(!std::fs::read(&object).unwrap().is_empty());
            std::fs::remove_file(&object).unwrap();
            let opposite = if triple.starts_with("x86_64") {
                "aarch64-unknown-linux-gnu"
            } else {
                "x86_64-unknown-linux-gnu"
            };
            let target = NativeTarget::select(&TargetOptions {
                selection: TargetSelection::Triple(Triple::new(opposite).unwrap()),
                ..TargetOptions::default()
            })
            .unwrap();
            assert!(matches!(
                jai_codegen::lower_for_target(&context, &program, &target),
                Err(jai_codegen::Error::Simd(_))
            ));
        }
    }
}
