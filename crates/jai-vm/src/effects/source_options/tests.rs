use super::*;
use jai_types::{Integer, RecordKind, TypeRegistry};
#[derive(Default)]
struct Effects(Vec<CompilerRequest>);
impl CompilerEffects for Effects {
    fn begin(&mut self) {
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        self.0.push(request);
        EffectOutcome::Ready(CompilerResponse::Unit)
    }
    fn finish(&mut self, _: bool) -> Result<(), Error> {
        Ok(())
    }
}
fn current() -> WorkspaceId {
    WorkspaceId::from_raw(9).unwrap()
}
fn current_argument() -> Value {
    Value::Int(Integer::checked(IntegerType::S64, -1).unwrap())
}

#[test]
fn source_levels_are_preserved_and_invalid_levels_stage_nothing() {
    let mut types = TypeRegistry::new();
    let enumeration = types.reserve_enum(IntegerType::U8);
    types
        .define_enum(
            enumeration,
            (0..7)
                .map(|value| Integer::checked(IntegerType::U8, value).unwrap())
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let llvm = types.reserve_record(RecordKind::Struct);
    types.define_record(llvm, vec![enumeration]).unwrap();
    let options = types.reserve_record(RecordKind::Struct);
    types.define_record(options, vec![llvm]).unwrap();
    let projection = BuildOptionsProjection {
        bitcode: Some(RecordFieldPath {
            leaf: None,
            outer: types.field(options, 0).unwrap().id,
            inner: Some(types.field(llvm, 0).unwrap().id),
        }),
        ..BuildOptionsProjection::default()
    };
    let value = |level| Value::Record {
        ty: options,
        fields: vec![Value::Record {
            ty: llvm,
            fields: vec![Value::Enum {
                ty: enumeration,
                value: Integer::checked(IntegerType::U8, level).unwrap(),
            }],
        }],
    };
    let levels = [
        BitcodeOptimization::Unset,
        BitcodeOptimization::O0,
        BitcodeOptimization::O1,
        BitcodeOptimization::O2,
        BitcodeOptimization::O3,
        BitcodeOptimization::Os,
        BitcodeOptimization::Oz,
    ];
    for (number, level) in levels.into_iter().enumerate() {
        let mut effects = Effects::default();
        assert!(
            invoke_options(
                projection,
                &[value(number as i128), current_argument()],
                current(),
                &mut effects,
                &types
            )
            .is_ok()
        );
        assert_eq!(
            effects.0,
            vec![CompilerRequest::SetBuildOption {
                workspace: current(),
                option: BuildOption::BitcodeOptimization(level)
            }]
        );
    }
    let mut effects = Effects::default();
    assert!(
        invoke_options(
            projection,
            &[value(7), current_argument()],
            current(),
            &mut effects,
            &types
        )
        .is_err()
    );
    assert!(effects.0.is_empty());
}

#[test]
fn field_projection_rejects_same_shape_foreign_nominal_record() {
    let mut types = TypeRegistry::new();
    let string = types.string();
    let left = types.reserve_record(RecordKind::Struct);
    let right = types.reserve_record(RecordKind::Struct);
    types.define_record(left, vec![string]).unwrap();
    types.define_record(right, vec![string]).unwrap();
    let projection = BuildOptionsProjection {
        output_path: Some(RecordFieldPath {
            leaf: None,
            outer: types.field(left, 0).unwrap().id,
            inner: None,
        }),
        ..BuildOptionsProjection::default()
    };
    let value = Value::Record {
        ty: right,
        fields: vec![Value::String(b"build".to_vec())],
    };
    let mut effects = Effects::default();
    assert!(
        invoke_options(
            projection,
            &[value, current_argument()],
            current(),
            &mut effects,
            &types
        )
        .is_err()
    );
    assert!(effects.0.is_empty());
}

#[test]
fn settings_snapshot_returns_the_declared_nominal_record_and_enum_values() {
    struct SnapshotEffects {
        snapshot: BuildOptionsSnapshot,
        requests: Vec<CompilerRequest>,
    }
    impl CompilerEffects for SnapshotEffects {
        fn begin(&mut self) {
        }
        fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
            self.requests.push(request);
            EffectOutcome::Ready(CompilerResponse::BuildOptions(self.snapshot.clone()))
        }
        fn finish(&mut self, _: bool) -> Result<(), Error> {
            Ok(())
        }
    }
    let mut types = TypeRegistry::new();
    let string = types.string();
    let enumeration = types.reserve_enum(IntegerType::U8);
    types
        .define_enum(
            enumeration,
            (0..7)
                .map(|value| Integer::checked(IntegerType::U8, value).unwrap())
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let llvm = types.reserve_record(RecordKind::Struct);
    types
        .define_record(llvm, vec![enumeration, string])
        .unwrap();
    let options = types.reserve_record(RecordKind::Struct);
    let temporary_storage_size = types.scalar(jai_types::ScalarType::Int(IntegerType::S32));
    types
        .define_record(options, vec![string, llvm, temporary_storage_size])
        .unwrap();
    let outer = types.field(options, 1).unwrap().id;
    let projection = BuildOptionsProjection {
        output_path: Some(RecordFieldPath {
            leaf: None,
            outer: types.field(options, 0).unwrap().id,
            inner: None,
        }),
        bitcode: Some(RecordFieldPath {
            leaf: None,
            outer,
            inner: Some(types.field(llvm, 0).unwrap().id),
        }),
        target: Some(RecordFieldPath {
            leaf: None,
            outer,
            inner: Some(types.field(llvm, 1).unwrap().id),
        }),
        temporary_storage_size: Some(RecordFieldPath {
            leaf: None,
            outer: types.field(options, 2).unwrap().id,
            inner: None,
        }),
        machine: None,
        ..Default::default()
    };
    let mut effects = SnapshotEffects {
        snapshot: BuildOptionsSnapshot {
            output_kind: BuildOutputKind::Executable,
            runtime_support: RuntimeSupportMode::Auto,
            backtrace_on_crash: BacktraceOnCrash::On,
            output_path: Some("out".into()),
            target: Some(TargetTriple::parse("aarch64-apple-darwin").unwrap()),
            bitcode: BitcodeOptimization::Oz,
            machine: MachineOptimization::Unset,
            temporary_storage_size: 4096,
        },
        requests: vec![],
    };
    let values = invoke_get_options(
        projection,
        options,
        &[current_argument()],
        current(),
        &mut effects,
        &types,
    )
    .unwrap_or_else(|_| panic!("valid snapshot must materialize"));
    assert_eq!(
        effects.requests,
        vec![CompilerRequest::GetBuildOptions {
            workspace: current()
        }]
    );
    assert_eq!(
        values,
        vec![Value::Record {
            ty: options,
            fields: vec![
                Value::String(b"out".to_vec()),
                Value::Record {
                    ty: llvm,
                    fields: vec![
                        Value::Enum {
                            ty: enumeration,
                            value: Integer::checked(IntegerType::U8, 6).unwrap()
                        },
                        Value::String(b"aarch64-apple-darwin".to_vec())
                    ]
                },
                Value::Int(Integer::checked(IntegerType::S32, 4096).unwrap())
            ]
        }]
    );
}

#[test]
fn three_level_projection_preserves_output_and_runtime_policy() {
    let mut types = TypeRegistry::new();
    let enums: Vec<_> = [5, 4, 2]
        .into_iter()
        .map(|count| {
            let ty = types.reserve_enum(IntegerType::U8);
            types
                .define_enum(
                    ty,
                    (0..count)
                        .map(|value| Integer::checked(IntegerType::U8, value).unwrap())
                        .collect::<Vec<_>>(),
                )
                .unwrap();
            ty
        })
        .collect();
    let inner = types.reserve_record(RecordKind::Struct);
    types.define_record(inner, enums.clone()).unwrap();
    let middle = types.reserve_record(RecordKind::Struct);
    types.define_record(middle, vec![inner]).unwrap();
    let outer = types.reserve_record(RecordKind::Struct);
    types.define_record(outer, vec![middle]).unwrap();
    let path = |index| RecordFieldPath {
        outer: types.field(outer, 0).unwrap().id,
        inner: Some(types.field(middle, 0).unwrap().id),
        leaf: Some(types.field(inner, index).unwrap().id),
    };
    let projection = BuildOptionsProjection {
        output_kind: Some(path(0)),
        runtime_support: Some(path(1)),
        backtrace_on_crash: Some(path(2)),
        ..Default::default()
    };
    let value = Value::Record {
        ty: outer,
        fields: vec![Value::Record {
            ty: middle,
            fields: vec![Value::Record {
                ty: inner,
                fields: enums
                    .iter()
                    .zip([4, 2, 0])
                    .map(|(&ty, value)| Value::Enum {
                        ty,
                        value: Integer::checked(IntegerType::U8, value).unwrap(),
                    })
                    .collect(),
            }],
        }],
    };
    let mut effects = Effects::default();
    invoke_options(
        projection,
        &[value.clone(), current_argument()],
        current(),
        &mut effects,
        &types,
    )
    .unwrap_or_else(|_| panic!("valid nested source options"));
    assert_eq!(
        effects.0,
        vec![
            CompilerRequest::SetBuildOption {
                workspace: current(),
                option: BuildOption::OutputKind(BuildOutputKind::Object)
            },
            CompilerRequest::SetBuildOption {
                workspace: current(),
                option: BuildOption::RuntimeSupport(RuntimeSupportMode::InitializationOnly)
            },
            CompilerRequest::SetBuildOption {
                workspace: current(),
                option: BuildOption::BacktraceOnCrash(BacktraceOnCrash::Off)
            },
        ]
    );
    struct PolicySnapshot;
    impl CompilerEffects for PolicySnapshot {
        fn begin(&mut self) {
        }
        fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
            assert_eq!(
                request,
                CompilerRequest::GetBuildOptions {
                    workspace: current()
                }
            );
            EffectOutcome::Ready(CompilerResponse::BuildOptions(BuildOptionsSnapshot {
                output_path: None,
                target: None,
                bitcode: BitcodeOptimization::Unset,
                machine: MachineOptimization::Unset,
                temporary_storage_size: 32768,
                output_kind: BuildOutputKind::Object,
                runtime_support: RuntimeSupportMode::InitializationOnly,
                backtrace_on_crash: BacktraceOnCrash::Off,
            }))
        }
        fn finish(&mut self, _: bool) -> Result<(), Error> {
            Ok(())
        }
    }
    let got = invoke_get_options(
        projection,
        outer,
        &[current_argument()],
        current(),
        &mut PolicySnapshot,
        &types,
    )
    .unwrap_or_else(|_| panic!("nested policy snapshot must preserve nominal enum identities"));
    assert_eq!(got, vec![value]);
}
