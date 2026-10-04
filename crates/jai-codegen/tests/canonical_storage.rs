//! Sealed storage proofs through checked IR and actual O0/O2 LLVM execution.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_ir::*;
use jai_types::*;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
fn integer(ty: IntegerType, value: i128) -> ValueExpr {
    ValueExpr::Int(IntExpr::constant(Integer::checked(ty, value).unwrap()))
}
fn cast(
    types: &TypeRegistry,
    target: &jai_codegen::target::NativeTarget,
    source_type: TypeId,
    target_type: TypeId,
    source: StorageBitcastSource,
    strength: StorageBitcastStrength,
) -> ValueExpr {
    ValueExpr::StorageBitcast {
        source,
        cast: StorageBitcast::prove(
            types,
            target.layout_policy().unwrap(),
            source_type,
            target_type,
            strength,
        )
        .unwrap(),
    }
}
fn finish_full(
    types: TypeRegistry,
    statements: Vec<Statement>,
    locals: Vec<Local>,
    places: Places,
    globals: Vec<Global>,
    mut procedures: Vec<Procedure>,
) -> Program {
    let mut types = types;
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: vec![types.scalar(ScalarType::Int(IntegerType::S64))].into_boxed_slice(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    procedures.push(Procedure {
        id: ProcedureId::new(0),
        signature,
        parameters: vec![],
        locals,
        cleanups: vec![],
        body: Block {
            statements,
            flow: Flow::Terminates,
        },
    });
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(procedures)
        .globals(globals)
        .places(places)
        .finish(EntryPoint::Int(ProcedureId::new(0)))
        .unwrap()
}
fn finish(
    types: TypeRegistry,
    statements: Vec<Statement>,
    locals: Vec<Local>,
    places: Places,
) -> Program {
    finish_full(types, statements, locals, places, vec![], vec![])
}
fn returned(value: IntExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnInt(value),
    })
}
fn execute(program: &Program, expected: Option<i32>) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-canonical-record-prototype-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    for optimization in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target =
            jai_codegen::target::NativeTarget::select(&jai_codegen::target::TargetOptions {
                optimization: jai_codegen::optimization::Optimization {
                    bitcode: optimization,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
        module.verify().unwrap();
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
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
        let mut process = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = process.try_wait().unwrap() {
                if let Some(expected) = expected {
                    assert_eq!(status.code(), Some(expected), "{optimization:?}");
                } else {
                    assert!(
                        !status.success(),
                        "invalid Bool storage did not trap: {optimization:?}"
                    );
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::ExitStatusExt;
                        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
                    }
                }
                break;
            }
            if Instant::now() >= deadline {
                process.kill().unwrap();
                process.wait().unwrap();
                panic!("storage bitcast fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

fn padded(types: &mut TypeRegistry) -> TypeId {
    let ty = types.reserve_record(RecordKind::Struct);
    types
        .define_record(
            ty,
            [
                types.scalar(ScalarType::Int(IntegerType::U8)),
                types.scalar(ScalarType::Int(IntegerType::U32)),
            ],
        )
        .unwrap();
    ty
}
fn checked_bits(value: ValueExpr, expected: i128) -> Statement {
    returned(IntExpr::new(
        IntegerType::S64,
        IntExprKind::Conditional(Box::new(Conditional {
            condition: BoolExpr::CompareInts(
                Relation::Equal,
                Box::new(IntExpr::new(
                    IntegerType::U64,
                    IntExprKind::Value(Box::new(value)),
                )),
                Box::new(IntExpr::constant(
                    Integer::checked(IntegerType::U64, expected).unwrap(),
                )),
            ),
            then_value: IntExpr::constant(Integer::checked(IntegerType::S64, 42).unwrap()),
            else_value: IntExpr::constant(Integer::checked(IntegerType::S64, 1).unwrap()),
        })),
    ))
}
fn view(
    types: &TypeRegistry,
    source: TypeId,
    target: TypeId,
    value: StorageBitcastSource,
) -> ValueExpr {
    cast(
        types,
        &jai_codegen::target::NativeTarget::new().unwrap(),
        source,
        target,
        value,
        StorageBitcastStrength::EqualSize,
    )
}
#[test]
fn known_gap_bytes_survive_padded_destination_and_source_values() {
    let mut types = TypeRegistry::new();
    let record = padded(&mut types);
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let bits = 0x11223344aabbcc2ai128;
    let value = view(
        &types,
        word,
        record,
        StorageBitcastSource::Value(Box::new(integer(IntegerType::U64, bits))),
    );
    let recovered = view(
        &types,
        record,
        word,
        StorageBitcastSource::Value(Box::new(value)),
    );
    execute(
        &finish(
            types,
            vec![checked_bits(recovered, bits)],
            vec![],
            PlaceRegistry::new().freeze(),
        ),
        Some(42),
    );
}
#[test]
fn ordinary_construction_zero_backs_actual_padding() {
    let mut types = TypeRegistry::new();
    let record = padded(&mut types);
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let value = ValueExpr::Record {
        ty: record,
        fields: vec![
            integer(IntegerType::U8, 42),
            integer(IntegerType::U32, 0x11223344),
        ],
    };
    let recovered = view(
        &types,
        record,
        word,
        StorageBitcastSource::Value(Box::new(value)),
    );
    execute(
        &finish(
            types,
            vec![checked_bits(recovered, 0x112233440000002a)],
            vec![],
            PlaceRegistry::new().freeze(),
        ),
        Some(42),
    );
}
#[test]
fn copies_keep_gap_bytes_and_are_independent_of_field_mutation() {
    let mut types = TypeRegistry::new();
    let record = padded(&mut types);
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let main = ProcedureId::new(0);
    let source = Local::new_typed(main, 0, record, &types).unwrap();
    let copy = Local::new_typed(main, 1, record, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let changed = places
        .field(source.place(), types.field(record, 1).unwrap().id, &types)
        .unwrap();
    let bits = 0x11223344aabbcc2ai128;
    let value = view(
        &types,
        word,
        record,
        StorageBitcastSource::Value(Box::new(integer(IntegerType::U64, bits))),
    );
    let recovered = view(
        &types,
        record,
        word,
        StorageBitcastSource::Place(copy.place()),
    );
    execute(
        &finish(
            types,
            vec![
                Statement::Store(source.place(), value),
                Statement::Store(copy.place(), ValueExpr::Load(source.place())),
                Statement::Store(changed, integer(IntegerType::U32, 99)),
                checked_bits(recovered, bits),
            ],
            vec![source, copy],
            places.freeze(),
        ),
        Some(42),
    );
}
#[test]
fn bound_ssa_preserves_the_complete_record_byte_value() {
    let mut types = TypeRegistry::new();
    let record = padded(&mut types);
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let bits = 0x11223344aabbcc2ai128;
    let capture = ExpressionBindingId::new(ProcedureId::new(0), 0);
    let value = view(
        &types,
        word,
        record,
        StorageBitcastSource::Value(Box::new(integer(IntegerType::U64, bits))),
    );
    let body = view(
        &types,
        record,
        word,
        StorageBitcastSource::Value(Box::new(ValueExpr::Bound {
            binding: capture,
            ty: record,
        })),
    );
    let recovered = ValueExpr::Bind {
        bindings: vec![(capture, value)],
        body: Box::new(body),
        ty: word,
    };
    execute(
        &finish(
            types,
            vec![checked_bits(recovered, bits)],
            vec![],
            PlaceRegistry::new().freeze(),
        ),
        Some(42),
    );
}

#[test]
fn known_padding_survives_direct_and_indirect_parameters_and_results() {
    let mut types = TypeRegistry::new();
    let record = padded(&mut types);
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let bits = 0x11223344aabbcc2ai128;
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([record]),
            results: Box::new([record]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let id = ProcedureId::new(1);
    let parameter = Local::new_typed(id, 0, record, &types).unwrap();
    let callback = Procedure {
        id,
        signature,
        parameters: vec![parameter],
        locals: vec![parameter],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnValues(vec![ValueExpr::Load(parameter.place())]),
            })],
        },
    };
    let source = view(
        &types,
        word,
        record,
        StorageBitcastSource::Value(Box::new(integer(IntegerType::U64, bits))),
    );
    let direct = ValueExpr::Call {
        call: Call::new(id, vec![(ParameterId::new(0), source)]),
        ty: record,
    };
    let indirect = ValueExpr::IndirectCall {
        inline_hint: InlineHint::Automatic,
        callee: Box::new(ValueExpr::ProcedureValue {
            procedure: id,
            ty: signature,
        }),
        arguments: vec![(ParameterId::new(0), direct)],
        ty: record,
    };
    let recovered = view(
        &types,
        record,
        word,
        StorageBitcastSource::Value(Box::new(indirect)),
    );
    execute(
        &finish_full(
            types,
            vec![checked_bits(recovered, bits)],
            vec![],
            PlaceRegistry::new().freeze(),
            vec![],
            vec![callback],
        ),
        Some(42),
    );
}
#[test]
fn ordinary_global_record_constants_emit_explicit_zero_padding() {
    let mut types = TypeRegistry::new();
    let record = padded(&mut types);
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: record,
            kind: ConstantKind::Record(vec![
                ConstantValue {
                    ty: types.scalar(ScalarType::Int(IntegerType::U8)),
                    kind: ConstantKind::Int(Integer::checked(IntegerType::U8, 42).unwrap()),
                },
                ConstantValue {
                    ty: types.scalar(ScalarType::Int(IntegerType::U32)),
                    kind: ConstantKind::Int(
                        Integer::checked(IntegerType::U32, 0x11223344).unwrap(),
                    ),
                },
            ]),
        },
        &types,
    )
    .unwrap();
    let recovered = view(
        &types,
        record,
        word,
        StorageBitcastSource::Place(global.place()),
    );
    execute(
        &finish_full(
            types,
            vec![checked_bits(recovered, 0x112233440000002a)],
            vec![],
            PlaceRegistry::new().freeze(),
            vec![global],
            vec![],
        ),
        Some(42),
    );
}

