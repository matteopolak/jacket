//! Runtime roots and dependencies over the shared checked IR, without frontend binding.
#[path = "support/native_tools.rs"]
mod native_tools;
use inkwell::values::BasicValue;
use jai_codegen::native_reachability::{Error, Publication, Reachable};
use jai_ir::*;
use jai_types::*;

fn signature(types: &mut TypeRegistry, parameters: Vec<TypeId>, result: TypeId) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into_boxed_slice(),
            results: vec![result].into_boxed_slice(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}
fn integer(n: i128) -> ValueExpr {
    ValueExpr::Int(IntExpr::constant(
        Integer::checked(IntegerType::S64, n).unwrap(),
    ))
}
fn returning(id: ProcedureId, signature: TypeId, value: ValueExpr) -> Procedure {
    Procedure {
        id,
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnValues(vec![value]),
            })],
        },
    }
}
fn compiler_fixture() -> Program {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = signature(&mut types, vec![], int);
    let compiler = ProcedureId::new(1);
    let helper = ProcedureId::new(3);
    let entry = ProcedureId::new(9);
    ProgramBuilder::new(types.freeze().unwrap())
        .prototypes(vec![ProcedurePrototype {
            id: compiler,
            signature,
            origin: PrototypeOrigin::Compiler,
        }])
        .procedures(vec![
            returning(
                helper,
                signature,
                ValueExpr::Call {
                    call: Call::new(compiler, vec![]),
                    ty: int,
                },
            ),
            returning(entry, signature, integer(42)),
        ])
        .finish(EntryPoint::Int(entry))
        .unwrap()
}

#[test]
fn unused_compiler_helpers_are_omitted_and_library_publication_is_explicit() {
    let program = compiler_fixture();
    let reachable = Reachable::executable(program.library(), program.entry()).unwrap();
    assert!(reachable.contains(ProcedureId::new(9)));
    assert!(!reachable.contains(ProcedureId::new(3)));
    assert!(!reachable.contains(ProcedureId::new(1)));
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower(&context, &program).unwrap();
    assert!(module.get_function("jai.p9").is_some());
    assert!(module.get_function("jai.p3").is_none());
    let selected = jai_codegen::lower_library(
        &context,
        program.library(),
        &Publication::Selected(vec![ProcedureId::new(9)]),
    )
    .unwrap();
    assert!(selected.get_function("jai.p9").is_some());
    assert!(selected.get_function("main").is_none());
    let error = Reachable::library(program.library(), &Publication::AllBodies).unwrap_err();
    let Error::CompilerRequest { chain } = error else {
        panic!("compiler diagnostic")
    };
    assert_eq!(chain, [ProcedureId::new(3), ProcedureId::new(1)]);
    assert!(matches!(
        Reachable::library(
            program.library(),
            &Publication::Selected(vec![ProcedureId::new(99)])
        ),
        Err(Error::MissingProcedure(_)),
    ));
}

