//! The Windows calls a sandboxed program makes when it targets Windows (the editor checks a
//! build metaprogram for every target on any host).
//!
//! They run on the same in-memory file system, descriptors and clock as the C library calls.
//! A file handle is its descriptor plus [`HANDLE_BASE`] (so no handle is null or
//! `INVALID_HANDLE_VALUE`), text is UTF-16 converted from and to UTF-8 whatever the code page,
//! and failures set the error `GetLastError` returns.
use super::*;

const HANDLE_BASE: u64 = 0x1_0000;
const INVALID_HANDLE_VALUE: u64 = u64::MAX;
const INVALID_FILE_ATTRIBUTES: u64 = 0xFFFF_FFFF;
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
const GENERIC_READ: u64 = 0x8000_0000;
const GENERIC_WRITE: u64 = 0x4000_0000;
const ERROR_FILE_NOT_FOUND: u64 = 2;
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
///
/// # Safety
/// `ptr` must point at that many units (or a zero-terminated run of them).
unsafe fn counted(ptr: u64, count: i64, size: usize) -> Vec<u8> {
    if ptr == 0 {
        return Vec::new();
    }
    let units = if count >= 0 {
        count as usize
    } else {
        let mut n = 0;
        while unsafe { std::slice::from_raw_parts((ptr as usize + n * size) as *const u8, size) }
            .iter()
            .any(|&b| b != 0)
        {
            n += 1;
        }
        n + 1
    };
    unsafe { std::slice::from_raw_parts(ptr as *const u8, units * size) }.to_vec()
}

/// Write `units` to `out` (room for `room` of them) the Windows way: a room of 0 asks for
/// the count only, and too little room writes nothing and returns 0.
///
/// # Safety
/// `out` must have room for `room` units.
unsafe fn put_counted<T: Copy>(units: &[T], out: u64, room: i64) -> u64 {
    if room == 0 {
        return units.len() as u64;
    }
    if out == 0 || (room as usize) < units.len() {
        return 0;
    }
    unsafe { std::ptr::copy_nonoverlapping(units.as_ptr(), out as *mut T, units.len()) };
    units.len() as u64
}

/// The UTF-16 text at `ptr` up to its terminator.
fn wide_string(ptr: u64) -> String {
    let wide = unsafe { counted(ptr, -1, 2) };
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
    unsafe { put_counted(&wide, out, room as i64) };
    n
}

fn write_u32(addr: u64, value: u32) {
    unsafe { std::ptr::write_unaligned(addr as *mut u32, value) }
}