#[test]
fn padded_global_record_preserves_nested_union_procedure_relocation() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
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
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [signature, int]).unwrap();
    let holder = types.reserve_record(RecordKind::Struct);
    types.define_record(holder, [byte, union]).unwrap();
    let member = types.field(union, 0).unwrap().id;
    let choice = types.field(holder, 1).unwrap().id;
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: holder,
            kind: ConstantKind::Record(vec![
                ConstantValue {
                    ty: byte,
                    kind: ConstantKind::Int(Integer::checked(IntegerType::U8, 7).unwrap()),
                },
                ConstantValue {
                    ty: union,
                    kind: ConstantKind::Union {
                        field: member,
                        value: Box::new(ConstantValue {
                            ty: signature,
                            kind: ConstantKind::Procedure(ProcedureId::new(1)),
                        }),
                    },
                },
            ]),
        },
        &types,
    )
    .unwrap();
    let mut places = PlaceRegistry::new();
    let slot = places.field(global.place(), choice, &types).unwrap();
    let callable = places.field(slot, member, &types).unwrap();
    let invoke = ValueExpr::IndirectCall {
        inline_hint: InlineHint::Automatic,
        callee: Box::new(ValueExpr::Load(callable)),
        arguments: vec![],
        ty: int,
    };
    let callback = Procedure {
        id: ProcedureId::new(1),
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![returned(IntExpr::constant(
                Integer::checked(IntegerType::S64, 42).unwrap(),
            ))],
        },
    };
    execute(
        &finish_full(
            types,
            vec![returned(IntExpr::new(
                IntegerType::S64,
                IntExprKind::Value(Box::new(invoke)),
            ))],
            vec![],
            places.freeze(),
            vec![global],
            vec![callback],
        ),
        Some(42),
    );
}

