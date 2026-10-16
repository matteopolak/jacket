//! The sandbox host: program output captured in memory and a small libc for runs that have no
//! dynamic linker (the browser playground, `jaic -interp -os wasm`).
//!
//! Everything here is virtual and deterministic: a clock that advances by a microsecond per query
//! (and by the requested time on `nanosleep`), a file system that overlays in-memory writes on a
//! read-only [`FileSystem`] (the playground's workspace and bundled stdlib), a working directory,
//! `FILE*`/descriptor/`DIR*` handles, and the number parsing and `localtime` helpers the standard
//! library calls. Thread primitives are not here: they need the interpreter (`threads_inline.rs`).
#![allow(unsafe_code)]

use super::*;
use crate::sema::FileSystem;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

mod numeric;
mod objc;
mod stdin;
mod windows;

const EPERM: u64 = 1;
const ENOENT: u64 = 2;
const EBADF: u64 = 9;
const EEXIST: u64 = 17;
const ENOTDIR: u64 = 20;
const EISDIR: u64 = 21;
const EINVAL: u64 = 22;
const ENOTEMPTY: u64 = 39;
const O_ACCMODE: u64 = 3;
const O_WRONLY: u64 = 1;
const O_RDWR: u64 = 2;
const O_CREAT: u64 = 0o100;
const O_EXCL: u64 = 0o200;
const O_TRUNC: u64 = 0o1000;
const O_APPEND: u64 = 0o2000;
const S_IFDIR: u32 = 0o040000;
const S_IFREG: u32 = 0o100000;

/// The first descriptor handed out by `open` (0 to 2 are the standard streams).
const FIRST_FD: i64 = 3;

/// Files the program wrote, laid over a read-only base.
#[derive(Default)]
pub struct SandboxFs {
    base: Option<Rc<dyn FileSystem>>,
    /// Contents of every file that was created, written, or opened (base files are copied on open).
    files: BTreeMap<String, Vec<u8>>,
    dirs: BTreeSet<String>,
    removed: HashSet<String>,
    /// Inode numbers given out by `link`, which the linked names share; other files' numbers
    /// come from their paths.
    inodes: HashMap<String, u64>,
    /// Modification times: every change of a file advances a virtual clock by one second.
    mtimes: HashMap<String, u64>,
    clock: u64,
}

/// The modification time of a file the program has not changed.
const BASE_MTIME: u64 = 1_700_000_000;

impl SandboxFs {
    fn inode(&self, path: &str) -> u64 {
        if let Some(&inode) = self.inodes.get(path) {
            return inode;
        }
        // FNV-1a, kept clear of the numbers `link` hands out.
        let hash = path.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        });
        hash | 1 << 63
    }

    /// `path` was created, written or truncated: stamp it, and give its other names (hard
    /// links) the same contents.
    fn changed(&mut self, path: &str) {
        self.clock += 1;
        let stamp = BASE_MTIME + self.clock;
        let inode = self.inode(path);
        let peers: Vec<String> = self
            .inodes
            .iter()
            .filter(|(p, i)| **i == inode && p.as_str() != path)
            .map(|(p, _)| p.clone())
            .collect();
        if let Some(data) = self.files.get(path).cloned() {
            for peer in peers {
                self.mtimes.insert(peer.clone(), stamp);
                self.files.insert(peer, data.clone());
            }
        }
        self.mtimes.insert(path.to_string(), stamp);
    }

    /// `path` no longer names a file (its other names keep the contents).
    fn forget(&mut self, path: &str) {
        self.inodes.remove(path);
        self.mtimes.remove(path);
    }

    fn exists_file(&self, path: &str) -> bool {
        if self.files.contains_key(path) {
            return true;
        }
        !self.removed.contains(path)
            && self
                .base
                .as_ref()
                .is_some_and(|b| b.is_file(Path::new(path)))
    }

    fn is_dir(&self, path: &str) -> bool {
        if path == "/" || self.dirs.contains(path) {
            return true;
        }
        let prefix = format!("{}/", path.trim_end_matches('/'));
        if self.files.keys().any(|k| k.starts_with(&prefix)) {
            return true;
        }
        !self.removed.contains(path)
            && self
                .base
                .as_ref()
                .is_some_and(|b| b.is_dir(Path::new(path)))
    }

    /// Bring a base file into the overlay so it can be read and written in place.
    fn load(&mut self, path: &str) -> bool {
        if self.files.contains_key(path) {
            return true;
        }
        if self.removed.contains(path) {
            return false;
        }
        match self.base.as_ref().and_then(|b| b.read(Path::new(path))) {
            Some(bytes) => {
                self.files.insert(path.to_string(), bytes);
                true
            }
            None => false,
        }
    }

    /// The length of an existing file, written or not.
    fn file_size(&self, path: &str) -> usize {
        match self.files.get(path) {
            Some(data) => data.len(),
            None => self
                .base
                .as_ref()
                .and_then(|b| b.read(Path::new(path)))
                .map_or(0, |d| d.len()),
        }
    }

    fn list(&self, path: &str) -> Vec<(String, bool)> {
        let mut out: BTreeMap<String, bool> = BTreeMap::new();
        if let Some(base) = &self.base {
            for (name, dir) in base.list_dir(Path::new(path)) {
                if !self.removed.contains(&join(path, &name)) {
                    out.insert(name, dir);
                }
            }
        }
        let prefix = format!("{}/", path.trim_end_matches('/'));
        for key in &self.files {
            if let Some(rest) = key.0.strip_prefix(&prefix) {
                match rest.split_once('/') {
                    Some((dir, _)) => {
                        out.insert(dir.to_string(), true);
                    }
                    None => {
                        out.insert(rest.to_string(), false);
                    }
                }
            }
        }
        for dir in &self.dirs {
            if let Some(rest) = dir.strip_prefix(&prefix) {
                let first = rest.split('/').next().unwrap_or(rest);
                if !first.is_empty() {
                    out.insert(first.to_string(), true);
                }
            }
        }
        out.into_iter().collect()
    }
}

fn join(dir: &str, name: &str) -> String {
    format!("{}/{}", dir.trim_end_matches('/'), name)
}

/// Resolve `path` against `cwd` and drop `.`/`..` components.
fn absolute(cwd: &str, path: &str) -> String {
    let full = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("{cwd}/{path}")
    };
    let mut parts: Vec<&str> = Vec::new();
    for part in full.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    format!("/{}", parts.join("/"))
}