#[test]
fn typed_procedure_values_keep_indirect_callback_bodies_in_the_native_object() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let callback_signature = signature(&mut types, vec![], int);
    let wrapper_signature = signature(&mut types, vec![callback_signature], int);
    let callback = ProcedureId::new(2);
    let wrapper = ProcedureId::new(4);
    let unused = ProcedureId::new(6);
    let entry = ProcedureId::new(8);
    let local = Local::new_typed(wrapper, 0, callback_signature, &types).unwrap();
    let mut wrapper_body = returning(
        wrapper,
        wrapper_signature,
        ValueExpr::IndirectCall {
            inline_hint: InlineHint::Automatic,
            callee: Box::new(ValueExpr::Load(local.place())),
            arguments: vec![],
            ty: int,
        },
    );
    wrapper_body.locals = vec![local];
    wrapper_body.parameters = vec![local];
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![
            returning(callback, callback_signature, integer(42)),
            wrapper_body,
            returning(unused, callback_signature, integer(99)),
            returning(
                entry,
                callback_signature,
                ValueExpr::Call {
                    ty: int,
                    call: Call::new(
                        wrapper,
                        vec![(
                            ParameterId::new(0),
                            ValueExpr::ProcedureValue {
                                procedure: callback,
                                ty: callback_signature,
                            },
                        )],
                    ),
                },
            ),
        ])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    let context = jai_codegen::Context::create();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    for id in [callback, wrapper, entry] {
        assert!(
            module
                .get_function(&format!("jai.p{}", id.index()))
                .is_some()
        );
    }
    assert!(module.get_function("jai.p6").is_none());
    let all =
        jai_codegen::lower_library(&context, program.library(), &Publication::AllBodies).unwrap();
    assert!(all.get_function("jai.p6").is_some());
    assert!(all.get_function("main").is_none());
    let directory =
        std::env::temp_dir().join(format!("jai-native-callback-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let object = directory.join("callback.o");
    let executable = directory.join("callback");
    target.write_object(&module, &object).unwrap();
    let output = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::process::Command::new(executable)
            .status()
            .unwrap()
            .code(),
        Some(42)
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn global_procedure_constants_keep_bodies_and_foreign_prototypes_reachable() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let callback_signature = signature(&mut types, vec![], int);
    let foreign_signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: vec![int].into_boxed_slice(),
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let callback = ProcedureId::new(2);
    let external = ProcedureId::new(3);
    let entry = ProcedureId::new(4);
    let unused = ProcedureId::new(5);
    let globals = vec![
        Global::new_typed(
            0,
            ConstantValue {
                ty: callback_signature,
                kind: ConstantKind::Procedure(callback),
            },
            &types,
        )
        .unwrap(),
        Global::new_typed(
            1,
            ConstantValue {
                ty: foreign_signature,
                kind: ConstantKind::Procedure(external),
            },
            &types,
        )
        .unwrap(),
    ];
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .globals(globals)
        .prototypes(vec![ProcedurePrototype {
            id: external,
            signature: foreign_signature,
            origin: PrototypeOrigin::Foreign {
                symbol: "source_callback_fixture".into(),
                library: None,
            },
        }])
        .procedures(vec![
            returning(callback, callback_signature, integer(42)),
            returning(entry, callback_signature, integer(42)),
            returning(unused, callback_signature, integer(99)),
        ])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    let reachable = Reachable::executable(program.library(), program.entry()).unwrap();
    assert!(reachable.contains(callback));
    assert!(reachable.contains(external));
    assert!(!reachable.contains(unused));
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower(&context, &program).unwrap();
    assert!(module.get_function("jai.p2").is_some());
    assert!(module.get_function("source_callback_fixture").is_some());
    assert!(module.get_function("jai.p5").is_none());
    assert!(
        module
            .get_global("jai.g0")
            .unwrap()
            .get_initializer()
            .unwrap()
            .is_const()
    );
}

#[test]
fn static_procedure_relocations_are_checked_and_keep_callback_bodies() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let callback_signature = signature(&mut types, vec![], int);
    let callback_pointer = types.pointer(callback_signature).unwrap();
    let callback = ProcedureId::new(2);
    let entry = ProcedureId::new(4);
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(callback_signature, &types).unwrap();
    builder
        .define(
            object,
            StaticValue::constant(ConstantValue {
                ty: callback_signature,
                kind: ConstantKind::Procedure(callback),
            }),
        )
        .unwrap();
    let data = std::sync::Arc::new(builder.finish(&types, StaticDataLimits::default()).unwrap());
    let mut places = PlaceRegistry::new();
    let callee = places
        .dereference(
            ValueExpr::StaticAddress {
                data,
                address: StaticAddress::new(object),
                ty: callback_pointer,
            },
            &types,
        )
        .unwrap();
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .places(places.freeze())
        .procedures(vec![
            returning(callback, callback_signature, integer(42)),
            returning(
                entry,
                callback_signature,
                ValueExpr::IndirectCall {
                    inline_hint: InlineHint::Automatic,
                    callee: Box::new(ValueExpr::Load(callee)),
                    arguments: vec![],
                    ty: int,
                },
            ),
        ])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    assert!(
        matches!(outcome, jai_vm::Outcome::Complete(values) if values[0].integer().unwrap().value() == 42)
    );
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower(&context, &program).unwrap();
    assert!(module.get_function("jai.p2").is_some());
    assert!(
        module
            .print_to_string()
            .to_string()
            .contains("constant ptr @jai.p2")
    );
}