#[test]
fn ordered_nested_overlapping_paths_preserve_interleaving_o0_o2() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let inner = types.reserve_record(RecordKind::Struct);
    types.define_record(inner, [word]).unwrap();
    let record = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_placements(record, [inner, inner], Default::default(), [None, Some(0)])
        .unwrap();
    let a = types.field(record, 0).unwrap().id;
    let b = types.field(record, 1).unwrap().id;
    let child = types.field(inner, 0).unwrap().id;
    let snapshot = ValueExpr::OrderedRecord {
        ty: record,
        backing: OrderedRecordBacking::Zeroed,
        initializers: [(a, 1), (b, 2), (a, 42)]
            .into_iter()
            .map(|(field, value)| (Box::from([field, child]), integer(IntegerType::U32, value)))
            .collect(),
    };
    let value = ValueExpr::Field {
        base: Box::new(ValueExpr::Field {
            base: Box::new(snapshot),
            field: b,
            ty: inner,
        }),
        field: child,
        ty: word,
    };
    execute(
        &finish(
            types,
            vec![returned(IntExpr::new(
                IntegerType::S64,
                IntExprKind::Cast(
                    CastMode::Checked,
                    Box::new(IntExpr::new(
                        IntegerType::U32,
                        IntExprKind::Value(Box::new(value)),
                    )),
                ),
            ))],
            vec![],
            Places::default(),
        ),
        Some(42),
    );
}

#[test]
fn placed_record_build_applies_overlapping_fields_in_initializer_order_o0_o2() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let record = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_placements(
            record,
            [word, word],
            RecordLayout::default(),
            [None, Some(0)],
        )
        .unwrap();
    let first = types.field(record, 0).unwrap().id;
    let second = types.field(record, 1).unwrap().id;
    let snapshot = ValueExpr::RecordBuild {
        ty: record,
        initializers: vec![
            (first, integer(IntegerType::U32, 1)),
            (second, integer(IntegerType::U32, 42)),
        ],
    };
    let selected = ValueExpr::Field {
        base: Box::new(snapshot),
        field: first,
        ty: word,
    };
    execute(
        &finish(
            types,
            vec![returned(IntExpr::new(
                IntegerType::S64,
                IntExprKind::Cast(
                    CastMode::Checked,
                    Box::new(IntExpr::new(
                        IntegerType::U32,
                        IntExprKind::Value(Box::new(selected)),
                    )),
                ),
            ))],
            vec![],
            Places::default(),
        ),
        Some(42),
    );
}

#[test]
fn ordered_uninitialized_backing_writes_only_the_selected_field_o0_o2() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [byte, word]).unwrap();
    let field = types.field(record, 0).unwrap().id;
    let snapshot = ValueExpr::OrderedRecord {
        ty: record,
        backing: OrderedRecordBacking::Uninitialized,
        initializers: vec![(Box::from([field]), integer(IntegerType::U8, 42))],
    };
    let value = ValueExpr::Field {
        base: Box::new(snapshot),
        field,
        ty: byte,
    };
    execute(
        &finish(
            types,
            vec![returned(IntExpr::new(
                IntegerType::S64,
                IntExprKind::Cast(
                    CastMode::Checked,
                    Box::new(IntExpr::new(
                        IntegerType::U8,
                        IntExprKind::Value(Box::new(value)),
                    )),
                ),
            ))],
            vec![],
            Places::default(),
        ),
        Some(42),
    );
}
