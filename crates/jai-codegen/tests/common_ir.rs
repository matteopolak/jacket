//! Native execution of shared checked IR without parsing source or using sema.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_ir::*;
use jai_types::*;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

fn integer(value: i128) -> IntExpr {
    IntExpr::constant(Integer::checked(IntegerType::S64, value).unwrap())
}
fn exit(values: Vec<ValueExpr>, cleanups: Vec<CleanupId>) -> Statement {
    Statement::Exit(Exit {
        cleanups,
        transfer: Transfer::ReturnValues(values),
    })
}
fn signature(types: &mut TypeRegistry, parameters: Vec<TypeId>, results: Vec<TypeId>) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into_boxed_slice(),
            results: results.into_boxed_slice(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: jai_types::Variadic::None,
        })
        .unwrap()
}
fn execute(program: &Program) -> i32 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-common-ir-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    let executable = scratch.0.join("program");
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
    let object = scratch.0.join("program.o");
    target.write_object(&module, &object).unwrap();
    let result = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stderr),
        module.print_to_string()
    );
    let mut process = Command::new(&executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = process.try_wait().unwrap() {
            return status
                .code()
                .expect("generated program terminated by signal");
        }
        if Instant::now() >= deadline {
            process.kill().unwrap();
            process.wait().unwrap();
            panic!("generated program exceeded deadline");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn nested_field_mutation_leaves_whole_record_copy_independent() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let inner = types.reserve_record(RecordKind::Struct);
    types.define_record(inner, [int]).unwrap();
    let outer = types.reserve_record(RecordKind::Struct);
    types.define_record(outer, [inner, int]).unwrap();
    let field_inner = types.field(outer, 0).unwrap().id;
    let field_value = types.field(inner, 0).unwrap().id;
    let pid = ProcedureId::new(0);
    let source = Local::new_typed(pid, 0, outer, &types).unwrap();
    let copy = Local::new_typed(pid, 1, outer, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let source_inner = places.field(source.place(), field_inner, &types).unwrap();
    let source_value = places.field(source_inner, field_value, &types).unwrap();
    let copy_inner = places.field(copy.place(), field_inner, &types).unwrap();
    let copy_value = places.field(copy_inner, field_value, &types).unwrap();
    let signature = signature(&mut types, vec![], vec![int]);
    let procedure = Procedure {
        id: pid,
        signature,
        parameters: vec![],
        locals: vec![source, copy],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(
                    source.place(),
                    ValueExpr::Record {
                        ty: outer,
                        fields: vec![
                            ValueExpr::Record {
                                ty: inner,
                                fields: vec![ValueExpr::Int(integer(17))],
                            },
                            ValueExpr::Int(integer(1)),
                        ],
                    },
                ),
                Statement::Store(copy.place(), ValueExpr::Load(source.place())),
                Statement::Store(source_value, ValueExpr::Int(integer(99))),
                exit(
                    vec![ValueExpr::Int(IntExpr::load(
                        IntPlace::try_from_place(copy_value, &types).unwrap(),
                    ))],
                    vec![],
                ),
            ],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .places(places.freeze())
        .finish(EntryPoint::Int(pid))
        .unwrap();
    assert_eq!(execute(&program), 17);
}

#[test]
fn native_selected_returns_preserve_valid_blocks_with_checked_casts() {
    for selected in [0, 1] {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let pid = ProcedureId::new(0);
        let local = Local::new_typed(pid, 0, int, &types).unwrap();
        let signature = signature(&mut types, vec![], vec![int]);
        let returning_arm = || Block {
            flow: Flow::FallsThrough,
            statements: vec![Statement::If(
                BoolExpr::Constant(true),
                Block {
                    flow: Flow::Terminates,
                    statements: vec![exit(
                        vec![ValueExpr::Int(IntExpr::new(
                            IntegerType::S64,
                            IntExprKind::Cast(
                                CastMode::Checked,
                                Box::new(IntExpr::constant(
                                    Integer::checked(IntegerType::U64, 42).unwrap(),
                                )),
                            ),
                        ))],
                        vec![],
                    )],
                },
                Block {
                    flow: Flow::FallsThrough,
                    statements: vec![],
                },
            )],
        };
        let procedure = Procedure {
            id: pid,
            signature,
            parameters: vec![],
            locals: vec![local],
            cleanups: vec![],
            body: Block {
                flow: Flow::Terminates,
                statements: vec![
                    Statement::Store(local.place(), ValueExpr::Int(integer(selected))),
                    Statement::If(
                        BoolExpr::CompareInts(
                            Relation::Equal,
                            Box::new(IntExpr::new(
                                IntegerType::S64,
                                IntExprKind::Value(Box::new(ValueExpr::Load(local.place()))),
                            )),
                            Box::new(integer(1)),
                        ),
                        returning_arm(),
                        returning_arm(),
                    ),
                    exit(vec![ValueExpr::Int(integer(1))], vec![]),
                ],
            },
        };
        let program = ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![procedure])
            .finish(EntryPoint::Int(pid))
            .unwrap();
        assert_eq!(execute(&program), 42);
    }
}

#[test]
fn aggregate_parameters_returns_and_deferred_cleanup_use_snapshots() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [int]).unwrap();
    let field = types.field(record, 0).unwrap().id;
    let helper_id = ProcedureId::new(0);
    let main_id = ProcedureId::new(1);
    let parameter = Local::new_typed(helper_id, 0, record, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let parameter_field = places.field(parameter.place(), field, &types).unwrap();
    let helper_signature = signature(&mut types, vec![record], vec![record]);
    let main_signature = signature(&mut types, vec![], vec![int]);
    let helper = Procedure {
        id: helper_id,
        signature: helper_signature,
        parameters: vec![parameter],
        locals: vec![parameter],
        cleanups: vec![
            Block {
                flow: Flow::FallsThrough,
                statements: vec![Statement::Store(
                    parameter_field,
                    ValueExpr::Int(integer(99)),
                )],
            }
            .into(),
        ],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(
                vec![ValueExpr::Load(parameter.place())],
                vec![CleanupId::new(0)],
            )],
        },
    };
    let returned = ValueExpr::Call {
        ty: record,
        call: Call::new(
            helper_id,
            vec![(
                ParameterId::new(0),
                ValueExpr::Record {
                    ty: record,
                    fields: vec![ValueExpr::Int(integer(23))],
                },
            )],
        ),
    };
    let field_value = ValueExpr::Field {
        base: Box::new(returned),
        field,
        ty: int,
    };
    let main = Procedure {
        id: main_id,
        signature: main_signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(
                vec![ValueExpr::Int(IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Value(Box::new(field_value)),
                ))],
                vec![],
            )],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![helper, main])
        .places(places.freeze())
        .finish(EntryPoint::Int(main_id))
        .unwrap();
    assert_eq!(execute(&program), 23);
}

