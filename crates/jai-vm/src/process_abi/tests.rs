use super::*;
use jai_source::Identities;
use jai_types::{ProcedureType, ScalarType, TypeRegistry};

fn target() -> BuildTarget {
    BuildTarget {
        operating_system: OperatingSystem::MacOS,
        architecture: Architecture::Arm64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    }
}
fn library() -> ForeignLibrary {
    ForeignLibrary {
        id: ForeignLibraryId::new(Identities::default().declaration()),
        kind: ForeignLibraryKind::System {
            name: "libc".into(),
        },
        options: Default::default(),
    }
}
fn signature(types: &mut TypeRegistry, parameters: &[TypeId], results: &[TypeId]) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}
fn nominal_enum(types: &mut TypeRegistry, repr: IntegerType) -> TypeId {
    let ty = types.reserve_enum(repr);
    types.define_enum(ty, []).unwrap();
    ty
}
fn nominals(types: &mut TypeRegistry) -> ProcessAbiNominals {
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let error_code = types.reserve_distinct(DistinctKind::IsA);
    types.define_distinct(error_code, s32).unwrap();
    let header = types.reserve_record(RecordKind::Struct);
    types.define_record(header, []).unwrap();
    ProcessAbiNominals {
        error_code,
        socket: Some(ProcessSocketTypes {
            socket_kind: nominal_enum(types, IntegerType::U32),
            protocol: nominal_enum(types, IntegerType::U32),
            message_flags: nominal_enum(types, IntegerType::S32),
            shutdown_kind: nominal_enum(types, IntegerType::U32),
            message_header: header,
            control_header: header,
            io_vector: header,
        }),
    }
}
fn prototype(
    id: ProcedureId,
    signature: TypeId,
    symbol: &str,
    library: ForeignLibrary,
) -> ProcedurePrototype {
    ProcedurePrototype {
        id,
        signature,
        origin: PrototypeOrigin::Foreign {
            symbol: symbol.into(),
            library: Some(library),
        },
    }
}
fn authority(
    types: &dyn TypeView,
    nominals: ProcessAbiNominals,
    id: ProcedureId,
    op: ProcessAbiOperation,
    signature: TypeId,
) -> ProcessAuthority {
    ProcessAuthority::from_verified_source(
        target(),
        library(),
        nominals,
        [(id, op, signature)],
        types,
    )
    .unwrap()
}

#[test]
fn inspected_base_and_socket_catalog_binds_only_complete_exact_source_shapes() {
    let mut types = TypeRegistry::new();
    let nominals = nominals(&mut types);
    let socket = nominals.socket.unwrap();
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let u64 = types.scalar(ScalarType::Int(IntegerType::U64));
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let text = types.pointer(byte).unwrap();
    let argv = types.pointer(text).unwrap();
    let buffer = types.pointer(types.void()).unwrap();
    let status = types.pointer(s32).unwrap();
    let array = types.fixed_array(s32, 2).unwrap();
    let pair = types.pointer(array).unwrap();
    let errno = types.pointer(nominals.error_code).unwrap();
    let header = types.pointer(socket.message_header).unwrap();
    let shapes = [
        (ProcessAbiOperation::Fork, "fork", vec![], vec![s32]),
        (ProcessAbiOperation::Pipe, "pipe", vec![pair], vec![s32]),
        (ProcessAbiOperation::Close, "close", vec![s32], vec![s32]),
        (
            ProcessAbiOperation::Read,
            "read",
            vec![s32, buffer, u64],
            vec![s64],
        ),
        (
            ProcessAbiOperation::Write,
            "write",
            vec![s32, buffer, u64],
            vec![s64],
        ),
        (ProcessAbiOperation::Dup2, "dup2", vec![s32, s32], vec![s32]),
        (
            ProcessAbiOperation::WaitPid,
            "waitpid",
            vec![s32, status, s32],
            vec![s32],
        ),
        (
            ProcessAbiOperation::ExecVp,
            "execvp",
            vec![text, argv],
            vec![s32],
        ),
        (ProcessAbiOperation::Exit, "_exit", vec![s32], vec![]),
        (
            ProcessAbiOperation::ErrnoLocation,
            "__error",
            vec![],
            vec![errno],
        ),
        (ProcessAbiOperation::GetPid, "getpid", vec![], vec![s32]),
        (
            ProcessAbiOperation::GetParentPid,
            "getppid",
            vec![],
            vec![s32],
        ),
        (
            ProcessAbiOperation::SocketPair,
            "socketpair",
            vec![s32, socket.socket_kind, socket.protocol, pair],
            vec![s32],
        ),
        (
            ProcessAbiOperation::SendMsg,
            "sendmsg",
            vec![s32, header, socket.message_flags],
            vec![s64],
        ),
        (
            ProcessAbiOperation::RecvMsg,
            "recvmsg",
            vec![s32, header, socket.message_flags],
            vec![s64],
        ),
        (
            ProcessAbiOperation::Shutdown,
            "shutdown",
            vec![s32, socket.shutdown_kind],
            vec![s32],
        ),
    ];
    for (index, (op, symbol, parameters, results)) in shapes.into_iter().enumerate() {
        let id = ProcedureId::new(index);
        let sig = signature(&mut types, &parameters, &results);
        let authority = authority(&types, nominals, id, op, sig);
        let proof = authority
            .bind(&prototype(id, sig, symbol, library()), &target(), &types)
            .unwrap();
        assert_eq!(proof.signature(), sig);
        assert_eq!(proof.operation(), op);
        assert_eq!(proof.procedure(), id);
        assert_eq!(proof.library(), library().id);
        assert_eq!(proof.nominals(), nominals);
        proof.validate(&target(), &types).unwrap();
        assert!(proof.validate(&target(), &TypeRegistry::new()).is_err());
    }
}

