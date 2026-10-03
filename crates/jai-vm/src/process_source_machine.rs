//! Source foreign calls over private branch memory and a shared virtual process world.
use crate::{
    ByteTarget, Error, LimitKind, Memory, Pointer, Value,
    host_effects::{HostRequestKey, ProcessTermination},
    process_abi::{ProcessAbiError, ProcessAbiOperation, ProcessAbiProcedure},
    virtual_process::{
        ProcessError, ProcessEvent, ProcessId, ProcessIo, VirtualProcesses, WaitOutcome,
    },
};
use jai_types::{BuildTarget, Integer, IntegerType, OperatingSystem, TypeId, TypeView};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ProcessBranchState {
    current: ProcessId,
    errno: HashMap<TypeId, Pointer>,
}
impl ProcessBranchState {
    pub fn new(current: ProcessId) -> Self {
        Self {
            current,
            errno: HashMap::new(),
        }
    }
    pub fn current(&self) -> ProcessId {
        self.current
    }
    /// Cached branch-local metadata retention; pointed-to cells belong to private Memory.
    pub fn retained_metadata_cells(&self) -> usize {
        self.errno.capacity()
    }
    /// The scheduler must also fork private Memory before using the cloned pointers.
    pub fn for_child(&self, child: ProcessId, world: &VirtualProcesses) -> Result<Self, Error> {
        if world.parent(child).map_err(process_error)? != Some(self.current) {
            return Err(Error::InvalidIr(
                "process branch is not an actual virtual child",
            ));
        }
        let mut branch = self.clone();
        branch.current = child;
        Ok(branch)
    }
    fn errno_pointer(
        &mut self,
        ty: TypeId,
        memory: &mut Memory,
        types: &dyn TypeView,
    ) -> Result<Pointer, Error> {
        if let Some(pointer) = self.errno.get(&ty) {
            // Reject stale or cross-memory branch state before publishing an errno pointer.
            memory.load(types, pointer)?;
            return Ok(pointer.clone());
        }
        let pointer = memory.allocate(types, ty, Some(errno_value(ty, 0)))?;
        self.errno.insert(ty, pointer.clone());
        Ok(pointer)
    }
    fn set_errno(
        &mut self,
        ty: TypeId,
        code: i32,
        memory: &mut Memory,
        types: &dyn TypeView,
    ) -> Result<(), Error> {
        let pointer = self.errno_pointer(ty, memory, types)?;
        memory.store(types, &pointer, errno_value(ty, code))
    }
}
/// No public constructor can grant a fork control transfer from a scalar or spelling.
#[derive(Clone, Debug)]
pub struct ProcessForkControl {
    procedure: ProcessAbiProcedure,
    parent: ProcessId,
}
impl ProcessForkControl {
    pub fn procedure(&self) -> &ProcessAbiProcedure {
        &self.procedure
    }
    pub fn parent(&self) -> ProcessId {
        self.parent
    }
}
#[derive(Clone, Debug)]
pub struct ProcessExitControl {
    procedure: ProcessAbiProcedure,
    process: ProcessId,
    status: i32,
}
impl ProcessExitControl {
    pub fn procedure(&self) -> &ProcessAbiProcedure {
        &self.procedure
    }
    pub fn process(&self) -> ProcessId {
        self.process
    }
    pub fn status(&self) -> i32 {
        self.status
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessExecStage {
    HostPending(HostRequestKey),
    Replaced,
}
/// Only the exact argv/catalog/host-completion route can construct this token.
#[derive(Clone, Debug)]
pub struct ProcessExecControl {
    procedure: ProcessAbiProcedure,
    process: ProcessId,
    stage: ProcessExecStage,
}
impl ProcessExecControl {
    pub fn procedure(&self) -> &ProcessAbiProcedure {
        &self.procedure
    }
    pub fn process(&self) -> ProcessId {
        self.process
    }
    pub fn stage(&self) -> ProcessExecStage {
        self.stage
    }
}
#[derive(Clone, Debug)]
pub enum ProcessCallOutcome {
    Values(Vec<Value>),
    Pending(ProcessEvent),
    ForkControl(ProcessForkControl),
    ExitControl(ProcessExitControl),
    ExecControl(ProcessExecControl),
}
fn process_error(error: ProcessError) -> Error {
    Error::EffectRejected(error.to_string())
}
fn binding_error(error: ProcessAbiError) -> Error {
    match error {
        ProcessAbiError::Type(error) => Error::Type(error),
        error => Error::EffectRejected(error.to_string()),
    }
}
fn plain(value: &Value, ty: IntegerType) -> Result<Integer, Error> {
    let integer = value.number()?.portable_integer()?;
    if integer.ty() != ty {
        return Err(Error::InvalidIr("process ABI scalar type mismatch"));
    }
    Ok(integer)
}
fn fd_number(value: &Value) -> Result<i32, Error> {
    i32::try_from(plain(value, IntegerType::S32)?.value()).map_err(|_| Error::CheckedCast)
}
fn count(value: &Value, world: &VirtualProcesses) -> Result<usize, Error> {
    let count =
        usize::try_from(plain(value, IntegerType::U64)?.bits()).map_err(|_| Error::CheckedCast)?;
    if count > world.limits().transfer_bytes {
        return Err(process_error(ProcessError::Budget("transfer bytes")));
    }
    Ok(count)
}
fn int(ty: IntegerType, value: i128) -> Value {
    Value::Int(Integer::checked(ty, value).expect("checked process integer result"))
}
fn errno_value(ty: TypeId, code: i32) -> Value {
    Value::Distinct {
        ty,
        value: Box::new(int(IntegerType::S32, i128::from(code))),
    }
}
fn values(ty: IntegerType, value: i128) -> ProcessCallOutcome {
    ProcessCallOutcome::Values(vec![int(ty, value)])
}
fn recoverable(error: &ProcessError, target: &BuildTarget) -> Option<i32> {
    match error {
        ProcessError::UnknownDescriptor
        | ProcessError::WrongDirection
        | ProcessError::InvalidDescriptor => Some(9),
        ProcessError::WouldBlock => Some(if target.operating_system == OperatingSystem::MacOS {
            35
        } else {
            11
        }),
        ProcessError::BrokenPipe => Some(32),
        _ => None,
    }
}
impl ProcessAbiProcedure {
    /// Validate only the exact bound ABI, including its closed fcntl C-vararg protocol.
    /// The central bridge uses this before dispatch rather than rejecting every C tail.
    pub(crate) fn validate_process_arguments(
        &self,
        args: &[Value],
        types: &dyn TypeView,
        target: &BuildTarget,
    ) -> Result<(), Error> {
        self.validate(target, types).map_err(binding_error)?;
        let signature = types.procedure_definition(self.signature())?;
        if self.operation() == ProcessAbiOperation::Fcntl {
            for (value, ty) in args.iter().zip(signature.parameters.iter()) {
                value.validate(types, *ty, 128)?;
            }
            return flags::validate_arguments(args);
        }
        if args.len() != signature.parameters.len() {
            return Err(Error::InvalidIr("process ABI argument count"));
        }
        for (value, ty) in args.iter().zip(signature.parameters.iter()) {
            value.validate(types, *ty, 128)?;
        }
        Ok(())
    }
    /// Central dispatch must retain the operands on Pending and implement sealed control transfers.
    /// Charge succeeds before any branch allocation, byte copy or shared-world mutation.
    #[allow(
        clippy::too_many_arguments,
        reason = "branch memory, shared world and fuel retain separate scheduler ownership"
    )]
    pub fn invoke_process(
        &self,
        args: &[Value],
        memory: &mut Memory,
        types: &dyn TypeView,
        target: &BuildTarget,
        branch: &mut ProcessBranchState,
        world: &mut VirtualProcesses,
        charge: &mut impl FnMut(u64) -> Result<(), Error>,
    ) -> Result<ProcessCallOutcome, Error> {
        self.validate_process_arguments(args, types, target)?;
        if memory.target() != ByteTarget::from(target) {
            return Err(Error::InvalidIr(
                "process branch memory differs from source target",
            ));
        }
        world
            .validate_running(branch.current)
            .map_err(process_error)?;
        let op = self.operation();
        if op == ProcessAbiOperation::Fcntl {
            return flags::invoke(self, args, memory, types, target, branch, world, charge);
        }
        if matches!(
            op,
            ProcessAbiOperation::SocketPair
                | ProcessAbiOperation::SendMsg
                | ProcessAbiOperation::RecvMsg
                | ProcessAbiOperation::Shutdown
        ) {
            return socket::invoke(self, args, memory, types, target, branch, world, charge);
        }
        let mut cost = world
            .work_cost()
            .map_err(process_error)?
            .checked_mul(2)
            .ok_or(Error::Limit(LimitKind::Fuel))?
            .checked_add(
                u64::try_from(branch.errno.capacity())
                    .map_err(|_| Error::Limit(LimitKind::Fuel))?,
            )
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        let extra = match op {
            ProcessAbiOperation::Read | ProcessAbiOperation::Write => {
                fd_number(&args[0])?;
                let count = count(&args[2], world)?;
                memory.intrinsic_work_cost(
                    types,
                    &[(args[1].pointer()?, op == ProcessAbiOperation::Read)],
                    count,
                )?
            }
            ProcessAbiOperation::Pipe => {
                memory.intrinsic_work_cost(types, &[(args[0].pointer()?, true)], 8)?
            }
            ProcessAbiOperation::Close => {
                fd_number(&args[0])?;
                1
            }
            ProcessAbiOperation::Dup2 => {
                fd_number(&args[0])?;
                fd_number(&args[1])?;
                1
            }
            ProcessAbiOperation::Exit => {
                fd_number(&args[0])?;
                1
            }
            ProcessAbiOperation::WaitPid => {
                if fd_number(&args[0])? <= 0 {
                    return Err(process_error(ProcessError::Unsupported(
                        "waitpid requires a selected positive child pid",
                    )));
                }
                if !matches!(fd_number(&args[2])?, 0 | 1) {
                    return Err(process_error(ProcessError::Unsupported(
                        "waitpid options are not supported",
                    )));
                }
                let status = args[1].pointer()?;
                if status.is_null() {
                    1
                } else {
                    memory.intrinsic_work_cost(types, &[(status, true)], 4)?
                }
            }
            ProcessAbiOperation::Fork
            | ProcessAbiOperation::GetPid
            | ProcessAbiOperation::GetParentPid
            | ProcessAbiOperation::ErrnoLocation => 8,
            _ => return Err(Error::UnsupportedForeignProcedure(self.procedure())),
        };
        cost = cost
            .checked_add(extra)
            .and_then(|cost| cost.checked_add(8))
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        if let Some(errno) = branch.errno.get(&self.nominals().error_code) {
            cost = cost
                .checked_add(memory.intrinsic_work_cost(types, &[(errno, true)], 4)?)
                .ok_or(Error::Limit(LimitKind::Fuel))?;
        }
        charge(cost)?;
        if op == ProcessAbiOperation::Fork {
            return Ok(ProcessCallOutcome::ForkControl(ProcessForkControl {
                procedure: self.clone(),
                parent: branch.current,
            }));
        }
        if op == ProcessAbiOperation::Exit {
            return Ok(ProcessCallOutcome::ExitControl(ProcessExitControl {
                procedure: self.clone(),
                process: branch.current,
                status: fd_number(&args[0])?,
            }));
        }
        if op == ProcessAbiOperation::GetPid {
            return Ok(values(
                IntegerType::S32,
                i128::from(branch.current.abi_value().map_err(process_error)?),
            ));
        }
        if op == ProcessAbiOperation::GetParentPid {
            let parent = world
                .parent(branch.current)
                .map_err(process_error)?
                .ok_or_else(|| {
                    process_error(ProcessError::Unsupported(
                        "virtual root has no modeled parent",
                    ))
                })?;
            return Ok(values(
                IntegerType::S32,
                i128::from(parent.abi_value().map_err(process_error)?),
            ));
        }
        if op == ProcessAbiOperation::ErrnoLocation {
            let pointer = branch.errno_pointer(self.nominals().error_code, memory, types)?;
            return Ok(ProcessCallOutcome::Values(vec![Value::Pointer(pointer)]));
        }
        let mut candidate = world.clone();
        let result = (|| -> Result<ProcessCallOutcome, CallError> {
            let current = branch.current;
            match op {
                ProcessAbiOperation::Pipe => {
                    let pair = candidate.pipe(current)?;
                    let pointer = args[0].pointer()?;
                    memory.store(
                        types,
                        pointer,
                        Value::Array {
                            ty: pointer.pointee(),
                            elements: pair
                                .into_iter()
                                .map(|fd| int(IntegerType::S32, i128::from(fd.number())))
                                .collect(),
                        },
                    )?;
                    Ok(values(IntegerType::S32, 0))
                }
                ProcessAbiOperation::Close => {
                    let fd = candidate.descriptor(current, fd_number(&args[0])?)?;
                    candidate.close(fd)?;
                    Ok(values(IntegerType::S32, 0))
                }
                ProcessAbiOperation::Dup2 => {
                    let fd = candidate.descriptor(current, fd_number(&args[0])?)?;
                    let duplicate = candidate.dup2(fd, fd_number(&args[1])?)?;
                    Ok(values(IntegerType::S32, i128::from(duplicate.number())))
                }
                ProcessAbiOperation::Write => {
                    let fd = candidate.descriptor(current, fd_number(&args[0])?)?;
                    let count = count(&args[2], &candidate)?;
                    let bytes = memory.host_read_bytes(types, args[1].pointer()?, count)?;
                    let written = candidate.write(fd, &bytes)?;
                    Ok(values(
                        IntegerType::S64,
                        i128::try_from(written).map_err(|_| Error::CheckedCast)?,
                    ))
                }
                ProcessAbiOperation::Read => {
                    let fd = candidate.descriptor(current, fd_number(&args[0])?)?;
                    match candidate.read(fd, count(&args[2], &candidate)?)? {
                        ProcessIo::Pending(event) => Ok(ProcessCallOutcome::Pending(event)),
                        ProcessIo::Ready(bytes) => {
                            memory.host_write_bytes(types, args[1].pointer()?, &bytes)?;
                            Ok(values(
                                IntegerType::S64,
                                i128::try_from(bytes.len()).map_err(|_| Error::CheckedCast)?,
                            ))
                        }
                    }
                }
                ProcessAbiOperation::WaitPid => {
                    let child = candidate.resolve_pid(current, fd_number(&args[0])?)?;
                    match candidate.wait(current, child, fd_number(&args[2])? == 1)? {
                        WaitOutcome::Pending(event) => Ok(ProcessCallOutcome::Pending(event)),
                        WaitOutcome::StillRunning => Ok(values(IntegerType::S32, 0)),
                        WaitOutcome::Reaped(termination) => {
                            let status = encode_wait_status(termination)?;
                            let pointer = args[1].pointer()?;
                            if !pointer.is_null() {
                                memory.store(
                                    types,
                                    pointer,
                                    int(IntegerType::S32, i128::from(status)),
                                )?;
                            }
                            Ok(values(IntegerType::S32, i128::from(child.abi_value()?)))
                        }
                    }
                }
                _ => Err(Error::InvalidIr("unsupported scalar process adapter").into()),
            }
        })();
        match result {
            Ok(outcome @ ProcessCallOutcome::Pending(_)) => Ok(outcome),
            Ok(outcome) => {
                *world = candidate;
                Ok(outcome)
            }
            Err(CallError::Vm(error)) => Err(error),
            Err(CallError::Process(error)) => {
                let errno = if op == ProcessAbiOperation::WaitPid
                    && matches!(error, ProcessError::NotChild | ProcessError::AlreadyReaped)
                {
                    Some(10)
                } else {
                    recoverable(&error, target)
                };
                if let Some(code) = errno {
                    branch.set_errno(self.nominals().error_code, code, memory, types)?;
                    Ok(values(
                        if matches!(op, ProcessAbiOperation::Read | ProcessAbiOperation::Write) {
                            IntegerType::S64
                        } else {
                            IntegerType::S32
                        },
                        -1,
                    ))
                } else {
                    Err(process_error(error))
                }
            }
        }
    }
}
fn encode_wait_status(termination: ProcessTermination) -> Result<i32, Error> {
    match termination {
        // Both inspected POSIX profiles place the normal exit byte in bits 8..15.
        ProcessTermination::Exited(code) if (0..=255).contains(&code) => Ok(code << 8),
        ProcessTermination::Exited(_) => Err(Error::InvalidIr(
            "observed process exit code is outside the POSIX exit byte",
        )),
        ProcessTermination::Signaled | ProcessTermination::TimedOut => Err(process_error(
            ProcessError::Unsupported("source wait status needs an actual signal number"),
        )),
    }
}
enum CallError {
    Vm(Error),
    Process(ProcessError),
}
impl From<Error> for CallError {
    fn from(error: Error) -> Self {
        Self::Vm(error)
    }
}
impl From<ProcessError> for CallError {
    fn from(error: ProcessError) -> Self {
        Self::Process(error)
    }
}

#[cfg(test)]
#[path = "process_source_machine/tests.rs"]
mod tests;

mod exec;
pub use exec::{ProcessExecLimits, ProcessExecScope};

mod socket;

mod flags;