struct OpenFile {
    path: String,
    pos: usize,
    readable: bool,
    writable: bool,
    append: bool,
    eof: bool,
}

struct DirStream {
    entries: Vec<(String, bool)>,
    next: usize,
    /// Host address of the `dirent` returned by `readdir`.
    entry: u64,
}

/// Collects output in memory and implements a small libc for sandboxed runs.
#[derive(Default)]
pub struct SandboxHost {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    /// Write order across the two streams: `(to_stderr, bytes)` runs, adjacent runs merged.
    pub order: Vec<(bool, usize)>,
    allocations: HashMap<u64, (ZeroedBlock, usize)>,
    /// Virtual clock ticks (nanoseconds) handed out by `clock_gettime`; the sandbox has no real
    /// time source on wasm32, so every query advances this by one microsecond.
    clock_ns: u64,
    fs: SandboxFs,
    cwd: String,
    fds: HashMap<i64, OpenFile>,
    next_fd: i64,
    /// `FILE*` handle address -> descriptor.
    streams: HashMap<u64, i64>,
    dirs: HashMap<u64, DirStream>,
    errno_cell: u64,
    /// Windows' `GetLastError`.
    last_error: u64,
    /// `FindFirstFileW` handle -> the entries left to return.
    finds: HashMap<u64, Vec<(String, bool)>>,
    /// `strerror` results by error number; they stay valid for the whole run, like libc's.
    error_texts: HashMap<u64, u64>,
    stdin: stdin::Stdin,
    objc: objc::Objc,
}

impl SandboxHost {
    /// Free the blocks compile-time code allocated and never freed. Only once no code runs on
    /// this host again (`Interp::release_code` on every interpreter using it): pointers to
    /// them are left dangling in program memory.
    pub fn release_memory(&mut self) {
        self.allocations = HashMap::default();
    }

    /// Back the sandbox's file system with `base` (read-only) and start in `cwd`.
    pub fn with_files(base: Rc<dyn FileSystem>, cwd: &str) -> Self {
        let mut host = SandboxHost::default();
        host.fs.base = Some(base);
        host.cwd = cwd.to_string();
        // Programs expect a scratch directory.
        host.fs.dirs.insert("/tmp".to_string());
        host
    }

    /// A zeroed 16-byte aligned block, or null when the request cannot be met (like `malloc`;
    /// an infallible allocation would abort the whole compiler on `alloc(1 << 60)`).
    fn alloc(&mut self, size: usize) -> u64 {
        let Some(block) = ZeroedBlock::new(size, 16) else {
            return 0;
        };
        let addr = block.addr();
        self.allocations.insert(addr, (block, size));
        addr
    }

    fn set_errno(&mut self, code: u64) -> u64 {
        if self.errno_cell == 0 {
            self.errno_cell = self.alloc(8);
        }
        unsafe { std::ptr::write_unaligned(self.errno_cell as *mut u32, code as u32) };
        u64::MAX
    }

    fn path(&self, ptr: u64) -> String {
        absolute(
            if self.cwd.is_empty() {
                "/"
            } else {
                &self.cwd
            },
            &cstr(ptr),
        )
    }

    fn open_file(&mut self, path: String, flags: u64) -> Result<i64, u64> {
        let mode = flags & O_ACCMODE;
        let (readable, writable) = (mode != O_WRONLY, mode == O_WRONLY || mode == O_RDWR);
        if self.fs.is_dir(&path) {
            if writable {
                return Err(EISDIR);
            }
        } else if self.fs.exists_file(&path) {
            if flags & O_CREAT != 0 && flags & O_EXCL != 0 {
                return Err(EEXIST);
            }
            self.fs.load(&path);
            if flags & O_TRUNC != 0 && writable {
                self.fs.files.insert(path.clone(), Vec::new());
                self.fs.changed(&path);
            }
        } else if flags & O_CREAT != 0 {
            if !self.parent_exists(&path) {
                return Err(ENOENT);
            }
            self.fs.removed.remove(&path);
            self.fs.files.insert(path.clone(), Vec::new());
            self.fs.changed(&path);
        } else {
            return Err(ENOENT);
        }
        if self.next_fd < FIRST_FD {
            self.next_fd = FIRST_FD;
        }
        let fd = self.next_fd;
        self.next_fd += 1;
        self.fds.insert(
            fd,
            OpenFile {
                path,
                pos: 0,
                readable,
                writable,
                append: flags & O_APPEND != 0,
                eof: false,
            },
        );
        Ok(fd)
    }

    fn parent_exists(&self, path: &str) -> bool {
        let parent = path.rsplit_once('/').map_or("/", |(p, _)| p);
        parent.is_empty() || self.fs.is_dir(parent)
    }

    /// Read up to `count` bytes at the descriptor's position. `None`: bad descriptor.
    fn read_fd(&mut self, fd: i64, out: u64, count: usize) -> Option<usize> {
        if fd == 0 {
            return Some(0);
        }
        let file = self.fds.get_mut(&fd)?;
        let data = self.fs.files.get(&file.path)?;
        if !file.readable {
            return None;
        }
        let available = data.len().saturating_sub(file.pos);
        let n = available.min(count);
        if n > 0 {
            unsafe { std::ptr::copy_nonoverlapping(data[file.pos..].as_ptr(), out as *mut u8, n) };
        }
        file.pos += n;
        if n < count {
            file.eof = true;
        }
        Some(n)
    }

    fn write_fd(&mut self, fd: i64, bytes: &[u8]) -> Option<usize> {
        let file = self.fds.get_mut(&fd)?;
        if !file.writable {
            return None;
        }
        let data = self.fs.files.get_mut(&file.path)?;
        if file.append {
            file.pos = data.len();
        }
        if data.len() < file.pos {
            data.resize(file.pos, 0);
        }
        let end = file.pos + bytes.len();
        if data.len() < end {
            data.resize(end, 0);
        }
        data[file.pos..end].copy_from_slice(bytes);
        file.pos = end;
        let path = file.path.clone();
        self.fs.changed(&path);
        Some(bytes.len())
    }

    fn file_len(&self, fd: i64) -> Option<usize> {
        let file = self.fds.get(&fd)?;
        Some(self.fs.files.get(&file.path)?.len())
    }

    fn seek(&mut self, fd: i64, offset: i64, whence: u64) -> Option<i64> {
        let len = self.file_len(fd)? as i64;
        let file = self.fds.get_mut(&fd)?;
        let base = match whence {
            0 => 0,
            1 => file.pos as i64,
            2 => len,
            _ => return None,
        };
        let target = base + offset;
        if target < 0 {
            return None;
        }
        file.pos = target as usize;
        file.eof = false;
        Some(target)
    }

