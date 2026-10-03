//! Bounded virtual stdio state. Handles refer to owned buffers, never native FILE pointers.
use crate::host_effects::{
    HostEffects, HostError, HostOutcome, HostPath, HostRequest, HostRequestKey, HostResponse,
};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VirtualFileHandle(u64);
fn handle() -> VirtualFileHandle {
    VirtualFileHandle(
        NEXT_HANDLE
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("virtual file identities exhausted"),
    )
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileOpenMode {
    Read,
    TruncateUpdate,
    AppendUpdate,
}
impl FileOpenMode {
    pub fn parse(bytes: &[u8]) -> Result<Self, FileError> {
        match bytes {
            b"rb" | b"r" => Ok(Self::Read),
            b"wb+" | b"w+" => Ok(Self::TruncateUpdate),
            b"a+" | b"ab+" => Ok(Self::AppendUpdate),
            _ => Err(FileError::UnsupportedMode),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeekOrigin {
    Start,
    Current,
    End,
}
impl SeekOrigin {
    pub fn from_c(value: i32) -> Result<Self, FileError> {
        match value {
            0 => Ok(Self::Start),
            1 => Ok(Self::Current),
            2 => Ok(Self::End),
            _ => Err(FileError::InvalidSeek),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct FileLimits {
    pub handles: usize,
    pub payload_bytes: usize,
    pub transfer_bytes: usize,
}
impl Default for FileLimits {
    fn default() -> Self {
        Self {
            handles: 1024,
            payload_bytes: 16 * 1024 * 1024,
            transfer_bytes: 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileError {
    UnknownHandle,
    UnsupportedMode,
    InvalidSeek,
    ReadOnly,
    InvalidCount,
    LiveHandles,
    Budget(&'static str),
    Host(HostError),
}
impl From<HostError> for FileError {
    fn from(error: HostError) -> Self {
        Self::Host(error)
    }
}
impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "virtual stdio: {self:?}")
    }
}
impl std::error::Error for FileError {}
#[derive(Clone, Debug)]
struct FileState {
    path: HostPath,
    bytes: Vec<u8>,
    mode: FileOpenMode,
    cursor: usize,
    eof: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRead {
    pub bytes: Vec<u8>,
    pub complete_items: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileCloseOutcome {
    Closed,
    Pending(HostRequestKey),
    Rejected(HostError),
}
/// One ledger belongs to one evaluation transaction. Drop/reset invalidates all handles;
/// global monotonic identities ensure a rolled-back handle cannot select a future file.
#[derive(Clone, Debug)]
pub struct VirtualFiles {
    files: HashMap<VirtualFileHandle, FileState>,
    limits: FileLimits,
    payload_bytes: usize,
}
impl Default for VirtualFiles {
    fn default() -> Self {
        Self::new(FileLimits::default())
    }
}
impl VirtualFiles {
    pub fn new(limits: FileLimits) -> Self {
        Self {
            files: HashMap::new(),
            limits,
            payload_bytes: 0,
        }
    }
    pub fn live_handles(&self) -> usize {
        self.files.len()
    }
    pub fn payload_bytes(&self) -> usize {
        self.payload_bytes
    }
    /// General rollback snapshots retain live streams and their private cursors.
    pub(crate) fn snapshot_bounds(
        &self,
        charge: &mut impl FnMut(u64) -> Result<(), crate::Error>,
    ) -> Result<usize, crate::Error> {
        let overflow = || crate::Error::Limit(crate::LimitKind::ValueCells);
        let mut cells = self.files.capacity().checked_add(1).ok_or_else(overflow)?;
        // Admit the table walk before inspecting any nested path or byte backing.
        charge(u64::try_from(cells).map_err(|_| crate::Error::Limit(crate::LimitKind::Fuel))?)?;
        for file in self.files.values() {
            // root/path and buffer headers, mode, cursor and EOF are retained even
            // for an empty file. Payload cells count backing, not live byte length.
            cells = cells
                .checked_add(6)
                .and_then(|cells| cells.checked_add(file.path.retained_path_capacity()))
                .and_then(|cells| cells.checked_add(file.bytes.capacity()))
                .ok_or_else(overflow)?;
        }
        Ok(cells)
    }
    pub(crate) fn closed_table_capacity(&self) -> Result<usize, FileError> {
        self.require_closed()?;
        Ok(self.files.capacity())
    }
    /// Install only a successful typed host observation or an explicitly staged writable open.
    /// This method grants no root authority; host reads/writes still validate HostPath capabilities.
    pub fn opened(
        &mut self,
        path: HostPath,
        mode: FileOpenMode,
        bytes: Vec<u8>,
    ) -> Result<VirtualFileHandle, FileError> {
        if self.files.len() >= self.limits.handles {
            return Err(FileError::Budget("file handles"));
        }
        if mode == FileOpenMode::TruncateUpdate && !bytes.is_empty() {
            return Err(FileError::InvalidCount);
        }
        let total = self
            .payload_bytes
            .checked_add(bytes.len())
            .ok_or(FileError::Budget("file payload"))?;
        if total > self.limits.payload_bytes {
            return Err(FileError::Budget("file payload"));
        }
        let handle = handle();
        let cursor = if mode == FileOpenMode::AppendUpdate {
            bytes.len()
        } else {
            0
        };
        self.files.insert(
            handle,
            FileState {
                path,
                bytes,
                mode,
                cursor,
                eof: false,
            },
        );
        self.payload_bytes = total;
        Ok(handle)
    }
    fn file(&self, handle: VirtualFileHandle) -> Result<&FileState, FileError> {
        self.files.get(&handle).ok_or(FileError::UnknownHandle)
    }
    fn byte_count(&self, size: usize, count: usize) -> Result<usize, FileError> {
        let bytes = size.checked_mul(count).ok_or(FileError::InvalidCount)?;
        if bytes > self.limits.transfer_bytes {
            return Err(FileError::Budget("file transfer"));
        }
        Ok(bytes)
    }
    /// fread returns complete items while still copying a final partial item's bytes.
    /// Zero-sized reads do not move the cursor or change EOF.
    pub fn read(
        &mut self,
        handle: VirtualFileHandle,
        size: usize,
        count: usize,
    ) -> Result<FileRead, FileError> {
        let requested = self.byte_count(size, count)?;
        let file = self
            .files
            .get_mut(&handle)
            .ok_or(FileError::UnknownHandle)?;
        if requested == 0 {
            return Ok(FileRead {
                bytes: vec![],
                complete_items: 0,
            });
        }
        let available = file.bytes.len().saturating_sub(file.cursor);
        let read = requested.min(available);
        let bytes = if read == 0 {
            vec![]
        } else {
            file.bytes[file.cursor..file.cursor + read].to_vec()
        };
        file.cursor += read;
        file.eof |= requested > available;
        Ok(FileRead {
            bytes,
            complete_items: read / size,
        })
    }
    /// fwrite must receive exactly the checked source byte range. Append modes force
    /// each write to the end even after fseek, matching C append stream behavior.
    pub fn write(
        &mut self,
        handle: VirtualFileHandle,
        size: usize,
        count: usize,
        bytes: &[u8],
    ) -> Result<usize, FileError> {
        let requested = self.byte_count(size, count)?;
        if bytes.len() != requested {
            return Err(FileError::InvalidCount);
        }
        let file = self.file(handle)?;
        if requested == 0 {
            return Ok(0);
        }
        if file.mode == FileOpenMode::Read {
            return Err(FileError::ReadOnly);
        }
        let cursor = if file.mode == FileOpenMode::AppendUpdate {
            file.bytes.len()
        } else {
            file.cursor
        };
        let end = cursor
            .checked_add(requested)
            .ok_or(FileError::InvalidCount)?;
        let length = file.bytes.len().max(end);
        let total = self
            .payload_bytes
            .checked_sub(file.bytes.len())
            .and_then(|n| n.checked_add(length))
            .ok_or(FileError::Budget("file payload"))?;
        if total > self.limits.payload_bytes {
            return Err(FileError::Budget("file payload"));
        }
        let file = self.files.get_mut(&handle).unwrap();
        file.bytes.resize(length, 0);
        file.bytes[cursor..end].copy_from_slice(bytes);
        file.cursor = end;
        self.payload_bytes = total;
        Ok(count)
    }
    pub fn seek(
        &mut self,
        handle: VirtualFileHandle,
        offset: i64,
        origin: SeekOrigin,
    ) -> Result<(), FileError> {
        let file = self.file(handle)?;
        let base = match origin {
            SeekOrigin::Start => 0,
            SeekOrigin::Current => file.cursor,
            SeekOrigin::End => file.bytes.len(),
        };
        let position =
            i128::try_from(base).map_err(|_| FileError::InvalidSeek)? + i128::from(offset);
        let position = usize::try_from(position).map_err(|_| FileError::InvalidSeek)?;
        if position > self.limits.payload_bytes {
            return Err(FileError::Budget("file position"));
        }
        let file = self.files.get_mut(&handle).unwrap();
        file.cursor = position;
        file.eof = false;
        Ok(())
    }
    pub fn tell(&self, handle: VirtualFileHandle) -> Result<i64, FileError> {
        i64::try_from(self.file(handle)?.cursor).map_err(|_| FileError::InvalidSeek)
    }
    pub fn eof(&self, handle: VirtualFileHandle) -> Result<bool, FileError> {
        Ok(self.file(handle)?.eof)
    }
    /// Only a successful staged write closes a writable handle. Pending/rejected
    /// effects leave it live for transaction rollback; no file is published here.
    pub fn close(
        &mut self,
        handle: VirtualFileHandle,
        effects: &mut impl HostEffects,
    ) -> Result<FileCloseOutcome, FileError> {
        let file = self.file(handle)?;
        if file.mode != FileOpenMode::Read {
            match effects.request(HostRequest::WriteEntireFile {
                path: file.path.clone(),
                bytes: file.bytes.clone(),
            }) {
                HostOutcome::Ready(HostResponse::WriteStaged) => {}
                HostOutcome::Ready(_) => {
                    return Err(FileError::Host(HostError::Transaction(
                        "file close response differs",
                    )));
                }
                HostOutcome::Pending(key) => return Ok(FileCloseOutcome::Pending(key)),
                HostOutcome::Rejected(error) => return Ok(FileCloseOutcome::Rejected(error)),
            }
        }
        let file = self.files.remove(&handle).unwrap();
        self.payload_bytes -= file.bytes.len();
        Ok(FileCloseOutcome::Closed)
    }
    /// Source transaction exit must close all handles explicitly; resetting on rollback
    /// frees buffers without publishing dirty contents or reviving stale identities.
    pub fn require_closed(&self) -> Result<(), FileError> {
        if self.files.is_empty() {
            Ok(())
        } else {
            Err(FileError::LiveHandles)
        }
    }
    pub fn reset(&mut self) {
        self.files.clear();
        self.payload_bytes = 0;
    }
}
#[cfg(test)]
#[path = "virtual_files/tests.rs"]
mod tests;
