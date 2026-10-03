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
fn finish(
    types: TypeRegistry,
    statements: Vec<Statement>,
    locals: Vec<Local>,
    places: Places,
) -> Program {
    let mut types = types;
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: vec![types.scalar(ScalarType::Int(IntegerType::S64))].into_boxed_slice(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![Procedure {
            id: ProcedureId::new(0),
            signature,
            parameters: vec![],
            locals,
            cleanups: vec![],
            body: Block {
                statements,
                flow: Flow::Terminates,
            },
        }])
        .places(places)
        .finish(EntryPoint::Int(ProcedureId::new(0)))
        .unwrap()
}
fn returned(value: IntExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnInt(value),
    })
}
fn result42() -> Statement {
    returned(IntExpr::constant(
        Integer::checked(IntegerType::S64, 42).unwrap(),
    ))
}
fn returned_u64(value: ValueExpr) -> Statement {
    returned(IntExpr::new(
        IntegerType::S64,
        IntExprKind::Cast(
            CastMode::Checked,
            Box::new(IntExpr::new(
                IntegerType::U64,
                IntExprKind::Value(Box::new(value)),
            )),
        ),
    ))
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
        "jai-storage-bitcasts-ir-{}-{}",
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
#[test]
fn u32_float_storage_view_preserves_bits() {
    let types = TypeRegistry::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let source = types.scalar(ScalarType::Int(IntegerType::U32));
    let dest = types.float(FloatType::F32);
    let view = cast(
        &types,
        &target,
        source,
        dest,
        StorageBitcastSource::Value(Box::new(integer(IntegerType::U32, 0x3f800000))),
        StorageBitcastStrength::EqualSize,
    );
    let float = FloatExpr::new(FloatType::F32, FloatExprKind::Value(Box::new(view)));
    let one = IntExpr::new(
        IntegerType::S64,
        IntExprKind::FromFloat(CastMode::Checked, Box::new(float)),
    );
    let result = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(
            IntOp::Multiply,
            Box::new(one),
            Box::new(IntExpr::constant(
                Integer::checked(IntegerType::S64, 42).unwrap(),
            )),
        ),
    );
    execute(
        &finish(
            types,
            vec![returned(result)],
            vec![],
            PlaceRegistry::new().freeze(),
        ),
        Some(42),
    );
}
#[test]
fn prefix_place_copies_initialized_bytes_without_typed_tail_load() {
    let mut types = TypeRegistry::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let int = types.scalar(ScalarType::Int(IntegerType::U64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![int, int]).unwrap();
    let prefix = types.reserve_record(RecordKind::Struct);
    types.define_record(prefix, vec![int]).unwrap();
    let local = Local::new_typed(ProcedureId::new(0), 0, wide, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let low = places
        .field(local.place(), types.field(wide, 0).unwrap().id, &types)
        .unwrap();
    let view = cast(
        &types,
        &target,
        wide,
        prefix,
        StorageBitcastSource::Place(local.place()),
        StorageBitcastStrength::Prefix,
    );
    let value = ValueExpr::Field {
        base: Box::new(view),
        field: types.field(prefix, 0).unwrap().id,
        ty: int,
    };
    execute(
        &finish(
            types,
            vec![
                Statement::Store(low, integer(IntegerType::U64, 42)),
                returned_u64(value),
            ],
            vec![local],
            places.freeze(),
        ),
        Some(42),
    );
}
#[test]
fn packed_source_snapshot_supports_stronger_destination_alignment() {
    let mut types = TypeRegistry::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let int = types.scalar(ScalarType::Int(IntegerType::U64));
    let source = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            source,
            vec![int, int],
            RecordLayout {
                packed: true,
                ..Default::default()
            },
        )
        .unwrap();
    let dest = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            dest,
            vec![int, int],
            RecordLayout {
                minimum_alignment: Some(16),
                ..Default::default()
            },
        )
        .unwrap();
    let view = cast(
        &types,
        &target,
        source,
        dest,
        StorageBitcastSource::Value(Box::new(ValueExpr::Record {
            ty: source,
            fields: vec![integer(IntegerType::U64, 42), integer(IntegerType::U64, 99)],
        })),
        StorageBitcastStrength::EqualSize,
    );
    let value = ValueExpr::Field {
        base: Box::new(view),
        field: types.field(dest, 0).unwrap().id,
        ty: int,
    };
    execute(
        &finish(
            types,
            vec![returned_u64(value)],
            vec![],
            PlaceRegistry::new().freeze(),
        ),
        Some(42),
    );
}
#[test]
fn noncanonical_scalar_bool_storage_traps_before_typed_load() {
    let types = TypeRegistry::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let source = types.scalar(ScalarType::Int(IntegerType::U8));
    let dest = types.scalar(ScalarType::Bool);
    let view = cast(
        &types,
        &target,
        source,
        dest,
        StorageBitcastSource::Value(Box::new(integer(IntegerType::U8, 2))),
        StorageBitcastStrength::EqualSize,
    );
    execute(
        &finish(
            types,
            vec![Statement::DiscardValue(view), result42()],
            vec![],
            PlaceRegistry::new().freeze(),
        ),
        None,
    );
}
#[test]
fn bool_array_loop_checks_every_element_before_typed_load() {
    let mut types = TypeRegistry::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let boolean = types.scalar(ScalarType::Bool);
    let source = types.fixed_array(byte, 3).unwrap();
    let dest = types.fixed_array(boolean, 3).unwrap();
    let value = ValueExpr::Array {
        ty: source,
        elements: vec![
            integer(IntegerType::U8, 0),
            integer(IntegerType::U8, 1),
            integer(IntegerType::U8, 2),
        ],
    };
    let view = cast(
        &types,
        &target,
        source,
        dest,
        StorageBitcastSource::Value(Box::new(value)),
        StorageBitcastStrength::EqualSize,
    );
    execute(
        &finish(
            types,
            vec![Statement::DiscardValue(view), result42()],
            vec![],
            PlaceRegistry::new().freeze(),
        ),
        None,
    );
}

