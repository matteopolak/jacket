//! Custom immutable graphs retain typed field addresses and native relocations.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
    types::TypeLowerer,
};
use jai_ir::*;
use jai_types::*;
use std::{
    fs,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

fn scalar(ty: TypeId, integer: IntegerType, value: i128) -> StaticValue {
    StaticValue::constant(ConstantValue {
        ty,
        kind: ConstantKind::Int(Integer::checked(integer, value).unwrap()),
    })
}
fn address(data: &Arc<StaticData>, address: StaticAddress, ty: TypeId) -> ValueExpr {
    ValueExpr::StaticAddress {
        data: data.clone(),
        address,
        ty,
    }
}
fn program(mut types: TypeRegistry, places: PlaceRegistry, result: IntExpr) -> Program {
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let id = ProcedureId::new(0);
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
    let procedure = Procedure {
        id,
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnValues(vec![ValueExpr::Int(result)]),
            })],
        },
    };
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .places(places.freeze())
        .finish(EntryPoint::Int(id))
        .unwrap()
}
fn check(program: &Program) {
    let outcome = jai_vm::execute(program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-static-layout-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    for bitcode in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target = NativeTarget::select(&TargetOptions {
            optimization: Optimization {
                bitcode,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();
        let context = jai_codegen::Context::create();
        let mut lowerer = TypeLowerer::with_target(&context, program.types(), &target.data);
        for (id, kind) in program.types().iter() {
            if matches!(kind, TypeKind::Record(_) | TypeKind::FixedArray { .. }) {
                lowerer.verify_layout(id, &target.data).unwrap();
            }
        }
        let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let mut command = native_tools::clang_command();
        command.arg(&object).arg("-o").arg(&executable);
        if cfg!(target_os = "macos") {
            command.arg("-Wl,-no_fixup_chains");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            module.print_to_string()
        );
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(
                    status.code(),
                    Some(42),
                    "{bitcode:?}\n{}",
                    module.print_to_string()
                );
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("static-layout fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn cyclic_static_record_relocations_follow_custom_fields_and_array_stride() {
    for layout in [
        RecordLayout {
            packed: true,
            ..Default::default()
        },
        RecordLayout {
            packed: true,
            field_alignments: Box::new([None, Some(4), Some(4)]),
            ..Default::default()
        },
        RecordLayout {
            packed: true,
            minimum_alignment: Some(32),
            ..Default::default()
        },
    ] {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let int_pointer = types.pointer(int).unwrap();
        let pointer_pointer = types.pointer(int_pointer).unwrap();
        let record = types.reserve_record(RecordKind::Struct);
        types
            .define_record_with_layout(record, [byte, int, int_pointer], layout)
            .unwrap();
        let number = types.field(record, 1).unwrap().id;
        let link = types.field(record, 2).unwrap().id;
        let array = types.fixed_array(record, 2).unwrap();
        let envelope = types.reserve_record(RecordKind::Struct);
        types
            .define_record_with_layout(
                envelope,
                [byte, array],
                RecordLayout {
                    packed: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let values = types.field(envelope, 1).unwrap().id;
        let mut builder = StaticDataBuilder::new();
        let root = builder.reserve(envelope, &types).unwrap();
        let elements = StaticAddress::new(root).project(StaticProjection::Field(values));
        let item = |index| elements.project(StaticProjection::Index(index));
        let mut records = vec![];
        for (index, value) in [20, 22].into_iter().enumerate() {
            records.push(StaticValue {
                ty: record,
                kind: StaticValueKind::Record(vec![
                    scalar(byte, IntegerType::U8, 7),
                    scalar(int, IntegerType::S64, value),
                    StaticValue {
                        ty: int_pointer,
                        kind: StaticValueKind::Address(
                            item(index as u64).project(StaticProjection::Field(number)),
                        ),
                    },
                ]),
            });
        }
        builder
            .define(
                root,
                StaticValue {
                    ty: envelope,
                    kind: StaticValueKind::Record(vec![
                        scalar(byte, IntegerType::U8, 9),
                        StaticValue {
                            ty: array,
                            kind: StaticValueKind::Array(records),
                        },
                    ]),
                },
            )
            .unwrap();
        let data = Arc::new(builder.finish(&types, StaticDataLimits::default()).unwrap());
        let mut places = PlaceRegistry::new();
        let indirect = places
            .dereference(
                address(
                    &data,
                    item(0).project(StaticProjection::Field(link)),
                    pointer_pointer,
                ),
                &types,
            )
            .unwrap();
        let first = places
            .dereference(ValueExpr::Load(indirect), &types)
            .unwrap();
        let second = places
            .dereference(
                address(
                    &data,
                    item(1).project(StaticProjection::Field(number)),
                    int_pointer,
                ),
                &types,
            )
            .unwrap();
        let result = IntExpr::new(
            IntegerType::S64,
            IntExprKind::Binary(
                IntOp::Add,
                Box::new(IntExpr::load(
                    IntPlace::try_from_place(first, &types).unwrap(),
                )),
                Box::new(IntExpr::load(
                    IntPlace::try_from_place(second, &types).unwrap(),
                )),
            ),
        );
        check(&program(types, places, result));
    }
}

#[test]
fn static_custom_union_member_relocations_preserve_zero_offset() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let union = types.reserve_record(RecordKind::Union);
    types
        .define_record_with_layout(
            union,
            [byte, int],
            RecordLayout {
                packed: true,
                minimum_alignment: Some(16),
                ..Default::default()
            },
        )
        .unwrap();
    let int_pointer = types.pointer(int).unwrap();
    let byte_pointer = types.pointer(byte).unwrap();
    let union_pointer = types.pointer(union).unwrap();
    let pointer_pointer = types.pointer(int_pointer).unwrap();
    let holder = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            holder,
            [byte, int_pointer],
            RecordLayout {
                packed: true,
                ..Default::default()
            },
        )
        .unwrap();
    let mut builder = StaticDataBuilder::new();
    let root = builder.reserve(union, &types).unwrap();
    let member = StaticAddress::new(root)
        .project(StaticProjection::Field(types.field(union, 1).unwrap().id));
    let relocation = builder.reserve(holder, &types).unwrap();
    builder
        .define(
            root,
            StaticValue::constant(ConstantValue {
                ty: union,
                kind: ConstantKind::Zero,
            }),
        )
        .unwrap();
    builder
        .define(
            relocation,
            StaticValue {
                ty: holder,
                kind: StaticValueKind::Record(vec![
                    scalar(byte, IntegerType::U8, 7),
                    StaticValue {
                        ty: int_pointer,
                        kind: StaticValueKind::Address(member.clone()),
                    },
                ]),
            },
        )
        .unwrap();
    let data = Arc::new(builder.finish(&types, StaticDataLimits::default()).unwrap());
    let base = address(&data, StaticAddress::new(root), union_pointer);
    let field = address(&data, member, int_pointer);
    let difference = IntExpr::new(
        IntegerType::S64,
        IntExprKind::PointerDifference {
            left: Box::new(ValueExpr::PointerCast {
                value: Box::new(field),
                ty: byte_pointer,
                mode: CastMode::Unchecked,
            }),
            right: Box::new(ValueExpr::PointerCast {
                value: Box::new(base),
                ty: byte_pointer,
                mode: CastMode::Unchecked,
            }),
        },
    );
    let mut places = PlaceRegistry::new();
    let slot = places
        .dereference(
            address(
                &data,
                StaticAddress::new(relocation)
                    .project(StaticProjection::Field(types.field(holder, 1).unwrap().id)),
                pointer_pointer,
            ),
            &types,
        )
        .unwrap();
    let target = places.dereference(ValueExpr::Load(slot), &types).unwrap();
    let zero = IntExpr::load(IntPlace::try_from_place(target, &types).unwrap());
    let result = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(
            IntOp::Add,
            Box::new(IntExpr::constant(
                Integer::checked(IntegerType::S64, 42).unwrap(),
            )),
            Box::new(IntExpr::new(
                IntegerType::S64,
                IntExprKind::Binary(IntOp::Add, Box::new(difference), Box::new(zero)),
            )),
        ),
    );
    check(&program(types, places, result));
}