    fn fill_stat(&self, path: &str, out: u64) -> Result<(), u64> {
        let links = 1 + self
            .fs
            .inodes
            .iter()
            .filter(|(p, i)| **i == self.fs.inode(path) && p.as_str() != path)
            .count() as u64;
        let (mode, size) = if self.fs.is_dir(path) {
            (S_IFDIR | 0o755, 4096)
        } else if self.fs.exists_file(path) {
            (S_IFREG | 0o644, self.fs.file_size(path) as i64)
        } else {
            return Err(ENOENT);
        };
        // Linux x86-64 `struct stat`.
        unsafe {
            std::ptr::write_bytes(out as *mut u8, 0, 144);
            let put = |off: u64, v: u64, width: usize| {
                std::ptr::copy_nonoverlapping(
                    v.to_le_bytes().as_ptr(),
                    (out + off) as *mut u8,
                    width,
                )
            };
            put(0, 1, 8); // st_dev
            put(8, self.fs.inode(path), 8); // st_ino
            put(16, links, 8); // st_nlink
            put(24, mode as u64, 4);
            put(48, size as u64, 8);
            put(56, 4096, 8); // st_blksize
            put(64, (size as u64).div_ceil(512), 8); // st_blocks
            let mtime = self.fs.mtimes.get(path).copied().unwrap_or(BASE_MTIME);
            for off in [72, 88, 104] {
                put(off, mtime, 8);
            }
        }
        Ok(())
    }

    fn libc_result(&mut self, result: Result<u64, u64>) -> u64 {
        match result {
            Ok(v) => v,
            Err(code) => self.set_errno(code),
        }
    }
}

fn cstr(ptr: u64) -> String {
    if ptr == 0 {
        return String::new();
    }
    let mut bytes = Vec::new();
    let mut p = ptr;
    loop {
        let b = unsafe { *(p as *const u8) };
        if b == 0 {
            break;
        }
        bytes.push(b);
        p += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn write_u64(addr: u64, value: u64) {
    unsafe { std::ptr::write_unaligned(addr as *mut u64, value) }
}

fn write_i32(addr: u64, value: i32) {
    unsafe { std::ptr::write_unaligned(addr as *mut i32, value) }
}

/// Days since 1970-01-01 to (year, month 1-12, day).
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 {
        mp + 3
    } else {
        mp - 9
    };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// `gmtime_r`/`localtime_r` (the sandbox's zone is UTC): fill a Linux `struct tm`.
fn fill_tm(seconds: i64, out: u64) {
    let days = seconds.div_euclid(86_400);
    let rem = seconds.rem_euclid(86_400);
    let (year, month, day) = civil(days);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    const CUMULATIVE: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let yday = CUMULATIVE[(month - 1) as usize] + day - 1 + i64::from(leap && month > 2);
    let fields = [
        rem % 60,
        rem / 60 % 60,
        rem / 3600,
        day,
        month - 1,
        year - 1900,
        (days + 4).rem_euclid(7),
        yday,
        0,
    ];
    for (i, v) in fields.iter().enumerate() {
        write_i32(out + 4 * i as u64, *v as i32);
    }
    write_i32(out + 36, 0);
    write_u64(out + 40, 0); // tm_gmtoff
    write_u64(out + 48, 0); // tm_zone
}

/// `strtol` family: returns (value, bytes consumed).
fn parse_int_prefix(text: &str, base: u32, signed: bool) -> (i128, usize) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let mut negative = false;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        negative = bytes[i] == b'-';
        i += 1;
    }
    let mut base = base;
    let has_hex_prefix =
        |at: usize| at + 1 < bytes.len() && bytes[at] == b'0' && (bytes[at + 1] | 0x20) == b'x';
    if (base == 0 || base == 16)
        && has_hex_prefix(i)
        && bytes.get(i + 2).is_some_and(|b| b.is_ascii_hexdigit())
    {
        i += 2;
        base = 16;
    } else if base == 0 {
        base = if bytes.get(i) == Some(&b'0') {
            8
        } else {
            10
        };
    }
    let first = i;
    let mut value: i128 = 0;
    while i < bytes.len() {
        let Some(digit) = (bytes[i] as char).to_digit(base) else {
            break;
        };
        value = value
            .saturating_mul(base as i128)
            .saturating_add(digit as i128);
        i += 1;
    }
    if i == first {
        return (0, 0);
    }
    let value = if negative {
        -value
    } else {
        value
    };
    let _ = signed;
    (value, i)
}

impl Host for SandboxHost {
    fn write(&mut self, bytes: &[u8], to_stderr: bool) {
        if bytes.is_empty() {
            return;
        }
        if to_stderr {
            self.stderr.extend_from_slice(bytes);
        } else {
            self.stdout.extend_from_slice(bytes);
        }
        match self.order.last_mut() {
            Some((stream, len)) if *stream == to_stderr => *len += bytes.len(),
            _ => self.order.push((to_stderr, bytes.len())),
        }
    }

