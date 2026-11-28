//! Compile-time calls to the C library of another POSIX OS than the host's (a Linux program's
//! `#run` on a macOS host), answered in the program's ABI over the host's files.
//!
//! The calls whose meaning is the same in both libraries run natively (`foreign.rs` decides
//! which), so descriptors and `FILE*` streams are the host's own. What differs is answered
//! here: `errno` (the program's own cell, in its numbers), the open flags, `struct stat`,
//! directory streams and `struct dirent`, `strerror`, and the stat-family system calls.
use std::ffi::{CStr, CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::sync::Arc;
use std::{fs, io};

use super::records::{self, DirEntry, FileStat};
use super::{Libraries, StatSyscall};
use crate::fxhash::HashMap;
use crate::interp::Window;
use crate::interp::errno::{EBADF, EINVAL};
use crate::interp::globals::ZeroedBlock;
use crate::interp::sandbox::guest::{read_u8, read_uint, write_bytes, write_uint};

/// A procedure answered here, by the name the program's C library gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::interp) enum PosixFn {
    ErrnoAddress,
    Stat,
    Lstat,
    Fstat,
    Fstatat,
    Open,
    Openat,
    Opendir,
    Readdir,
    Closedir,
    Strerror,
    Syscall,
}

impl PosixFn {
    /// The procedure `symbol` names; Darwin's x86-64 names (`stat$INODE64`) are those of its
    /// 64-bit-inode records, the only ones answered.
    pub(in crate::interp) fn from_name(symbol: &str) -> Option<PosixFn> {
        let name = symbol.strip_suffix("$INODE64").unwrap_or(symbol);

        Some(match name {
            "__errno_location" | "__error" => PosixFn::ErrnoAddress,
            "stat" | "stat64" | "__stat64_time64" => PosixFn::Stat,
            "lstat" | "lstat64" | "__lstat64_time64" => PosixFn::Lstat,
            "fstat" | "fstat64" | "__fstat64_time64" => PosixFn::Fstat,
            "fstatat" | "fstatat64" | "__fstatat64_time64" => PosixFn::Fstatat,
            "open" | "open64" => PosixFn::Open,
            "openat" | "openat64" => PosixFn::Openat,
            "opendir" => PosixFn::Opendir,
            "readdir" | "readdir64" => PosixFn::Readdir,
            "closedir" => PosixFn::Closedir,
            "strerror" => PosixFn::Strerror,
            "syscall" => PosixFn::Syscall,
            _ => return None,
        })
    }
}

/// An open directory stream: its entries, read when it was opened.
struct Dir {
    /// What `DIR*` points at; only its address means anything.
    _handle: ZeroedBlock,
    /// The `struct dirent` each `readdir` fills.
    entry: ZeroedBlock,
    entries: Vec<(Vec<u8>, u64, u8)>,
    next: usize,
}

/// The state the answered calls keep for one interpreter.
#[derive(Default)]
pub(in crate::interp) struct NativePosix {
    /// The program's `errno`, in its own numbers; made when it first asks for the address.
    errno: Option<ZeroedBlock>,
    dirs: HashMap<u64, Dir>,
    /// `strerror` results by error number; they stay valid for the whole run, like libc's.
    texts: HashMap<u64, ZeroedBlock>,
}

/// A zero-terminated string of the program, as a host path.
fn c_string(addr: u64) -> CString {
    let mut bytes = Vec::new();

    if addr != 0 {
        loop {
            let byte = read_u8(addr + bytes.len() as u64);

            if byte == 0 {
                break;
            }

            bytes.push(byte);
        }
    }

    CString::new(bytes).unwrap_or_default()
}

fn file_stat(metadata: &fs::Metadata) -> FileStat {
    let birth = metadata
        .created()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or((0, 0), |since| {
            (since.as_secs() as i64, since.subsec_nanos())
        });

    FileStat {
        dev: metadata.dev(),
        inode: metadata.ino(),
        mode: metadata.mode(),
        links: metadata.nlink(),
        uid: metadata.uid(),
        gid: metadata.gid(),
        rdev: metadata.rdev(),
        size: metadata.size(),
        blksize: metadata.blksize(),
        blocks: metadata.blocks(),
        times: [
            (metadata.atime(), metadata.atime_nsec() as u32),
            (metadata.mtime(), metadata.mtime_nsec() as u32),
            (metadata.ctime(), metadata.ctime_nsec() as u32),
            birth,
        ],
    }
}

/// The `d_type` of a directory entry of type `kind`; the numbers are the same everywhere.
fn entry_kind(kind: fs::FileType) -> u8 {
    if kind.is_dir() {
        records::DT_DIR
    } else if kind.is_file() {
        records::DT_REG
    } else if kind.is_symlink() {
        10
    } else if kind.is_fifo() {
        1
    } else if kind.is_char_device() {
        2
    } else if kind.is_block_device() {
        6
    } else if kind.is_socket() {
        12
    } else {
        0
    }
}

