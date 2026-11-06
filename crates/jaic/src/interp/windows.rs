//! Shared Win32 file, path, text and time calls for sandbox and native cross evaluation.
use super::errno::{EBADF, EEXIST, EISDIR, ENOENT, ENOTDIR, ENOTEMPTY, EPERM};
use super::globals::ZeroedBlock;
use super::sandbox::guest::{fill, read_bytes, read_u8, write_u32, write_u64, write_units};
use super::sandbox::join;
use super::sandbox::symbols::WinFn;
use super::sandbox::{O_CREAT, O_EXCL, O_RDWR, O_TRUNC, O_WRONLY};
use crate::fxhash::HashMap;
use crate::interp::word_or_zero;

/// Host-neutral file details, with times already expressed as Windows FILETIME ticks.
pub(super) struct FileDetails {
    pub attributes: u32,
    pub size: u64,
    pub created: u64,
    pub accessed: u64,
    pub written: u64,
    pub identity: [u64; 3],
}

/// Backing operations use the interpreter's portable open flags and errno values.
pub(super) trait WindowsBackend {
    fn details(&self, path: &str) -> Option<FileDetails>;
    fn fd_details(&self, fd: i64) -> Option<FileDetails>;
    fn canonical(&self, fd: i64) -> Option<String>;
    fn contents(&self, fd: i64) -> Option<Vec<u8>>;
    fn flush(&mut self, fd: i64) -> Result<(), u64>;
    fn copy_file(&mut self, from: &str, to: &str) -> Result<(), u64>;
    fn hard_link(&mut self, from: &str, to: &str) -> Result<(), u64>;
    fn cwd(&self) -> String;
    fn set_cwd(&mut self, path: String) -> Result<(), u64>;
    fn is_dir(&self, path: &str) -> bool;
    fn exists_file(&self, path: &str) -> bool;
    fn list(&self, path: &str) -> Vec<(String, bool)>;
    fn create_dir(&mut self, path: &str) -> Result<(), u64>;
    fn remove_dir(&mut self, path: &str) -> Result<(), u64>;
    fn delete_file(&mut self, path: &str) -> Result<(), u64>;
    fn rename_file(&mut self, from: &str, to: &str) -> Result<(), u64>;
    fn open_file(&mut self, path: String, flags: u64) -> Result<i64, u64>;
    fn has_fd(&self, fd: i64) -> bool;
    fn close_fd(&mut self, fd: i64);
    fn read_fd(&mut self, fd: i64, out: u64, count: usize) -> Option<usize>;
    fn write_fd(&mut self, fd: i64, bytes: &[u8]) -> Option<usize>;
    fn file_len(&self, fd: i64) -> Option<usize>;
    fn seek(&mut self, fd: i64, offset: i64, whence: u64) -> Option<i64>;
    fn write(&mut self, bytes: &[u8], stderr: bool);
    fn wall_ns(&mut self) -> u64;
    fn monotonic_ns(&mut self) -> u64;
}

#[derive(Default)]
pub(super) struct WindowsCalls {
    pub(super) last_error: u64,
    finds: HashMap<u64, Vec<(String, bool)>>,
    next_find: u64,
    mappings: HashMap<u64, Vec<u8>>,
    views: HashMap<u64, ZeroedBlock>,
}

const HANDLE_BASE: u64 = 0x1_0000;
const INVALID_HANDLE_VALUE: u64 = u64::MAX;
const INVALID_FILE_ATTRIBUTES: u64 = 0xFFFF_FFFF;
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
const GENERIC_READ: u64 = 0x8000_0000;
const GENERIC_WRITE: u64 = 0x4000_0000;
const ERROR_FILE_NOT_FOUND: u64 = 2;
const ERROR_NOT_SUPPORTED: u64 = 50;
const ERROR_INVALID_NAME: u64 = 123;
const ERROR_PATH_NOT_FOUND: u64 = 3;
const ERROR_ACCESS_DENIED: u64 = 5;
const ERROR_INVALID_HANDLE: u64 = 6;
const ERROR_NO_MORE_FILES: u64 = 18;
const ERROR_FILE_EXISTS: u64 = 80;
const ERROR_INVALID_PARAMETER: u64 = 87;
const ERROR_DIR_NOT_EMPTY: u64 = 145;
const ERROR_ALREADY_EXISTS: u64 = 183;

