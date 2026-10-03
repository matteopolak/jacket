//! Inspected POSIX Unix-stream socket and ancillary descriptor ABI. No native calls.
use super::*;
use crate::process_abi::ProcessSocketTypes;
use jai_types::{CastMode, RecordKind, ScalarType, TypeKind};

#[derive(Clone, Copy)]
struct Profile {
    socket: ProcessSocketTypes,
    header_bytes: usize,
    control_bytes: usize,
    alignment: usize,
    sol_socket: i32,
    ctrunc: i32,
    length: IntegerType,
    iov_count: IntegerType,
}
fn unsupported(message: &'static str) -> Error {
    process_error(ProcessError::Unsupported(message))
}
fn add(left: u64, right: u64) -> Result<u64, Error> {
    left.checked_add(right).ok_or(Error::Limit(LimitKind::Fuel))
}
fn record_fields(types: &dyn TypeView, ty: TypeId, fields: &[TypeId]) -> Result<(), Error> {
    let record = types.record_definition(ty)?;
    if record.kind != RecordKind::Struct
        || record.fields.as_ref() != fields
        || record.layout != Default::default()
    {
        return Err(Error::InvalidIr(
            "Socket source record layout differs from inspected target",
        ));
    }
    Ok(())
}
fn profile(
    proof: &ProcessAbiProcedure,
    types: &dyn TypeView,
    target: &BuildTarget,
) -> Result<Profile, Error> {
    let socket = proof
        .nominals()
        .socket
        .ok_or_else(|| unsupported("Socket nominal receipt is absent"))?;
    let mac = target.operating_system == OperatingSystem::MacOS;
    let length = if mac {
        IntegerType::U32
    } else {
        IntegerType::U64
    };
    let iov_count = if mac {
        IntegerType::S32
    } else {
        IntegerType::U64
    };
    let integer = |repr| types.scalar(ScalarType::Int(repr));
    let void = types
        .record_definition(socket.io_vector)?
        .fields
        .first()
        .copied()
        .ok_or(Error::InvalidIr("iovec field count"))?;
    if !matches!(types.kind(void)?, TypeKind::Pointer(pointee) if matches!(types.kind(*pointee)?,TypeKind::Void))
    {
        return Err(Error::InvalidIr(
            "iovec base requires its source void pointer",
        ));
    }
    record_fields(types, socket.io_vector, &[void, integer(IntegerType::U64)])?;
    let header = types.record_definition(socket.message_header)?;
    let iov_pointer = header
        .fields
        .get(2)
        .copied()
        .ok_or(Error::InvalidIr("msghdr field count"))?;
    if !matches!(types.kind(iov_pointer)?, TypeKind::Pointer(pointee) if *pointee==socket.io_vector)
    {
        return Err(Error::InvalidIr("msghdr requires its actual iovec pointer"));
    }
    record_fields(
        types,
        socket.message_header,
        &[
            void,
            integer(IntegerType::U32),
            iov_pointer,
            integer(iov_count),
            void,
            integer(length),
            integer(IntegerType::S32),
        ],
    )?;
    record_fields(
        types,
        socket.control_header,
        &[
            integer(length),
            integer(IntegerType::S32),
            integer(IntegerType::S32),
        ],
    )?;
    Ok(Profile {
        socket,
        header_bytes: if mac { 48 } else { 56 },
        control_bytes: if mac { 12 } else { 16 },
        alignment: if mac { 4 } else { 8 },
        sol_socket: if mac { 0xffff } else { 1 },
        ctrunc: if mac { 0x20 } else { 0x8 },
        length,
        iov_count,
    })
}
fn enum_bits(value: &Value, ty: TypeId, repr: IntegerType) -> Result<i128, Error> {
    let Value::Enum { ty: actual, value } = value.semantic() else {
        return Err(Error::InvalidIr("Socket requires its actual enum value"));
    };
    if *actual != ty || value.ty() != repr {
        return Err(Error::InvalidIr("Socket nominal enum mismatch"));
    }
    Ok(value.value())
}
fn size(value: &Value, repr: IntegerType, limit: usize) -> Result<usize, Error> {
    let value = usize::try_from(plain(value, repr)?.value()).map_err(|_| Error::CheckedCast)?;
    if value > limit {
        return Err(process_error(ProcessError::Budget("Socket ABI transfer")));
    }
    Ok(value)
}
fn fields(value: &Value, ty: TypeId) -> Result<&[Value], Error> {
    match value.semantic() {
        Value::Record { ty: actual, fields } if *actual == ty => Ok(fields),
        _ => Err(Error::InvalidIr("Socket record value mismatch")),
    }
}
fn at(
    memory: &Memory,
    types: &dyn TypeView,
    base: &Pointer,
    offset: usize,
    ty: TypeId,
) -> Result<Pointer, Error> {
    let bytes = memory.cast_pointer(
        types,
        base,
        types.scalar(ScalarType::Int(IntegerType::U8)),
        CastMode::Unchecked,
    )?;
    let bytes = memory.offset(
        types,
        &bytes,
        isize::try_from(offset).map_err(|_| Error::CheckedCast)?,
    )?;
    memory.cast_pointer(types, &bytes, ty, CastMode::Unchecked)
}
#[derive(Clone)]
struct Vector {
    pointer: Pointer,
    bytes: usize,
}
struct Message {
    header: Pointer,
    vectors: Vec<Vector>,
    control: Pointer,
    control_capacity: usize,
    total: usize,
    writable_retention: u64,
}
fn message(
    profile: Profile,
    args: &[Value],
    memory: &Memory,
    types: &dyn TypeView,
    world: &VirtualProcesses,
    receiving: bool,
    charge: &mut impl FnMut(u64) -> Result<(), Error>,
) -> Result<Message, Error> {
    if enum_bits(&args[2], profile.socket.message_flags, IntegerType::S32)? != 0 {
        return Err(unsupported(
            "Socket message flags other than zero are unsupported",
        ));
    }
    let header = args[1].pointer()?;
    let header_work =
        memory.intrinsic_work_cost(types, &[(header, receiving)], profile.header_bytes)?;
    charge(header_work)?;
    let mut writable_retention = if receiving { header_work } else { 0 };
    let value = memory.load(types, header)?;
    let header_fields = fields(&value, profile.socket.message_header)?;
    if !header_fields[0].pointer()?.is_null()
        || plain(&header_fields[1], IntegerType::U32)?.bits() != 0
    {
        return Err(unsupported("Unix socket address messages are unsupported"));
    }
    let iov_count = size(
        &header_fields[3],
        profile.iov_count,
        world.limits().descriptors,
    )?;
    let control_capacity = size(
        &header_fields[5],
        profile.length,
        world.limits().transfer_bytes,
    )?;
    let control = header_fields[4].pointer()?.clone();
    let control_work =
        memory.intrinsic_work_cost(types, &[(&control, receiving)], control_capacity)?;
    charge(control_work)?;
    if receiving {
        writable_retention = add(writable_retention, control_work)?;
    }
    let mut vectors = Vec::new();
    let mut total = 0usize;
    for index in 0..iov_count {
        charge(128)?;
        let pointer = memory.offset(
            types,
            header_fields[2].pointer()?,
            isize::try_from(index).map_err(|_| Error::CheckedCast)?,
        )?;
        charge(memory.intrinsic_work_cost(types, &[(&pointer, false)], 16)?)?;
        let value = memory.load(types, &pointer)?;
        let fields = fields(&value, profile.socket.io_vector)?;
        let bytes = size(&fields[1], IntegerType::U64, world.limits().transfer_bytes)?;
        total = total
            .checked_add(bytes)
            .filter(|total| *total <= world.limits().transfer_bytes)
            .ok_or_else(|| process_error(ProcessError::Budget("Socket vector bytes")))?;
        let pointer = fields[0].pointer()?.clone();
        let output_work = memory.intrinsic_work_cost(types, &[(&pointer, receiving)], bytes)?;
        charge(output_work)?;
        if receiving {
            writable_retention = add(writable_retention, output_work)?;
        }
        vectors.push(Vector { pointer, bytes });
    }
    Ok(Message {
        header: header.clone(),
        vectors,
        control,
        control_capacity,
        total,
        writable_retention,
    })
}
fn send(
    message: &Message,
    profile: Profile,
    memory: &Memory,
    types: &dyn TypeView,
    world: &VirtualProcesses,
    process: ProcessId,
    charge: &mut impl FnMut(u64) -> Result<(), Error>,
) -> Result<(Vec<u8>, Vec<crate::virtual_process::FileDescriptor>), CallError> {
    let mut bytes = Vec::new();
    for vector in &message.vectors {
        bytes.extend(memory.host_read_bytes(types, &vector.pointer, vector.bytes)?);
    }
    let mut rights = Vec::new();
    let mut offset = 0usize;
    while offset < message.control_capacity {
        let remaining = message.control_capacity - offset;
        if remaining < profile.control_bytes {
            break;
        }
        charge(128)?;
        let pointer = at(
            memory,
            types,
            &message.control,
            offset,
            profile.socket.control_header,
        )?;
        let value = memory.load(types, &pointer)?;
        let fields = fields(&value, profile.socket.control_header)?;
        let length = size(&fields[0], profile.length, remaining)?;
        if length < profile.control_bytes || !(length - profile.control_bytes).is_multiple_of(4) {
            return Err(Error::InvalidIr("invalid SCM_RIGHTS control length").into());
        }
        if fd_number(&fields[1])? != profile.sol_socket || fd_number(&fields[2])? != 1 {
            return Err(unsupported("ancillary data other than SCM_RIGHTS is unsupported").into());
        }
        let count = (length - profile.control_bytes) / 4;
        if rights
            .len()
            .checked_add(count)
            .is_none_or(|count| count > world.limits().transferred_descriptors)
        {
            return Err(ProcessError::Budget("SCM_RIGHTS descriptor count").into());
        }
        for index in 0..count {
            charge(128)?;
            let pointer = at(
                memory,
                types,
                &message.control,
                offset + profile.control_bytes + index * 4,
                types.scalar(ScalarType::Int(IntegerType::S32)),
            )?;
            rights.push(world.descriptor(process, fd_number(&memory.load(types, &pointer)?)?)?);
        }
        let aligned = length
            .checked_add(profile.alignment - 1)
            .map(|length| length & !(profile.alignment - 1))
            .ok_or(Error::CheckedCast)?;
        if aligned > remaining {
            break;
        }
        offset += aligned;
    }
    Ok((bytes, rights))
}
#[allow(
    clippy::too_many_arguments,
    reason = "atomic receive must admit all transient memory and world ownership"
)]
fn receive(
    message: &Message,
    profile: Profile,
    candidate: &mut VirtualProcesses,
    memory: &mut Memory,
    types: &dyn TypeView,
    descriptor: crate::virtual_process::FileDescriptor,
    charge: &mut impl FnMut(u64) -> Result<(), Error>,
    original_world_work: u64,
    branch_metadata: usize,
) -> Result<ProcessCallOutcome, CallError> {
    let received = match candidate.recv_rights(
        descriptor,
        message.total,
        candidate.limits().transferred_descriptors,
    )? {
        ProcessIo::Pending(event) => return Ok(ProcessCallOutcome::Pending(event)),
        ProcessIo::Ready(message) => message,
    };
    let work = memory.snapshot_work_cost()?;
    charge(
        u64::try_from(work)
            .map_err(|_| Error::Limit(LimitKind::Fuel))?
            .checked_mul(2)
            .ok_or(Error::Limit(LimitKind::Fuel))?,
    )?;
    let world_work = original_world_work.max(candidate.work_cost()?);
    let retained = memory
        .value_cells()
        .checked_mul(2)
        .and_then(|cells| {
            usize::try_from(world_work)
                .ok()
                .and_then(|world| world.checked_mul(2))
                .and_then(|world| cells.checked_add(world))
        })
        .and_then(|cells| cells.checked_add(branch_metadata))
        .and_then(|cells| {
            usize::try_from(message.writable_retention)
                .ok()
                .and_then(|growth| cells.checked_add(growth))
        })
        .ok_or(Error::Limit(LimitKind::ValueCells))?;
    if retained > memory.value_cell_limit() {
        return Err(Error::Limit(LimitKind::ValueCells).into());
    }
    let mut output = memory.fork_private_branch(work)?;
    let mut cursor = 0usize;
    for vector in &message.vectors {
        let count = vector.bytes.min(received.bytes.len() - cursor);
        output.host_write_bytes(
            types,
            &vector.pointer,
            &received.bytes[cursor..cursor + count],
        )?;
        cursor += count;
        if cursor == received.bytes.len() {
            break;
        }
    }
    let capacity = message
        .control_capacity
        .saturating_sub(profile.control_bytes)
        / 4;
    let fits = if message.control_capacity < profile.control_bytes {
        0
    } else {
        capacity.min(received.descriptors.len())
    };
    let truncated = fits < received.descriptors.len();
    let mut control_bytes = 0usize;
    if fits != 0 {
        charge(128)?;
        let pointer = at(
            &output,
            types,
            &message.control,
            0,
            profile.socket.control_header,
        )?;
        let length = profile.control_bytes + fits * 4;
        output.store(
            types,
            &pointer,
            Value::Record {
                ty: profile.socket.control_header,
                fields: vec![
                    int(profile.length, length as i128),
                    int(IntegerType::S32, profile.sol_socket as i128),
                    int(IntegerType::S32, 1),
                ],
            },
        )?;
        for (index, fd) in received.descriptors.iter().take(fits).enumerate() {
            charge(128)?;
            let pointer = at(
                &output,
                types,
                &message.control,
                profile.control_bytes + index * 4,
                types.scalar(ScalarType::Int(IntegerType::S32)),
            )?;
            output.store(types, &pointer, int(IntegerType::S32, fd.number() as i128))?;
        }
        control_bytes = length
            .checked_add(profile.alignment - 1)
            .map(|length| length & !(profile.alignment - 1))
            .ok_or(Error::CheckedCast)?
            .min(message.control_capacity);
    }
    for descriptor in received.descriptors.iter().skip(fits) {
        candidate.close(*descriptor)?;
    }
    let length = output.field(types, &message.header, 5)?;
    let flags = output.field(types, &message.header, 6)?;
    output.store(types, &length, int(profile.length, control_bytes as i128))?;
    output.store(
        types,
        &flags,
        int(
            IntegerType::S32,
            if truncated { profile.ctrunc as i128 } else { 0 },
        ),
    )?;
    let final_retained = memory
        .value_cells()
        .checked_add(output.value_cells())
        .and_then(|cells| {
            usize::try_from(world_work)
                .ok()
                .and_then(|world| world.checked_mul(2))
                .and_then(|world| cells.checked_add(world))
        })
        .and_then(|cells| cells.checked_add(branch_metadata))
        .ok_or(Error::Limit(LimitKind::ValueCells))?;
    if final_retained > memory.value_cell_limit() {
        return Err(Error::Limit(LimitKind::ValueCells).into());
    }
    *memory = output;
    Ok(values(IntegerType::S64, received.bytes.len() as i128))
}
#[allow(
    clippy::too_many_arguments,
    reason = "socket ABI keeps branch memory and scheduler transport ownership separate"
)]
pub(super) fn invoke(
    proof: &ProcessAbiProcedure,
    args: &[Value],
    memory: &mut Memory,
    types: &dyn TypeView,
    target: &BuildTarget,
    branch: &mut ProcessBranchState,
    world: &mut VirtualProcesses,
    charge: &mut impl FnMut(u64) -> Result<(), Error>,
) -> Result<ProcessCallOutcome, Error> {
    let profile = profile(proof, types, target)?;
    let original_world_work = world.work_cost().map_err(process_error)?;
    let mut cost = original_world_work
        .checked_mul(2)
        .ok_or(Error::Limit(LimitKind::Fuel))?;
    cost = add(
        cost,
        u64::try_from(branch.errno.capacity()).map_err(|_| Error::Limit(LimitKind::Fuel))?,
    )?;
    cost = add(cost, 8)?;
    if let Some(pointer) = branch.errno.get(&proof.nominals().error_code) {
        cost = add(
            cost,
            memory.intrinsic_work_cost(types, &[(pointer, true)], 4)?,
        )?;
    }
    charge(cost)?;
    let op = proof.operation();
    let result = (|| -> Result<(ProcessCallOutcome, VirtualProcesses), CallError> {
        match op {
            ProcessAbiOperation::SocketPair => {
                if fd_number(&args[0])? != 1
                    || enum_bits(&args[1], profile.socket.socket_kind, IntegerType::U32)? != 1
                    || enum_bits(&args[2], profile.socket.protocol, IntegerType::U32)? != 0
                {
                    return Err(unsupported(
                        "socketpair supports inspected AF_UNIX/STREAM/protocol zero only",
                    )
                    .into());
                }
                let pointer = args[3].pointer()?;
                charge(memory.intrinsic_work_cost(types, &[(pointer, true)], 8)?)?;
                let mut candidate = world.clone();
                let pair = candidate.socketpair(branch.current)?;
                memory.store(
                    types,
                    pointer,
                    Value::Array {
                        ty: pointer.pointee(),
                        elements: pair
                            .into_iter()
                            .map(|fd| int(IntegerType::S32, fd.number() as i128))
                            .collect(),
                    },
                )?;
                Ok((values(IntegerType::S32, 0), candidate))
            }
            ProcessAbiOperation::Shutdown => {
                if enum_bits(&args[1], profile.socket.shutdown_kind, IntegerType::U32)? != 1 {
                    return Err(
                        unsupported("socket shutdown supports inspected SHUT_WR only").into(),
                    );
                }
                let mut candidate = world.clone();
                candidate
                    .shutdown_write(candidate.descriptor(branch.current, fd_number(&args[0])?)?)?;
                Ok((values(IntegerType::S32, 0), candidate))
            }
            ProcessAbiOperation::SendMsg | ProcessAbiOperation::RecvMsg => {
                fd_number(&args[0])?;
                let message = message(
                    profile,
                    args,
                    memory,
                    types,
                    world,
                    op == ProcessAbiOperation::RecvMsg,
                    charge,
                )?;
                let mut candidate = world.clone();
                let descriptor = candidate.descriptor(branch.current, fd_number(&args[0])?)?;
                let result = if op == ProcessAbiOperation::SendMsg {
                    let (bytes, rights) = send(
                        &message,
                        profile,
                        memory,
                        types,
                        world,
                        branch.current,
                        charge,
                    )?;
                    values(
                        IntegerType::S64,
                        candidate.send_rights(descriptor, &bytes, &rights)? as i128,
                    )
                } else {
                    receive(
                        &message,
                        profile,
                        &mut candidate,
                        memory,
                        types,
                        descriptor,
                        charge,
                        original_world_work,
                        branch.retained_metadata_cells(),
                    )?
                };
                Ok((result, candidate))
            }
            _ => Err(Error::InvalidIr("non-socket proof reached socket adapter").into()),
        }
    })();
    match result {
        Ok((outcome @ ProcessCallOutcome::Pending(_), _)) => Ok(outcome),
        Ok((outcome, candidate)) => {
            *world = candidate;
            Ok(outcome)
        }
        Err(CallError::Vm(error)) => Err(error),
        Err(CallError::Process(error)) => {
            if let Some(code) = recoverable(&error, target) {
                branch.set_errno(proof.nominals().error_code, code, memory, types)?;
                Ok(values(
                    if matches!(
                        op,
                        ProcessAbiOperation::SendMsg | ProcessAbiOperation::RecvMsg
                    ) {
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