    fn foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
        _sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>> {
        if let Some(result) = self.stdin_foreign(symbol, args) {
            return Some(Ok(vec![result]));
        }
        if let Some(result) = self.windows_foreign(symbol, args) {
            return Some(Ok(vec![result]));
        }
        if let Some(result) = self.objc_foreign(symbol, args) {
            return Some(Ok(vec![result]));
        }
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        Some(Ok(vec![match symbol {
            "write" => {
                let (fd, ptr, len) = (arg(0) as i32 as i64, arg(1), arg(2) as usize);
                let bytes = if len == 0 {
                    Vec::new()
                } else {
                    unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }.to_vec()
                };
                if fd == 1 || fd == 2 {
                    self.write(&bytes, fd == 2);
                    len as u64
                } else {
                    match self.write_fd(fd, &bytes) {
                        Some(n) => n as u64,
                        None => self.set_errno(EBADF),
                    }
                }
            }
            "wasm_write_string" => {
                let (len, ptr, to_stderr) = (arg(0) as usize, arg(1), arg(2) & 1 != 0);
                let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }.to_vec();
                self.write(&bytes, to_stderr);
                0
            }
            "malloc" => self.alloc(arg(0) as usize),
            "calloc" => (arg(0) as usize)
                .checked_mul(arg(1) as usize)
                .map_or(0, |size| self.alloc(size)),
            "realloc" => {
                let (old, size) = (arg(0), arg(1) as usize);
                let new = self.alloc(size);
                if new == 0 {
                    // The old block stays valid when growing fails.
                    return Some(Ok(vec![0]));
                }
                if let Some((_, old_size)) = self.allocations.get(&old) {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            old as *const u8,
                            new as *mut u8,
                            (*old_size).min(size),
                        )
                    };
                    self.allocations.remove(&old);
                }
                new
            }
            "free" => {
                self.allocations.remove(&arg(0));
                0
            }
            "memcpy" | "memmove" => {
                unsafe { std::ptr::copy(arg(1) as *const u8, arg(0) as *mut u8, arg(2) as usize) };
                arg(0)
            }
            "memset" => {
                unsafe { std::ptr::write_bytes(arg(0) as *mut u8, arg(1) as u8, arg(2) as usize) };
                arg(0)
            }
            "memcmp" => {
                let (a, b) = unsafe {
                    (
                        std::slice::from_raw_parts(arg(0) as *const u8, arg(2) as usize),
                        std::slice::from_raw_parts(arg(1) as *const u8, arg(2) as usize),
                    )
                };
                match a.cmp(b) {
                    std::cmp::Ordering::Less => -1i64 as u64,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }
            }
            "strlen" => {
                let mut n = 0;
                while unsafe { *((arg(0) + n) as *const u8) } != 0 {
                    n += 1;
                }
                n
            }
            "strcmp" | "strncmp" => {
                let limit = if symbol == "strncmp" {
                    arg(2) as usize
                } else {
                    usize::MAX
                };
                let (mut a, mut b) = (arg(0), arg(1));
                let mut seen = 0;
                loop {
                    if seen == limit {
                        break 0;
                    }
                    let (x, y) = unsafe { (*(a as *const u8), *(b as *const u8)) };
                    if x != y {
                        break (x as i64 - y as i64) as u64;
                    }
                    if x == 0 {
                        break 0;
                    }
                    a += 1;
                    b += 1;
                    seen += 1;
                }
            }
            "exit" | "_exit" => return Some(Err(format!("exit({})", arg(0) as i32))),
            "abort" => return Some(Err("abort()".into())),
            // Threads are scheduled by the interpreter; these only run when no thread exists.
            "pthread_mutexattr_init"
            | "pthread_mutexattr_destroy"
            | "pthread_mutexattr_settype"
            | "pthread_attr_init"
            | "pthread_attr_destroy"
            | "pthread_attr_setstacksize"
            | "pthread_mutex_lock"
            | "pthread_mutex_unlock"
            | "pthread_mutex_init"
            | "pthread_mutex_destroy" => 0,
            "isatty" => 0,
            "getenv" => 0,
            // Whether the embedding page offers a host procedure (stdlib/Extensions/WebGPU): not here. The
            // browser playground's host (jai-wasm `host_bridge`) asks its page instead.
            "jai_host_provides" => 0,
            "strerror" => {
                let code = arg(0) as u32 as u64;
                if let Some(&text) = self.error_texts.get(&code) {
                    return Some(Ok(vec![text]));
                }
                let message = match code {
                    0 => "Success".to_string(),
                    ENOENT => "No such file or directory".to_string(),
                    EBADF => "Bad file descriptor".to_string(),
                    EEXIST => "File exists".to_string(),
                    ENOTDIR => "Not a directory".to_string(),
                    EISDIR => "Is a directory".to_string(),
                    EINVAL => "Invalid argument".to_string(),
                    ENOTEMPTY => "Directory not empty".to_string(),
                    other => format!("Unknown error {other}"),
                };
                let text = self.alloc(message.len() + 1);
                if text != 0 {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            message.as_ptr(),
                            text as *mut u8,
                            message.len(),
                        )
                    };
                    self.error_texts.insert(code, text);
                }
                text
            }
            "wasm_debug_break" => return Some(Err("debug_break() was called".into())),
            "nanosleep" => {
                // struct timespec { tv_sec; tv_nsec }: sleeping only moves the virtual clock.
                let req = arg(0);
                if req != 0 {
                    let secs = unsafe { std::ptr::read_unaligned(req as *const u64) };
                    let nanos = unsafe { std::ptr::read_unaligned((req + 8) as *const u64) };
                    self.clock_ns = self
                        .clock_ns
                        .saturating_add(secs.saturating_mul(1_000_000_000))
                        .saturating_add(nanos);
                }
                0
            }
            "usleep" => {
                self.clock_ns += (arg(0) as u32 as u64) * 1_000;
                0
            }
            "sleep" => {
                self.clock_ns += (arg(0) as u32 as u64) * 1_000_000_000;
                0
            }
            "sched_yield" => 0,
            // One processor, 4 KiB pages: the browser sandbox is a single cooperative thread.
            "sysconf" => match arg(0) as i32 {
                83 | 84 => 1,
                30 => 4096,
                _ => self.set_errno(EINVAL),
            },
            "getpid" => 1,
            "clock_gettime" => {
                // struct timespec { tv_sec: s64; tv_nsec: s64 }; a fixed epoch plus the virtual
                // clock keeps runs deterministic.
                self.clock_ns += 1_000;
                let ns = self.clock_ns + 1_700_000_000u64 * 1_000_000_000;
                let out = arg(1);
                if out == 0 {
                    return Some(Ok(vec![u64::MAX]));
                }
                write_u64(out, ns / 1_000_000_000);
                write_u64(out + 8, ns % 1_000_000_000);
                0
            }
            "gettimeofday" => {
                self.clock_ns += 1_000;
                let ns = self.clock_ns + 1_700_000_000u64 * 1_000_000_000;
                if arg(0) != 0 {
                    write_u64(arg(0), ns / 1_000_000_000);
                    write_u64(arg(0) + 8, ns % 1_000_000_000 / 1_000);
                }
                0
            }
            "time" => {
                self.clock_ns += 1_000;
                let secs = 1_700_000_000u64 + self.clock_ns / 1_000_000_000;
                if arg(0) != 0 {
                    write_u64(arg(0), secs);
                }
                secs
            }
            "gmtime_r" | "localtime_r" => {
                let seconds = unsafe { std::ptr::read_unaligned(arg(0) as *const i64) };
                fill_tm(seconds, arg(1));
                arg(1)
            }
            "atof" | "strtod" | "strtof" => {
                let text = cstr(arg(0));
                let parsed = numeric::parse(&text, symbol == "strtof");
                if symbol != "atof" && arg(1) != 0 {
                    write_u64(arg(1), arg(0) + parsed.used as u64);
                }
                if parsed.range_error {
                    self.set_errno(34);
                }
                parsed.bits
            }
            "setlocale" => {
                if arg(0) > 6 {
                    0
                } else if arg(1) == 0 || matches!(cstr(arg(1)).as_str(), "" | "C" | "POSIX") {
                    c"C".as_ptr() as u64
                } else {
                    0
                }
            }
            "atoi" | "atol" | "atoll" | "strtol" | "strtoll" | "strtoul" | "strtoull" => {
                let text = cstr(arg(0));
                let base = if symbol.starts_with("ato") {
                    10
                } else {
                    arg(2) as u32
                };
                let (value, used) = parse_int_prefix(&text, base, true);
                if !symbol.starts_with("ato") && arg(1) != 0 {
                    write_u64(arg(1), arg(0) + used as u64);
                }
                value as i64 as u64
            }
            "__errno_location" | "__error" => {
                // The cell keeps the last error: reading it must not clear it.
                if self.errno_cell == 0 {
                    self.errno_cell = self.alloc(8);
                }
                self.errno_cell
            }
            "getcwd" => {
                let cwd = if self.cwd.is_empty() {
                    "/".to_string()
                } else {
                    self.cwd.clone()
                };
                if (cwd.len() as u64) + 1 > arg(1) {
                    self.set_errno(34); // ERANGE
                    0
                } else {
                    unsafe {
                        std::ptr::copy_nonoverlapping(cwd.as_ptr(), arg(0) as *mut u8, cwd.len());
                        *((arg(0) + cwd.len() as u64) as *mut u8) = 0;
                    }
                    arg(0)
                }
            }
            "chdir" => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) {
                    self.cwd = path;
                    0
                } else {
                    self.set_errno(ENOENT)
                }
            }
            "realpath" | "realpath$DARWIN_EXTSN" => {
                let path = self.path(arg(0));
                if !self.fs.is_dir(&path) && !self.fs.exists_file(&path) {
                    self.set_errno(ENOENT);
                    0
                } else {
                    let out = if arg(1) != 0 {
                        arg(1)
                    } else {
                        self.alloc(path.len() + 1)
                    };
                    unsafe {
                        std::ptr::copy_nonoverlapping(path.as_ptr(), out as *mut u8, path.len());
                        *((out + path.len() as u64) as *mut u8) = 0;
                    }
                    out
                }
            }
            "access" => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) || self.fs.exists_file(&path) {
                    0
                } else {
                    self.set_errno(ENOENT)
                }
            }
            // The virtual file system keeps no permission bits (every file is readable and
            // writable), so changing them succeeds on any path that exists.
            "chmod" => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) || self.fs.exists_file(&path) {
                    0
                } else {
                    self.set_errno(ENOENT)
                }
            }
            "fchmod" => match self.fds.contains_key(&(arg(0) as i32 as i64)) {
                true => 0,
                false => self.set_errno(EBADF),
            },
            "stat" | "lstat" => {
                let path = self.path(arg(0));
                let result = self.fill_stat(&path, arg(1)).map(|_| 0);
                self.libc_result(result)
            }
            "fstat" => {
                let fd = arg(0) as i32 as i64;
                match self.fds.get(&fd).map(|f| f.path.clone()) {
                    Some(path) => {
                        let result = self.fill_stat(&path, arg(1)).map(|_| 0);
                        self.libc_result(result)
                    }
                    None => self.set_errno(EBADF),
                }
            }
            "open" | "open64" => {
                let path = self.path(arg(0));
                let result = self
                    .open_file(path, arg(1) as u32 as u64)
                    .map(|fd| fd as u64);
                self.libc_result(result)
            }
            "close" => {
                if self.fds.remove(&(arg(0) as i32 as i64)).is_some() {
                    0
                } else {
                    self.set_errno(EBADF)
                }
            }
            "read" => match self.read_fd(arg(0) as i32 as i64, arg(1), arg(2) as usize) {
                Some(n) => n as u64,
                None => self.set_errno(EBADF),
            },
            "lseek" | "lseek64" => match self.seek(arg(0) as i32 as i64, arg(1) as i64, arg(2)) {
                Some(p) => p as u64,
                None => self.set_errno(EINVAL),
            },
            "fopen" | "fopen64" => {
                let path = self.path(arg(0));
                let mode = cstr(arg(1));
                let plus = mode.contains('+');
                let flags = match mode.chars().next() {
                    Some('w') => {
                        O_CREAT
                            | O_TRUNC
                            | if plus {
                                O_RDWR
                            } else {
                                O_WRONLY
                            }
                    }
                    Some('a') => {
                        O_CREAT
                            | O_APPEND
                            | if plus {
                                O_RDWR
                            } else {
                                O_WRONLY
                            }
                    }
                    _ => {
                        if plus {
                            O_RDWR
                        } else {
                            0
                        }
                    }
                };
                match self.open_file(path, flags) {
                    Ok(fd) => {
                        let handle = self.alloc(256);
                        self.streams.insert(handle, fd);
                        handle
                    }
                    Err(code) => {
                        self.set_errno(code);
                        0
                    }
                }
            }
            "fclose" => match self.streams.remove(&arg(0)) {
                Some(fd) => {
                    self.fds.remove(&fd);
                    self.allocations.remove(&arg(0));
                    0
                }
                None => self.set_errno(EBADF),
            },
            "fflush" => 0,
            "fread" => {
                let (size, count) = (arg(1) as usize, arg(2) as usize);
                match self.streams.get(&arg(3)).copied() {
                    Some(fd) if size > 0 => {
                        match self.read_fd(fd, arg(0), size.saturating_mul(count)) {
                            Some(n) => (n / size) as u64,
                            None => 0,
                        }
                    }
                    _ => 0,
                }
            }
            "fwrite" => {
                let (size, count) = (arg(1) as usize, arg(2) as usize);
                let total = size.saturating_mul(count);
                match self.streams.get(&arg(3)).copied() {
                    Some(fd) if total > 0 => {
                        let bytes =
                            unsafe { std::slice::from_raw_parts(arg(0) as *const u8, total) }
                                .to_vec();
                        match self.write_fd(fd, &bytes) {
                            Some(n) => (n / size) as u64,
                            None => 0,
                        }
                    }
                    _ => 0,
                }
            }
            "fseek" | "fseeko" | "fseeko64" => match self.streams.get(&arg(0)).copied() {
                Some(fd) => match self.seek(fd, arg(1) as i64, arg(2) as u32 as u64) {
                    Some(_) => 0,
                    None => self.set_errno(EINVAL),
                },
                None => self.set_errno(EBADF),
            },
            "ftell" | "ftello" | "ftello64" => match self.streams.get(&arg(0)).copied() {
                Some(fd) => self.fds.get(&fd).map_or(u64::MAX, |f| f.pos as u64),
                None => self.set_errno(EBADF),
            },
            "rewind" => {
                if let Some(fd) = self.streams.get(&arg(0)).copied() {
                    self.seek(fd, 0, 0);
                }
                0
            }
            "feof" => match self.streams.get(&arg(0)).copied() {
                Some(fd) => u64::from(self.fds.get(&fd).is_some_and(|f| f.eof)),
                None => 0,
            },
            "ferror" => 0,
            "fileno" => self.streams.get(&arg(0)).map_or(u64::MAX, |fd| *fd as u64),
            "unlink" | "remove" => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) && symbol == "remove" {
                    return self.rmdir(&path);
                }
                if self.fs.exists_file(&path) {
                    self.fs.files.remove(&path);
                    self.fs.forget(&path);
                    self.fs.removed.insert(path);
                    0
                } else {
                    self.set_errno(ENOENT)
                }
            }
            "rename" => {
                let (from, to) = (self.path(arg(0)), self.path(arg(1)));
                let result = self.rename_file(&from, &to).map(|_| 0);
                self.libc_result(result)
            }
            "link" => {
                let (from, to) = (self.path(arg(0)), self.path(arg(1)));
                if !self.fs.exists_file(&from) {
                    self.set_errno(ENOENT)
                } else if self.fs.exists_file(&to) {
                    self.set_errno(EEXIST)
                } else {
                    // A second name for the same file: writes through either show in both.
                    self.fs.load(&from);
                    let data = self.fs.files.get(&from).cloned().unwrap_or_default();
                    let inode = self.fs.inode(&from);
                    self.fs.inodes.insert(from.clone(), inode);
                    self.fs.inodes.insert(to.clone(), inode);
                    let stamp = self.fs.mtimes.get(&from).copied();
                    if let Some(stamp) = stamp {
                        self.fs.mtimes.insert(to.clone(), stamp);
                    }
                    self.fs.removed.remove(&to);
                    self.fs.files.insert(to, data);
                    0
                }
            }
            "mkdir" => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) || self.fs.exists_file(&path) {
                    self.set_errno(EEXIST)
                } else if !self.parent_exists(&path) {
                    self.set_errno(ENOENT)
                } else {
                    self.fs.removed.remove(&path);
                    self.fs.dirs.insert(path);
                    0
                }
            }
            "rmdir" => {
                let path = self.path(arg(0));
                return self.rmdir(&path);
            }
            "opendir" => {
                let path = self.path(arg(0));
                if !self.fs.is_dir(&path) {
                    self.set_errno(if self.fs.exists_file(&path) {
                        ENOTDIR
                    } else {
                        ENOENT
                    });
                    0
                } else {
                    let mut entries = vec![(".".to_string(), true), ("..".to_string(), true)];
                    entries.extend(self.fs.list(&path));
                    let handle = self.alloc(16);
                    let entry = self.alloc(280);
                    self.dirs.insert(
                        handle,
                        DirStream {
                            entries,
                            next: 0,
                            entry,
                        },
                    );
                    handle
                }
            }
            "readdir" | "readdir64" => match self.dirs.get_mut(&arg(0)) {
                Some(stream) if stream.next < stream.entries.len() => {
                    let (name, dir) = stream.entries[stream.next].clone();
                    stream.next += 1;
                    let entry = stream.entry;
                    unsafe {
                        std::ptr::write_bytes(entry as *mut u8, 0, 280);
                        write_u64(entry, stream.next as u64); // d_ino
                        write_u64(entry + 8, stream.next as u64); // d_off
                        std::ptr::write_unaligned((entry + 16) as *mut u16, 280);
                        *((entry + 18) as *mut u8) = if dir {
                            4
                        } else {
                            8
                        };
                        let n = name.len().min(255);
                        std::ptr::copy_nonoverlapping(name.as_ptr(), (entry + 19) as *mut u8, n);
                    }
                    entry
                }
                _ => 0,
            },
            "closedir" => match self.dirs.remove(&arg(0)) {
                Some(stream) => {
                    self.allocations.remove(&stream.entry);
                    self.allocations.remove(&arg(0));
                    0
                }
                None => self.set_errno(EBADF),
            },
            // Nothing a sandboxed program runs may start another process. It fails as a
            // refused call does, so code with a fallback (a build stamping its version from
            // `git`) carries on, and code that called it holding a lock lets go of it.
            "pipe" | "pipe2" | "socketpair" | "fork" | "vfork" | "execv" | "execve" | "execvp" => {
                self.set_errno(EPERM)
            }
            "posix_spawn" | "posix_spawnp" => EPERM,
            // Nor share memory with one (the memory debugger's visualizer channel).
            "shm_open" | "shm_unlink" => self.set_errno(EPERM),
            _ => return None,
        }]))
    }

    fn foreign_data(&mut self, symbol: &str) -> Option<u64> {
        self.stdin_variable(symbol)
    }

    fn native_linking(&self) -> bool {
        false
    }

    fn cooperative_threads(&self) -> bool {
        true
    }

    fn advance_clock(&mut self, nanoseconds: u64) {
        self.clock_ns = self.clock_ns.saturating_add(nanoseconds);
    }

    fn virtual_now_ns(&mut self) -> Option<u64> {
        Some(self.clock_ns + 1_700_000_000u64 * 1_000_000_000)
    }

    fn working_directory(&self) -> Option<String> {
        (!self.cwd.is_empty()).then(|| self.cwd.clone())
    }
}