/// `WIN32_FIND_DATAW`: attributes, three times, size, two reserved words, then the name.
const FIND_DATA_SIZE: usize = 592;

const FIND_DATA_NAME: u64 = 44;

/// `QueryPerformanceFrequency`: ticks of 100 ns, like the system's own.
const PERFORMANCE_FREQUENCY: u64 = 10_000_000;

/// The Windows error for a C library errno.
fn windows_error(errno: u64) -> u64 {
    match errno {
        ENOENT => ERROR_FILE_NOT_FOUND,
        EEXIST => ERROR_FILE_EXISTS,
        EBADF => ERROR_INVALID_HANDLE,
        ENOTEMPTY => ERROR_DIR_NOT_EMPTY,
        EISDIR | EPERM => ERROR_ACCESS_DENIED,
        ENOTDIR => ERROR_PATH_NOT_FOUND,
        _ => ERROR_INVALID_PARAMETER,
    }
}

/// The bytes of `count` units of `size` bytes at `ptr`; a count of -1 reads up to and
/// including a zero unit.
fn counted(ptr: u64, count: i64, size: usize) -> Vec<u8> {
    if ptr == 0 {
        return Vec::new();
    }
    let units = if count >= 0 {
        count as usize
    } else {
        let mut n = 0;
        while (0..size as u64).any(|i| read_u8(ptr + (n * size) as u64 + i) != 0) {
            n += 1;
        }
        n + 1
    };
    read_bytes(ptr, units * size)
}

/// Write `units` to `out` (room for `room` of them) the Windows way: a room of 0 asks for
/// the count only, and too little room writes nothing and returns 0.
fn put_counted<T: Copy>(units: &[T], out: u64, room: i64) -> u64 {
    if room == 0 {
        return units.len() as u64;
    }
    if out == 0 || (room as usize) < units.len() {
        return 0;
    }
    write_units(out, units);
    units.len() as u64
}

/// The UTF-16 text at `ptr` up to its terminator.
fn wide_string(ptr: u64) -> String {
    let wide = counted(ptr, -1, 2);
    let units: Vec<u16> = wide
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .take_while(|&u| u != 0)
        .collect();
    String::from_utf16_lossy(&units)
}

/// Write `text` and a terminator to `out` (room for `room` units) the way
/// `GetCurrentDirectoryW` does: the length without the terminator, or the room it needs.
fn put_wide_path(text: &str, out: u64, room: u64) -> u64 {
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    if out == 0 || room as usize <= wide.len() {
        return (wide.len() + 1) as u64;
    }
    let n = wide.len() as u64;
    wide.push(0);
    put_counted(&wide, out, room as i64);
    n
}