#[test]
fn compiler_only_procedure_constants_are_runtime_dependencies() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let callback_signature = signature(&mut types, vec![], int);
    let compiler = ProcedureId::new(2);
    let entry = ProcedureId::new(4);
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: callback_signature,
            kind: ConstantKind::Procedure(compiler),
        },
        &types,
    )
    .unwrap();
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .globals(vec![global])
        .prototypes(vec![ProcedurePrototype {
            id: compiler,
            signature: callback_signature,
            origin: PrototypeOrigin::Compiler,
        }])
        .procedures(vec![returning(entry, callback_signature, integer(42))])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    assert!(
        matches!(Reachable::executable(program.library(), program.entry()),
        Err(Error::CompilerRequest { chain }) if chain == [compiler])
    );
}

#[test]
fn expression_binding_producers_preserve_compiler_only_callback_roots() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let callback_signature = signature(&mut types, vec![], int);
    let compiler = ProcedureId::new(2);
    let entry = ProcedureId::new(4);
    let binding = ExpressionBindingId::new(entry, 0);
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .prototypes(vec![ProcedurePrototype {
            id: compiler,
            signature: callback_signature,
            origin: PrototypeOrigin::Compiler,
        }])
        .procedures(vec![returning(
            entry,
            callback_signature,
            ValueExpr::Bind {
                bindings: vec![(
                    binding,
                    ValueExpr::ProcedureValue {
                        procedure: compiler,
                        ty: callback_signature,
                    },
                )],
                body: Box::new(integer(42)),
                ty: int,
            },
        )])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    assert!(matches!(
        Reachable::executable(program.library(), program.entry()),
        Err(Error::CompilerRequest { chain }) if chain == [entry, compiler]
    ));
}