fn padded(types: &mut TypeRegistry) -> TypeId {
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, vec![byte, word]).unwrap();
    record
}
#[test]
fn padded_destination_value_is_rejected_until_canonical_byte_storage() {
    let mut types = TypeRegistry::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let source = types.scalar(ScalarType::Int(IntegerType::U64));
    let dest = padded(&mut types);
    let view = cast(
        &types,
        &target,
        source,
        dest,
        StorageBitcastSource::Value(Box::new(integer(IntegerType::U64, 42))),
        StorageBitcastStrength::EqualSize,
    );
    let program = finish(
        types,
        vec![Statement::DiscardValue(view), result42()],
        vec![],
        PlaceRegistry::new().freeze(),
    );
    let context = jai_codegen::Context::create();
    assert!(
        matches!(jai_codegen::lower_for_target(&context,&program,&target),Err(jai_codegen::Error::UnsupportedStorageBitcast {ty,role:jai_codegen::StorageValueRole::DestinationValue,..}) if ty==dest)
    );
}
#[test]
fn padded_source_value_is_rejected_until_canonical_byte_storage() {
    let mut types = TypeRegistry::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let source = padded(&mut types);
    let dest = types.scalar(ScalarType::Int(IntegerType::U64));
    let value = ValueExpr::Record {
        ty: source,
        fields: vec![integer(IntegerType::U8, 42), integer(IntegerType::U32, 0)],
    };
    let view = cast(
        &types,
        &target,
        source,
        dest,
        StorageBitcastSource::Value(Box::new(value)),
        StorageBitcastStrength::EqualSize,
    );
    let program = finish(
        types,
        vec![Statement::DiscardValue(view), result42()],
        vec![],
        PlaceRegistry::new().freeze(),
    );
    let context = jai_codegen::Context::create();
    assert!(
        matches!(jai_codegen::lower_for_target(&context,&program,&target),Err(jai_codegen::Error::UnsupportedStorageBitcast {ty,role:jai_codegen::StorageValueRole::SourceValue,..}) if ty==source)
    );
}
#[test]
fn padded_source_place_can_recover_actual_raw_bytes_without_ssa_loss() {
    let mut types = TypeRegistry::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let source = padded(&mut types);
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let source_pointer = types.pointer(source).unwrap();
    let word_pointer = types.pointer(word).unwrap();
    let local = Local::new_typed(ProcedureId::new(0), 0, source, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let alias = places
        .dereference(
            ValueExpr::PointerCast {
                value: Box::new(ValueExpr::AddressOf {
                    place: local.place(),
                    ty: source_pointer,
                }),
                ty: word_pointer,
                mode: CastMode::Unchecked,
            },
            &types,
        )
        .unwrap();
    let view = cast(
        &types,
        &target,
        source,
        word,
        StorageBitcastSource::Place(local.place()),
        StorageBitcastStrength::EqualSize,
    );
    execute(
        &finish(
            types,
            vec![
                Statement::Store(alias, integer(IntegerType::U64, 42)),
                returned_u64(view),
            ],
            vec![local],
            places.freeze(),
        ),
        Some(42),
    );
}