#[test]
fn spelling_ids_library_metadata_origin_and_target_cannot_replace_receipt() {
    let mut types = TypeRegistry::new();
    let nominals = nominals(&mut types);
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let sig = signature(&mut types, &[], &[s32]);
    let id = ProcedureId::new(7);
    let auth = authority(&types, nominals, id, ProcessAbiOperation::Fork, sig);
    let good = prototype(id, sig, "fork", library());
    let mut wrong = good.clone();
    wrong.id = ProcedureId::new(8);
    assert!(auth.bind(&wrong, &target(), &types).is_err());
    let mut wrong = good.clone();
    wrong.origin = PrototypeOrigin::Compiler;
    assert!(auth.bind(&wrong, &target(), &types).is_err());
    let mut wrong = good.clone();
    wrong.origin = PrototypeOrigin::Foreign {
        symbol: "fork".into(),
        library: None,
    };
    assert!(auth.bind(&wrong, &target(), &types).is_err());
    let mut identities = Identities::default();
    identities.declaration();
    let mut other = library();
    other.id = ForeignLibraryId::new(identities.declaration());
    assert!(
        auth.bind(&prototype(id, sig, "fork", other), &target(), &types)
            .is_err()
    );
    assert!(
        auth.bind(&prototype(id, sig, "getpid", library()), &target(), &types)
            .is_err()
    );
    let mut changed = target();
    changed.architecture = Architecture::X86_64;
    assert!(auth.bind(&good, &changed, &types).is_err());
    changed = target();
    changed.operating_system = OperatingSystem::Linux;
    assert!(auth.bind(&good, &changed, &types).is_err());
}