#[test]
fn expression_binding_callback_producers_keep_actual_targets_in_vm_and_native_code() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let callback_signature = signature(&mut types, vec![], int);
    let callback = ProcedureId::new(2);
    let entry = ProcedureId::new(4);
    let binding = ExpressionBindingId::new(entry, 0);
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![
            returning(callback, callback_signature, integer(42)),
            returning(
                entry,
                callback_signature,
                ValueExpr::Bind {
                    bindings: vec![(
                        binding,
                        ValueExpr::ProcedureValue {
                            procedure: callback,
                            ty: callback_signature,
                        },
                    )],
                    body: Box::new(ValueExpr::IndirectCall {
                        inline_hint: InlineHint::Automatic,
                        callee: Box::new(ValueExpr::Bound {
                            binding,
                            ty: callback_signature,
                        }),
                        arguments: vec![],
                        ty: int,
                    }),
                    ty: int,
                },
            ),
        ])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    assert!(
        Reachable::executable(program.library(), program.entry())
            .unwrap()
            .contains(callback)
    );
    assert!(matches!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Complete(values) if values[0].integer().unwrap().value() == 42
    ));
    let context = jai_codegen::Context::create();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    module.verify().unwrap();
    assert!(module.get_function("jai.p2").is_some());
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(
        std::env::temp_dir().join(format!("jai-expression-callback-{}", std::process::id())),
    );
    std::fs::create_dir(&fixture.0).unwrap();
    let object = fixture.0.join("callback.o");
    let executable = fixture.0.join("callback");
    target.write_object(&module, &object).unwrap();
    let linked = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let mut child = std::process::Command::new(executable).spawn().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(42));
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("generated expression-binding callback exceeded five seconds");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn source_contracts_remain_typed_object_declarations_and_reject_executable_demand() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = types.slice(int).unwrap();
    let callback_signature = types
        .procedure(ProcedureType {
            parameters: vec![slice].into(),
            results: vec![int].into(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::Jai {
                parameter: 0,
                element: int,
            },
        })
        .unwrap();
    let body_signature = signature(&mut types, vec![], int);
    let contract = ProcedureId::new(2);
    let helper = ProcedureId::new(4);
    let entry = ProcedureId::new(6);
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .prototypes(vec![ProcedurePrototype {
            id: contract,
            signature: callback_signature,
            origin: PrototypeOrigin::SourceContract {
                symbol: "genuine_source_contract".into(),
            },
        }])
        .procedures(vec![
            returning(
                helper,
                body_signature,
                ValueExpr::Call {
                    call: Call::new(
                        contract,
                        vec![(
                            ParameterId::new(0),
                            ConstantValue {
                                ty: slice,
                                kind: ConstantKind::Zero,
                            }
                            .into_expression(),
                        )],
                    ),
                    ty: int,
                },
            ),
            returning(entry, body_signature, integer(42)),
        ])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    let context = jai_codegen::Context::create();
    let executable = jai_codegen::lower(&context, &program).unwrap();
    assert!(executable.get_function("genuine_source_contract").is_none());
    for publication in [Publication::AllBodies, Publication::Selected(vec![helper])] {
        let module = jai_codegen::lower_library(&context, program.library(), &publication).unwrap();
        module.verify().unwrap();
        let declaration = module.get_function("genuine_source_contract").unwrap();
        assert_eq!(declaration.count_basic_blocks(), 0);
        assert!(!declaration.get_type().is_var_arg());
        assert_eq!(declaration.count_params(), 1);
        assert_eq!(
            declaration
                .get_nth_param(0)
                .unwrap()
                .get_type()
                .into_struct_type()
                .count_fields(),
            2
        );
        assert!(module.get_function("jai.p4").is_some());
        assert!(module.get_function("main").is_none());
    }
    let error = Reachable::executable(program.library(), EntryPoint::Int(helper)).unwrap_err();
    assert!(error.to_string().contains("without a checked provider"));
    assert!(matches!(error,Error::SourceContract{chain} if chain == [helper,contract]));
}

#[test]
fn source_contract_callback_globals_reject_executable_demand() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = types.slice(int).unwrap();
    let callback_signature = types
        .procedure(ProcedureType {
            parameters: vec![slice].into(),
            results: vec![int].into(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::Jai {
                parameter: 0,
                element: int,
            },
        })
        .unwrap();
    let body_signature = signature(&mut types, vec![], int);
    let contract = ProcedureId::new(2);
    let entry = ProcedureId::new(4);
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: callback_signature,
            kind: ConstantKind::Procedure(contract),
        },
        &types,
    )
    .unwrap();
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .globals(vec![global])
        .prototypes(vec![ProcedurePrototype {
            id: contract,
            signature: callback_signature,
            origin: PrototypeOrigin::SourceContract {
                symbol: "genuine_global_contract".into(),
            },
        }])
        .procedures(vec![returning(entry, body_signature, integer(42))])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    assert!(
        matches!(Reachable::executable(program.library(),program.entry()),
        Err(Error::SourceContract{chain}) if chain == [contract])
    );
    let context = jai_codegen::Context::create();
    let module =
        jai_codegen::lower_library(&context, program.library(), &Publication::AllBodies).unwrap();
    module.verify().unwrap();
    assert_eq!(
        module
            .get_function("genuine_global_contract")
            .unwrap()
            .count_basic_blocks(),
        0
    );
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let object = jai_codegen::lower_program_object_for_target(&context, &program, &target).unwrap();
    object.verify().unwrap();
    assert!(object.get_function("main").is_some());
    assert_eq!(
        object
            .get_function("genuine_global_contract")
            .unwrap()
            .count_basic_blocks(),
        0
    );
    assert!(matches!(
        jai_codegen::lower_for_target(&context, &program, &target),
        Err(jai_codegen::Error::Reachability(
            Error::SourceContract { .. }
        ))
    ));
}