#[test]
fn ordered_multiple_result_carrier_destructures_once() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [int]).unwrap();
    let field = types.field(record, 0).unwrap().id;
    let helper_id = ProcedureId::new(0);
    let main_id = ProcedureId::new(1);
    let helper_signature = signature(&mut types, vec![], vec![record, int]);
    let main_signature = signature(&mut types, vec![], vec![int]);
    let result = Local::new_typed(main_id, 0, record, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let result_field = places.field(result.place(), field, &types).unwrap();
    let helper = Procedure {
        id: helper_id,
        signature: helper_signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(
                vec![
                    ValueExpr::Record {
                        ty: record,
                        fields: vec![ValueExpr::Int(integer(31))],
                    },
                    ValueExpr::Int(integer(2)),
                ],
                vec![],
            )],
        },
    };
    let main = Procedure {
        id: main_id,
        signature: main_signature,
        parameters: vec![],
        locals: vec![result],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::CallResults {
                    call: Call::new(helper_id, vec![]),
                    destinations: vec![Some(result.place()), None],
                },
                exit(
                    vec![ValueExpr::Int(IntExpr::load(
                        IntPlace::try_from_place(result_field, &types).unwrap(),
                    ))],
                    vec![],
                ),
            ],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![helper, main])
        .places(places.freeze())
        .finish(EntryPoint::Int(main_id))
        .unwrap();
    assert_eq!(execute(&program), 31);
}

