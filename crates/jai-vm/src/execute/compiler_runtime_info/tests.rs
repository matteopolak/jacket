use super::*;
use jai_types::{
    IntegerType, LayoutPolicy, RecordKind, ReflectionGraph, ReflectionMetadata,
    ReflectionReadiness, ScalarLayout, ScalarType, TypeRegistry,
};
use std::sync::Arc;

struct Provider {
    types: TypeRegistry,
    workspace: WorkspaceId,
    snapshot: RuntimeInfoSnapshot,
    availability: Option<Option<Dependency>>,
    represented: TypeId,
}
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
        ProcedureAvailability::Missing
    }
    fn runtime_info(&self) -> RuntimeInfoAvailability<'_> {
        match &self.availability {
            None => RuntimeInfoAvailability::Ready {
                workspace: self.workspace,
                snapshot: &self.snapshot,
            },
            Some(Some(dependency)) => RuntimeInfoAvailability::Pending(dependency.clone()),
            Some(None) => RuntimeInfoAvailability::Missing,
        }
    }
}
fn provider() -> Provider {
    let mut types = TypeRegistry::new();
    let tag = types.reserve_enum(IntegerType::U32);
    types
        .define_enum(tag, [Integer::checked(IntegerType::U32, 2).unwrap()])
        .unwrap();
    let size = types.scalar(ScalarType::Int(IntegerType::S64));
    let header = types.reserve_record(RecordKind::Struct);
    types.define_record(header, [tag, size]).unwrap();
    types.bind_runtime_type_header(header).unwrap();
    let header_pointer = types.pointer(header).unwrap();
    let table = types.slice(header_pointer).unwrap();
    let segment_tag = types.reserve_enum(IntegerType::U16);
    types
        .define_enum(
            segment_tag,
            [0, 1, 2, 3, 5].map(|n| Integer::checked(IntegerType::U16, n).unwrap()),
        )
        .unwrap();
    let bytes = types
        .slice(types.scalar(ScalarType::Int(IntegerType::U8)))
        .unwrap();
    let segment = types.reserve_record(RecordKind::Struct);
    types.define_record(segment, [segment_tag, bytes]).unwrap();
    let segments = types.slice(segment).unwrap();
    let global = types.reserve_record(RecordKind::Struct);
    types
        .define_record(
            global,
            [types.scalar(ScalarType::Int(IntegerType::U64)), segments],
        )
        .unwrap();
    let global_pointer = types.pointer(global).unwrap();
    let info = types.reserve_record(RecordKind::Struct);
    types.define_record(info, [table, global_pointer]).unwrap();
    let schema = RuntimeInfoSchema::validate(&types, info, global).unwrap();
    let represented = types.scalar(ScalarType::Bool);
    let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
        &types,
        represented,
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    )
    .unwrap() else {
        panic!("builtin boolean descriptor must be ready")
    };
    let mut storage = StaticDataBuilder::new();
    let descriptor = storage.reserve(header, &types).unwrap();
    storage
        .define_type_descriptor(
            descriptor,
            StaticValue {
                ty: header,
                kind: StaticValueKind::Record(vec![
                    StaticValue::constant(ConstantValue {
                        ty: tag,
                        kind: ConstantKind::Enum(Integer::checked(IntegerType::U32, 2).unwrap()),
                    }),
                    StaticValue::constant(ConstantValue {
                        ty: size,
                        kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 1).unwrap()),
                    }),
                ]),
            },
            &graph,
            graph.root(),
            &types,
        )
        .unwrap();
    let backing = types.fixed_array(header_pointer, 1).unwrap();
    let backing_object = storage.reserve(backing, &types).unwrap();
    storage
        .define(
            backing_object,
            StaticValue {
                ty: backing,
                kind: StaticValueKind::Array(vec![StaticValue {
                    ty: header_pointer,
                    kind: StaticValueKind::Address(StaticAddress::new(descriptor)),
                }]),
            },
        )
        .unwrap();
    let object = storage.reserve(info, &types).unwrap();
    storage
        .define(
            object,
            StaticValue {
                ty: info,
                kind: StaticValueKind::Record(vec![
                    StaticValue {
                        ty: table,
                        kind: StaticValueKind::Slice {
                            data: Some(
                                StaticAddress::new(backing_object)
                                    .project(StaticProjection::Index(0)),
                            ),
                            count: 1,
                        },
                    },
                    StaticValue::constant(ConstantValue {
                        ty: global_pointer,
                        kind: ConstantKind::Zero,
                    }),
                ]),
            },
        )
        .unwrap();
    let data = Arc::new(
        storage
            .publish(&types, StaticDataLimits::default())
            .unwrap(),
    );
    let snapshot = RuntimeInfoSnapshot::new_compile_time(
        data,
        object,
        schema,
        &types,
        LayoutPolicy::lp64(),
        &[represented],
    )
    .unwrap();
    Provider {
        types,
        workspace: WorkspaceId::from_raw(7).unwrap(),
        snapshot,
        availability: None,
        represented,
    }
}
fn current() -> Value {
    Value::Int(Integer::checked(IntegerType::S64, -1).unwrap())
}