#[test]
fn compile_time_bodies_are_omitted_from_ordinary_library_publication() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let callback_signature = signature(&mut types, vec![], int);
    let compile_time = ProcedureId::new(17);
    let entry = ProcedureId::new(31);
    let mut phases = ProcedurePhases::default();
    phases.insert(compile_time, ProcedureExecution::CompileTimeOnly);
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedure_phases(phases)
        .procedures(vec![
            returning(compile_time, callback_signature, integer(42)),
            returning(entry, callback_signature, integer(42)),
        ])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    let reachable = Reachable::library(program.library(), &Publication::AllBodies).unwrap();
    assert!(reachable.contains(entry));
    assert!(!reachable.contains(compile_time));
    let context = jai_codegen::Context::create();
    let module =
        jai_codegen::lower_library(&context, program.library(), &Publication::AllBodies).unwrap();
    assert!(module.get_function("jai.p31").is_some());
    assert!(module.get_function("jai.p17").is_none());
    assert!(matches!(
        Reachable::library(program.library(), &Publication::Selected(vec![compile_time])),
        Err(Error::CompileTimeOnly { chain }) if chain == [compile_time]
    ));
}

#[test]
fn compile_time_body_calls_and_stored_callbacks_are_runtime_dependencies() {
    for stored_callback in [false, true] {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let callback_signature = signature(&mut types, vec![], int);
        let compile_time = ProcedureId::new(17);
        let entry = ProcedureId::new(31);
        let globals = if stored_callback {
            vec![
                Global::new_typed(
                    0,
                    ConstantValue {
                        ty: callback_signature,
                        kind: ConstantKind::Procedure(compile_time),
                    },
                    &types,
                )
                .unwrap(),
            ]
        } else {
            vec![]
        };
        let mut phases = ProcedurePhases::default();
        phases.insert(compile_time, ProcedureExecution::CompileTimeOnly);
        let program = ProgramBuilder::new(types.freeze().unwrap())
            .procedure_phases(phases)
            .globals(globals)
            .procedures(vec![
                returning(compile_time, callback_signature, integer(42)),
                returning(
                    entry,
                    callback_signature,
                    ValueExpr::Call {
                        call: Call::new(compile_time, vec![]),
                        ty: int,
                    },
                ),
            ])
            .finish(EntryPoint::Int(entry))
            .unwrap();
        assert!(matches!(
            jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
            jai_vm::Outcome::Complete(values) if values[0].integer().unwrap().value() == 42
        ));
        let error = Reachable::executable(program.library(), program.entry()).unwrap_err();
        let Error::CompileTimeOnly { chain } = error else {
            panic!("compile-time procedure diagnostic")
        };
        assert_eq!(
            chain,
            if stored_callback {
                vec![compile_time]
            } else {
                vec![entry, compile_time]
            }
        );
    }
}

