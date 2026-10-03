//! Native/VM pack storage checks independent of source resolver readiness.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_ir::*;
use jai_types::*;
use std::{fs, process::Command};

fn integer(value: i128) -> ValueExpr {
    ValueExpr::Int(IntExpr::constant(
        Integer::checked(IntegerType::S64, value).unwrap(),
    ))
}
fn signature(
    types: &mut TypeRegistry,
    parameters: Vec<TypeId>,
    results: Vec<TypeId>,
    variadic: Variadic,
) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            variadic,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
        })
        .unwrap()
}
fn exit(value: ValueExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnValues(vec![value]),
    })
}
fn fixture(escape: bool) -> Program {
    fixture_with_null_spread(escape, false)
}
fn fixture_with_null_spread(escape: bool, null_spread: bool) -> Program {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = types.slice(int).unwrap();
    let array = types.fixed_array(int, 1).unwrap();
    let borrow_id = ProcedureId::new(0);
    let leak_id = ProcedureId::new(1);
    let main_id = ProcedureId::new(2);
    let borrow_signature = signature(
        &mut types,
        vec![slice],
        vec![slice],
        Variadic::Jai {
            parameter: 0,
            element: int,
        },
    );
    let leak_signature = signature(&mut types, vec![], vec![slice], Variadic::None);
    let main_signature = signature(&mut types, vec![], vec![int], Variadic::None);
    let parameter = Local::new_typed(borrow_id, 0, slice, &types).unwrap();
    let borrow = Procedure {
        id: borrow_id,
        signature: borrow_signature,
        parameters: vec![parameter],
        locals: vec![parameter],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![exit(ValueExpr::Load(parameter.place()))],
        },
    };
    let call = ValueExpr::Call {
        ty: slice,
        call: Call::new(
            borrow_id,
            vec![(
                ParameterId::new(0),
                ValueExpr::SequenceConcat {
                    ty: slice,
                    parts: vec![
                        SequencePackPart::Element(integer(20)),
                        SequencePackPart::Spread(if null_spread {
                            ValueExpr::SequenceBuild {
                                ty: slice,
                                initializers: vec![(SequenceField::Count, integer(1))],
                            }
                        } else {
                            ValueExpr::ArrayView {
                                ty: slice,
                                array: Box::new(ValueExpr::Array {
                                    ty: array,
                                    elements: vec![integer(22)],
                                }),
                            }
                        }),
                    ],
                },
            )],
        ),
    };
    let alias = Local::new_typed(leak_id, 0, slice, &types).unwrap();
    let leak = Procedure {
        id: leak_id,
        signature: leak_signature,
        parameters: vec![],
        locals: vec![alias],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(alias.place(), call.clone()),
                exit(ValueExpr::Load(alias.place())),
            ],
        },
    };
    let value = Local::new_typed(main_id, 0, slice, &types).unwrap();
    let element = |index| {
        IntExpr::new(
            IntegerType::S64,
            IntExprKind::Value(Box::new(ValueExpr::Index {
                ty: int,
                base: Box::new(ValueExpr::Load(value.place())),
                index: match integer(index) {
                    ValueExpr::Int(value) => value,
                    _ => unreachable!(),
                },
                check: CheckMode::Enabled,
            })),
        )
    };
    let main = Procedure {
        id: main_id,
        signature: main_signature,
        parameters: vec![],
        locals: vec![value],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(
                    value.place(),
                    if escape {
                        ValueExpr::Call {
                            ty: slice,
                            call: Call::new(leak_id, vec![]),
                        }
                    } else {
                        call
                    },
                ),
                exit(ValueExpr::Int(IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Binary(IntOp::Add, Box::new(element(0)), Box::new(element(1))),
                ))),
            ],
        },
    };
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![borrow, leak, main])
        .finish(EntryPoint::Int(main_id))
        .unwrap()
}
fn native(program: &Program) -> std::process::ExitStatus {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "jai-pack-ir-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(path.clone());
    fs::create_dir(&path).unwrap();
    let context = jai_codegen::Context::create();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
    target.write_object(&module, &path.join("pack.o")).unwrap();
    let linked = native_tools::clang_command()
        .arg(path.join("pack.o"))
        .arg("-o")
        .arg(path.join("pack"))
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let mut process = Command::new(path.join("pack")).spawn().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(status) = process.try_wait().unwrap() {
            drop(scratch);
            return status;
        }
        if std::time::Instant::now() >= deadline {
            process.kill().unwrap();
            process.wait().unwrap();
            panic!("generated pack fixture timed out");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
#[test]
fn returned_pack_view_survives_borrowed_callee() {
    let program = fixture(false);
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("{:?}", execution.outcome)
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
    assert_eq!(native(&program).code(), Some(42));
}
#[test]
fn returning_owned_pack_alias_traps_before_frame_exit() {
    let program = fixture(true);
    assert_eq!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::SequenceTemporaryEscape)
    );
    let status = native(&program);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(status.signal().is_some());
    }
}