impl SandboxHost {
    fn rmdir(&mut self, path: &str) -> Option<Result<Vec<u64>, String>> {
        let result = self.remove_dir(path).map(|_| 0);
        Some(Ok(vec![self.libc_result(result)]))
    }

    /// Remove an empty directory; the error is an errno.
    fn remove_dir(&mut self, path: &str) -> Result<(), u64> {
        if !self.fs.is_dir(path) {
            return Err(if self.fs.exists_file(path) {
                ENOTDIR
            } else {
                ENOENT
            });
        }
        if !self.fs.list(path).is_empty() {
            return Err(ENOTEMPTY);
        }
        self.fs.dirs.remove(path);
        self.fs.removed.insert(path.to_string());
        Ok(())
    }

    /// Move a file to `to`, replacing what is there; the error is an errno.
    fn rename_file(&mut self, from: &str, to: &str) -> Result<(), u64> {
        if self.fs.is_dir(from) {
            return Err(EINVAL);
        }
        if !self.fs.exists_file(from) {
            return Err(ENOENT);
        }
        self.fs.load(from);
        let data = self.fs.files.remove(from).unwrap_or_default();
        // The file keeps its identity and time under the new name.
        let inode = self.fs.inode(from);
        let stamp = self.fs.mtimes.get(from).copied();
        self.fs.forget(from);
        self.fs.forget(to);
        self.fs.inodes.insert(to.to_string(), inode);
        if let Some(stamp) = stamp {
            self.fs.mtimes.insert(to.to_string(), stamp);
        }
        self.fs.removed.insert(from.to_string());
        self.fs.removed.remove(to);
        self.fs.files.insert(to.to_string(), data);
        Ok(())
    }
}

