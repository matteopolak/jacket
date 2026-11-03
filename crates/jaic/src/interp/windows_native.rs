//! Real host filesystem backing for Windows compile-time calls on another OS.
use super::errno::{EEXIST, EINVAL, EISDIR, ENOENT, ENOTDIR, ENOTEMPTY, EPERM};
use super::sandbox::guest::write_bytes;
use super::sandbox::symbols::WinFn;
use super::sandbox::{O_CREAT, O_EXCL, O_RDWR, O_TRUNC, O_WRONLY};
use super::windows::{FileDetails, WindowsBackend, WindowsCalls};
use crate::fxhash::HashMap;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub(super) struct NativeWindows {
    calls: WindowsCalls,
    backend: NativeFiles,
}

impl NativeWindows {
    pub(super) fn foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
        sig: &crate::ir::Sig,
    ) -> Option<u64> {
        self.calls
            .foreign(&mut self.backend, WinFn::from_name(symbol)?, args, sig)
    }
}

#[derive(Default)]
struct NativeFiles {
    files: HashMap<i64, File>,
    next_fd: i64,
    paths: HashMap<i64, String>,
    started: Option<Instant>,
}

fn errno(error: io::Error) -> u64 {
    match error.kind() {
        io::ErrorKind::NotFound => ENOENT,
        io::ErrorKind::AlreadyExists => EEXIST,
        io::ErrorKind::NotADirectory => ENOTDIR,
        io::ErrorKind::IsADirectory => EISDIR,
        io::ErrorKind::DirectoryNotEmpty => ENOTEMPTY,
        io::ErrorKind::PermissionDenied => EPERM,
        _ => EINVAL,
    }
}

fn file_details(metadata: &fs::Metadata) -> FileDetails {
    let ticks = |time: io::Result<SystemTime>| {
        time.ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| (d.as_nanos() / 100) as u64)
            + 116_444_736_000_000_000
    };
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        [metadata.dev(), metadata.ino(), 0]
    };
    #[cfg(not(unix))]
    let identity = [0; 3];
    FileDetails {
        attributes: if metadata.is_dir() {
            0x10
        } else {
            0x80
        },
        size: if metadata.is_dir() {
            0
        } else {
            metadata.len()
        },
        created: ticks(metadata.created()),
        accessed: ticks(metadata.accessed()),
        written: ticks(metadata.modified()),
        identity,
    }
}

impl WindowsBackend for NativeFiles {
    fn details(&self, path: &str) -> Option<FileDetails> {
        let mut details = file_details(&fs::metadata(path).ok()?);
        if fs::symlink_metadata(path).ok()?.is_symlink() {
            details.attributes |= 0x400;
        }
        Some(details)
    }

    fn fd_details(&self, fd: i64) -> Option<FileDetails> {
        Some(file_details(&self.files.get(&fd)?.metadata().ok()?))
    }

    fn canonical(&self, fd: i64) -> Option<String> {
        Some(
            fs::canonicalize(self.paths.get(&fd)?)
                .ok()?
                .to_string_lossy()
                .into_owned(),
        )
    }

    fn contents(&self, fd: i64) -> Option<Vec<u8>> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            let file = self.files.get(&fd)?;
            let mut bytes = vec![0; file.metadata().ok()?.len().try_into().ok()?];
            file.read_exact_at(&mut bytes, 0).ok()?;
            Some(bytes)
        }
        #[cfg(not(unix))]
        {
            let _ = fd;
            None
        }
    }

    fn flush(&mut self, fd: i64) -> Result<(), u64> {
        self.files
            .get(&fd)
            .ok_or(super::errno::EBADF)?
            .sync_all()
            .map_err(errno)
    }

    fn copy_file(&mut self, from: &str, to: &str) -> Result<(), u64> {
        fs::copy(from, to).map(|_| ()).map_err(errno)
    }

    fn hard_link(&mut self, from: &str, to: &str) -> Result<(), u64> {
        fs::hard_link(from, to).map_err(errno)
    }

    fn cwd(&self) -> String {
        std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }

    fn set_cwd(&mut self, path: String) -> Result<(), u64> {
        std::env::set_current_dir(path).map_err(errno)
    }

    fn is_dir(&self, path: &str) -> bool {
        std::path::Path::new(path).is_dir()
    }

    fn exists_file(&self, path: &str) -> bool {
        std::path::Path::new(path).is_file()
    }

    fn list(&self, path: &str) -> Vec<(String, bool)> {
        let mut entries: Vec<_> = fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    e.path().is_dir(),
                )
            })
            .collect();
        entries.sort();
        entries
    }

    fn create_dir(&mut self, path: &str) -> Result<(), u64> {
        fs::create_dir(path).map_err(errno)
    }

    fn remove_dir(&mut self, path: &str) -> Result<(), u64> {
        fs::remove_dir(path).map_err(errno)
    }

    fn delete_file(&mut self, path: &str) -> Result<(), u64> {
        fs::remove_file(path).map_err(errno)
    }

    fn rename_file(&mut self, from: &str, to: &str) -> Result<(), u64> {
        fs::rename(from, to).map_err(errno)
    }

    fn open_file(&mut self, path: String, flags: u64) -> Result<i64, u64> {
        let mode = flags & 3;
        let file = OpenOptions::new()
            .read(mode != O_WRONLY)
            .write(mode == O_WRONLY || mode == O_RDWR)
            .create(flags & O_CREAT != 0)
            .create_new(flags & O_EXCL != 0)
            .truncate(flags & O_TRUNC != 0)
            .open(&path)
            .map_err(errno)?;
        self.next_fd = self.next_fd.max(2) + 1;
        self.paths.insert(self.next_fd, path);
        self.files.insert(self.next_fd, file);
        Ok(self.next_fd)
    }

    fn has_fd(&self, fd: i64) -> bool {
        self.files.contains_key(&fd)
    }

    fn close_fd(&mut self, fd: i64) {
        self.files.remove(&fd);
        self.paths.remove(&fd);
    }

    fn read_fd(&mut self, fd: i64, out: u64, count: usize) -> Option<usize> {
        let mut bytes = vec![0; count];
        let n = if fd == 0 {
            io::stdin().read(&mut bytes).ok()?
        } else {
            self.files.get_mut(&fd)?.read(&mut bytes).ok()?
        };
        write_bytes(out, &bytes[..n]);
        Some(n)
    }

    fn write_fd(&mut self, fd: i64, bytes: &[u8]) -> Option<usize> {
        self.files.get_mut(&fd)?.write(bytes).ok()
    }

    fn file_len(&self, fd: i64) -> Option<usize> {
        Some(self.files.get(&fd)?.metadata().ok()?.len() as usize)
    }

    fn seek(&mut self, fd: i64, offset: i64, whence: u64) -> Option<i64> {
        let pos = match whence {
            0 => SeekFrom::Start(u64::try_from(offset).ok()?),
            1 => SeekFrom::Current(offset),
            2 => SeekFrom::End(offset),
            _ => return None,
        };
        self.files
            .get_mut(&fd)?
            .seek(pos)
            .ok()
            .and_then(|n| i64::try_from(n).ok())
    }

    fn write(&mut self, bytes: &[u8], stderr: bool) {
        super::Host::write(&mut super::NativeHost, bytes, stderr);
    }

    fn wall_ns(&mut self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    fn monotonic_ns(&mut self) -> u64 {
        self.started
            .get_or_insert_with(Instant::now)
            .elapsed()
            .as_nanos() as u64
    }
}