impl SandboxHost {
    /// A Windows path argument, resolved like the C library's.
    fn wide_path(&self, ptr: u64) -> String {
        let path = wide_string(ptr).replace('\\', "/");
        absolute(
            if self.cwd.is_empty() {
                "/"
            } else {
                &self.cwd
            },
            &path,
        )
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

    fn handle_fd(&self, handle: u64) -> Option<i64> {
        let fd = handle.checked_sub(HANDLE_BASE)? as i64;
        (fd <= 2 || self.fds.contains_key(&fd)).then_some(fd)
    }

    /// Fill a `WIN32_FIND_DATAW` at `out` for `name` in `dir`.
    fn fill_find_data(&self, dir: &str, name: &str, is_dir: bool, out: u64) {
        unsafe { std::ptr::write_bytes(out as *mut u8, 0, FIND_DATA_SIZE) };
        let attributes = if is_dir {
            FILE_ATTRIBUTE_DIRECTORY
        } else {
            FILE_ATTRIBUTE_NORMAL
        };
        write_u32(out, attributes);
        if !is_dir {
            let size = self.fs.file_size(&join(dir, name)) as u64;
            write_u32(out + 28, (size >> 32) as u32);
            write_u32(out + 32, size as u32);
        }
        let mut wide: Vec<u16> = name.encode_utf16().take(259).collect();
        wide.push(0);
        unsafe { put_counted(&wide, out + FIND_DATA_NAME, 260) };
    }

    /// The Windows calls; `None` for any other symbol.
    pub(super) fn windows_foreign(&mut self, symbol: &str, args: &[u64]) -> Option<u64> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        Some(match symbol {
            "GetLastError" => self.last_error,
            "SetLastError" => {
                self.last_error = arg(0) as u32 as u64;
                0
            }
            // Every code page is taken as UTF-8.
            "MultiByteToWideChar" => {
                let text = unsafe { counted(arg(2), arg(3) as i32 as i64, 1) };
                let wide: Vec<u16> = String::from_utf8_lossy(&text).encode_utf16().collect();
                unsafe { put_counted(&wide, arg(4), arg(5) as i32 as i64) }
            }
            "WideCharToMultiByte" => {
                let wide = unsafe { counted(arg(2), arg(3) as i32 as i64, 2) };
                let units: Vec<u16> = wide
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| u16::from_le_bytes(*c))
                    .collect();
                let text = String::from_utf16_lossy(&units).into_bytes();
                unsafe { put_counted(&text, arg(4), arg(5) as i32 as i64) }
            }
            "GetSystemTimeAsFileTime" | "GetSystemTimePreciseAsFileTime" => {
                // FILETIME: 100 ns ticks since 1601.
                self.clock_ns += 1_000;
                let ns = self.clock_ns + BASE_MTIME * 1_000_000_000;
                write_u64(arg(0), ns / 100 + 116_444_736_000_000_000);
                0
            }
            "QueryPerformanceCounter" => {
                self.clock_ns += 1_000;
                write_u64(arg(0), self.clock_ns / 100);
                1
            }
            "QueryPerformanceFrequency" => {
                write_u64(arg(0), PERFORMANCE_FREQUENCY);
                1
            }
            "GetCurrentDirectoryW" => {
                let cwd = if self.cwd.is_empty() {
                    "/"
                } else {
                    &self.cwd
                };
                put_wide_path(cwd, arg(1), arg(0) as u32 as u64)
            }
            "SetCurrentDirectoryW" => {
                let path = self.wide_path(arg(0));
                if !self.fs.is_dir(&path) {
                    return Some(self.fail(ERROR_PATH_NOT_FOUND));
                }
                self.cwd = path;
                1
            }
            "GetFullPathNameW" => {
                let path = self.wide_path(arg(0));
                if arg(3) != 0 {
                    write_u64(arg(3), 0);
                }
                put_wide_path(&path, arg(2), arg(1) as u32 as u64)
            }
            "GetFileAttributesW" => {
                let path = self.wide_path(arg(0));
                if self.fs.is_dir(&path) {
                    FILE_ATTRIBUTE_DIRECTORY as u64
                } else if self.fs.exists_file(&path) {
                    FILE_ATTRIBUTE_NORMAL as u64
                } else {
                    self.last_error = ERROR_FILE_NOT_FOUND;
                    INVALID_FILE_ATTRIBUTES
                }
            }
            "CreateDirectoryW" => {
                let path = self.wide_path(arg(0));
                if self.fs.is_dir(&path) || self.fs.exists_file(&path) {
                    return Some(self.fail(ERROR_ALREADY_EXISTS));
                }
                if !self.parent_exists(&path) {
                    return Some(self.fail(ERROR_PATH_NOT_FOUND));
                }
                self.fs.removed.remove(&path);
                self.fs.dirs.insert(path);
                1
            }
            "RemoveDirectoryW" => {
                let path = self.wide_path(arg(0));
                let result = self.remove_dir(&path);
                self.win_result(result)
            }
            "DeleteFileW" => {
                let path = self.wide_path(arg(0));
                if !self.fs.exists_file(&path) {
                    return Some(self.fail(ERROR_FILE_NOT_FOUND));
                }
                self.fs.files.remove(&path);
                self.fs.forget(&path);
                self.fs.removed.insert(path);
                1
            }
            "MoveFileExW" | "MoveFileW" => {
                let (from, to) = (self.wide_path(arg(0)), self.wide_path(arg(1)));
                // Without MOVEFILE_REPLACE_EXISTING an existing target stays.
                if (symbol == "MoveFileW" || arg(2) & 1 == 0) && self.fs.exists_file(&to) {
                    return Some(self.fail(ERROR_ALREADY_EXISTS));
                }
                let result = self.rename_file(&from, &to);
                self.win_result(result)
            }
            "CreateFileW" => {
                let path = self.wide_path(arg(0));
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
                match self.open_file(path, flags) {
                    Ok(fd) => fd as u64 + HANDLE_BASE,
                    Err(errno) => {
                        self.last_error = windows_error(errno);
                        INVALID_HANDLE_VALUE
                    }
                }
            }
            "GetStdHandle" => match arg(0) as u32 {
                0xFFFF_FFF6 => HANDLE_BASE,     // STD_INPUT_HANDLE
                0xFFFF_FFF5 => HANDLE_BASE + 1, // STD_OUTPUT_HANDLE
                0xFFFF_FFF4 => HANDLE_BASE + 2, // STD_ERROR_HANDLE
                _ => INVALID_HANDLE_VALUE,
            },
            "CloseHandle" => {
                if self.finds.remove(&arg(0)).is_some() {
                    self.allocations.remove(&arg(0));
                    return Some(1);
                }
                match self.handle_fd(arg(0)) {
                    Some(fd) if fd > 2 => {
                        self.fds.remove(&fd);
                        1
                    }
                    Some(_) => 1,
                    None => self.fail(ERROR_INVALID_HANDLE),
                }
            }
            "ReadFile" => {
                let Some(n) = self
                    .handle_fd(arg(0))
                    .and_then(|fd| self.read_fd(fd, arg(1), arg(2) as u32 as usize))
                else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                if arg(3) != 0 {
                    write_u32(arg(3), n as u32);
                }
                1
            }
            "WriteFile" => {
                let len = arg(2) as u32 as usize;
                let bytes = unsafe { counted(arg(1), len as i64, 1) };
                let written = match self.handle_fd(arg(0)) {
                    Some(fd @ (1 | 2)) => {
                        self.write(&bytes, fd == 2);
                        Some(len)
                    }
                    Some(fd) => self.write_fd(fd, &bytes),
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
            "GetFileSizeEx" => {
                let Some(len) = self.handle_fd(arg(0)).and_then(|fd| self.file_len(fd)) else {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                };
                write_u64(arg(1), len as u64);
                1
            }
            "SetFilePointerEx" => {
                let Some(pos) = self
                    .handle_fd(arg(0))
                    .and_then(|fd| self.seek(fd, arg(1) as i64, arg(3)))
                else {
                    return Some(self.fail(ERROR_INVALID_PARAMETER));
                };
                if arg(2) != 0 {
                    write_u64(arg(2), pos as u64);
                }
                1
            }
            "FindFirstFileW" | "FindFirstFileExW" => {
                // `dir/*`, `dir/*.*` or one exact name.
                let pattern = self.wide_path(arg(0));
                let out = if symbol == "FindFirstFileExW" {
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
                    all.extend(self.fs.list(dir));
                    all
                } else {
                    let path = join(dir, name);
                    if self.fs.is_dir(&path) {
                        vec![(name.to_string(), true)]
                    } else if self.fs.exists_file(&path) {
                        vec![(name.to_string(), false)]
                    } else {
                        Vec::new()
                    }
                };
                if entries.is_empty() || !self.fs.is_dir(dir) {
                    self.last_error = ERROR_FILE_NOT_FOUND;
                    return Some(INVALID_HANDLE_VALUE);
                }
                let (first, is_dir) = entries.remove(0);
                self.fill_find_data(dir, &first, is_dir, out);
                let handle = self.alloc(16);
                let rest = entries
                    .into_iter()
                    .rev()
                    .map(|(n, d)| (join(dir, &n), d))
                    .collect();
                self.finds.insert(handle, rest);
                handle
            }
            "FindNextFileW" => {
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
                self.fill_find_data(dir, name, is_dir, arg(1));
                1
            }
            "FindClose" => {
                if self.finds.remove(&arg(0)).is_none() {
                    return Some(self.fail(ERROR_INVALID_HANDLE));
                }
                self.allocations.remove(&arg(0));
                1
            }
            // Nothing a sandboxed program runs may start another process or share memory
            // with one (see `pipe` and `shm_open`).
            "CreatePipe" | "CreateProcessW" | "CreateProcessA" | "CreateFileMappingW"
            | "CreateFileMappingA" | "OpenFileMappingW" | "OpenFileMappingA" => {
                self.fail(ERROR_ACCESS_DENIED)
            }
            _ => return None,
        })
    }
}