/// Shares a [`SandboxHost`] with the embedder so output can be read after the run (and so
/// metaprogram workspaces write to the same buffers).
pub struct SharedHost(pub Rc<RefCell<SandboxHost>>);

impl Host for SharedHost {
    fn write(&mut self, bytes: &[u8], to_stderr: bool) {
        self.0.borrow_mut().write(bytes, to_stderr);
    }

    fn foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>> {
        self.0.borrow_mut().foreign(symbol, args, sig)
    }

    fn foreign_data(&mut self, symbol: &str) -> Option<u64> {
        self.0.borrow_mut().foreign_data(symbol)
    }

    fn native_linking(&self) -> bool {
        false
    }

    fn cooperative_threads(&self) -> bool {
        true
    }

    fn advance_clock(&mut self, nanoseconds: u64) {
        self.0.borrow_mut().advance_clock(nanoseconds);
    }

    fn virtual_now_ns(&mut self) -> Option<u64> {
        self.0.borrow_mut().virtual_now_ns()
    }

    fn working_directory(&self) -> Option<String> {
        self.0.borrow().working_directory()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig() -> ir::Sig {
        ir::Sig {
            params: vec![],
            returns: vec![],
            conv: ir::Conv::C,
            c_varargs: false,
            c_fixed: 0,
            c_abi: None,
        }
    }

    fn host() -> SandboxHost {
        let mut host = SandboxHost {
            cwd: "/tmp".to_string(),
            ..Default::default()
        };
        host.fs.dirs.insert("/tmp".to_string());
        host
    }

    /// A zero-terminated UTF-16 string.
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain([0]).collect()
    }