#[test]
fn null_nonempty_spread_traps_before_copying() {
    let program = fixture_with_null_spread(false, true);
    assert_eq!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::NullPointer)
    );
    let status = native(&program);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(status.signal().is_some());
    }
}

#[test]
fn mutable_string_descriptor_indices_alias_byte_array_storage() {
    let program = string_mutation_fixture(false);
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("{:?}", execution.outcome)
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
    assert_eq!(native(&program).code(), Some(42));
}

#[test]
fn string_descriptor_writes_preserve_immutable_literal_backing() {
    let program = string_mutation_fixture(true);
    assert_eq!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::ReadOnlyStorage)
    );
    let status = native(&program);
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(status.signal().is_some());
    }
}

fn string_mutation_fixture(literal: bool) -> Program {
    string_mutation_fixture_at(literal, 1)
}

#[test]
fn string_descriptor_write_checks_its_count_before_addressing_backing() {
    let program = string_mutation_fixture_at(false, 3);
    assert_eq!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::OutOfBounds {
            index: 3,
            length: 3,
        })
    );
    let status = native(&program);
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(status.signal().is_some());
    }
}

fn string_mutation_fixture_at(literal: bool, selected_index: i128) -> Program {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let string = types.string();
    let pointer = types.pointer(byte).unwrap();
    let array = types.fixed_array(byte, 3).unwrap();
    let id = ProcedureId::new(0);
    let signature = signature(&mut types, vec![], vec![int], Variadic::None);
    let bytes = Local::new_typed(id, 0, array, &types).unwrap();
    let text = Local::new_typed(id, 1, string, &types).unwrap();
    let alias = Local::new_typed(id, 2, string, &types).unwrap();
    let mut places = PlaceRegistry::default();
    let index = |value| IntExpr::constant(Integer::checked(IntegerType::S64, value).unwrap());
    let selected = places
        .index(alias.place(), index(selected_index), &types)
        .unwrap();
    let byte_value = |value| {
        ValueExpr::Int(IntExpr::constant(
            Integer::checked(IntegerType::U8, value).unwrap(),
        ))
    };
    let read = |offset| {
        IntExpr::new(
            IntegerType::S64,
            IntExprKind::Cast(
                CastMode::Checked,
                Box::new(IntExpr::new(
                    IntegerType::U8,
                    IntExprKind::Value(Box::new(ValueExpr::Index {
                        base: Box::new(ValueExpr::Load(text.place())),
                        index: index(offset),
                        ty: byte,
                        check: CheckMode::Enabled,
                    })),
                )),
            ),
        )
    };
    let procedure = Procedure {
        id,
        signature,
        parameters: vec![],
        locals: vec![bytes, text, alias],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(
                    bytes.place(),
                    ValueExpr::Array {
                        ty: array,
                        elements: vec![byte_value(20), byte_value(99), byte_value(22)],
                    },
                ),
                Statement::Store(
                    text.place(),
                    if literal {
                        ValueExpr::StringBytes {
                            ty: string,
                            bytes: vec![20, 99, 22],
                        }
                    } else {
                        ValueExpr::SequenceBuild {
                            ty: string,
                            initializers: vec![
                                (SequenceField::Count, integer(3)),
                                (
                                    SequenceField::Data,
                                    ValueExpr::SequenceField {
                                        base: Box::new(ValueExpr::Load(bytes.place())),
                                        field: SequenceField::Data,
                                        ty: pointer,
                                    },
                                ),
                            ],
                        }
                    },
                ),
                Statement::Store(alias.place(), ValueExpr::Load(text.place())),
                Statement::Store(selected, byte_value(20)),
                exit(ValueExpr::Int(IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Binary(IntOp::Add, Box::new(read(1)), Box::new(read(2))),
                ))),
            ],
        },
    };
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .places(places.freeze())
        .finish(EntryPoint::Int(id))
        .unwrap()
}