#[test]
fn enum_runtime_numeric_bridge_preserves_representation_and_nominal_storage() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let enumeration = types.reserve_enum(IntegerType::U8);
    types
        .define_enum(
            enumeration,
            [
                Integer::checked(IntegerType::U8, 1).unwrap(),
                Integer::checked(IntegerType::U8, 4).unwrap(),
            ],
        )
        .unwrap();
    let pid = ProcedureId::new(0);
    let local = Local::new_typed(pid, 0, enumeration, &types).unwrap();
    let signature = signature(&mut types, vec![], vec![int]);
    let flags = IntExpr::new(
        IntegerType::U8,
        IntExprKind::Binary(
            IntOp::BitOr,
            Box::new(IntExpr::constant(
                Integer::checked(IntegerType::U8, 1).unwrap(),
            )),
            Box::new(IntExpr::constant(
                Integer::checked(IntegerType::U8, 4).unwrap(),
            )),
        ),
    );
    let numeric = IntExpr::new(
        IntegerType::U8,
        IntExprKind::EnumValue(Box::new(ValueExpr::Load(local.place()))),
    );
    let widened = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Cast(CastMode::Checked, Box::new(numeric)),
    );
    let procedure = Procedure {
        id: pid,
        signature,
        parameters: vec![],
        locals: vec![local],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(
                    local.place(),
                    ValueExpr::EnumFromInt {
                        ty: enumeration,
                        value: flags,
                    },
                ),
                exit(vec![ValueExpr::Int(widened)], vec![]),
            ],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .finish(EntryPoint::Int(pid))
        .unwrap();
    assert_eq!(execute(&program), 5);
}

#[test]
fn default_record_values_use_their_typed_storage() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let boolean = types.scalar(ScalarType::Bool);
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [boolean, int]).unwrap();
    let field = types.field(record, 1).unwrap().id;
    let pid = ProcedureId::new(0);
    let local = Local::new_typed(pid, 0, record, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let number = places.field(local.place(), field, &types).unwrap();
    let signature = signature(&mut types, vec![], vec![int]);
    let procedure = Procedure {
        id: pid,
        signature,
        parameters: vec![],
        locals: vec![local],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(local.place(), ValueExpr::Zero(record)),
                exit(
                    vec![ValueExpr::Int(IntExpr::load(
                        IntPlace::try_from_place(number, &types).unwrap(),
                    ))],
                    vec![],
                ),
            ],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .places(places.freeze())
        .finish(EntryPoint::Int(pid))
        .unwrap();
    assert_eq!(execute(&program), 0);
}