    fn call(host: &mut SandboxHost, symbol: &str, args: &[u64]) -> u64 {
        host.foreign(symbol, args, &sig())
            .unwrap_or_else(|| panic!("{symbol} is not provided"))
            .unwrap()[0]
    }

    fn errno(host: &SandboxHost) -> u32 {
        unsafe { std::ptr::read_unaligned(host.errno_cell as *const u32) }
    }

    #[test]
    fn process_creation_and_shared_memory_are_refused() {
        let mut h = host();
        for symbol in [
            "fork",
            "vfork",
            "pipe",
            "pipe2",
            "socketpair",
            "execv",
            "execve",
            "execvp",
            "shm_open",
            "shm_unlink",
        ] {
            assert_eq!(call(&mut h, symbol, &[0, 0, 0]), u64::MAX, "{symbol}");
            assert_eq!(errno(&h), EPERM as u32, "{symbol}");
        }
        // posix_spawn returns the error number itself.
        assert_eq!(call(&mut h, "posix_spawn", &[0, 0, 0, 0, 0, 0]), EPERM);
        assert_eq!(call(&mut h, "posix_spawnp", &[0, 0, 0, 0, 0, 0]), EPERM);
    }

    #[test]
    fn darwin_error_and_realpath_names_are_served() {
        let mut h = host();
        let cell = call(&mut h, "__error", &[]);
        assert_ne!(cell, 0);
        assert_eq!(cell, call(&mut h, "__errno_location", &[]));
    }

    #[test]
    fn c_numeric_strtof_rounds_the_decimal_directly_to_binary32() {
        let mut h = host();
        let text = b"1.0000000596046447753906250000000000000001!\0";
        let mut end = 0u64;
        let bits = call(
            &mut h,
            "strtof",
            &[text.as_ptr() as u64, (&mut end as *mut u64) as u64],
        );
        assert_eq!(bits, 0x3f800001);
        assert_eq!(end, text.as_ptr() as u64 + text.len() as u64 - 2);
    }

    #[test]
    fn c_numeric_hexadecimal_and_nan_payloads_preserve_endptr() {
        let mut h = host();
        for (text, expected, consumed) in [
            (b"0x1.8p+2suffix\0".as_slice(), 6.0f64.to_bits(), 8),
            (b"nan(payload)tail\0".as_slice(), f64::NAN.to_bits(), 12),
        ] {
            let mut end = 0u64;
            assert_eq!(
                call(
                    &mut h,
                    "strtod",
                    &[text.as_ptr() as u64, (&mut end as *mut u64) as u64]
                ),
                expected
            );
            assert_eq!(end, text.as_ptr() as u64 + consumed);
        }
    }

    #[test]
    fn c_numeric_range_errors_set_errno_and_success_does_not_clear_it() {
        let mut h = host();
        call(&mut h, "__errno_location", &[]);
        let text = b"1e9999\0";
        assert_eq!(
            call(&mut h, "strtod", &[text.as_ptr() as u64, 0]),
            f64::INFINITY.to_bits()
        );
        assert_eq!(errno(&h), 34);
        let text = b"7\0";
        assert_eq!(
            call(&mut h, "strtod", &[text.as_ptr() as u64, 0]),
            7.0f64.to_bits()
        );
        assert_eq!(errno(&h), 34);
    }

    #[test]
    fn c_numeric_locale_queries_and_unsupported_locales_stay_in_the_sandbox() {
        let mut h = host();
        assert_ne!(call(&mut h, "setlocale", &[6, 0]), 0);
        let c = b"C\0";
        assert_ne!(call(&mut h, "setlocale", &[6, c.as_ptr() as u64]), 0);
        let other = b"de_DE.UTF-8\0";
        assert_eq!(call(&mut h, "setlocale", &[6, other.as_ptr() as u64]), 0);
        let text = b"1,5\0";
        let mut end = 0u64;
        assert_eq!(
            call(
                &mut h,
                "strtod",
                &[text.as_ptr() as u64, (&mut end as *mut u64) as u64]
            ),
            1.0f64.to_bits()
        );
        assert_eq!(end, text.as_ptr() as u64 + 1);
    }

