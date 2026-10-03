//! Closed foreign ABI proofs. These select control operations, never fake scalar fork/exec results.
use jai_ir::{
    ForeignLibrary, ForeignLibraryId, ForeignLibraryKind, ProcedureId, ProcedurePrototype,
    PrototypeOrigin,
};
use jai_types::{
    Architecture, BuildTarget, ByteOrder, CallingConvention, ContextMode, DistinctKind,
    IntegerType, LayoutPolicy, OperatingSystem, RecordKind, TypeError, TypeId, TypeKind, TypeView,
    Variadic,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessAbiOperation {
    Fcntl,
    Fork,
    Pipe,
    Close,
    Read,
    Write,
    Dup2,
    WaitPid,
    ExecVp,
    Exit,
    ErrnoLocation,
    GetPid,
    GetParentPid,
    SocketPair,
    SendMsg,
    RecvMsg,
    Shutdown,
}
impl ProcessAbiOperation {
    fn symbol_matches(self, symbol: &str, target: &BuildTarget) -> bool {
        match self {
            Self::Fcntl => symbol == "fcntl",
            Self::Fork => symbol == "fork",
            Self::Pipe => symbol == "pipe",
            Self::Close => symbol == "close",
            Self::Read => symbol == "read",
            Self::Write => symbol == "write",
            Self::Dup2 => symbol == "dup2",
            Self::WaitPid => symbol == "waitpid",
            Self::ExecVp => symbol == "execvp",
            // Basic.exit is ordinary source (#asm/syscall); it never acquires this proof.
            Self::Exit => symbol == "_exit",
            Self::ErrnoLocation => match target.operating_system {
                OperatingSystem::MacOS => symbol == "__error",
                OperatingSystem::Linux => symbol == "__errno_location",
                _ => false,
            },
            Self::GetPid => symbol == "getpid",
            Self::GetParentPid => symbol == "getppid",
            Self::SocketPair => symbol == "socketpair",
            Self::SendMsg => symbol == "sendmsg",
            Self::RecvMsg => symbol == "recvmsg",
            Self::Shutdown => symbol == "shutdown",
        }
    }
}
/// Actual source nominal types; width-compatible substitutes do not confer authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessSocketTypes {
    pub socket_kind: TypeId,
    pub protocol: TypeId,
    pub message_flags: TypeId,
    pub shutdown_kind: TypeId,
    pub message_header: TypeId,
    pub control_header: TypeId,
    pub io_vector: TypeId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessAbiNominals {
    pub error_code: TypeId,
    pub socket: Option<ProcessSocketTypes>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessAbiError {
    Type(TypeError),
    Authority(&'static str),
    Signature(&'static str),
    Target(&'static str),
}
impl From<TypeError> for ProcessAbiError {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}
impl std::fmt::Display for ProcessAbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "process ABI binding: {self:?}")
    }
}
impl std::error::Error for ProcessAbiError {}

/// Trusted Rust receipt boundary, unavailable to Jai source. The embedding must verify
/// immutable source bytes/paths, graph identity, canonical library table and nominal origins.
/// Use a separate authority for each genuine base/header library declaration.
#[derive(Clone, Debug)]
pub struct ProcessAuthority {
    target: BuildTarget,
    library: ForeignLibrary,
    nominals: ProcessAbiNominals,
    declarations: HashMap<ProcedureId, (ProcessAbiOperation, TypeId)>,
}
/// All fields are private: callers cannot mutate a checked proof's signature or target.
#[derive(Clone, Debug)]
pub struct ProcessAbiProcedure {
    target: BuildTarget,
    procedure: ProcedureId,
    library: ForeignLibraryId,
    signature: TypeId,
    operation: ProcessAbiOperation,
    nominals: ProcessAbiNominals,
}
impl ProcessAbiProcedure {
    pub fn target(&self) -> &BuildTarget {
        &self.target
    }
    pub fn procedure(&self) -> ProcedureId {
        self.procedure
    }
    pub fn library(&self) -> ForeignLibraryId {
        self.library
    }
    pub fn signature(&self) -> TypeId {
        self.signature
    }
    pub fn operation(&self) -> ProcessAbiOperation {
        self.operation
    }
    pub fn nominals(&self) -> ProcessAbiNominals {
        self.nominals
    }
    pub fn validate(
        &self,
        target: &BuildTarget,
        types: &dyn TypeView,
    ) -> Result<(), ProcessAbiError> {
        if target != &self.target {
            return Err(ProcessAbiError::Target(
                "proof belongs to another source target",
            ));
        }
        validate_target(target)?;
        validate_nominals(self.nominals, types)?;
        validate_signature(self.operation, self.signature, self.nominals, types)
    }
}
impl ProcessAuthority {
    pub fn from_verified_source(
        target: BuildTarget,
        library: ForeignLibrary,
        nominals: ProcessAbiNominals,
        declarations: impl IntoIterator<Item = (ProcedureId, ProcessAbiOperation, TypeId)>,
        types: &dyn TypeView,
    ) -> Result<Self, ProcessAbiError> {
        validate_target(&target)?;
        library
            .validate()
            .map_err(|_| ProcessAbiError::Authority("invalid canonical library metadata"))?;
        if !matches!(library.id, ForeignLibraryId::File(_))
            || !matches!(&library.kind, ForeignLibraryKind::System { name } if name == "libc")
        {
            return Err(ProcessAbiError::Authority(
                "process ABI requires its canonical source libc declaration",
            ));
        }
        validate_nominals(nominals, types)?;
        let mut bindings = HashMap::new();
        for (id, operation, signature) in declarations {
            validate_signature(operation, signature, nominals, types)?;
            if bindings.insert(id, (operation, signature)).is_some() {
                return Err(ProcessAbiError::Authority(
                    "duplicate process declaration identity",
                ));
            }
        }
        Ok(Self {
            target,
            library,
            nominals,
            declarations: bindings,
        })
    }
    pub fn bind(
        &self,
        prototype: &ProcedurePrototype,
        target: &BuildTarget,
        types: &dyn TypeView,
    ) -> Result<ProcessAbiProcedure, ProcessAbiError> {
        if target != &self.target {
            return Err(ProcessAbiError::Target(
                "source receipt belongs to another target",
            ));
        }
        let (operation, expected) =
            self.declarations
                .get(&prototype.id)
                .copied()
                .ok_or(ProcessAbiError::Authority(
                    "procedure is not in the source process catalog",
                ))?;
        let PrototypeOrigin::Foreign {
            symbol,
            library: Some(library),
        } = &prototype.origin
        else {
            return Err(ProcessAbiError::Authority(
                "process declaration must retain its foreign library binding",
            ));
        };
        if library != &self.library || !operation.symbol_matches(symbol, target) {
            return Err(ProcessAbiError::Authority(
                "prototype differs from its exact source library/symbol receipt",
            ));
        }
        if prototype.signature != expected {
            return Err(ProcessAbiError::Signature(
                "prototype signature differs from its source receipt",
            ));
        }
        let bound = ProcessAbiProcedure {
            target: self.target.clone(),
            procedure: prototype.id,
            library: library.id,
            signature: expected,
            operation,
            nominals: self.nominals,
        };
        bound.validate(target, types)?;
        Ok(bound)
    }
}
fn validate_target(target: &BuildTarget) -> Result<(), ProcessAbiError> {
    if !matches!(
        target.operating_system,
        OperatingSystem::MacOS | OperatingSystem::Linux
    ) || !matches!(
        target.architecture,
        Architecture::Arm64 | Architecture::X86_64
    ) || target.byte_order != ByteOrder::Little
        || target.layout != LayoutPolicy::lp64()
    {
        return Err(ProcessAbiError::Target(
            "only inspected little-endian LP64 macOS/Linux profiles are supported",
        ));
    }
    Ok(())
}
fn validate_nominals(
    nominals: ProcessAbiNominals,
    types: &dyn TypeView,
) -> Result<(), ProcessAbiError> {
    let error = types.distinct_definition(nominals.error_code)?;
    if error.kind != DistinctKind::IsA || !integer(error.representation, IntegerType::S32, types)? {
        return Err(ProcessAbiError::Authority(
            "OS_Error_Code requires its actual IsA s32 nominal",
        ));
    }
    if let Some(socket) = nominals.socket {
        for (ty, representation) in [
            (socket.socket_kind, IntegerType::U32),
            (socket.protocol, IntegerType::U32),
            (socket.message_flags, IntegerType::S32),
            (socket.shutdown_kind, IntegerType::U32),
        ] {
            if types.enum_definition(ty)?.representation != representation {
                return Err(ProcessAbiError::Authority(
                    "Socket source enum representation differs",
                ));
            }
        }
        for ty in [
            socket.message_header,
            socket.control_header,
            socket.io_vector,
        ] {
            if types.record_definition(ty)?.kind != RecordKind::Struct {
                return Err(ProcessAbiError::Authority(
                    "Socket layouts require their actual source structs",
                ));
            }
        }
    }
    Ok(())
}
fn integer(ty: TypeId, expected: IntegerType, types: &dyn TypeView) -> Result<bool, TypeError> {
    Ok(matches!(types.kind(ty)?, TypeKind::Integer(actual) if *actual == expected))
}
fn pointer(ty: TypeId, expected: TypeId, types: &dyn TypeView) -> Result<bool, TypeError> {
    Ok(matches!(types.kind(ty)?, TypeKind::Pointer(actual) if *actual == expected))
}
fn void_pointer(ty: TypeId, types: &dyn TypeView) -> Result<bool, TypeError> {
    Ok(
        matches!(types.kind(ty)?, TypeKind::Pointer(actual) if matches!(types.kind(*actual)?, TypeKind::Void)),
    )
}
fn byte_pointer(ty: TypeId, types: &dyn TypeView) -> Result<bool, TypeError> {
    Ok(match types.kind(ty)? {
        TypeKind::Pointer(actual) => integer(*actual, IntegerType::U8, types)?,
        _ => false,
    })
}
fn descriptor_pair(ty: TypeId, types: &dyn TypeView) -> Result<bool, TypeError> {
    Ok(match types.kind(ty)? {
        TypeKind::Pointer(actual) => match types.kind(*actual)? {
            TypeKind::FixedArray { element, count: 2 } => {
                integer(*element, IntegerType::S32, types)?
            }
            _ => false,
        },
        _ => false,
    })
}
fn validate_signature(
    op: ProcessAbiOperation,
    signature: TypeId,
    nominals: ProcessAbiNominals,
    types: &dyn TypeView,
) -> Result<(), ProcessAbiError> {
    let signature = types.procedure_definition(signature)?;
    let expected_variadic = if op == ProcessAbiOperation::Fcntl {
        Variadic::C {
            fixed_parameters: 2,
        }
    } else {
        Variadic::None
    };
    if signature.convention != CallingConvention::C
        || signature.context != ContextMode::None
        || signature.variadic != expected_variadic
    {
        return Err(ProcessAbiError::Signature(
            "process ABI requires its exact C pack mode and no context",
        ));
    }
    let s32 = |ty| integer(ty, IntegerType::S32, types);
    let s64 = |ty| integer(ty, IntegerType::S64, types);
    let u64 = |ty| integer(ty, IntegerType::U64, types);
    let valid = match (
        op,
        signature.parameters.as_ref(),
        signature.results.as_ref(),
    ) {
        (ProcessAbiOperation::Fcntl, [fd, command], [result]) => {
            s32(*fd)? && s32(*command)? && s32(*result)?
        }
        (
            ProcessAbiOperation::Fork
            | ProcessAbiOperation::GetPid
            | ProcessAbiOperation::GetParentPid,
            [],
            [result],
        ) => s32(*result)?,
        (ProcessAbiOperation::Pipe, [pair], [result]) => {
            descriptor_pair(*pair, types)? && s32(*result)?
        }
        (ProcessAbiOperation::Close, [fd], [result]) => s32(*fd)? && s32(*result)?,
        (ProcessAbiOperation::Read | ProcessAbiOperation::Write, [fd, buffer, count], [result]) => {
            s32(*fd)? && void_pointer(*buffer, types)? && u64(*count)? && s64(*result)?
        }
        (ProcessAbiOperation::Dup2, [source, destination], [result]) => {
            s32(*source)? && s32(*destination)? && s32(*result)?
        }
        (ProcessAbiOperation::WaitPid, [pid, status, options], [result]) => {
            s32(*pid)?
                && matches!(types.kind(*status)?, TypeKind::Pointer(actual) if s32(*actual)?)
                && s32(*options)?
                && s32(*result)?
        }
        (ProcessAbiOperation::ExecVp, [file, argv], [result]) => {
            byte_pointer(*file, types)?
                && matches!(types.kind(*argv)?, TypeKind::Pointer(actual) if byte_pointer(*actual, types)?)
                && s32(*result)?
        }
        // Sema normalizes a sole source void result to an empty runtime result list.
        (ProcessAbiOperation::Exit, [status], []) => s32(*status)?,
        (ProcessAbiOperation::ErrnoLocation, [], [result]) => {
            pointer(*result, nominals.error_code, types)?
        }
        (ProcessAbiOperation::SocketPair, [domain, kind, protocol, pair], [result]) => {
            nominals
                .socket
                .is_some_and(|socket| *kind == socket.socket_kind && *protocol == socket.protocol)
                && s32(*domain)?
                && descriptor_pair(*pair, types)?
                && s32(*result)?
        }
        (
            ProcessAbiOperation::SendMsg | ProcessAbiOperation::RecvMsg,
            [fd, header, flags],
            [result],
        ) => match nominals.socket {
            Some(socket) => {
                s32(*fd)?
                    && pointer(*header, socket.message_header, types)?
                    && *flags == socket.message_flags
                    && s64(*result)?
            }
            None => false,
        },
        (ProcessAbiOperation::Shutdown, [fd, how], [result]) => {
            nominals
                .socket
                .is_some_and(|socket| *how == socket.shutdown_kind)
                && s32(*fd)?
                && s32(*result)?
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(ProcessAbiError::Signature(
            "parameters/results or actual nominal types differ",
        ))
    }
}

#[cfg(test)]
#[path = "process_abi/tests.rs"]
mod tests;
