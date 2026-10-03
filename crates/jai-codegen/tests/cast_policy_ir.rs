//! Isolated checked-IR regression while source semantic adapters are migrating.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_ir::*;
use jai_types::*;
use std::{
    fs,
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn truncation_keeps_unsigned_low_bits_in_vm_and_o0_o2_native_code() {
    let root = std::env::temp_dir().join(format!("jai-cast-policy-ir-{}", std::process::id()));
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(root);
    fs::create_dir_all(&scratch.0).unwrap();
    for value in [298, -214] {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let signature = types
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: Box::new([int]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let id = ProcedureId::new(0);
        let local = Local::new_typed(id, 0, int, &types).unwrap();
        let narrowed = IntExpr::new(
            IntegerType::U8,
            IntExprKind::Cast(
                CastMode::Truncate,
                Box::new(IntExpr::load(
                    IntPlace::try_from_place(local.place(), &types).unwrap(),
                )),
            ),
        );
        let widened = IntExpr::new(
            IntegerType::S64,
            IntExprKind::Cast(CastMode::Checked, Box::new(narrowed)),
        );
        let procedure = Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![local],
            cleanups: vec![],
            body: Block {
                flow: Flow::Terminates,
                statements: vec![
                    Statement::Store(
                        local.place(),
                        ValueExpr::Int(IntExpr::constant(
                            Integer::checked(IntegerType::S64, value).unwrap(),
                        )),
                    ),
                    Statement::Exit(Exit {
                        cleanups: vec![],
                        transfer: Transfer::ReturnValues(vec![ValueExpr::Int(widened)]),
                    }),
                ],
            },
        };
        let program = ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![procedure])
            .finish(EntryPoint::Int(id))
            .unwrap();
        let jai_vm::Outcome::Complete(values) =
            jai_vm::execute(&program, jai_vm::Limits::default()).outcome
        else {
            panic!("VM truncation failed")
        };
        assert_eq!(values[0].integer().unwrap().value(), 42);
        let context = jai_codegen::Context::create();
        let target = jai_codegen::target::NativeTarget::new().unwrap();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let input = scratch.0.join("program.ll");
        fs::write(&input, module.print_to_string().to_string()).unwrap();
        for optimization in ["-O0", "-O2"] {
            let output = scratch.0.join(format!("program-{value}-{optimization}"));
            let compile = native_tools::clang_command()
                .args(["-x", "ir", optimization])
                .arg(&input)
                .arg("-o")
                .arg(&output)
                .output()
                .unwrap();
            assert!(
                compile.status.success(),
                "{}",
                String::from_utf8_lossy(&compile.stderr)
            );
            let mut child = Command::new(output).spawn().unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if let Some(status) = child.try_wait().unwrap() {
                    assert_eq!(status.code(), Some(42));
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    panic!("native cast deadline");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}