    #[test]
    fn windows_file_round_trip() {
        let mut h = host();
        let name = wide("data.txt");
        let (create_always, open_existing) = (2, 3);
        let handle = call(
            &mut h,
            "CreateFileW",
            &[
                name.as_ptr() as u64,
                0x4000_0000,
                0,
                0,
                create_always,
                0x80,
                0,
            ],
        );
        assert_ne!(handle, u64::MAX);
        let mut done = 0u32;
        let payload = b"hello windows";
        let ok = call(
            &mut h,
            "WriteFile",
            &[
                handle,
                payload.as_ptr() as u64,
                payload.len() as u64,
                &mut done as *mut u32 as u64,
                0,
            ],
        );
        assert_eq!((ok, done as usize), (1, payload.len()));
        assert_eq!(call(&mut h, "CloseHandle", &[handle]), 1);

        let handle = call(
            &mut h,
            "CreateFileW",
            &[
                name.as_ptr() as u64,
                0x8000_0000,
                0,
                0,
                open_existing,
                0x80,
                0,
            ],
        );
        assert_ne!(handle, u64::MAX);
        let mut buf = [0u8; 32];
        let ok = call(
            &mut h,
            "ReadFile",
            &[
                handle,
                buf.as_mut_ptr() as u64,
                buf.len() as u64,
                &mut done as *mut u32 as u64,
                0,
            ],
        );
        assert_eq!((ok, done as usize), (1, payload.len()));
        assert_eq!(&buf[..payload.len()], payload);
        let mut size = 0u64;
        assert_eq!(
            call(
                &mut h,
                "GetFileSizeEx",
                &[handle, &mut size as *mut u64 as u64]
            ),
            1
        );
        assert_eq!(size as usize, payload.len());
        assert_eq!(call(&mut h, "CloseHandle", &[handle]), 1);

        // A missing file: INVALID_HANDLE_VALUE and ERROR_FILE_NOT_FOUND.
        let missing = wide("nope.txt");
        let bad = call(
            &mut h,
            "CreateFileW",
            &[
                missing.as_ptr() as u64,
                0x8000_0000,
                0,
                0,
                open_existing,
                0x80,
                0,
            ],
        );
        assert_eq!(bad, u64::MAX);
        assert_eq!(call(&mut h, "GetLastError", &[]), 2);
    }

    #[test]
    fn windows_find_files_lists_a_directory() {
        let mut h = host();
        h.fs.dirs.insert("/tmp/d".to_string());
        h.fs.files
            .insert("/tmp/d/a.txt".to_string(), b"abc".to_vec());
        h.fs.files.insert("/tmp/d/b.txt".to_string(), Vec::new());
        let pattern = wide("d\\*");
        let mut data = vec![0u8; 592];
        let find = call(
            &mut h,
            "FindFirstFileW",
            &[pattern.as_ptr() as u64, data.as_mut_ptr() as u64],
        );
        assert_ne!(find, u64::MAX);
        let name_of = |data: &[u8]| {
            let units: Vec<u16> = data[44..44 + 520]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .take_while(|&u| u != 0)
                .collect();
            String::from_utf16(&units).unwrap()
        };
        let mut names = vec![name_of(&data)];
        loop {
            let ok = call(&mut h, "FindNextFileW", &[find, data.as_mut_ptr() as u64]);
            if ok == 0 {
                assert_eq!(call(&mut h, "GetLastError", &[]), 18);
                break;
            }
            names.push(name_of(&data));
        }
        names.sort();
        assert_eq!(names, [".", "..", "a.txt", "b.txt"]);
        assert_eq!(call(&mut h, "FindClose", &[find]), 1);
        // A directory with nothing matching.
        let none = wide("d\\*.zzz");
        let bad = call(
            &mut h,
            "FindFirstFileW",
            &[none.as_ptr() as u64, data.as_mut_ptr() as u64],
        );
        assert_eq!(bad, u64::MAX);
    }

    #[test]
    fn windows_text_conversion_and_current_directory() {
        let mut h = host();
        let text = "h\u{e9}llo \u{4e16}";
        let mut utf16 = [0u16; 16];
        let n = call(
            &mut h,
            "MultiByteToWideChar",
            &[
                65001,
                0,
                text.as_ptr() as u64,
                text.len() as u64,
                utf16.as_mut_ptr() as u64,
                16,
            ],
        );
        let expected: Vec<u16> = text.encode_utf16().collect();
        assert_eq!(n as usize, expected.len());
        assert_eq!(&utf16[..expected.len()], &expected[..]);
        // A room of 0 asks for the count only.
        assert_eq!(
            call(
                &mut h,
                "MultiByteToWideChar",
                &[65001, 0, text.as_ptr() as u64, text.len() as u64, 0, 0],
            ),
            n
        );
        let mut back = [0u8; 32];
        let m = call(
            &mut h,
            "WideCharToMultiByte",
            &[
                65001,
                0,
                utf16.as_ptr() as u64,
                n,
                back.as_mut_ptr() as u64,
                32,
                0,
                0,
            ],
        );
        assert_eq!(&back[..m as usize], text.as_bytes());

        let mut cwd = [0u16; 16];
        let len = call(
            &mut h,
            "GetCurrentDirectoryW",
            &[16, cwd.as_mut_ptr() as u64],
        );
        assert_eq!(String::from_utf16(&cwd[..len as usize]).unwrap(), "/tmp");
        // Too small: the size needed, terminator included.
        assert_eq!(
            call(
                &mut h,
                "GetCurrentDirectoryW",
                &[2, cwd.as_mut_ptr() as u64]
            ),
            5
        );
    }

    #[test]
    fn windows_process_creation_is_access_denied() {
        let mut h = host();
        for symbol in [
            "CreateProcessW",
            "CreateProcessA",
            "CreatePipe",
            "CreateFileMappingW",
        ] {
            h.last_error = 0;
            assert_eq!(call(&mut h, symbol, &[0; 10]), 0, "{symbol}");
            assert_eq!(call(&mut h, "GetLastError", &[]), 5, "{symbol}");
        }
    }
}
