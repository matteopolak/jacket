//! Typed unsigned index and 32-bit object contracts independent of source binding.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_ir::*;
use jai_types::*;
use std::{fs, process::Command};

fn number(ty: IntegerType, value: i128) -> ValueExpr {
    ValueExpr::Int(IntExpr::constant(Integer::checked(ty, value).unwrap()))
}
fn fixture(index: u64) -> Program {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let unsigned = types.scalar(ScalarType::Int(IntegerType::U64));
    let slice = types.slice(int).unwrap();
    let array = types.fixed_array(int, 3).unwrap();
    let reader = ProcedureId::new(0);
    let main = ProcedureId::new(1);
    let reader_signature = types
        .procedure(ProcedureType {
            parameters: vec![slice, unsigned].into(),
            results: vec![int].into(),
            variadic: Variadic::None,
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
        })
        .unwrap();
    let main_signature = types
        .procedure(ProcedureType {
            parameters: vec![].into(),
            results: vec![int].into(),
            variadic: Variadic::None,
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
        })
        .unwrap();
    let values = Local::new_typed(reader, 0, slice, &types).unwrap();
    let selected = Local::new_typed(reader, 1, unsigned, &types).unwrap();
    let reader_body = Procedure {
        id: reader,
        signature: reader_signature,
        parameters: vec![values, selected],
        locals: vec![values, selected],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnValues(vec![ValueExpr::Index {
                    base: Box::new(ValueExpr::Load(values.place())),
                    index: IntExpr::load(
                        IntPlace::try_from_place(selected.place(), &types).unwrap(),
                    ),
                    ty: int,
                    check: CheckMode::Enabled,
                }]),
            })],
        },
    };
    let main_body = Procedure {
        id: main,
        signature: main_signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnValues(vec![ValueExpr::Call {
                    ty: int,
                    call: Call::new(
                        reader,
                        vec![
                            (
                                ParameterId::new(0),
                                ValueExpr::ArrayView {
                                    ty: slice,
                                    array: Box::new(ValueExpr::Array {
                                        ty: array,
                                        elements: vec![
                                            number(IntegerType::S64, 20),
                                            number(IntegerType::S64, 22),
                                            number(IntegerType::S64, 42),
                                        ],
                                    }),
                                },
                            ),
                            (
                                ParameterId::new(1),
                                number(IntegerType::U64, i128::from(index)),
                            ),
                        ],
                    ),
                }]),
            })],
        },
    };
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![reader_body, main_body])
        .finish(EntryPoint::Int(main))
        .unwrap()
}
struct Scratch(std::path::PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-index-width-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn native(program: &Program) -> std::process::ExitStatus {
    let scratch = Scratch::new();
    let context = jai_codegen::Context::create();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
    let object = scratch.0.join("index.o");
    target.write_object(&module, &object).unwrap();
    let output = scratch.0.join("index");
    let linked = native_tools::clang_command()
        .arg(object)
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let mut process = Command::new(output).spawn().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(status) = process.try_wait().unwrap() {
            return status;
        }
        if std::time::Instant::now() >= deadline {
            process.kill().unwrap();
            process.wait().unwrap();
            panic!("generated index fixture timed out");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
#[test]
fn unsigned_index_reads_the_same_element_in_native_and_vm() {
    let program = fixture(2);
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("{:?}", execution.outcome)
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
    assert_eq!(native(&program).code(), Some(42));
}
#[test]
fn maximum_unsigned_index_is_a_bounds_failure_before_any_signed_cast() {
    let program = fixture(u64::MAX);
    assert_eq!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::OutOfBounds {
            index: usize::MAX,
            length: 3
        })
    );
    let status = native(&program);
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(status.signal().is_some());
    }
}
#[test]
fn actual_32_bit_object_checks_stride_before_pointer_index_normalization() {
    use jai_codegen::target::{NativeTarget, TargetOptions, TargetSelection, Triple};
    let scratch = Scratch::new();
    let target = NativeTarget::select(&TargetOptions {
        selection: TargetSelection::Triple(Triple::new("i386-unknown-linux-gnu").unwrap()),
        ..Default::default()
    })
    .unwrap();
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &fixture(2), &target).unwrap();
    let ir = module.print_to_string().to_string();
    // Read the emitted LLVM module; no LLVM text is constructed or parsed.
    assert!(
        ir.contains("index.unsigned.displacement.fits = icmp ule i64"),
        "{ir}"
    );
    assert!(
        ir.contains(", 536870911"),
        "the eight-byte stride must bound 32-bit displacement: {ir}"
    );
    assert!(
        ir.contains("index.target.width = trunc i64") && ir.contains(" to i32"),
        "{ir}"
    );
    let object = scratch.0.join("index32.o");
    target.write_object(&module, &object).unwrap();
    let bytes = fs::read(object).unwrap();
    assert_eq!(&bytes[..5], b"\x7fELF\x01");
}

#[test]
fn fixed_array_storage_must_fit_the_selected_target_address_domain() {
    use jai_codegen::target::{NativeTarget, TargetOptions, TargetSelection, Triple};
    let target = NativeTarget::select(&TargetOptions {
        selection: TargetSelection::Triple(Triple::new("i386-unknown-linux-gnu").unwrap()),
        ..Default::default()
    })
    .unwrap();
    let mut registry = TypeRegistry::new();
    let element = registry.scalar(ScalarType::Int(IntegerType::S64));
    let array = registry.fixed_array(element, 536_870_912).unwrap();
    let types = registry.freeze().unwrap();
    let context = jai_codegen::Context::create();
    let mut lowerer = jai_codegen::types::TypeLowerer::with_target(&context, &types, &target.data);
    assert!(matches!(
        lowerer.basic(array),
        Err(jai_codegen::types::Error::ArrayStorageTooLarge {
            ty, count:536_870_912, stride:8, address_bits:32
        }) if ty == array
    ));
}

#[test]
fn dynamic_descriptor_uses_the_adopted_nominal_allocator_storage() {
    use jai_codegen::target::{NativeTarget, TargetOptions, TargetSelection, Triple};
    let mut registry = TypeRegistry::new();
    let mode = registry.reserve_enum(IntegerType::S64);
    registry
        .define_enum(mode, AllocatorMode::ALL.map(AllocatorMode::value))
        .unwrap();
    let size = registry.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = registry.pointer(registry.void()).unwrap();
    let procedure = registry
        .procedure(ProcedureType {
            parameters: vec![mode, size, size, pointer, pointer].into(),
            results: vec![pointer].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let allocator = registry.reserve_record(RecordKind::Struct);
    registry
        .define_record(allocator, [procedure, pointer])
        .unwrap();
    registry.bind_allocator(allocator, mode).unwrap();
    let dynamic = registry.dynamic_array(size).unwrap();
    let types = registry.freeze().unwrap();
    for triple in ["i386-unknown-linux-gnu", "x86_64-unknown-linux-gnu"] {
        let target = NativeTarget::select(&TargetOptions {
            selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
            ..Default::default()
        })
        .unwrap();
        let context = jai_codegen::Context::create();
        let mut lowerer =
            jai_codegen::types::TypeLowerer::with_target(&context, &types, &target.data);
        let descriptor = lowerer.basic(dynamic).unwrap().into_struct_type();
        assert_eq!(
            descriptor.get_field_type_at_index(3),
            Some(lowerer.basic(allocator).unwrap())
        );
        lowerer.verify_layout(dynamic, &target.data).unwrap();
        lowerer.verify_layout(allocator, &target.data).unwrap();
    }
}