#[test]
fn runtime_type_descriptor_graphs_keep_nested_procedure_constants_reachable() {
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let boolean = types.scalar(ScalarType::Bool);
    let callback_signature = signature(&mut types, vec![], int);
    let tag = types.reserve_enum(IntegerType::U32);
    let bool_tag = Integer::checked(IntegerType::U32, TypeInfoTag::Bool as i128).unwrap();
    types.define_enum(tag, [bool_tag]).unwrap();
    let header = types.reserve_record(RecordKind::Struct);
    types.define_record(header, [tag, int]).unwrap();
    types.bind_runtime_type_header(header).unwrap();
    let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
        &types,
        boolean,
        Some(target.layout_policy().unwrap()),
        &ReflectionMetadata::default(),
    )
    .unwrap() else {
        panic!("boolean descriptor must be complete")
    };
    let callback = ProcedureId::new(2);
    let entry = ProcedureId::new(4);
    let mut builder = StaticDataBuilder::new();
    let descriptor = builder.reserve(header, &types).unwrap();
    let callback_cell = builder.reserve(callback_signature, &types).unwrap();
    builder
        .define_type_descriptor(
            descriptor,
            StaticValue {
                ty: header,
                kind: StaticValueKind::Record(vec![
                    StaticValue::constant(ConstantValue {
                        ty: tag,
                        kind: ConstantKind::Enum(bool_tag),
                    }),
                    StaticValue::constant(ConstantValue {
                        ty: int,
                        kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 1).unwrap()),
                    }),
                ]),
            },
            &graph,
            graph.root(),
            &types,
        )
        .unwrap();
    builder
        .define(
            callback_cell,
            StaticValue::constant(ConstantValue {
                ty: callback_signature,
                kind: ConstantKind::Procedure(callback),
            }),
        )
        .unwrap();
    let data = std::sync::Arc::new(builder.finish(&types, StaticDataLimits::default()).unwrap());
    let runtime_type = RuntimeTypeConstant::new(data, descriptor, &types).unwrap();
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: runtime_type.ty(),
            kind: ConstantKind::RuntimeType(runtime_type),
        },
        &types,
    )
    .unwrap();
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .globals(vec![global])
        .procedures(vec![
            returning(callback, callback_signature, integer(42)),
            returning(entry, callback_signature, integer(42)),
        ])
        .finish(EntryPoint::Int(entry))
        .unwrap();
    assert!(
        Reachable::executable(program.library(), program.entry())
            .unwrap()
            .contains(callback)
    );
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    assert!(module.get_function("jai.p2").is_some());
    assert!(
        module
            .print_to_string()
            .to_string()
            .contains("constant ptr @jai.p2")
    );
}

#[test]
fn compiler_dependencies_inside_projected_places_and_exit_cleanups_are_followed() {
    for in_cleanup in [false, true] {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let signature = signature(&mut types, vec![], int);
        let array = types.fixed_array(int, 1).unwrap();
        let compiler = ProcedureId::new(1);
        let entry = ProcedureId::new(7);
        let local = Local::new_typed(entry, 0, array, &types).unwrap();
        let mut places = PlaceRegistry::new();
        let projected = places
            .index(
                local.place(),
                IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Call(Call::new(compiler, vec![])),
                ),
                &types,
            )
            .unwrap();
        let statement = Statement::Store(projected, integer(42));
        let mut procedure = returning(entry, signature, integer(42));
        procedure.locals = vec![local];
        if in_cleanup {
            procedure.cleanups = vec![
                Block {
                    flow: Flow::FallsThrough,
                    statements: vec![statement],
                }
                .into(),
            ];
            let Statement::Exit(exit) = &mut procedure.body.statements[0] else {
                unreachable!()
            };
            exit.cleanups.push(CleanupId::new(0));
        } else {
            procedure.body.statements.insert(0, statement);
        }
        let program = ProgramBuilder::new(types.freeze().unwrap())
            .prototypes(vec![ProcedurePrototype {
                id: compiler,
                signature,
                origin: PrototypeOrigin::Compiler,
            }])
            .procedures(vec![procedure])
            .places(places.freeze())
            .finish(EntryPoint::Int(entry))
            .unwrap();
        let error = Reachable::executable(program.library(), program.entry()).unwrap_err();
        assert!(error.to_string().contains("7 -> 1"));
        let Error::CompilerRequest { chain } = error else {
            panic!("compiler diagnostic")
        };
        assert_eq!(chain, [entry, compiler]);
    }
}