#[test]
fn source_order_record_build_and_rvalue_field_evaluate_effects_once() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [int, int]).unwrap();
    let first = types.field(record, 0).unwrap().id;
    let second = types.field(record, 1).unwrap().id;
    let counter = Global::new(
        0,
        GlobalInitializer::Int(Integer::checked(IntegerType::S64, 0).unwrap()),
        &types,
    );
    let counter_place = IntPlace::try_from_place(counter.place(), &types).unwrap();
    let bump_id = ProcedureId::new(0);
    let create_id = ProcedureId::new(1);
    let main_id = ProcedureId::new(2);
    let bump_signature = signature(&mut types, vec![], vec![int]);
    let create_signature = signature(&mut types, vec![], vec![record]);
    let main_signature = signature(&mut types, vec![], vec![int]);
    let increment = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(
            IntOp::Add,
            Box::new(IntExpr::load(counter_place)),
            Box::new(integer(1)),
        ),
    );
    let bump = Procedure {
        id: bump_id,
        signature: bump_signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::StoreInt(counter_place, increment),
                exit(vec![ValueExpr::Int(IntExpr::load(counter_place))], vec![]),
            ],
        },
    };
    let bump_call = || ValueExpr::Call {
        ty: int,
        call: Call::new(bump_id, vec![]),
    };
    let create = Procedure {
        id: create_id,
        signature: create_signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(
                vec![ValueExpr::RecordBuild {
                    ty: record,
                    initializers: vec![(second, bump_call()), (first, bump_call())],
                }],
                vec![],
            )],
        },
    };
    let field = ValueExpr::Field {
        ty: int,
        field: first,
        base: Box::new(ValueExpr::Call {
            ty: record,
            call: Call::new(create_id, vec![]),
        }),
    };
    let projected = IntExpr::new(IntegerType::S64, IntExprKind::Value(Box::new(field)));
    let count = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(
            IntOp::Multiply,
            Box::new(IntExpr::load(counter_place)),
            Box::new(integer(10)),
        ),
    );
    let result = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(IntOp::Add, Box::new(projected), Box::new(count)),
    );
    let main = Procedure {
        id: main_id,
        signature: main_signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(vec![ValueExpr::Int(result)], vec![])],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![bump, create, main])
        .globals(vec![counter])
        .finish(EntryPoint::Int(main_id))
        .unwrap();
    assert_eq!(execute(&program), 22);
}

#[test]
fn nested_record_global_constants_and_field_stores_share_storage() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let enumeration = types.reserve_enum(IntegerType::U16);
    types
        .define_enum(
            enumeration,
            [Integer::checked(IntegerType::U16, 7).unwrap()],
        )
        .unwrap();
    let inner = types.reserve_record(RecordKind::Struct);
    types.define_record(inner, [enumeration, int]).unwrap();
    let outer = types.reserve_record(RecordKind::Struct);
    types.define_record(outer, [inner]).unwrap();
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: outer,
            kind: ConstantKind::Record(vec![ConstantValue {
                ty: inner,
                kind: ConstantKind::Record(vec![
                    ConstantValue {
                        ty: enumeration,
                        kind: ConstantKind::Enum(Integer::checked(IntegerType::U16, 7).unwrap()),
                    },
                    ConstantValue {
                        ty: int,
                        kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 40).unwrap()),
                    },
                ]),
            }]),
        },
        &types,
    )
    .unwrap();
    let mut places = PlaceRegistry::new();
    let nested = places
        .field(global.place(), types.field(outer, 0).unwrap().id, &types)
        .unwrap();
    let field = places
        .field(nested, types.field(inner, 1).unwrap().id, &types)
        .unwrap();
    let place = IntPlace::try_from_place(field, &types).unwrap();
    let pid = ProcedureId::new(0);
    let signature = signature(&mut types, vec![], vec![int]);
    let increment = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(
            IntOp::Add,
            Box::new(IntExpr::load(place)),
            Box::new(integer(1)),
        ),
    );
    let procedure = Procedure {
        id: pid,
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(field, ValueExpr::Int(increment)),
                exit(vec![ValueExpr::Int(IntExpr::load(place))], vec![]),
            ],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .globals(vec![global])
        .places(places.freeze())
        .finish(EntryPoint::Int(pid))
        .unwrap();
    assert_eq!(execute(&program), 41);
}

