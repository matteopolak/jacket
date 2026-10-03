//! Typed stdio calls over virtual buffers and explicit host observations.
use crate::{
    Dependency, Error, Memory, Value,
    effects::EffectError,
    file_abi::{FileAbiOperation, FileAbiProcedure},
    file_tokens::FileTokens,
    host_effects::*,
    virtual_files::*,
};
use jai_types::{Integer, IntegerType, TypeId, TypeView};
use std::collections::HashMap;
const MAXIMUM_PATH: usize = 4096;
const MAXIMUM_MODE: usize = 16;
#[derive(Clone, Debug)]
pub struct HostFileMachine {
    files: VirtualFiles,
    tokens: HashMap<TypeId, FileTokens>,
    limits: FileLimits,
}
impl Default for HostFileMachine {
    fn default() -> Self {
        Self::new(FileLimits::default())
    }
}
impl HostFileMachine {
    pub fn new(limits: FileLimits) -> Self {
        Self {
            files: VirtualFiles::new(limits),
            tokens: HashMap::new(),
            limits,
        }
    }
    pub fn require_closed(&self) -> Result<(), Error> {
        self.files.require_closed().map_err(file_error)?;
        if self.tokens.values().any(|tokens| tokens.live_tokens() != 0) {
            return Err(Error::InvalidIr("unclosed FILE capability"));
        }
        Ok(())
    }
    /// Admit actual FILE backing before a general rollback checkpoint.
    /// The caller admits the combined owners and charges this returned clone bound.
    pub(crate) fn snapshot_bounds(
        &self,
        charge: &mut impl FnMut(u64) -> Result<(), Error>,
    ) -> Result<usize, Error> {
        let mut cells = self
            .tokens
            .capacity()
            .checked_add(1)
            .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
        // General snapshots may contain live streams. Only fork requires closed.
        charge(u64::try_from(cells).map_err(|_| Error::Limit(crate::LimitKind::Fuel))?)?;
        cells = cells
            .checked_add(self.files.snapshot_bounds(charge)?)
            .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
        for tokens in self.tokens.values() {
            cells = cells
                .checked_add(tokens.snapshot_bounds(charge)?)
                .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
        }
        Ok(cells)
    }
    /// Admit closed FILE ledger backing before a private branch snapshot.
    /// Live streams cannot be copied because their cursors require shared open state.
    pub(crate) fn closed_fork_bounds(
        &self,
        charge: &mut impl FnMut(u64) -> Result<(), Error>,
    ) -> Result<usize, Error> {
        let overflow = || Error::Limit(crate::LimitKind::Fuel);
        let outer = self.tokens.capacity();
        // require_closed and the nested-capacity pass each walk the outer table.
        let work = outer
            .checked_mul(2)
            .and_then(|work| work.checked_add(2))
            .ok_or_else(overflow)?;
        charge(u64::try_from(work).map_err(|_| overflow())?)?;
        self.require_closed()?;
        let file_capacity = self.files.closed_table_capacity().map_err(file_error)?;
        let mut cells = outer
            .checked_add(2) // machine and VirtualFiles headers
            .and_then(|cells| cells.checked_add(file_capacity))
            .ok_or_else(overflow)?;
        for tokens in self.tokens.values() {
            let capacity = tokens.closed_table_capacity()?;
            cells = cells
                .checked_add(1) // nested FileTokens header
                .and_then(|cells| cells.checked_add(capacity))
                .ok_or_else(overflow)?;
        }
        Ok(cells)
    }
}
fn file_error(error: FileError) -> Error {
    Error::EffectRejected(error.to_string())
}
fn host_error(error: HostError) -> EffectError {
    EffectError::Failed(Error::EffectRejected(error.to_string()))
}
fn failed(error: Error) -> EffectError {
    EffectError::Failed(error)
}
fn request(
    effects: &mut impl crate::CompilerEffects,
    request: HostRequest,
) -> Result<HostResponse, EffectError> {
    match effects.host_request(request) {
        HostOutcome::Ready(response) => Ok(response),
        HostOutcome::Pending(key) => Err(EffectError::Pending(Dependency::Host(key))),
        HostOutcome::Rejected(error) => Err(host_error(error)),
    }
}
#[cfg(unix)]
fn source_path(bytes: Vec<u8>) -> Result<std::path::PathBuf, Error> {
    use std::os::unix::ffi::OsStringExt;
    Ok(std::ffi::OsString::from_vec(bytes).into())
}
#[cfg(not(unix))]
fn source_path(bytes: Vec<u8>) -> Result<std::path::PathBuf, Error> {
    String::from_utf8(bytes)
        .map(Into::into)
        .map_err(|_| Error::InvalidIr("stdio host path is not UTF-8"))
}
fn unsigned(value: &Value) -> Result<usize, Error> {
    let value = value.number()?.portable_integer()?;
    if value.ty() != IntegerType::U64 {
        return Err(Error::InvalidIr("stdio count must be u64"));
    }
    usize::try_from(value.bits()).map_err(|_| Error::CheckedCast)
}
fn integer(ty: IntegerType, value: i128) -> Vec<Value> {
    vec![Value::Int(
        Integer::checked(ty, value).expect("stdio adapter validated integer result"),
    )]
}
struct Requests<'a, E>(&'a mut E);
impl<E: crate::CompilerEffects> HostEffects for Requests<'_, E> {
    fn begin(&mut self, _: crate::SourceOrigin) -> Result<(), HostError> {
        Err(HostError::Transaction(
            "stdio cannot begin a host transaction",
        ))
    }
    fn request(&mut self, request: HostRequest) -> HostOutcome {
        self.0.host_request(request)
    }
    fn finish(&mut self, _: bool) -> Result<(), HostError> {
        Err(HostError::Transaction(
            "stdio cannot finish a host transaction",
        ))
    }
}
impl FileAbiProcedure {
    pub(crate) fn work_cost(
        self,
        args: &[Value],
        memory: &Memory,
        types: &dyn TypeView,
        machine: &HostFileMachine,
    ) -> Result<u64, Error> {
        self.validate(types).map_err(|error| match error {
            crate::file_abi::FileAbiError::Type(error) => Error::Type(error),
            error => Error::EffectRejected(error.to_string()),
        })?;
        let signature = types.procedure_definition(self.signature)?;
        if signature.parameters.len() != args.len() {
            return Err(Error::InvalidIr("stdio argument count"));
        }
        let mut cost = u64::try_from(machine.files.payload_bytes())
            .map_err(|_| Error::Limit(crate::LimitKind::Fuel))?;
        for (value, ty) in args.iter().zip(signature.parameters.iter()) {
            value.validate(types, *ty, 128)?;
        }
        let extra = match self.operation() {
            FileAbiOperation::Open => memory
                .host_c_string_cost(types, args[0].pointer()?, MAXIMUM_PATH)?
                .checked_add(memory.host_c_string_cost(types, args[1].pointer()?, MAXIMUM_MODE)?)
                .ok_or(Error::Limit(crate::LimitKind::Fuel))?,
            FileAbiOperation::Read | FileAbiOperation::Write => {
                let count = unsigned(&args[1])?
                    .checked_mul(unsigned(&args[2])?)
                    .ok_or(Error::CheckedCast)?;
                if count > machine.limits.transfer_bytes {
                    return Err(file_error(FileError::Budget("file transfer")));
                }
                memory.intrinsic_work_cost(
                    types,
                    &[(
                        args[0].pointer()?,
                        self.operation() == FileAbiOperation::Read,
                    )],
                    count,
                )?
            }
            _ => 1,
        };
        cost = cost
            .checked_add(extra)
            .ok_or(Error::Limit(crate::LimitKind::Fuel))?;
        Ok(cost)
    }
    pub(crate) fn invoke(
        self,
        args: &[Value],
        memory: &mut Memory,
        types: &dyn TypeView,
        machine: &mut HostFileMachine,
        effects: &mut impl crate::CompilerEffects,
        charge: &mut impl FnMut(u64) -> Result<(), Error>,
    ) -> Result<Vec<Value>, EffectError> {
        // Central VM validates the exact signature and charges work_cost before entry.
        if self.operation() == FileAbiOperation::Open {
            let path = memory
                .host_c_string(types, args[0].pointer().map_err(failed)?, MAXIMUM_PATH)
                .map_err(failed)?;
            let mode = memory
                .host_c_string(types, args[1].pointer().map_err(failed)?, MAXIMUM_MODE)
                .map_err(failed)?;
            let mode = FileOpenMode::parse(&mode).map_err(|error| failed(file_error(error)))?;
            let path = source_path(path).map_err(failed)?;
            let scope = effects
                .host_file_scope()
                .ok_or_else(|| host_error(HostError::Unavailable))?;
            let path = scope
                .resolve(std::path::Path::new(&path))
                .map_err(host_error)?;
            let bytes = if mode == FileOpenMode::TruncateUpdate {
                Vec::new()
            } else {
                match request(effects, HostRequest::ReadFileForOpen(path.clone()))? {
                    HostResponse::FileOpen(FileOpenObservation::Bytes(bytes)) => bytes,
                    HostResponse::FileOpen(FileOpenObservation::Failed(
                        FileOpenFailure::NotFound,
                    )) if mode == FileOpenMode::AppendUpdate => Vec::new(),
                    HostResponse::FileOpen(FileOpenObservation::Failed(_)) => {
                        return Ok(vec![Value::Pointer(crate::Pointer::null(self.file_type()))]);
                    }
                    _ => {
                        return Err(failed(Error::EffectResponse(
                            "stdio open requires FileOpen response",
                        )));
                    }
                }
            };
            charge(
                u64::try_from(bytes.len())
                    .map_err(|_| failed(Error::Limit(crate::LimitKind::Fuel)))?,
            )
            .map_err(failed)?;
            if mode != FileOpenMode::Read
                && request(
                    effects,
                    HostRequest::WriteEntireFile {
                        path: path.clone(),
                        bytes: bytes.clone(),
                    },
                )? != HostResponse::WriteStaged
            {
                return Err(failed(Error::EffectResponse(
                    "writable stdio open requires staged write",
                )));
            }
            let handle = machine
                .files
                .opened(path, mode, bytes)
                .map_err(|error| failed(file_error(error)))?;
            if let std::collections::hash_map::Entry::Vacant(entry) =
                machine.tokens.entry(self.file_type())
            {
                entry.insert(FileTokens::new(types, self.file_type()).map_err(failed)?);
            }
            let pointer = machine
                .tokens
                .get_mut(&self.file_type())
                .unwrap()
                .mint(types, memory, handle)
                .map_err(failed)?;
            return Ok(vec![Value::Pointer(pointer)]);
        }
        let stream = if matches!(
            self.operation(),
            FileAbiOperation::Read | FileAbiOperation::Write
        ) {
            3
        } else {
            0
        };
        let pointer = args[stream].pointer().map_err(failed)?;
        let tokens = machine
            .tokens
            .get_mut(&self.file_type())
            .ok_or_else(|| failed(Error::InvalidIr("no live FILE ledger")))?;
        let handle = tokens.lookup(types, memory, pointer).map_err(failed)?;
        match self.operation() {
            FileAbiOperation::Read => {
                let size = unsigned(&args[1]).map_err(failed)?;
                let count = unsigned(&args[2]).map_err(failed)?;
                let read = machine
                    .files
                    .read(handle, size, count)
                    .map_err(|error| failed(file_error(error)))?;
                memory
                    .host_write_bytes(types, args[0].pointer().map_err(failed)?, &read.bytes)
                    .map_err(failed)?;
                Ok(integer(IntegerType::U64, read.complete_items as i128))
            }
            FileAbiOperation::Write => {
                let size = unsigned(&args[1]).map_err(failed)?;
                let count = unsigned(&args[2]).map_err(failed)?;
                let length = size
                    .checked_mul(count)
                    .ok_or_else(|| failed(Error::CheckedCast))?;
                let bytes = memory
                    .host_read_bytes(types, args[0].pointer().map_err(failed)?, length)
                    .map_err(failed)?;
                let count = machine
                    .files
                    .write(handle, size, count, &bytes)
                    .map_err(|error| failed(file_error(error)))?;
                Ok(integer(IntegerType::U64, count as i128))
            }
            FileAbiOperation::Seek => {
                let offset = i64::try_from(
                    args[1]
                        .number()
                        .and_then(|number| number.portable_integer())
                        .map_err(failed)?
                        .value(),
                )
                .map_err(|_| failed(Error::CheckedCast))?;
                let origin = i32::try_from(
                    args[2]
                        .number()
                        .and_then(|number| number.portable_integer())
                        .map_err(failed)?
                        .value(),
                )
                .map_err(|_| failed(Error::CheckedCast))?;
                let status = match SeekOrigin::from_c(origin)
                    .and_then(|origin| machine.files.seek(handle, offset, origin))
                {
                    Ok(()) => 0,
                    Err(FileError::InvalidSeek) => -1,
                    Err(error) => return Err(failed(file_error(error))),
                };
                Ok(integer(IntegerType::S32, status))
            }
            FileAbiOperation::Tell => Ok(integer(
                IntegerType::S64,
                machine
                    .files
                    .tell(handle)
                    .map_err(|error| failed(file_error(error)))? as i128,
            )),
            FileAbiOperation::Eof => Ok(integer(
                IntegerType::S32,
                i128::from(
                    machine
                        .files
                        .eof(handle)
                        .map_err(|error| failed(file_error(error)))?,
                ),
            )),
            FileAbiOperation::Close => {
                match machine
                    .files
                    .close(handle, &mut Requests(effects))
                    .map_err(|error| failed(file_error(error)))?
                {
                    FileCloseOutcome::Closed => {}
                    FileCloseOutcome::Pending(key) => {
                        return Err(EffectError::Pending(Dependency::Host(key)));
                    }
                    FileCloseOutcome::Rejected(error) => return Err(host_error(error)),
                }
                let retired = tokens.remove(types, memory, pointer).map_err(failed)?;
                memory.release_opaque_host_token(retired).map_err(failed)?;
                if tokens.live_tokens() == 0 {
                    machine.tokens.remove(&self.file_type());
                }
                Ok(integer(IntegerType::S32, 0))
            }
            FileAbiOperation::Open => unreachable!(),
        }
    }
}
#[cfg(test)]
mod tests;
