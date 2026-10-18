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
use crate::fxhash::HashSet;
use crate::interp::word_or_zero;
use crate::sema::FileSystem;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

mod dirs;
mod files;
mod guest;
mod libc_fn;
mod memory;
mod numeric;
mod objc;
mod process;
mod stdin;
mod streams;
mod time;
mod windows;

use guest::{
    copy, fill, read_bytes, read_i64, read_u8, read_u64, write_bytes, write_i32, write_u8,
    write_u16, write_u32, write_u64, write_units,
};
use libc_fn::LibcFn;

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
const S_IFDIR: u32 = 0o040_000;
const S_IFREG: u32 = 0o100_000;

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
        write_u32(self.errno_cell, code as u32);
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
            write_bytes(out, &data[file.pos..file.pos + n]);
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
        fill(out, 0, 144);
        let put = |off: u64, v: u64, width: usize| {
            write_bytes(out + off, &v.to_le_bytes()[..width]);
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
        let b = read_u8(p);
        if b == 0 {
            break;
        }
        bytes.push(b);
        p += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
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
        if let Some(result) = self.windows_foreign(symbol, args) {
            return Some(Ok(vec![result]));
        }
        if let Some(result) = self.objc_foreign(symbol, args) {
            return Some(Ok(vec![result]));
        }
        // Each area answers the procedures it implements and passes on the rest.
        let fun = LibcFn::from_name(symbol)?;
        let arg = |i: usize| word_or_zero(args, i);
        let served = self
            .stdin_foreign(fun, args)
            .or_else(|| self.memory_call(fun, arg))
            .or_else(|| self.time_call(fun, arg))
            .or_else(|| self.numeric_call(fun, arg))
            .or_else(|| self.file_call(fun, arg))
            .or_else(|| self.dir_call(fun, arg))
            .or_else(|| self.stream_call(fun, arg))
            .map(Ok)
            .or_else(|| self.process_call(fun, arg))?;
        Some(served.map(|value| vec![value]))
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
    use super::guest::read_u32;
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
        read_u32(host.errno_cell)
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
            &[text.as_ptr() as u64, (&raw mut end) as u64],
        );
        assert_eq!(bits, 0x3f80_0001);
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
                    &[text.as_ptr() as u64, (&raw mut end) as u64]
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
                &[text.as_ptr() as u64, (&raw mut end) as u64]
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
                &raw mut done as u64,
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
                &raw mut done as u64,
                0,
            ],
        );
        assert_eq!((ok, done as usize), (1, payload.len()));
        assert_eq!(&buf[..payload.len()], payload);
        let mut size = 0u64;
        assert_eq!(
            call(&mut h, "GetFileSizeEx", &[handle, &raw mut size as u64]),
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