#[test]
fn foreign_prototype_and_procedure_pointer_use_sparse_checked_identities() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let external = ProcedureId::new(3);
    let main = ProcedureId::new(9);
    let foreign_signature = types
        .procedure(ProcedureType {
            parameters: vec![int].into_boxed_slice(),
            results: vec![int].into_boxed_slice(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let main_signature = signature(&mut types, vec![], vec![int]);
    let direct = ValueExpr::Call {
        call: Call::new(
            external,
            vec![(ParameterId::new(0), ValueExpr::Int(integer(-20)))],
        ),
        ty: int,
    };
    let indirect = ValueExpr::IndirectCall {
        inline_hint: jai_types::InlineHint::Automatic,
        callee: Box::new(ValueExpr::ProcedureValue {
            procedure: external,
            ty: foreign_signature,
        }),
        arguments: vec![(ParameterId::new(0), ValueExpr::Int(integer(-22)))],
        ty: int,
    };
    let result = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(
            IntOp::Add,
            Box::new(IntExpr::new(
                IntegerType::S64,
                IntExprKind::Value(Box::new(direct)),
            )),
            Box::new(IntExpr::new(
                IntegerType::S64,
                IntExprKind::Value(Box::new(indirect)),
            )),
        ),
    );
    let procedure = Procedure {
        id: main,
        signature: main_signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(vec![ValueExpr::Int(result)], vec![])],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .prototypes(vec![ProcedurePrototype {
            id: external,
            signature: foreign_signature,
            origin: PrototypeOrigin::Foreign {
                symbol: "labs".into(),
                library: None,
            },
        }])
        .finish(EntryPoint::Int(main))
        .unwrap();
    assert_eq!(execute(&program), 42);
}

#[test]
fn union_members_share_storage_and_copies_remain_independent() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [int, int]).unwrap();
    let pid = ProcedureId::new(0);
    let source = Local::new_typed(pid, 0, union, &types).unwrap();
    let copy = Local::new_typed(pid, 1, union, &types).unwrap();
    let first = types.field(union, 0).unwrap().id;
    let second = types.field(union, 1).unwrap().id;
    let mut places = PlaceRegistry::new();
    let source_first = places.field(source.place(), first, &types).unwrap();
    let copy_second = places.field(copy.place(), second, &types).unwrap();
    let signature = signature(&mut types, vec![], vec![int]);
    let procedure = Procedure {
        id: pid,
        signature,
        parameters: vec![],
        locals: vec![source, copy],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(
                    source.place(),
                    ValueExpr::Union {
                        ty: union,
                        field: first,
                        value: Box::new(ValueExpr::Int(integer(27))),
                    },
                ),
                Statement::Store(copy.place(), ValueExpr::Load(source.place())),
                Statement::Store(source_first, ValueExpr::Int(integer(99))),
                exit(
                    vec![ValueExpr::Int(IntExpr::load(
                        IntPlace::try_from_place(copy_second, &types).unwrap(),
                    ))],
                    vec![],
                ),
            ],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .places(places.freeze())
        .finish(EntryPoint::Int(pid))
        .unwrap();
    assert_eq!(execute(&program), 27);
}