#[test]
fn certified_table_reads_actual_registered_descriptors_and_null_global_data() {
    let provider = provider();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let values = vm
        .compiler_runtime_info(provider.workspace, provider.snapshot.schema(), &[current()])
        .unwrap();
    let Value::Record {
        ty,
        fields,
    } = values[0].semantic()
    else {
        panic!("actual Runtime_Info value")
    };
    assert_eq!(*ty, provider.snapshot.schema().ty());
    let [
        Value::Slice {
            pointer: data,
            count,
            ..
        },
        Value::Pointer(global),
    ] = fields.as_slice()
    else {
        panic!("actual slice and global pointer")
    };
    assert_eq!(*count, 1);
    assert!(global.is_null());
    let Value::Pointer(descriptor) = vm.memory.load(&provider.types, data).unwrap() else {
        panic!("stored Type_Info address")
    };
    assert_eq!(
        vm.memory
            .runtime_type_identity(
                &provider.types,
                &Value::Type {
                    descriptor: Some(descriptor.clone())
                }
            )
            .unwrap()
            .ty(),
        provider.represented,
    );
    assert!(matches!(
        vm.memory
            .store(&provider.types, &descriptor, Value::Bool(false)),
        Err(Error::ReadOnlyStorage)
    ));
    assert!(vm.statistics.steps > 1);
}

#[test]
fn unavailable_and_pending_checkpoints_do_not_publish_empty_tables() {
    let mut provider = provider();
    provider.availability = Some(None);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.compiler_runtime_info(provider.workspace, provider.snapshot.schema(), &[current()]),
        Err(Halt::Failed(Error::EffectRejected(_)))
    ));
    drop(vm);
    let dependency = Dependency::Type(provider.represented);
    provider.availability = Some(Some(dependency.clone()));
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.compiler_runtime_info(provider.workspace, provider.snapshot.schema(), &[current()]),
        Err(Halt::Pending(actual)) if actual == dependency
    ));
}

#[test]
fn foreign_workspace_and_invalid_source_handles_cannot_borrow_the_checkpoint() {
    let provider = provider();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let foreign = Value::Int(Integer::checked(IntegerType::S64, 8).unwrap());
    assert!(matches!(
        vm.compiler_runtime_info(provider.workspace, provider.snapshot.schema(), &[foreign]),
        Err(Halt::Failed(Error::EffectRejected(_)))
    ));
    assert!(matches!(
        vm.compiler_runtime_info(
            WorkspaceId::from_raw(8).unwrap(),
            provider.snapshot.schema(),
            &[current()]
        ),
        Err(Halt::Failed(Error::InvalidIr(_)))
    ));
    for raw in [0, -2] {
        assert!(matches!(
            vm.compiler_runtime_info(
                provider.workspace,
                provider.snapshot.schema(),
                &[Value::Int(Integer::checked(IntegerType::S64, raw).unwrap())]
            ),
            Err(Halt::Failed(Error::InvalidIr(_)))
        ));
    }
}

#[test]
fn static_graph_admission_consumes_fuel_before_returning_the_table() {
    let provider = provider();
    let mut vm = Vm::new(
        &provider,
        crate::NoEffects,
        Limits {
            fuel: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        vm.compiler_runtime_info(provider.workspace, provider.snapshot.schema(), &[current()]),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(vm.statistics.steps, 2);
}

#[test]
fn a_certified_snapshot_cannot_cross_the_selected_vm_target() {
    let provider = provider();
    let policy = LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 4),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
        ScalarLayout::new(1, 1),
    )
    .unwrap();
    let mut vm = Vm::new_with_target(
        &provider,
        crate::NoEffects,
        Limits::default(),
        crate::ByteTarget {
            policy,
            endian: crate::Endian::Little,
        },
    )
    .unwrap();
    assert!(matches!(
        vm.compiler_runtime_info(provider.workspace, provider.snapshot.schema(), &[current()]),
        Err(Halt::Failed(Error::IrValidation(_)))
    ));
    assert!(vm.static_publications.is_empty());
}

#[test]
fn checked_compiler_call_returns_the_actual_runtime_info_record() {
    struct Callable {
        base: Provider,
        signature: TypeId,
        signatures: std::collections::HashMap<ProcedureId, TypeId>,
    }
    impl ProcedureProvider for Callable {
        fn types(&self) -> &dyn TypeView {
            &self.base.types
        }
        fn signatures(&self) -> &std::collections::HashMap<ProcedureId, TypeId> {
            &self.signatures
        }
        fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
            ProcedureAvailability::Compiler(crate::CompilerProcedure {
                signature: self.signature,
                intrinsic: crate::CompilerIntrinsic::SourceRuntimeInfo {
                    current_workspace: self.base.workspace,
                    schema: self.base.snapshot.schema(),
                },
            })
        }
        fn runtime_info(&self) -> RuntimeInfoAvailability<'_> {
            self.base.runtime_info()
        }
    }
    let mut base = provider();
    let id = ProcedureId::new(0);
    let signature = base
        .types
        .procedure(ProcedureType {
            parameters: vec![base.types.scalar(ScalarType::Int(IntegerType::S64))].into(),
            results: vec![base.snapshot.schema().ty()].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: jai_types::CallingConvention::Jai,
            context: jai_types::ContextMode::Implicit,
            variadic: jai_types::Variadic::None,
        })
        .unwrap();
    let provider = Callable {
        base,
        signature,
        signatures: [(id, signature)].into(),
    };
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let execution = vm.execute(id, vec![current()]);
    let Outcome::Complete(values) = execution.outcome else {
        panic!("{execution:?}")
    };
    let Value::Record {
        ty,
        fields,
    } = values[0].semantic()
    else {
        panic!("checked compiler result is a stored Runtime_Info")
    };
    assert_eq!(*ty, provider.base.snapshot.schema().ty());
    assert!(matches!(
        &fields[0],
        Value::Slice {
            count: 1,
            ..
        }
    ));
    assert!(matches!(&fields[1], Value::Pointer(pointer) if pointer.is_null()));
}
