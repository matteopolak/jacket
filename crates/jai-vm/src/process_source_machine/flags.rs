//! The inspected Process wrapper's closed C-int fcntl protocol, never a native call.
use super::*;
use crate::virtual_process::DescriptorAccess;

const GETFD: i32 = 1;
const SETFD: i32 = 2;
const GETFL: i32 = 3;
const SETFL: i32 = 4;
const CLOEXEC: i32 = 1;
const ACCESS_MASK: i32 = 3;

pub(super) fn validate_arguments(args: &[Value]) -> Result<(), Error> {
    if args.len() < 2 {
        return Err(Error::InvalidIr("fcntl ABI argument count"));
    }
    fd_number(&args[0])?;
    let command = fd_number(&args[1])?;
    let expected = match command {
        GETFD | GETFL => 2,
        SETFD | SETFL => 3,
        _ => return Err(process_error(ProcessError::Unsupported("fcntl command"))),
    };
    if args.len() != expected {
        return Err(Error::InvalidIr("fcntl ABI argument count"));
    }
    if expected == 3 {
        // The actual source wrapper supplies a promoted C int, not an Any box,
        // native address, pointer, wider integer or address-derived integer.
        fd_number(&args[2])?;
    }
    Ok(())
}

fn access_bits(access: DescriptorAccess) -> i32 {
    match access {
        DescriptorAccess::ReadOnly => 0,
        DescriptorAccess::WriteOnly => 1,
        DescriptorAccess::ReadWrite => 2,
    }
}

fn admit_candidate(
    memory: &Memory,
    branch: &ProcessBranchState,
    world_work: u64,
    argument_cells: usize,
) -> Result<(), Error> {
    let retained = usize::try_from(world_work)
        .ok()
        .and_then(|cells| cells.checked_mul(2))
        .and_then(|cells| cells.checked_add(memory.value_cells()))
        .and_then(|cells| cells.checked_add(branch.retained_metadata_cells()))
        .and_then(|cells| cells.checked_add(argument_cells))
        .ok_or(Error::Limit(LimitKind::ValueCells))?;
    if retained > memory.value_cell_limit() {
        return Err(Error::Limit(LimitKind::ValueCells));
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "branch memory, shared world and fuel retain separate scheduler ownership"
)]
pub(super) fn invoke(
    procedure: &ProcessAbiProcedure,
    args: &[Value],
    memory: &mut Memory,
    types: &dyn TypeView,
    target: &BuildTarget,
    branch: &mut ProcessBranchState,
    world: &mut VirtualProcesses,
    charge: &mut impl FnMut(u64) -> Result<(), Error>,
) -> Result<ProcessCallOutcome, Error> {
    validate_arguments(args)?;
    let world_work = world.work_cost().map_err(process_error)?;
    let mut work = world_work
        .checked_mul(2)
        .and_then(|work| work.checked_add(8))
        .and_then(|work| work.checked_add(u64::try_from(branch.retained_metadata_cells()).ok()?))
        .ok_or(Error::Limit(LimitKind::Fuel))?;
    if let Some(errno) = branch.errno.get(&procedure.nominals().error_code) {
        work = work
            .checked_add(memory.intrinsic_work_cost(types, &[(errno, true)], 4)?)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
    }
    charge(work)?;

    // Resolve the actual descriptor before restricting flags: nested getter
    // failure followed by a setter must still report the bad descriptor.
    let fd = match world.descriptor(branch.current, fd_number(&args[0])?) {
        Ok(fd) => fd,
        Err(error) if recoverable(&error, target) == Some(9) => {
            branch.set_errno(procedure.nominals().error_code, 9, memory, types)?;
            return Ok(values(IntegerType::S32, -1));
        }
        Err(error) => return Err(process_error(error)),
    };
    let nonblock = if target.operating_system == OperatingSystem::MacOS {
        4
    } else {
        0x800
    };
    let command = fd_number(&args[1])?;
    match command {
        GETFD => Ok(values(
            IntegerType::S32,
            if world.close_on_exec(fd).map_err(process_error)? {
                i128::from(CLOEXEC)
            } else {
                0
            },
        )),
        GETFL => {
            let access = access_bits(world.access_mode(fd).map_err(process_error)?);
            let status = if world.nonblocking(fd).map_err(process_error)? {
                nonblock
            } else {
                0
            };
            Ok(values(IntegerType::S32, i128::from(access | status)))
        }
        SETFD => {
            let flags = fd_number(&args[2])?;
            if flags & !CLOEXEC != 0 {
                return Err(process_error(ProcessError::Unsupported(
                    "fcntl descriptor flags beyond FD_CLOEXEC",
                )));
            }
            admit_candidate(memory, branch, world_work, args.len())?;
            let mut candidate = world.clone();
            candidate
                .set_close_on_exec(fd, flags & CLOEXEC != 0)
                .map_err(process_error)?;
            *world = candidate;
            Ok(values(IntegerType::S32, 0))
        }
        SETFL => {
            let flags = fd_number(&args[2])?;
            let access = access_bits(world.access_mode(fd).map_err(process_error)?);
            if flags & !(ACCESS_MASK | nonblock) != 0 || flags & ACCESS_MASK != access {
                return Err(process_error(ProcessError::Unsupported(
                    "fcntl status flags require the retained access mode and optional O_NONBLOCK",
                )));
            }
            admit_candidate(memory, branch, world_work, args.len())?;
            let mut candidate = world.clone();
            candidate
                .set_nonblocking(fd, flags & nonblock != 0)
                .map_err(process_error)?;
            *world = candidate;
            Ok(values(IntegerType::S32, 0))
        }
        _ => Err(Error::InvalidIr("unvalidated fcntl command")),
    }
}