/// The host's error number of `error`.
fn host_code(error: &io::Error) -> u64 {
    error.raw_os_error().map_or(5, |code| code as u64)
}

unsafe extern "C" {
    #[link_name = "open"]
    fn host_open(path: *const std::ffi::c_char, flags: i32, ...) -> i32;

    #[link_name = "openat"]
    fn host_openat(dir: i32, path: *const std::ffi::c_char, flags: i32, ...) -> i32;

    #[link_name = "strerror"]
    fn host_strerror(code: i32) -> *const std::ffi::c_char;
}

/// The host's `AT_FDCWD`.
fn host_working_directory(libraries: Libraries) -> i32 {
    libraries.host.at_working_directory() as i32
}

impl NativePosix {
    /// Answer `fun` for a program of `libraries.target`, with its arguments `args`. Runs
    /// inside `guest::with_memory`. `None`: these arguments ask for something not answered
    /// here (another system call, a directory descriptor other than the working directory).
    pub(in crate::interp) fn foreign(
        &mut self,
        libraries: Libraries,
        window: Option<&Arc<Window>>,
        fun: PosixFn,
        args: &[u64],
    ) -> Option<u64> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);

        Some(match fun {
            PosixFn::ErrnoAddress => self.errno_address(window)?,
            PosixFn::Stat => self.stat(libraries, arg(0), arg(1), false),
            PosixFn::Lstat => self.stat(libraries, arg(0), arg(1), true),
            PosixFn::Fstat => self.fstat(libraries, arg(0) as i32, arg(1)),
            PosixFn::Fstatat => self.fstatat(libraries, args)?,
            PosixFn::Open => self.open(libraries, None, arg(0), arg(1), arg(2)),
            PosixFn::Openat => {
                let dir = arg(0) as i32;
                self.open(libraries, Some(dir), arg(1), arg(2), arg(3))
            }
            PosixFn::Opendir => self.opendir(libraries, window, arg(0))?,
            PosixFn::Readdir => self.readdir(libraries, arg(0)),
            PosixFn::Closedir => match self.dirs.remove(&arg(0)) {
                Some(_) => 0,
                None => self.fail(libraries, libraries.host.errno(EBADF)),
            },
            PosixFn::Strerror => self.strerror(libraries, window, arg(0) as i32 as i64 as u64)?,
            PosixFn::Syscall => {
                let rest = args.get(1..).unwrap_or_default();
                let rest_arg = |i: usize| rest.get(i).copied().unwrap_or(0);

                match libraries.target.stat_syscall(arg(0))? {
                    StatSyscall::Stat => self.stat(libraries, rest_arg(0), rest_arg(1), false),
                    StatSyscall::Lstat => self.stat(libraries, rest_arg(0), rest_arg(1), true),
                    StatSyscall::Fstat => self.fstat(libraries, rest_arg(0) as i32, rest_arg(1)),
                    StatSyscall::Fstatat => self.fstatat(libraries, rest)?,
                }
            }
        })
    }

    /// The program's `errno`, in the host's numbers: what a native call should start with. The
    /// cell is made here if need be, so what the call leaves is kept even when the program
    /// asks for `errno`'s address only after it.
    pub(in crate::interp) fn host_errno(
        &mut self,
        libraries: Libraries,
        window: Option<&Arc<Window>>,
    ) -> u32 {
        match self.errno_address(window) {
            Some(cell) => libraries.to_host(read_uint(cell, 4)) as u32,
            None => 0,
        }
    }

    /// A native call left the host's error `code`: the program now reads it as its own.
    pub(in crate::interp) fn set_from_host_errno(&mut self, libraries: Libraries, code: u32) {
        if let Some(cell) = &self.errno {
            write_uint(cell.addr(), 4, libraries.to_target(u64::from(code)));
        }
    }

    fn errno_address(&mut self, window: Option<&Arc<Window>>) -> Option<u64> {
        if self.errno.is_none() {
            self.errno = Some(ZeroedBlock::in_window(window, 4, 4)?);
        }

        self.errno.as_ref().map(ZeroedBlock::addr)
    }

    /// Set the program's `errno` to the host's error `code`; returns -1.
    fn fail(&mut self, libraries: Libraries, code: u64) -> u64 {
        self.set_from_host_errno(libraries, code as u32);
        u64::MAX
    }

    /// Fill the program's `struct stat` at `out` from `metadata`, or fail with its error.
    fn answer_stat(
        &mut self,
        libraries: Libraries,
        metadata: io::Result<fs::Metadata>,
        out: u64,
    ) -> u64 {
        match metadata {
            Ok(metadata) => {
                records::fill_stat(libraries.target, out, &file_stat(&metadata));
                0
            }
            Err(error) => self.fail(libraries, host_code(&error)),
        }
    }

    fn stat(&mut self, libraries: Libraries, path: u64, out: u64, no_follow: bool) -> u64 {
        let path = c_string(path);
        let path = OsStr::from_bytes(path.as_bytes());
        let metadata = if no_follow {
            fs::symlink_metadata(path)
        } else {
            fs::metadata(path)
        };

        self.answer_stat(libraries, metadata, out)
    }

    fn fstat(&mut self, libraries: Libraries, fd: i32, out: u64) -> u64 {
        use std::os::fd::FromRawFd;

        if fd < 0 {
            return self.fail(libraries, libraries.host.errno(EBADF));
        }

        // SAFETY: the descriptor is the program's, borrowed for this one query and never
        // closed here; a number nothing has open fails the query with EBADF, as `fstat` does.
        let file = std::mem::ManuallyDrop::new(unsafe { fs::File::from_raw_fd(fd) });
        let metadata = file.metadata();

        self.answer_stat(libraries, metadata, out)
    }

    /// `fstatat(dir, path, out, flags)`, for the working directory or an absolute path.
    fn fstatat(&mut self, libraries: Libraries, args: &[u64]) -> Option<u64> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        let path = arg(1);
        let relative = read_u8(path) != b'/';

        if relative && arg(0) as i32 as i64 != libraries.target.at_working_directory() {
            return None;
        }

        let no_follow = arg(3) & libraries.target.at_no_follow() != 0;
        Some(self.stat(libraries, path, arg(2), no_follow))
    }

    /// `open` (`dir` `None`) or `openat`, with the program's flags and the creation mode.
    fn open(
        &mut self,
        libraries: Libraries,
        dir: Option<i32>,
        path: u64,
        flags: u64,
        mode: u64,
    ) -> u64 {
        let (open, unknown) = libraries.target.decode_open(flags as u32 as u64);

        if unknown != 0 {
            return self.fail(libraries, libraries.host.errno(EINVAL));
        }

        let flags = libraries.host.encode_open(open) as i32;
        let path = c_string(path);
        let mode = mode as u32;
        // SAFETY: `path` is zero-terminated; the mode is read only with `O_CREAT`.
        let fd = unsafe {
            match dir {
                None => host_open(path.as_ptr(), flags, mode),
                Some(dir) => {
                    let dir = if i64::from(dir) == libraries.target.at_working_directory() {
                        host_working_directory(libraries)
                    } else {
                        dir
                    };
                    host_openat(dir, path.as_ptr(), flags, mode)
                }
            }
        };

        if fd < 0 {
            return self.fail(libraries, host_code(&io::Error::last_os_error()));
        }

        fd as u64
    }

    fn opendir(
        &mut self,
        libraries: Libraries,
        window: Option<&Arc<Window>>,
        path: u64,
    ) -> Option<u64> {
        let path = c_string(path);
        let path = std::path::Path::new(OsStr::from_bytes(path.as_bytes()));
        let listing = match fs::read_dir(path) {
            Ok(listing) => listing,
            Err(error) => {
                self.fail(libraries, host_code(&error));
                return Some(0);
            }
        };
        let inode = |path: &std::path::Path| fs::metadata(path).map_or(0, |m| m.ino());
        let mut entries = vec![
            (b".".to_vec(), inode(path), records::DT_DIR),
            (b"..".to_vec(), inode(&path.join("..")), records::DT_DIR),
        ];

        for entry in listing.flatten() {
            use std::os::unix::fs::DirEntryExt;
            let kind = entry.file_type().map_or(0, entry_kind);

            entries.push((entry.file_name().as_bytes().to_vec(), entry.ino(), kind));
        }

        let handle = ZeroedBlock::in_window(window, 16, 16)?;
        let entry = ZeroedBlock::in_window(window, records::dirent_size(libraries.target), 8)?;
        let address = handle.addr();

        self.dirs.insert(
            address,
            Dir {
                _handle: handle,
                entry,
                entries,
                next: 0,
            },
        );
        Some(address)
    }

    fn readdir(&mut self, libraries: Libraries, handle: u64) -> u64 {
        let Some(dir) = self.dirs.get_mut(&handle) else {
            self.fail(libraries, libraries.host.errno(EBADF));
            return 0;
        };
        let Some((name, inode, kind)) = dir.entries.get(dir.next) else {
            return 0;
        };

        dir.next += 1;
        let record = DirEntry {
            inode: *inode,
            next: dir.next as u64,
            kind: *kind,
            name,
        };

        records::fill_dirent(libraries.target, dir.entry.addr(), &record);
        dir.entry.addr()
    }

    /// The text of the program's error `code`: the host's text for the same error.
    fn strerror(
        &mut self,
        libraries: Libraries,
        window: Option<&Arc<Window>>,
        code: u64,
    ) -> Option<u64> {
        if let Some(text) = self.texts.get(&code) {
            return Some(text.addr());
        }

        // SAFETY: `strerror` returns a zero-terminated string the C library keeps; it is
        // copied before any other call can change it.
        let text = unsafe { CStr::from_ptr(host_strerror(libraries.to_host(code) as i32)) }
            .to_bytes()
            .to_vec();
        let block = ZeroedBlock::in_window(window, text.len() + 1, 1)?;

        write_bytes(block.addr(), &text);
        let address = block.addr();

        self.texts.insert(code, block);
        Some(address)
    }
}
