//! Custom immutable graphs retain typed field addresses and native relocations.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions, TargetSelection, Triple},
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
fn program(
    mut types: TypeRegistry,
    places: PlaceRegistry,
    result: IntExpr,
    mut prelude: Vec<Statement>,
) -> Program {
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
    prelude.push(Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnValues(vec![ValueExpr::Int(result)]),
    }));
    let procedure = Procedure {
        id,
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: prelude,
        },
    };
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![
            procedure,
            Procedure {
                id: ProcedureId::new(1),
                signature,
                parameters: vec![],
                locals: vec![],
                cleanups: vec![],
                body: Block {
                    flow: Flow::Terminates,
                    statements: vec![Statement::Exit(Exit {
                        cleanups: vec![],
                        transfer: Transfer::ReturnValues(vec![ValueExpr::Int(IntExpr::constant(
                            Integer::wrapping(IntegerType::S64, 0),
                        ))]),
                    })],
                },
            },
        ])
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

fn heterogeneous_fixture(target: &NativeTarget) -> Program {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let int_pointer = types.pointer(int).unwrap();
    let byte_pointer = types.pointer(byte).unwrap();
    let record = types.reserve_record(RecordKind::Struct);
    let callable = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([int]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let type_cell = types.meta_type();
    let tag = types.reserve_enum(IntegerType::U32);
    types
        .define_enum(
            tag,
            [0, 1, 13].map(|v| Integer::wrapping(IntegerType::U32, v)),
        )
        .unwrap();
    let header = types.reserve_record(RecordKind::Struct);
    let descriptor = types.reserve_record(RecordKind::Struct);
    let boolean = types.scalar(ScalarType::Bool);
    types.define_record(header, [tag, int]).unwrap();
    types.define_record(descriptor, [header, boolean]).unwrap();
    types.bind_runtime_type_header(header).unwrap();
    types
        .define_record(record, [int, int_pointer, type_cell, callable])
        .unwrap();
    let context = jai_codegen::Context::create();
    let policy = jai_codegen::types::layout_policy(&context, &target.data).unwrap();
    let mut layouts = LayoutEngine::new(&types, policy);
    let layout = layouts.layout(record).unwrap().clone();
    let mut builder = StaticDataBuilder::new();
    let ReflectionReadiness::Ready(graph) =
        ReflectionGraph::build(&types, int, Some(policy), &ReflectionMetadata::default()).unwrap()
    else {
        panic!("scalar descriptor is ready")
    };
    let descriptor_object = builder.reserve(descriptor, &types).unwrap();
    builder
        .define_type_descriptor(
            descriptor_object,
            StaticValue {
                ty: descriptor,
                kind: StaticValueKind::Record(vec![
                    StaticValue {
                        ty: header,
                        kind: StaticValueKind::Record(vec![
                            StaticValue::constant(ConstantValue {
                                ty: tag,
                                kind: ConstantKind::Enum(Integer::wrapping(IntegerType::U32, 0)),
                            }),
                            scalar(int, IntegerType::S64, 8),
                        ]),
                    },
                    StaticValue::constant(ConstantValue {
                        ty: boolean,
                        kind: ConstantKind::Bool(true),
                    }),
                ]),
            },
            &graph,
            graph.root(),
            &types,
        )
        .unwrap();
    let referent = builder.reserve(int, &types).unwrap();
    let root = builder.reserve(record, &types).unwrap();
    let view = builder
        .byte_view(root, policy, 0, layout.size, &types)
        .unwrap();
    builder
        .define(referent, scalar(int, IntegerType::S64, 22))
        .unwrap();
    builder
        .define(
            root,
            StaticValue {
                ty: record,
                kind: StaticValueKind::Record(vec![
                    scalar(int, IntegerType::S64, 20),
                    StaticValue {
                        ty: int_pointer,
                        kind: StaticValueKind::Address(StaticAddress::new(referent)),
                    },
                    StaticValue {
                        ty: type_cell,
                        kind: StaticValueKind::Address(
                            StaticAddress::new(descriptor_object).project(StaticProjection::Field(
                                types.field(descriptor, 0).unwrap().id,
                            )),
                        ),
                    },
                    StaticValue::constant(ConstantValue {
                        ty: callable,
                        kind: ConstantKind::Procedure(ProcedureId::new(1)),
                    }),
                ]),
            },
        )
        .unwrap();
    let data = Arc::new(builder.finish(&types, StaticDataLimits::default()).unwrap());
    let mut places = PlaceRegistry::new();
    let bytes = address(&data, view.address(), byte_pointer);
    let number = places
        .dereference(
            ValueExpr::PointerCast {
                value: Box::new(bytes.clone()),
                ty: int_pointer,
                mode: CastMode::Checked,
            },
            &types,
        )
        .unwrap();
    let pointer_pointer = types.pointer(int_pointer).unwrap();
    let cell = ValueExpr::PointerOffset {
        pointer: Box::new(bytes.clone()),
        subtract: false,
        offset: IntExpr::constant(Integer::wrapping(
            IntegerType::S64,
            layout.field_offsets[1] as i128,
        )),
        ty: byte_pointer,
    };
    let relocation = places
        .dereference(
            ValueExpr::PointerCast {
                value: Box::new(cell),
                ty: pointer_pointer,
                mode: CastMode::Checked,
            },
            &types,
        )
        .unwrap();
    let referent = places
        .dereference(ValueExpr::Load(relocation), &types)
        .unwrap();
    let sum = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(
            IntOp::Add,
            Box::new(IntExpr::load(
                IntPlace::try_from_place(number, &types).unwrap(),
            )),
            Box::new(IntExpr::load(
                IntPlace::try_from_place(referent, &types).unwrap(),
            )),
        ),
    );
    let type_pointer = types.pointer(type_cell).unwrap();
    let callable_pointer = types.pointer(callable).unwrap();
    let mut cell = |offset, ty| {
        places
            .dereference(
                ValueExpr::PointerCast {
                    value: Box::new(ValueExpr::PointerOffset {
                        pointer: Box::new(bytes.clone()),
                        subtract: false,
                        offset: IntExpr::constant(Integer::wrapping(
                            IntegerType::S64,
                            offset as i128,
                        )),
                        ty: byte_pointer,
                    }),
                    ty,
                    mode: CastMode::Checked,
                },
                &types,
            )
            .unwrap()
    };
    let type_cell = cell(layout.field_offsets[2], type_pointer);
    let callable_cell = cell(layout.field_offsets[3], callable_pointer);
    let recovered_call = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Value(Box::new(ValueExpr::IndirectCall {
            inline_hint: InlineHint::default(),
            callee: Box::new(ValueExpr::Load(callable_cell)),
            arguments: vec![],
            ty: int,
        })),
    );
    let sum = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(IntOp::Add, Box::new(sum), Box::new(recovered_call)),
    );
    let header_pointer = types.pointer(header).unwrap();
    let recovered_descriptor = ValueExpr::TypeDescriptor {
        value: Box::new(ValueExpr::Load(type_cell)),
        ty: header_pointer,
    };
    let expected_descriptor = address(
        &data,
        StaticAddress::new(descriptor_object).project(StaticProjection::Field(
            types.field(descriptor, 0).unwrap().id,
        )),
        header_pointer,
    );
    let mismatch = BoolExpr::ComparePointers(
        Equality::NotEqual,
        Box::new(recovered_descriptor),
        Box::new(expected_descriptor),
    );
    program(
        types,
        places,
        sum,
        vec![Statement::If(
            mismatch,
            Block {
                flow: Flow::Terminates,
                statements: vec![Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnValues(vec![ValueExpr::Int(IntExpr::constant(
                        Integer::wrapping(IntegerType::S64, 1),
                    ))]),
                })],
            },
            Block {
                flow: Flow::FallsThrough,
                statements: vec![],
            },
        )],
    )
}