#[test]
fn union_global_constants_preserve_nested_layout_and_string_pointer_relocations() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let string = types.string();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [string, int]).unwrap();
    let array = types.fixed_array(union, 2).unwrap();
    let outer = types.reserve_record(RecordKind::Struct);
    types.define_record(outer, [int, array]).unwrap();
    let string_field = types.field(union, 0).unwrap().id;
    let int_field = types.field(union, 1).unwrap().id;
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: outer,
            kind: ConstantKind::Record(vec![
                ConstantValue {
                    ty: int,
                    kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 5).unwrap()),
                },
                ConstantValue {
                    ty: array,
                    kind: ConstantKind::Array(vec![
                        ConstantValue {
                            ty: union,
                            kind: ConstantKind::Union {
                                field: string_field,
                                value: Box::new(ConstantValue {
                                    ty: string,
                                    kind: ConstantKind::StringBytes(b"hello".to_vec()),
                                }),
                            },
                        },
                        ConstantValue {
                            ty: union,
                            kind: ConstantKind::Union {
                                field: int_field,
                                value: Box::new(ConstantValue {
                                    ty: int,
                                    kind: ConstantKind::Int(
                                        Integer::checked(IntegerType::S64, 37).unwrap(),
                                    ),
                                }),
                            },
                        },
                    ]),
                },
            ]),
        },
        &types,
    )
    .unwrap();
    let mut places = PlaceRegistry::new();
    let array_place = places
        .field(global.place(), types.field(outer, 1).unwrap().id, &types)
        .unwrap();
    let first = places.index(array_place, integer(0), &types).unwrap();
    let second = places.index(array_place, integer(1), &types).unwrap();
    let first_string = places.field(first, string_field, &types).unwrap();
    let second_int = places.field(second, int_field, &types).unwrap();
    let count = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Value(Box::new(ValueExpr::SequenceField {
            base: Box::new(ValueExpr::Load(first_string)),
            field: SequenceField::Count,
            ty: int,
        })),
    );
    let result = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(
            IntOp::Add,
            Box::new(count),
            Box::new(IntExpr::load(
                IntPlace::try_from_place(second_int, &types).unwrap(),
            )),
        ),
    );
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let first_byte = IntExpr::new(
        IntegerType::U8,
        IntExprKind::Value(Box::new(ValueExpr::Index {
            check: CheckMode::Enabled,
            base: Box::new(ValueExpr::Load(first_string)),
            index: integer(0),
            ty: byte,
        })),
    );
    let widened = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Cast(CastMode::Checked, Box::new(first_byte)),
    );
    let byte_check = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(IntOp::Subtract, Box::new(widened), Box::new(integer(104))),
    );
    let result = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(IntOp::Add, Box::new(result), Box::new(byte_check)),
    );
    let pid = ProcedureId::new(0);
    let signature = signature(&mut types, vec![], vec![int]);
    let procedure = Procedure {
        id: pid,
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(vec![ValueExpr::Int(result)], vec![])],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .globals(vec![global])
        .places(places.freeze())
        .finish(EntryPoint::Int(pid))
        .unwrap();
    assert_eq!(execute(&program), 42);
}

#[test]
fn c_definition_parameters_and_returns_use_the_foreign_adapter() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [int, int, int]).unwrap();
    let function = ProcedureId::new(2);
    let main = ProcedureId::new(9);
    let local = Local::new_typed(function, 0, record, &types).unwrap();
    let c_signature = types
        .procedure(ProcedureType {
            parameters: vec![record].into_boxed_slice(),
            results: vec![record].into_boxed_slice(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let main_signature = signature(&mut types, vec![], vec![int]);
    let callback = Procedure {
        id: function,
        signature: c_signature,
        parameters: vec![local],
        locals: vec![local],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(vec![ValueExpr::Load(local.place())], vec![])],
        },
    };
    let call = ValueExpr::IndirectCall {
        inline_hint: jai_types::InlineHint::Automatic,
        callee: Box::new(ValueExpr::ProcedureValue {
            procedure: function,
            ty: c_signature,
        }),
        arguments: vec![(
            ParameterId::new(0),
            ValueExpr::Record {
                ty: record,
                fields: vec![
                    ValueExpr::Int(integer(11)),
                    ValueExpr::Int(integer(22)),
                    ValueExpr::Int(integer(42)),
                ],
            },
        )],
        ty: record,
    };
    let value = ValueExpr::Field {
        base: Box::new(call),
        field: types.field(record, 2).unwrap().id,
        ty: int,
    };
    let main_body = Procedure {
        id: main,
        signature: main_signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(vec![value], vec![])],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![callback, main_body])
        .finish(EntryPoint::Int(main))
        .unwrap();
    assert_eq!(execute(&program), 42);
}

