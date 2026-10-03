use super::*;
use crate::file_abi::StdioAuthority;
use crate::{Limits, NoEffects, Pointer};
use jai_ir::{
    ForeignLibrary, ForeignLibraryId, ForeignLibraryKind, ProcedureId, ProcedurePrototype,
    PrototypeOrigin,
};
use jai_source::Identities;
use jai_types::{
    CallingConvention, CastMode, ContextMode, ProcedureType, RecordKind, ScalarType, TypeRegistry,
    Variadic,
};
fn catalog(types: &mut TypeRegistry) -> (TypeId, Vec<FileAbiProcedure>) {
    let file = types.reserve_record(RecordKind::Struct);
    types.define_record(file, []).unwrap();
    let stream = types.pointer(file).unwrap();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let text = types.pointer(byte).unwrap();
    let void = types.pointer(types.void()).unwrap();
    let u64 = types.scalar(ScalarType::Int(IntegerType::U64));
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let library = ForeignLibrary {
        id: ForeignLibraryId::new(Identities::default().declaration()),
        kind: ForeignLibraryKind::System {
            name: "libc".into(),
        },
        options: Default::default(),
    };
    let shapes = [
        (FileAbiOperation::Open, "fopen", vec![text, text], stream),
        (
            FileAbiOperation::Read,
            "fread",
            vec![void, u64, u64, stream],
            u64,
        ),
        (
            FileAbiOperation::Write,
            "fwrite",
            vec![void, u64, u64, stream],
            u64,
        ),
        (FileAbiOperation::Seek, "fseek", vec![stream, s64, s32], s32),
        (FileAbiOperation::Tell, "ftello", vec![stream], s64),
        (FileAbiOperation::Eof, "feof", vec![stream], s32),
        (FileAbiOperation::Close, "fclose", vec![stream], s32),
    ];
    let mut bindings = Vec::new();
    for (index, (operation, symbol, parameters, result)) in shapes.into_iter().enumerate() {
        let id = ProcedureId::new(index);
        let signature = types
            .procedure(ProcedureType {
                parameters: parameters.into(),
                results: vec![result].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let authority =
            StdioAuthority::from_verified_source(library.clone(), file, [(id, operation)], types)
                .unwrap();
        bindings.push(
            authority
                .bind(
                    &ProcedurePrototype {
                        id,
                        signature,
                        origin: PrototypeOrigin::Foreign {
                            symbol: symbol.into(),
                            library: Some(library.clone()),
                        },
                    },
                    types,
                )
                .unwrap(),
        );
    }
    (file, bindings)
}
fn buffer(types: &mut TypeRegistry, memory: &mut Memory, bytes: &[u8]) -> Pointer {
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let array = types.fixed_array(byte, bytes.len() as u64).unwrap();
    let pointer = memory
        .allocate(
            types,
            array,
            Some(Value::Array {
                ty: array,
                elements: bytes
                    .iter()
                    .map(|byte| Value::Int(Integer::wrapping(IntegerType::U8, i128::from(*byte))))
                    .collect(),
            }),
        )
        .unwrap();
    memory
        .cast_pointer(types, &pointer, byte, CastMode::Unchecked)
        .unwrap()
}
struct Effects {
    scope: FilePathScope,
    responses: Vec<HostOutcome>,
    requests: Vec<HostRequest>,
}
impl crate::CompilerEffects for Effects {
    fn begin(&mut self) {
    }
    fn request(&mut self, _: crate::CompilerRequest) -> crate::EffectOutcome {
        crate::EffectOutcome::Rejected("not a compiler request".into())
    }
    fn finish(&mut self, _: bool) -> Result<(), Error> {
        Ok(())
    }
    fn host_request(&mut self, request: HostRequest) -> HostOutcome {
        self.requests.push(request);
        self.responses.remove(0)
    }
    fn host_file_scope(&self) -> Option<FilePathScope> {
        Some(self.scope.clone())
    }
}
fn effects(responses: Vec<HostOutcome>) -> Effects {
    Effects {
        scope: FilePathScope::new(FileRootId::allocate(), "/own-fixture", "").unwrap(),
        responses,
        requests: Vec::new(),
    }
}
fn call(
    binding: FileAbiProcedure,
    args: Vec<Value>,
    memory: &mut Memory,
    types: &TypeRegistry,
    machine: &mut HostFileMachine,
    effects: &mut impl crate::CompilerEffects,
) -> Vec<Value> {
    binding.work_cost(&args, memory, types, machine).unwrap();
    match binding.invoke(&args, memory, types, machine, effects, &mut |_| Ok(())) {
        Ok(result) => result,
        Err(EffectError::Failed(error)) => panic!("{error:?}"),
        Err(EffectError::Pending(dependency)) => panic!("{dependency:?}"),
    }
}
fn u64(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::U64, value))
}
fn s64(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S64, value))
}
fn s32(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S32, value))
}
#[test]
fn closed_fork_bounds_include_retained_empty_nested_ledgers() {
    let mut types = TypeRegistry::new();
    let (file_type, _) = catalog(&mut types);
    let mut memory = Memory::new(Limits::default());
    let mut machine = HostFileMachine::default();
    let handle = machine
        .files
        .opened(
            HostPath::new(FileRootId::allocate(), "closed-fork-fixture").unwrap(),
            FileOpenMode::Read,
            vec![1],
        )
        .unwrap();
    let mut tokens = FileTokens::new(&types, file_type).unwrap();
    tokens.mint(&types, &mut memory, handle).unwrap();
    tokens.reset();
    let inner_capacity = tokens.closed_table_capacity().unwrap();
    assert!(inner_capacity > 0);
    machine.tokens.reserve(32);
    machine.tokens.insert(file_type, tokens);
    machine.files.reset();
    let expected = machine.tokens.capacity()
        + 2
        + machine.files.closed_table_capacity().unwrap()
        + 1
        + inner_capacity;
    let mut charged = 0;
    let cells = machine
        .closed_fork_bounds(&mut |work| {
            charged += work;
            Ok(())
        })
        .unwrap();
    assert_eq!(cells, expected);
    assert_eq!(charged, (machine.tokens.capacity() * 2 + 2) as u64);
}
#[test]
fn fork_bounds_charge_before_inspection_and_reject_live_streams() {
    let mut machine = HostFileMachine::default();
    machine
        .files
        .opened(
            HostPath::new(FileRootId::allocate(), "live-fork-fixture").unwrap(),
            FileOpenMode::Read,
            vec![1],
        )
        .unwrap();
    assert_eq!(
        machine.closed_fork_bounds(&mut |_| Err(Error::Limit(crate::LimitKind::Fuel))),
        Err(Error::Limit(crate::LimitKind::Fuel))
    );
    assert!(machine.closed_fork_bounds(&mut |_| Ok(())).is_err());
    assert!(machine.require_closed().is_err());
}
#[test]
fn adapter_reads_partial_items_seeks_eof_and_retires_opaque_storage() {
    let mut types = TypeRegistry::new();
    let (_, bindings) = catalog(&mut types);
    let mut memory = Memory::new(Limits::default());
    let path = buffer(&mut types, &mut memory, b"input\0");
    let mode = buffer(&mut types, &mut memory, b"rb\0");
    let output = buffer(&mut types, &mut memory, b".....");
    let void = memory
        .cast_pointer(&types, &output, types.void(), CastMode::Unchecked)
        .unwrap();
    let mut machine = HostFileMachine::default();
    let mut effects = effects(vec![HostOutcome::Ready(HostResponse::FileOpen(
        FileOpenObservation::Bytes(b"abc".to_vec()),
    ))]);
    let stream = call(
        bindings[0],
        vec![Value::Pointer(path), Value::Pointer(mode)],
        &mut memory,
        &types,
        &mut machine,
        &mut effects,
    )
    .remove(0);
    assert_eq!(
        memory.release(stream.pointer().unwrap()),
        Err(Error::ReadOnlyStorage)
    );
    assert!(machine.require_closed().is_err());
    assert_eq!(
        call(
            bindings[1],
            vec![Value::Pointer(void), u64(2), u64(2), stream.clone()],
            &mut memory,
            &types,
            &mut machine,
            &mut effects
        ),
        integer(IntegerType::U64, 1)
    );
    assert_eq!(
        memory.host_read_bytes(&types, &output, 5).unwrap(),
        b"abc.."
    );
    assert_eq!(
        call(
            bindings[5],
            vec![stream.clone()],
            &mut memory,
            &types,
            &mut machine,
            &mut effects
        ),
        integer(IntegerType::S32, 1)
    );
    assert_eq!(
        call(
            bindings[3],
            vec![stream.clone(), s64(-1), s32(2)],
            &mut memory,
            &types,
            &mut machine,
            &mut effects
        ),
        integer(IntegerType::S32, 0)
    );
    assert_eq!(
        call(
            bindings[4],
            vec![stream.clone()],
            &mut memory,
            &types,
            &mut machine,
            &mut effects
        ),
        integer(IntegerType::S64, 2)
    );
    assert_eq!(
        call(
            bindings[5],
            vec![stream.clone()],
            &mut memory,
            &types,
            &mut machine,
            &mut effects
        ),
        integer(IntegerType::S32, 0)
    );
    call(
        bindings[6],
        vec![stream.clone()],
        &mut memory,
        &types,
        &mut machine,
        &mut effects,
    );
    machine.require_closed().unwrap();
    assert!(machine.tokens.is_empty());
    assert_eq!(
        memory.load(&types, stream.pointer().unwrap()),
        Err(Error::DanglingPointer)
    );
}
#[test]
fn adapter_denies_missing_authority_and_keeps_pending_and_budget_observations_inert() {
    let mut types = TypeRegistry::new();
    let (_, bindings) = catalog(&mut types);
    let mut memory = Memory::new(Limits::default());
    let path = buffer(&mut types, &mut memory, b"input\0");
    let mode = buffer(&mut types, &mut memory, b"rb\0");
    let args = vec![Value::Pointer(path), Value::Pointer(mode)];
    let mut machine = HostFileMachine::default();
    assert!(matches!(
        bindings[0].invoke(
            &args,
            &mut memory,
            &types,
            &mut machine,
            &mut NoEffects,
            &mut |_| Ok(())
        ),
        Err(EffectError::Failed(Error::EffectRejected(_)))
    ));
    let key = HostRequestKey::allocate();
    let mut pending = effects(vec![HostOutcome::Pending(key)]);
    assert!(
        matches!(bindings[0].invoke(&args,&mut memory,&types,&mut machine,&mut pending,&mut |_|Ok(())),Err(EffectError::Pending(Dependency::Host(actual))) if actual==key)
    );
    machine.require_closed().unwrap();
    let mut observed = effects(vec![HostOutcome::Ready(HostResponse::FileOpen(
        FileOpenObservation::Bytes(b"payload".to_vec()),
    ))]);
    assert!(matches!(
        bindings[0].invoke(
            &args,
            &mut memory,
            &types,
            &mut machine,
            &mut observed,
            &mut |_| Err(Error::Limit(crate::LimitKind::Fuel))
        ),
        Err(EffectError::Failed(Error::Limit(crate::LimitKind::Fuel)))
    ));
    machine.require_closed().unwrap();
    assert_eq!(observed.requests.len(), 1);
}
#[test]
fn writable_adapter_stages_open_and_close_and_retains_token_on_pending_close() {
    let mut types = TypeRegistry::new();
    let (_, bindings) = catalog(&mut types);
    let mut memory = Memory::new(Limits::default());
    let path = buffer(&mut types, &mut memory, b"output\0");
    let mode = buffer(&mut types, &mut memory, b"wb+\0");
    let bytes = buffer(&mut types, &mut memory, b"data");
    let void = memory
        .cast_pointer(&types, &bytes, types.void(), CastMode::Unchecked)
        .unwrap();
    let mut machine = HostFileMachine::default();
    let pending = HostRequestKey::allocate();
    let mut effects = effects(vec![
        HostOutcome::Ready(HostResponse::WriteStaged),
        HostOutcome::Pending(pending),
        HostOutcome::Ready(HostResponse::WriteStaged),
    ]);
    let stream = call(
        bindings[0],
        vec![Value::Pointer(path), Value::Pointer(mode)],
        &mut memory,
        &types,
        &mut machine,
        &mut effects,
    )
    .remove(0);
    assert_eq!(
        call(
            bindings[2],
            vec![Value::Pointer(void), u64(1), u64(4), stream.clone()],
            &mut memory,
            &types,
            &mut machine,
            &mut effects
        ),
        integer(IntegerType::U64, 4)
    );
    assert!(
        matches!(bindings[6].invoke(std::slice::from_ref(&stream),&mut memory,&types,&mut machine,&mut effects,&mut |_|Ok(())),Err(EffectError::Pending(Dependency::Host(key))) if key==pending)
    );
    assert!(machine.require_closed().is_err());
    assert_eq!(
        memory.release(stream.pointer().unwrap()),
        Err(Error::ReadOnlyStorage)
    );
    call(
        bindings[6],
        vec![stream],
        &mut memory,
        &types,
        &mut machine,
        &mut effects,
    );
    machine.require_closed().unwrap();
    assert!(
        matches!(&effects.requests[0],HostRequest::WriteEntireFile {bytes,..} if bytes.is_empty())
    );
    assert!(
        matches!(&effects.requests[1],HostRequest::WriteEntireFile {bytes,..} if bytes==b"data")
    );
    assert_eq!(effects.requests[1], effects.requests[2]);
}
struct Provider {
    types: TypeRegistry,
    binding: FileAbiProcedure,
    signatures: HashMap<ProcedureId, TypeId>,
}
impl crate::ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        &self.signatures
    }
    fn procedure(&self, _: ProcedureId) -> crate::ProcedureAvailability<'_> {
        crate::ProcedureAvailability::FileAbi(self.binding)
    }
}
fn vm_string(memory: &mut Memory, types: &TypeRegistry, bytes: &[u8]) -> Pointer {
    let pointer = memory
        .allocate(types, types.string(), Some(Value::String(bytes.to_vec())))
        .unwrap();
    memory
        .cast_pointer(
            types,
            &pointer,
            types.scalar(ScalarType::Int(IntegerType::U8)),
            CastMode::Unchecked,
        )
        .unwrap()
}
#[test]
fn vm_dispatch_rolls_back_pending_and_open_stream_exit_and_rejects_wrong_procedure_identity() {
    let mut types = TypeRegistry::new();
    let (_, bindings) = catalog(&mut types);
    let binding = bindings[0];
    let provider = Provider {
        types,
        binding,
        signatures: HashMap::from([
            (binding.procedure(), binding.signature),
            (ProcedureId::new(99), binding.signature),
        ]),
    };
    let host = effects(vec![HostOutcome::Ready(HostResponse::FileOpen(
        FileOpenObservation::Bytes(b"abc".to_vec()),
    ))]);
    let mut vm = crate::Vm::new(&provider, host, Limits::default()).unwrap();
    let path = vm_string(vm.memory_mut(), &provider.types, b"input\0");
    let mode = vm_string(vm.memory_mut(), &provider.types, b"rb\0");
    let args = vec![Value::Pointer(path.clone()), Value::Pointer(mode.clone())];
    assert!(matches!(
        vm.execute(binding.procedure(), args.clone()).outcome,
        crate::Outcome::Failed(Error::EffectRejected(_))
    ));
    vm.configure_file_limits(FileLimits::default()).unwrap();
    assert_eq!(
        vm.memory()
            .host_c_string(&provider.types, &path, 64)
            .unwrap(),
        b"input"
    );
    assert!(matches!(
        vm.execute(ProcedureId::new(99), args).outcome,
        crate::Outcome::Failed(Error::InvalidIr("FILE capability names another procedure"))
    ));
    assert_eq!(vm.effects().requests.len(), 1);
    let pending = HostRequestKey::allocate();
    let host = effects(vec![HostOutcome::Pending(pending)]);
    let mut vm = crate::Vm::new(&provider, host, Limits::default()).unwrap();
    let path = vm_string(vm.memory_mut(), &provider.types, b"input\0");
    let mode = vm_string(vm.memory_mut(), &provider.types, b"rb\0");
    assert_eq!(
        vm.execute(
            binding.procedure(),
            vec![Value::Pointer(path), Value::Pointer(mode)]
        )
        .outcome,
        crate::Outcome::Pending(vec![Dependency::Host(pending)])
    );
    vm.configure_file_limits(FileLimits::default()).unwrap();
}