/// Preserve a Windows drive/share root while normalizing only its rooted suffix.
/// The backend provides one current directory; another drive's relative directory is unknown.
/// For example, `C:\base` plus `\file` resolves to `C:/file`, while `C:other` uses that base.
fn windows_path(cwd: &str, path: &str) -> Result<String, u64> {
    fn spelling(path: &str) -> Result<String, u64> {
        // Stripping the verbatim prefix must not turn an invalid drive name into C:relative.
        if let Some(rest) = path.strip_prefix(r"\\?\")
            && !rest.starts_with(r"UNC\")
            && !matches!(rest.as_bytes(), [drive, b':', b'\\', ..] if drive.is_ascii_alphabetic())
        {
            return Err(ERROR_NOT_SUPPORTED);
        }
        crate::paths::native_spelling(path, true).ok_or(ERROR_NOT_SUPPORTED)
    }
    fn drive(path: &str) -> bool {
        path.as_bytes().get(1) == Some(&b':') && path.as_bytes()[0].is_ascii_alphabetic()
    }
    fn root(path: &str) -> Result<(&str, &str), u64> {
        if let Some(unc) = path.strip_prefix("//") {
            let mut parts = unc.splitn(3, '/');
            let server = parts.next().unwrap_or_default();
            let share = parts.next().unwrap_or_default();
            if matches!(server, "." | "?") {
                return Err(ERROR_NOT_SUPPORTED);
            }
            if server.is_empty()
                || share.is_empty()
                || server == ".."
                || matches!(share, "." | "..")
            {
                return Err(ERROR_INVALID_NAME);
            }
            let end = 2 + server.len() + 1 + share.len();
            return Ok((&path[..end], &path[end..]));
        }
        if drive(path) && path[2..].starts_with('/') {
            return Ok((&path[..2], &path[2..]));
        }
        if path.starts_with('/') {
            return Ok(("", path));
        }
        Err(ERROR_NOT_SUPPORTED)
    }
    fn normalized(prefix: &str, tail: &str) -> String {
        let rooted = format!("/{}", tail.trim_start_matches('/'));
        let suffix = crate::paths::slashed(&crate::paths::normalize(std::path::Path::new(&rooted)));
        format!("{prefix}{suffix}")
    }
    if path.is_empty() {
        return Err(ERROR_INVALID_NAME);
    }
    let path = spelling(path)?;
    if path.starts_with("//") || (drive(&path) && path[2..].starts_with('/')) {
        let (prefix, tail) = root(&path)?;
        return Ok(normalized(prefix, tail));
    }
    let cwd = spelling(cwd)?;
    let (prefix, tail) = root(&cwd)?;
    if drive(&path) {
        if !drive(prefix) || !prefix.eq_ignore_ascii_case(&path[..2]) {
            return Err(ERROR_NOT_SUPPORTED);
        }
        return Ok(normalized(prefix, &join(tail, &path[2..])));
    }
    if path.as_bytes().get(1) == Some(&b':') {
        return Err(ERROR_INVALID_NAME);
    }
    if path.starts_with('/') {
        return Ok(normalized(prefix, &path));
    }
    Ok(normalized(prefix, &join(tail, &path)))
}

// Every caller keeps its own Win32 failure sentinel; unsupported paths never reach the backend.
macro_rules! resolved_path {
    ($calls:ident, $backend:ident, $ptr:expr, $failure:expr) => {
        match $calls.wide_path($backend, $ptr) {
            Ok(path) => path,
            Err(error) => {
                $calls.last_error = error;
                return Some($failure);
            }
        }
    };
}

impl WindowsCalls {
    /// A Windows path argument, resolved like the C library's.
    fn wide_path(&self, backend: &impl WindowsBackend, ptr: u64) -> Result<String, u64> {
        windows_path(&backend.cwd(), &wide_string(ptr))
    }

    fn fail(&mut self, error: u64) -> u64 {
        self.last_error = error;
        0
    }

    /// `BOOL` for a result whose error is an errno.
    fn win_result(&mut self, result: Result<(), u64>) -> u64 {
        match result {
            Ok(()) => 1,
            Err(errno) => self.fail(windows_error(errno)),
        }
    }

    fn handle_fd(&self, backend: &impl WindowsBackend, handle: u64) -> Option<i64> {
        let fd = handle.checked_sub(HANDLE_BASE)? as i64;
        ((0..=2).contains(&fd) || backend.has_fd(fd)).then_some(fd)
    }

    /// Fill a `WIN32_FIND_DATAW` at `out` for `name` in `dir`.
    fn fill_find_data(
        &self,
        backend: &impl WindowsBackend,
        dir: &str,
        name: &str,
        is_dir: bool,
        out: u64,
    ) {
        fill(out, 0, FIND_DATA_SIZE);
        let attributes = if is_dir {
            FILE_ATTRIBUTE_DIRECTORY
        } else {
            FILE_ATTRIBUTE_NORMAL
        };
        write_u32(out, attributes);
        if let Some(details) = backend.details(&join(dir, name)) {
            write_u32(out, details.attributes);
            if details.attributes & 0x400 != 0 {
                write_u32(out + 36, 0xa000_000c);
            }
            write_u64(out + 4, details.created);
            write_u64(out + 12, details.accessed);
            write_u64(out + 20, details.written);
            let size = details.size;
            write_u32(out + 28, (size >> 32) as u32);
            write_u32(out + 32, size as u32);
        }
        let mut wide: Vec<u16> = name.encode_utf16().take(259).collect();
        wide.push(0);
        put_counted(&wide, out + FIND_DATA_NAME, 260);
    }

    /// The Windows calls; `None` for any other symbol.
    pub(super) fn foreign(
        &mut self,
        backend: &mut impl WindowsBackend,
        fun: WinFn,
        args: &[u64],
        sig: &crate::ir::Sig,
    ) -> Option<u64> {
        // Aggregate C arguments live at addresses in interpreter IR. LARGE_INTEGER is
        // a union, although the native ABI passes its eight bytes in a register.
        let mut unpacked;
        let args = if fun == WinFn::SetFilePointerEx
            && sig
                .c_abi
                .as_ref()
                .and_then(|abi| abi.params.get(1))
                .is_some_and(Option::is_some)
        {
            unpacked = args.to_vec();
            unpacked[1] = super::sandbox::guest::read_u64(unpacked[1]);
            &unpacked
        } else {
            args
        };
        self.windows_text_and_time(backend, fun, args)
            .or_else(|| self.windows_paths(backend, fun, args))
            .or_else(|| self.windows_handles(backend, fun, args))
            .or_else(|| self.windows_mappings(backend, fun, args))
    }

    /// Error codes, code page conversion and the clocks.
    fn windows_text_and_time(
        &mut self,
        backend: &mut impl WindowsBackend,
        fun: WinFn,
        args: &[u64],
    ) -> Option<u64> {
        let arg = |i: usize| word_or_zero(args, i);
        Some(match fun {
            WinFn::GetLastError => self.last_error,
            WinFn::SetLastError => {
                self.last_error = arg(0) as u32 as u64;
                0
            }
            // Every code page is taken as UTF-8.
            WinFn::MultiByteToWideChar => {
                let text = counted(arg(2), arg(3) as i32 as i64, 1);
                let wide: Vec<u16> = String::from_utf8_lossy(&text).encode_utf16().collect();
                put_counted(&wide, arg(4), arg(5) as i32 as i64)
            }
            WinFn::WideCharToMultiByte => {
                let wide = counted(arg(2), arg(3) as i32 as i64, 2);
                let units: Vec<u16> = wide
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| u16::from_le_bytes(*c))
                    .collect();
                let text = String::from_utf16_lossy(&units).into_bytes();
                put_counted(&text, arg(4), arg(5) as i32 as i64)
            }
            WinFn::GetSystemTimeAsFileTime | WinFn::GetSystemTimePreciseAsFileTime => {
                // FILETIME: 100 ns ticks since 1601.
                let ns = backend.wall_ns();
                write_u64(arg(0), ns / 100 + 116_444_736_000_000_000);
                0
            }
            WinFn::QueryPerformanceCounter => {
                write_u64(arg(0), backend.monotonic_ns() / 100);
                1
            }
            WinFn::QueryPerformanceFrequency => {
                write_u64(arg(0), PERFORMANCE_FREQUENCY);
                1
            }
            _ => return None,
        })
    }

    /// Directories and file names.
    fn windows_paths(
        &mut self,
        backend: &mut impl WindowsBackend,
        fun: WinFn,
        args: &[u64],
    ) -> Option<u64> {
        let arg = |i: usize| word_or_zero(args, i);
        Some(match fun {
            WinFn::GetCurrentDirectoryW => {
                put_wide_path(&backend.cwd(), arg(1), arg(0) as u32 as u64)
            }
            WinFn::SetCurrentDirectoryW => {
                let path = resolved_path!(self, backend, arg(0), 0);
                if !backend.is_dir(&path) {
                    return Some(self.fail(ERROR_PATH_NOT_FOUND));
                }
                let result = backend.set_cwd(path);
                self.win_result(result)
            }
            WinFn::GetFullPathNameW => {
                let path = resolved_path!(self, backend, arg(0), 0);
                if arg(3) != 0 {
                    write_u64(arg(3), 0);
                }
                put_wide_path(&path, arg(2), arg(1) as u32 as u64)
            }
            WinFn::GetFileAttributesW => {
                let path = resolved_path!(self, backend, arg(0), INVALID_FILE_ATTRIBUTES);
                if let Some(details) = backend.details(&path) {
                    details.attributes as u64
                } else {
                    self.last_error = ERROR_FILE_NOT_FOUND;
                    INVALID_FILE_ATTRIBUTES
                }
            }
            WinFn::GetFileAttributesExW => {
                let path = resolved_path!(self, backend, arg(0), 0);
                let Some(details) = backend.details(&path) else {
                    return Some(self.fail(ERROR_FILE_NOT_FOUND));
                };
                if arg(1) != 0 || arg(2) == 0 {
                    return Some(self.fail(ERROR_INVALID_PARAMETER));
                }
                let out = arg(2);
                write_u32(out, details.attributes);
                write_u64(out + 4, details.created);
                write_u64(out + 12, details.accessed);
                write_u64(out + 20, details.written);
                write_u32(out + 28, (details.size >> 32) as u32);
                write_u32(out + 32, details.size as u32);
                1
            }
            WinFn::CopyFileW | WinFn::CreateHardLinkW => {
                let first = resolved_path!(self, backend, arg(0), 0);
                let second = resolved_path!(self, backend, arg(1), 0);
                let result = if fun == WinFn::CopyFileW {
                    if arg(2) != 0 && backend.exists_file(&second) {
                        return Some(self.fail(ERROR_FILE_EXISTS));
                    }
                    backend.copy_file(&first, &second)
                } else {
                    backend.hard_link(&second, &first)
                };
                self.win_result(result)
            }
            WinFn::CreateDirectoryW => {
                let path = resolved_path!(self, backend, arg(0), 0);
                if backend.is_dir(&path) || backend.exists_file(&path) {
                    return Some(self.fail(ERROR_ALREADY_EXISTS));
                }
                let result = backend.create_dir(&path);
                self.win_result(result)
            }

            WinFn::RemoveDirectoryW => {
                let path = resolved_path!(self, backend, arg(0), 0);
                let result = backend.remove_dir(&path);
                self.win_result(result)
            }
            WinFn::DeleteFileW => {
                let path = resolved_path!(self, backend, arg(0), 0);
                let result = backend.delete_file(&path);
                self.win_result(result)
            }

            WinFn::MoveFileExW | WinFn::MoveFileW => {
                let (from, to) = (
                    resolved_path!(self, backend, arg(0), 0),
                    resolved_path!(self, backend, arg(1), 0),
                );
                // Without MOVEFILE_REPLACE_EXISTING an existing target stays.
                if (fun == WinFn::MoveFileW || arg(2) & 1 == 0) && backend.exists_file(&to) {
                    return Some(self.fail(ERROR_ALREADY_EXISTS));
                }
                let result = backend.rename_file(&from, &to);
                self.win_result(result)
            }
            _ => return None,
        })
    }

    /// File handles, the standard handles, directory searches, and what the sandbox refuses.
    fn windows_handles(
        &mut self,
        backend: &mut impl WindowsBackend,
        fun: WinFn,
        args: &[u64],
    ) -> Option<u64> {
        let arg = |i: usize| word_or_zero(args, i);
        Some(match fun {
            WinFn::FlushFileBuffers => {
                let Some(fd) = self.handle_fd(backend, arg(0)) else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                let result = backend.flush(fd);
                self.win_result(result)
            }
            WinFn::GetFinalPathNameByHandleW => {
                let Some(path) = self
                    .handle_fd(backend, arg(0))
                    .and_then(|fd| backend.canonical(fd))
                else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                put_wide_path(&path, arg(1), arg(2) as u32 as u64)
            }
            WinFn::GetFileInformationByHandleEx => {
                if arg(1) != 18 || arg(3) < 24 {
                    return Some(self.fail(ERROR_INVALID_PARAMETER));
                }
                let Some(details) = self
                    .handle_fd(backend, arg(0))
                    .and_then(|fd| backend.fd_details(fd))
                else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                write_units(arg(2), &details.identity);
                1
            }
            WinFn::CreateFileW => {
                let path = resolved_path!(self, backend, arg(0), INVALID_HANDLE_VALUE);
                let access = arg(1) & (GENERIC_READ | GENERIC_WRITE);
                let mut flags = match access {
                    GENERIC_WRITE => O_WRONLY,
                    a if a == GENERIC_READ | GENERIC_WRITE => O_RDWR,
                    _ => 0,
                };
                flags |= match arg(4) {
                    1 => O_CREAT | O_EXCL,  // CREATE_NEW
                    2 => O_CREAT | O_TRUNC, // CREATE_ALWAYS
                    4 => O_CREAT,           // OPEN_ALWAYS
                    5 => O_TRUNC,           // TRUNCATE_EXISTING
                    _ => 0,                 // OPEN_EXISTING
                };
                match backend.open_file(path, flags) {
                    Ok(fd) => fd as u64 + HANDLE_BASE,
                    Err(errno) => {
                        self.last_error = windows_error(errno);
                        INVALID_HANDLE_VALUE
                    }
                }
            }
            WinFn::GetStdHandle => match arg(0) as u32 {
                0xFFFF_FFF6 => HANDLE_BASE,     // STD_INPUT_HANDLE
                0xFFFF_FFF5 => HANDLE_BASE + 1, // STD_OUTPUT_HANDLE
                0xFFFF_FFF4 => HANDLE_BASE + 2, // STD_ERROR_HANDLE
                _ => INVALID_HANDLE_VALUE,
            },
            WinFn::CloseHandle => {
                if self.mappings.remove(&arg(0)).is_some() {
                    return Some(1);
                }
                if self.finds.remove(&arg(0)).is_some() {
                    return Some(1);
                }
                match self.handle_fd(backend, arg(0)) {
                    Some(fd) if fd > 2 => {
                        backend.close_fd(fd);
                        1
                    }
                    Some(_) => 1,
                    None => self.fail(ERROR_INVALID_HANDLE),
                }
            }
            WinFn::ReadFile => {
                let Some(n) = self
                    .handle_fd(backend, arg(0))
                    .and_then(|fd| backend.read_fd(fd, arg(1), arg(2) as u32 as usize))
                else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                if arg(3) != 0 {
                    write_u32(arg(3), n as u32);
                }
                1
            }
            WinFn::WriteFile => {
                let len = arg(2) as u32 as usize;
                let bytes = counted(arg(1), len as i64, 1);
                let written = match self.handle_fd(backend, arg(0)) {
                    Some(fd @ (1 | 2)) => {
                        backend.write(&bytes, fd == 2);
                        Some(len)
                    }
                    Some(fd) => backend.write_fd(fd, &bytes),
                    None => None,
                };
                let Some(n) = written else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                if arg(3) != 0 {
                    write_u32(arg(3), n as u32);
                }
                1
            }
            WinFn::GetFileSizeEx => {
                let Some(len) = self
                    .handle_fd(backend, arg(0))
                    .and_then(|fd| backend.file_len(fd))
                else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                write_u64(arg(1), len as u64);
                1
            }
            WinFn::SetFilePointerEx => {
                let Some(pos) = self
                    .handle_fd(backend, arg(0))
                    .and_then(|fd| backend.seek(fd, arg(1) as i64, arg(3)))
                else {
                    return Some(self.fail(ERROR_INVALID_PARAMETER));
                };
                if arg(2) != 0 {
                    write_u64(arg(2), pos as u64);
                }
                1
            }
            WinFn::FindFirstFileW | WinFn::FindFirstFileExW => {
                // `dir/*`, `dir/*.*` or one exact name.
                let pattern = resolved_path!(self, backend, arg(0), INVALID_HANDLE_VALUE);
                let out = if fun == WinFn::FindFirstFileExW {
                    arg(2)
                } else {
                    arg(1)
                };
                let (dir, name) = pattern.rsplit_once('/').unwrap_or(("/", &pattern));
                let dir = if dir.is_empty() {
                    "/"
                } else {
                    dir
                };
                let mut entries: Vec<(String, bool)> = if name == "*" || name == "*.*" {
                    let mut all = vec![(".".to_string(), true), ("..".to_string(), true)];
                    all.extend(backend.list(dir));
                    all
                } else {
                    let path = join(dir, name);
                    if backend.is_dir(&path) {
                        vec![(name.to_string(), true)]
                    } else if backend.exists_file(&path) {
                        vec![(name.to_string(), false)]
                    } else {
                        Vec::new()
                    }
                };
                if entries.is_empty() || !backend.is_dir(dir) {
                    self.last_error = ERROR_FILE_NOT_FOUND;
                    return Some(INVALID_HANDLE_VALUE);
                }
                let (first, is_dir) = entries.remove(0);
                self.fill_find_data(backend, dir, &first, is_dir, out);
                self.next_find += 1;
                let handle = 0x1_0000_0000 + self.next_find;
                let rest = entries
                    .into_iter()
                    .rev()
                    .map(|(n, d)| (join(dir, &n), d))
                    .collect();
                self.finds.insert(handle, rest);
                handle
            }
            WinFn::FindNextFileW => {
                let Some(next) = self.finds.get_mut(&arg(0)).map(Vec::pop) else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                let Some((path, is_dir)) = next else {
                    return Some(self.fail(ERROR_NO_MORE_FILES));
                };
                let (dir, name) = path.rsplit_once('/').unwrap_or(("/", &path));
                let dir = if dir.is_empty() {
                    "/"
                } else {
                    dir
                };
                self.fill_find_data(backend, dir, name, is_dir, arg(1));
                1
            }
            WinFn::FindClose => {
                if self.finds.remove(&arg(0)).is_none() {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                }

                1
            }
            // Nothing a sandboxed program runs may start another process or share memory
            // with one (see `pipe` and `shm_open`).
            WinFn::CreatePipe
            | WinFn::CreateProcessW
            | WinFn::CreateProcessA
            | WinFn::CreateFileMappingA
            | WinFn::OpenFileMappingW
            | WinFn::OpenFileMappingA => self.fail(ERROR_ACCESS_DENIED),

            _ => return None,
        })
    }

    /// Read-only views of files; no process-shared or anonymous mappings.
    fn windows_mappings(
        &mut self,
        backend: &mut impl WindowsBackend,
        fun: WinFn,
        args: &[u64],
    ) -> Option<u64> {
        let arg = |i: usize| word_or_zero(args, i);
        Some(match fun {
            // Only unnamed, read-only file mappings are supported. Anonymous or shared
            // mappings retain the sandbox's refusal; views own a snapshot until unmapped.
            WinFn::CreateFileMappingW => {
                if arg(2) != 2 || arg(3) != 0 || arg(4) != 0 || arg(5) != 0 {
                    return Some(self.fail(ERROR_ACCESS_DENIED));
                }
                let Some(bytes) = self
                    .handle_fd(backend, arg(0))
                    .and_then(|fd| backend.contents(fd))
                else {
                    return Some(self.fail(ERROR_ACCESS_DENIED));
                };
                self.next_find += 1;
                let handle = 0x1_0000_0000 + self.next_find;
                self.mappings.insert(handle, bytes);
                handle
            }
            WinFn::MapViewOfFile => {
                let Some(bytes) = self.mappings.get(&arg(0)) else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                if arg(1) != 4 {
                    return Some(self.fail(ERROR_ACCESS_DENIED));
                }
                let offset = ((arg(2) as u32 as u64) << 32) | arg(3) as u32 as u64;
                let Some(rest) = usize::try_from(offset).ok().and_then(|n| bytes.get(n..)) else {
                    return Some(self.fail(ERROR_INVALID_PARAMETER));
                };
                let count = if arg(4) == 0 {
                    rest.len()
                } else {
                    arg(4) as usize
                };
                let Some(bytes) = rest.get(..count) else {
                    return Some(self.fail(ERROR_INVALID_PARAMETER));
                };
                let Some(block) = ZeroedBlock::new(count.max(1), 8) else {
                    return Some(self.fail(ERROR_ACCESS_DENIED));
                };
                let address = block.addr();
                super::sandbox::guest::write_bytes(address, bytes);
                self.views.insert(address, block);
                address
            }
            WinFn::UnmapViewOfFile => {
                if self.views.remove(&arg(0)).is_some() {
                    1
                } else {
                    self.fail(ERROR_INVALID_PARAMETER)
                }
            }
            _ => return None,
        })
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn windows_roots_and_known_current_directories_are_host_independent() {
        for (cwd, path, expected) in [
            (r"C:\base\dir", "C:/absolute/../flag", "C:/flag"),
            (r"C:\base\dir", r"\flag", "C:/flag"),
            (r"\\server\share\base", r"\flag", "//server/share/flag"),
            (r"C:\base\dir", "../../../../flag", "C:/flag"),
            (r"C:\base", "C:flag", "C:/base/flag"),
            (r"C:\base", "C:", "C:/base"),
            (r"C:\base", r"c:\flag", "C:/flag"),
            ("/tmp/base", "../flag", "/tmp/flag"),
            ("/tmp", "/elsewhere/flag", "/elsewhere/flag"),
            ("/tmp", r"\\server\share\file", "//server/share/file"),
            ("/tmp", r"\\server\share\..\file", "//server/share/file"),
            ("/tmp", r"\\?\C:\file", "C:/file"),
            ("/tmp", r"\\?\UNC\server\share\file", "//server/share/file"),
        ] {
            assert_eq!(
                windows_path(cwd, path),
                Ok(expected.into()),
                "{cwd:?}, {path:?}"
            );
        }
    }

    #[test]
    fn unsupported_windows_contexts_never_become_missing_file_paths() {
        for (cwd, path, code) in [
            (r"C:\base", "D:relative", ERROR_NOT_SUPPORTED),
            ("/tmp", "C:relative", ERROR_NOT_SUPPORTED),
            (r"C:\base", "", ERROR_INVALID_NAME),
            (r"C:\base", r"\\server", ERROR_INVALID_NAME),
            (r"C:\base", r"\\server\", ERROR_INVALID_NAME),
            (r"C:\base", r"\\.\pipe\name", ERROR_NOT_SUPPORTED),
            (r"C:\base", r"\\?\C:\odd.\file", ERROR_NOT_SUPPORTED),
            (r"C:\base", r"\\?\C:relative", ERROR_NOT_SUPPORTED),
            (r"C:\base", r"\\?\C:", ERROR_NOT_SUPPORTED),
        ] {
            assert_eq!(windows_path(cwd, path), Err(code));
        }
    }
}