#[test]
fn direct_and_indirect_call_results_capture_index_and_pointer_destinations_before_call() {
    for indirect in [false, true] {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let array = types.fixed_array(int, 3).unwrap();
        let pointer = types.pointer(int).unwrap();
        let values = Global::new_typed(
            0,
            ConstantValue {
                ty: array,
                kind: ConstantKind::Array(
                    [10, 20, 14]
                        .into_iter()
                        .map(|value| ConstantValue {
                            ty: int,
                            kind: ConstantKind::Int(
                                Integer::checked(IntegerType::S64, value).unwrap(),
                            ),
                        })
                        .collect(),
                ),
            },
            &types,
        )
        .unwrap();
        let index = Global::new_typed(
            1,
            ConstantValue {
                ty: int,
                kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 0).unwrap()),
            },
            &types,
        )
        .unwrap();
        let selected = Global::new_typed(
            2,
            ConstantValue {
                ty: pointer,
                kind: ConstantKind::Zero,
            },
            &types,
        )
        .unwrap();
        let mut places = PlaceRegistry::new();
        let first = places.index(values.place(), integer(0), &types).unwrap();
        let middle = places.index(values.place(), integer(1), &types).unwrap();
        let last = places.index(values.place(), integer(2), &types).unwrap();
        let indexed = places
            .index(
                values.place(),
                IntExpr::load(IntPlace::try_from_place(index.place(), &types).unwrap()),
                &types,
            )
            .unwrap();
        let dereferenced = places
            .dereference(ValueExpr::Load(selected.place()), &types)
            .unwrap();
        let callee_id = ProcedureId::new(0);
        let main_id = ProcedureId::new(1);
        let callee_signature = signature(&mut types, vec![], vec![int, int]);
        let main_signature = signature(&mut types, vec![], vec![int]);
        let callee = Procedure {
            id: callee_id,
            signature: callee_signature,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: Block {
                flow: Flow::Terminates,
                statements: vec![
                    Statement::Store(index.place(), ValueExpr::Int(integer(1))),
                    Statement::Store(
                        selected.place(),
                        ValueExpr::AddressOf {
                            place: last,
                            ty: pointer,
                        },
                    ),
                    exit(
                        vec![ValueExpr::Int(integer(7)), ValueExpr::Int(integer(8))],
                        vec![],
                    ),
                ],
            },
        };
        // Both destinations initially alias values[0]. Their addresses must be
        // captured before the callee changes the index and selected pointer.
        let destinations = vec![Some(indexed), Some(dereferenced)];
        let assignment = if indirect {
            Statement::IndirectCallResults {
                inline_hint: jai_types::InlineHint::Automatic,
                callee: Box::new(ValueExpr::ProcedureValue {
                    procedure: callee_id,
                    ty: callee_signature,
                }),
                arguments: vec![],
                destinations,
            }
        } else {
            Statement::CallResults {
                call: Call::new(callee_id, vec![]),
                destinations,
            }
        };
        let sum = IntExpr::new(
            IntegerType::S64,
            IntExprKind::Binary(
                IntOp::Add,
                Box::new(IntExpr::load(
                    IntPlace::try_from_place(first, &types).unwrap(),
                )),
                Box::new(IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Binary(
                        IntOp::Add,
                        Box::new(IntExpr::load(
                            IntPlace::try_from_place(middle, &types).unwrap(),
                        )),
                        Box::new(IntExpr::load(
                            IntPlace::try_from_place(last, &types).unwrap(),
                        )),
                    ),
                )),
            ),
        );
        let main = Procedure {
            id: main_id,
            signature: main_signature,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: Block {
                flow: Flow::Terminates,
                statements: vec![
                    Statement::Store(
                        selected.place(),
                        ValueExpr::AddressOf {
                            place: first,
                            ty: pointer,
                        },
                    ),
                    assignment,
                    exit(vec![ValueExpr::Int(sum)], vec![]),
                ],
            },
        };
        let program = ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![callee, main])
            .globals(vec![values, index, selected])
            .places(places.freeze())
            .finish(EntryPoint::Int(main_id))
            .unwrap();
        assert_eq!(execute(&program), 42, "indirect={indirect}");
    }
}
