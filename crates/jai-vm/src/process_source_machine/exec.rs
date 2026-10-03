//! Exact reviewed argv selection and one-shot host completion. No native calls.
use super::*;
use crate::{
    CompilerEffects, SourceOrigin,
    host_effects::{
        HostEffects, HostError, HostOutcome, HostRequest, HostResponse, ReviewedPrograms,
    },
    virtual_process::ExecOutcome,
};
use std::{
    ffi::OsString,
    path::{Component, PathBuf},
};

#[derive(Clone, Copy, Debug)]
pub struct ProcessExecLimits {
    pub arguments: usize,
    /// Aggregate executable/argument bytes, including their source terminators.
    pub bytes: usize,
}
impl Default for ProcessExecLimits {
    fn default() -> Self {
        Self {
            arguments: 4096,
            bytes: 1024 * 1024,
        }
    }
}
/// Trusted embedding authority. Source cannot add grants, choose cwd or search PATH.
#[derive(Debug)]
pub struct ProcessExecScope {
    programs: ReviewedPrograms,
    canonical_working_directory: PathBuf,
    limits: ProcessExecLimits,
}
impl ProcessExecScope {
    pub fn new(
        programs: ReviewedPrograms,
        canonical_working_directory: PathBuf,
        limits: ProcessExecLimits,
    ) -> Result<Self, HostError> {
        if !canonical_working_directory.is_absolute()
            || canonical_working_directory
                .as_os_str()
                .as_encoded_bytes()
                .contains(&0)
            || canonical_working_directory
                .components()
                .any(|part| matches!(part, Component::CurDir | Component::ParentDir))
        {
            return Err(HostError::InvalidPath);
        }
        if limits.arguments == 0 || limits.arguments > isize::MAX as usize || limits.bytes == 0 {
            return Err(HostError::Budget("source exec limits"));
        }
        Ok(Self {
            programs,
            canonical_working_directory,
            limits,
        })
    }
}
fn host_error(error: HostError) -> Error {
    Error::EffectRejected(error.to_string())
}
fn plus(left: u64, right: u64) -> Result<u64, Error> {
    left.checked_add(right).ok_or(Error::Limit(LimitKind::Fuel))
}
fn byte_cost(bytes: usize) -> Result<u64, Error> {
    u64::try_from(bytes).map_err(|_| Error::Limit(LimitKind::Fuel))
}
fn exec_work(
    proof: &ProcessAbiProcedure,
    memory: &Memory,
    types: &dyn TypeView,
    branch: &ProcessBranchState,
    world: &VirtualProcesses,
) -> Result<u64, Error> {
    let mut work = world
        .work_cost()
        .map_err(process_error)?
        .checked_mul(4)
        .ok_or(Error::Limit(LimitKind::Fuel))?;
    work = plus(work, byte_cost(branch.errno.capacity())?)?;
    work = plus(work, 8)?;
    if let Some(pointer) = branch.errno.get(&proof.nominals().error_code) {
        work = plus(
            work,
            memory.intrinsic_work_cost(types, &[(pointer, true)], 4)?,
        )?;
    }
    Ok(work)
}
fn validate_exec(
    proof: &ProcessAbiProcedure,
    memory: &Memory,
    types: &dyn TypeView,
    target: &BuildTarget,
) -> Result<(), Error> {
    proof.validate(target, types).map_err(binding_error)?;
    if proof.operation() != ProcessAbiOperation::ExecVp {
        return Err(Error::UnsupportedForeignProcedure(proof.procedure()));
    }
    if memory.target() != ByteTarget::from(target) {
        return Err(Error::InvalidIr(
            "exec branch memory differs from source target",
        ));
    }
    Ok(())
}
#[cfg(unix)]
fn os_bytes(bytes: Vec<u8>) -> Result<OsString, Error> {
    use std::os::unix::ffi::OsStringExt;
    Ok(OsString::from_vec(bytes))
}
#[cfg(not(unix))]
fn os_bytes(_: Vec<u8>) -> Result<OsString, Error> {
    Err(Error::EffectRejected(
        "byte-preserving POSIX exec requires a Unix host".into(),
    ))
}
fn c_string(
    memory: &Memory,
    types: &dyn TypeView,
    pointer: &Pointer,
    remaining: &mut usize,
    charge: &mut impl FnMut(u64) -> Result<(), Error>,
) -> Result<OsString, Error> {
    charge(memory.host_c_string_cost(types, pointer, *remaining)?)?;
    let bytes = memory.host_c_string(types, pointer, *remaining)?;
    *remaining = remaining
        .checked_sub(
            bytes
                .len()
                .checked_add(1)
                .ok_or(Error::Limit(LimitKind::Fuel))?,
        )
        .ok_or_else(|| host_error(HostError::Budget("source exec bytes")))?;
    os_bytes(bytes)
}
fn output_work(outcome: &HostOutcome) -> Result<u64, Error> {
    match outcome {
        HostOutcome::Ready(HostResponse::Process(output)) => byte_cost(output.stdout.len())?
            .checked_add(byte_cost(output.stderr.len())?)
            .and_then(|cost| cost.checked_mul(2))
            .ok_or(Error::Limit(LimitKind::Fuel)),
        _ => Ok(1),
    }
}
struct ChargedRequests<'a, E, C> {
    effects: &'a mut E,
    charge: &'a mut C,
    error: Option<Error>,
}
impl<E: CompilerEffects, C: FnMut(u64) -> Result<(), Error>> HostEffects
    for ChargedRequests<'_, E, C>
{
    fn begin(&mut self, _: SourceOrigin) -> Result<(), HostError> {
        Err(HostError::Transaction(
            "exec shares the outer source transaction",
        ))
    }
    fn finish(&mut self, _: bool) -> Result<(), HostError> {
        Err(HostError::Transaction(
            "exec shares the outer source transaction",
        ))
    }
    fn request(&mut self, request: HostRequest) -> HostOutcome {
        let observation = self.effects.host_request(request);
        let charged = output_work(&observation).and_then(|cost| (self.charge)(cost));
        if let Err(error) = charged {
            self.error = Some(error);
            return HostOutcome::Rejected(HostError::Budget("exec output work"));
        }
        observation
    }
}
impl ProcessAbiProcedure {
    #[allow(
        clippy::too_many_arguments,
        reason = "branch memory, shared world, reviewed authority and effects retain separate owners"
    )]
    pub fn invoke_exec(
        &self,
        args: &[Value],
        memory: &mut Memory,
        types: &dyn TypeView,
        target: &BuildTarget,
        branch: &mut ProcessBranchState,
        world: &mut VirtualProcesses,
        scope: &ProcessExecScope,
        effects: &mut impl CompilerEffects,
        charge: &mut impl FnMut(u64) -> Result<(), Error>,
    ) -> Result<ProcessCallOutcome, Error> {
        validate_exec(self, memory, types, target)?;
        world
            .validate_running(branch.current)
            .map_err(process_error)?;
        let signature = types.procedure_definition(self.signature())?;
        if args.len() != signature.parameters.len() {
            return Err(Error::InvalidIr("exec ABI argument count"));
        }
        for (value, ty) in args.iter().zip(signature.parameters.iter()) {
            value.validate(types, *ty, 128)?;
        }
        charge(plus(
            exec_work(self, memory, types, branch, world)?,
            scope
                .programs
                .work_cost()
                .checked_mul(2)
                .ok_or(Error::Limit(LimitKind::Fuel))?,
        )?)?;
        let mut remaining = scope.limits.bytes;
        let executable = c_string(memory, types, args[0].pointer()?, &mut remaining, charge)?;
        let array = args[1].pointer()?;
        let mut argv = Vec::new();
        for index in 0..=scope.limits.arguments {
            charge(1)?;
            let entry = memory.offset(
                types,
                array,
                isize::try_from(index).map_err(|_| Error::CheckedCast)?,
            )?;
            charge(memory.intrinsic_work_cost(types, &[(&entry, false)], 8)?)?;
            let value = memory.load(types, &entry)?;
            let pointer = value.pointer()?;
            if pointer.is_null() {
                break;
            }
            if index == scope.limits.arguments {
                return Err(host_error(HostError::Budget("source argument count")));
            }
            argv.push(c_string(memory, types, pointer, &mut remaining, charge)?);
        }
        let invocation = scope
            .programs
            .resolve(&executable, &argv, &scope.canonical_working_directory)
            .map_err(host_error)?;
        let mut candidate = world.clone();
        let mut requests = ChargedRequests {
            effects,
            charge,
            error: None,
        };
        let outcome = candidate.exec(branch.current, invocation, &mut requests);
        if let Some(error) = requests.error {
            return Err(error);
        }
        let outcome = outcome.map_err(process_error)?;
        finish_exec(
            self, outcome, candidate, memory, types, target, branch, world,
        )
    }
    /// Consume only the retained key's observation; never resolve argv or request again.
    #[allow(
        clippy::too_many_arguments,
        reason = "completion preserves separate scheduler ownership"
    )]
    pub fn complete_process_exec(
        &self,
        control: &ProcessExecControl,
        outcome: HostOutcome,
        memory: &mut Memory,
        types: &dyn TypeView,
        target: &BuildTarget,
        branch: &mut ProcessBranchState,
        world: &mut VirtualProcesses,
        charge: &mut impl FnMut(u64) -> Result<(), Error>,
    ) -> Result<ProcessCallOutcome, Error> {
        validate_exec(self, memory, types, target)?;
        control
            .procedure
            .validate(target, types)
            .map_err(binding_error)?;
        if self.procedure() != control.procedure.procedure()
            || self.library() != control.procedure.library()
            || self.signature() != control.procedure.signature()
            || branch.current != control.process
        {
            return Err(Error::InvalidIr("exec completion differs from sealed call"));
        }
        let ProcessExecStage::HostPending(key) = control.stage else {
            return Err(Error::InvalidIr("exec replacement cannot complete twice"));
        };
        charge(plus(
            exec_work(self, memory, types, branch, world)?,
            output_work(&outcome)?,
        )?)?;
        let mut candidate = world.clone();
        let outcome = candidate
            .complete_exec(branch.current, key, outcome)
            .map_err(process_error)?;
        finish_exec(
            self, outcome, candidate, memory, types, target, branch, world,
        )
    }
}
#[allow(
    clippy::too_many_arguments,
    reason = "atomic publication keeps private errno memory and scheduler world separate"
)]
fn finish_exec(
    proof: &ProcessAbiProcedure,
    outcome: ExecOutcome,
    candidate: VirtualProcesses,
    memory: &mut Memory,
    types: &dyn TypeView,
    target: &BuildTarget,
    branch: &mut ProcessBranchState,
    world: &mut VirtualProcesses,
) -> Result<ProcessCallOutcome, Error> {
    let outcome = match outcome {
        ExecOutcome::Rejected(error) => return Err(host_error(error)),
        ExecOutcome::Failed(failure) => {
            if !failure.matches_platform(&target.operating_system) {
                return Err(Error::InvalidIr(
                    "exec launch errno platform differs from source target",
                ));
            }
            branch.set_errno(proof.nominals().error_code, failure.errno(), memory, types)?;
            values(IntegerType::S32, -1)
        }
        ExecOutcome::Pending(key) => ProcessCallOutcome::ExecControl(ProcessExecControl {
            procedure: proof.clone(),
            process: branch.current,
            stage: ProcessExecStage::HostPending(key),
        }),
        ExecOutcome::Replaced => ProcessCallOutcome::ExecControl(ProcessExecControl {
            procedure: proof.clone(),
            process: branch.current,
            stage: ProcessExecStage::Replaced,
        }),
    };
    *world = candidate;
    Ok(outcome)
}