#[test]
fn immutable_byte_views_load_real_cells_data_type_and_callable_relocations_at_o0_and_o2() {
    let target = NativeTarget::select(&TargetOptions::default()).unwrap();
    check(&heterogeneous_fixture(&target));
}

#[test]
fn immutable_byte_views_execute_vm_and_emit_native_for_32_and_64_bit_targets() {
    for triple in ["wasm32-unknown-unknown", "wasm64-unknown-unknown"] {
        for bitcode in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
            let target = NativeTarget::select(&TargetOptions {
                selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
                optimization: Optimization {
                    bitcode,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
            let program = heterogeneous_fixture(&target);
            let policy = target.layout_policy().unwrap();
            let mut vm = jai_vm::Vm::new_with_target(
                &program,
                jai_vm::NoEffects,
                jai_vm::Limits::default(),
                jai_vm::ByteTarget {
                    policy,
                    endian: jai_vm::Endian::Little,
                },
            )
            .unwrap();
            let outcome = vm.execute(ProcedureId::new(0), vec![]).outcome;
            assert!(
                matches!(outcome, jai_vm::Outcome::Complete(values) if values[0].integer().unwrap().value() == 42),
                "{triple} {bitcode:?}"
            );
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            module.verify().unwrap();
        }
    }
}