#[test]
fn general_snapshot_permits_live_file_tokens_and_charges_each_walk_before_copy() {
    let mut types = TypeRegistry::new();
    let (file_type, _) = catalog(&mut types);
    let mut memory = Memory::new(Limits::default());
    let mut machine = HostFileMachine::default();
    let mut bytes = Vec::with_capacity(4096);
    bytes.extend_from_slice(b"abc");
    let handle = machine
        .files
        .opened(
            HostPath::new(FileRootId::allocate(), "snapshot-live-fixture").unwrap(),
            FileOpenMode::Read,
            bytes,
        )
        .unwrap();
    machine.files.seek(handle, 1, SeekOrigin::Start).unwrap();
    let mut tokens = FileTokens::new(&types, file_type).unwrap();
    let pointer = tokens.mint(&types, &mut memory, handle).unwrap();
    machine.tokens.insert(file_type, tokens);
    let expected = machine.tokens.capacity()
        + 1
        + machine.files.snapshot_bounds(&mut |_| Ok(())).unwrap()
        + machine.tokens[&file_type]
            .snapshot_bounds(&mut |_| Ok(()))
            .unwrap();
    let mut calls = 0;
    assert_eq!(
        machine
            .snapshot_bounds(&mut |_| {
                calls += 1;
                Ok(())
            })
            .unwrap(),
        expected
    );
    assert_eq!(calls, 3);
    assert!(expected > 4096);
    assert!(machine.closed_fork_bounds(&mut |_| Ok(())).is_err());
    assert_eq!(machine.files.tell(handle).unwrap(), 1);
    assert_eq!(
        machine.tokens[&file_type]
            .lookup(&types, &memory, &pointer)
            .unwrap(),
        handle
    );
    for denied_call in 1..=3 {
        let mut calls = 0;
        let result = machine.snapshot_bounds(&mut |_| {
            calls += 1;
            if calls == denied_call {
                Err(Error::Limit(crate::LimitKind::Fuel))
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Err(Error::Limit(crate::LimitKind::Fuel)));
        assert_eq!(calls, denied_call);
        assert_eq!(machine.files.tell(handle).unwrap(), 1);
        assert_eq!(machine.tokens[&file_type].live_tokens(), 1);
    }
}