#[test]
fn wrong_width_varargs_context_calling_convention_and_void_success_are_rejected() {
    let mut types = TypeRegistry::new();
    let nominals = nominals(&mut types);
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let id = ProcedureId::new(0);
    for (convention, context, variadic, result) in [
        (CallingConvention::C, ContextMode::None, Variadic::None, s64),
        (
            CallingConvention::Jai,
            ContextMode::None,
            Variadic::None,
            s32,
        ),
        (
            CallingConvention::C,
            ContextMode::Implicit,
            Variadic::None,
            s32,
        ),
        (
            CallingConvention::C,
            ContextMode::None,
            Variadic::C {
                fixed_parameters: 0,
            },
            s32,
        ),
    ] {
        let sig = types
            .procedure(ProcedureType {
                parameters: [].into(),
                results: [result].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention,
                context,
                variadic,
            })
            .unwrap();
        assert!(
            ProcessAuthority::from_verified_source(
                target(),
                library(),
                nominals,
                [(id, ProcessAbiOperation::Fork, sig)],
                &types
            )
            .is_err()
        );
    }
    let sig = signature(&mut types, &[s32], &[]);
    let auth = authority(&types, nominals, id, ProcessAbiOperation::Exit, sig);
    assert!(
        auth.bind(&prototype(id, sig, "exit", library()), &target(), &types)
            .is_err()
    );
    assert!(
        types
            .procedure(ProcedureType {
                parameters: [s32].into(),
                results: [types.void()].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .is_err()
    );
    let wrong = signature(&mut types, &[s32], &[s32]);
    assert!(
        ProcessAuthority::from_verified_source(
            target(),
            library(),
            nominals,
            [(id, ProcessAbiOperation::Exit, wrong)],
            &types,
        )
        .is_err()
    );
}

#[test]
fn same_layout_nominals_and_pointer_shapes_are_not_interchangeable() {
    let mut types = TypeRegistry::new();
    let nominals = nominals(&mut types);
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let other_nominals = self::nominals(&mut types);
    let wrong_errno = types.pointer(other_nominals.error_code).unwrap();
    let wrong_sig = signature(&mut types, &[], &[wrong_errno]);
    assert!(
        ProcessAuthority::from_verified_source(
            target(),
            library(),
            nominals,
            [(
                ProcedureId::new(0),
                ProcessAbiOperation::ErrnoLocation,
                wrong_sig
            )],
            &types
        )
        .is_err()
    );
    let scalar_ptr = types.pointer(s32).unwrap();
    let wrong_pipe = signature(&mut types, &[scalar_ptr], &[s32]);
    assert!(
        ProcessAuthority::from_verified_source(
            target(),
            library(),
            nominals,
            [(ProcedureId::new(0), ProcessAbiOperation::Pipe, wrong_pipe)],
            &types
        )
        .is_err()
    );
    let socket = nominals.socket.unwrap();
    let other = other_nominals.socket.unwrap();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let other_header = types.pointer(other.message_header).unwrap();
    let sig = signature(
        &mut types,
        &[s32, other_header, socket.message_flags],
        &[s64],
    );
    assert!(
        ProcessAuthority::from_verified_source(
            target(),
            library(),
            nominals,
            [(ProcedureId::new(0), ProcessAbiOperation::SendMsg, sig)],
            &types
        )
        .is_err()
    );
    let correct_header = types.pointer(socket.message_header).unwrap();
    let sig = signature(
        &mut types,
        &[s32, correct_header, other.message_flags],
        &[s64],
    );
    assert!(
        ProcessAuthority::from_verified_source(
            target(),
            library(),
            nominals,
            [(ProcedureId::new(0), ProcessAbiOperation::SendMsg, sig)],
            &types
        )
        .is_err()
    );
    let incomplete = types.reserve_record(RecordKind::Struct);
    let mut bad_nominals = nominals;
    bad_nominals.socket.as_mut().unwrap().message_header = incomplete;
    assert!(
        ProcessAuthority::from_verified_source(target(), library(), bad_nominals, [], &types)
            .is_err()
    );
}

#[test]
fn exact_source_profile_selects_errno_symbol_and_refuses_uninspected_targets() {
    let mut types = TypeRegistry::new();
    let nominals = nominals(&mut types);
    let errno = types.pointer(nominals.error_code).unwrap();
    let sig = signature(&mut types, &[], &[errno]);
    let id = ProcedureId::new(0);
    let mut linux = target();
    linux.operating_system = OperatingSystem::Linux;
    let auth = ProcessAuthority::from_verified_source(
        linux.clone(),
        library(),
        nominals,
        [(id, ProcessAbiOperation::ErrnoLocation, sig)],
        &types,
    )
    .unwrap();
    assert!(
        auth.bind(
            &prototype(id, sig, "__errno_location", library()),
            &linux,
            &types
        )
        .is_ok()
    );
    assert!(
        auth.bind(&prototype(id, sig, "__error", library()), &linux, &types)
            .is_err()
    );
    for (os, arch, order) in [
        (
            OperatingSystem::Windows,
            Architecture::X86_64,
            ByteOrder::Little,
        ),
        (
            OperatingSystem::Android,
            Architecture::Arm64,
            ByteOrder::Little,
        ),
        (OperatingSystem::MacOS, Architecture::X86, ByteOrder::Little),
        (OperatingSystem::Linux, Architecture::Arm64, ByteOrder::Big),
    ] {
        let mut bad = target();
        bad.operating_system = os;
        bad.architecture = arch;
        bad.byte_order = order;
        assert!(
            ProcessAuthority::from_verified_source(bad, library(), nominals, [], &types).is_err()
        );
    }
    assert!(
        ProcessAuthority::from_verified_source(
            target(),
            library(),
            nominals,
            [
                (id, ProcessAbiOperation::ErrnoLocation, sig),
                (id, ProcessAbiOperation::ErrnoLocation, sig)
            ],
            &types
        )
        .is_err()
    );
}

#[test]
fn fcntl_requires_actual_c_ellipsis_two_fixed_parameters_and_s32_result() {
    let mut types = TypeRegistry::new();
    let nominals = nominals(&mut types);
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let sig = types
        .procedure(ProcedureType {
            parameters: [s32, s32].into(),
            results: [s32].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::C {
                fixed_parameters: 2,
            },
        })
        .unwrap();
    let id = ProcedureId::new(99);
    let auth = authority(&types, nominals, id, ProcessAbiOperation::Fcntl, sig);
    auth.bind(&prototype(id, sig, "fcntl", library()), &target(), &types)
        .unwrap();
    let fixed = signature(&mut types, &[s32, s32], &[s32]);
    assert!(
        ProcessAuthority::from_verified_source(
            target(),
            library(),
            nominals,
            [(id, ProcessAbiOperation::Fcntl, fixed)],
            &types
        )
        .is_err()
    );
    assert!(
        auth.bind(&prototype(id, fixed, "fcntl", library()), &target(), &types)
            .is_err()
    );
}
