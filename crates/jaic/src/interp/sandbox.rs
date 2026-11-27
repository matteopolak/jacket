//! The sandbox host: program output captured in memory and a small libc for runs that have no
//! dynamic linker (the browser playground, `jaic -interp -os wasm`).
//!
//! Everything here is virtual and deterministic: a clock that advances by a microsecond per query
//! (and by the requested time on `nanosleep`), a file system that overlays in-memory writes on a
//! read-only [`FileSystem`] (the playground's workspace and bundled stdlib), a working directory,
//! `FILE*`/descriptor/`DIR*` handles, and the number parsing and `localtime` helpers the standard
//! library calls. Thread primitives are not here: they need the interpreter (`threads_inline.rs`).
#![allow(unsafe_code)]

use super::{HashMap, Host, Rc, ZeroedBlock, ir};
use crate::fxhash::HashSet;
use crate::interp::word_or_zero;
use crate::sema::FileSystem;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

mod dirs;
mod files;
pub(super) mod guest;
mod memory;
mod numeric;
mod objc;
mod process;
mod records;
mod sockets;
mod stdin;
mod streams;
pub(super) mod symbols;
#[cfg(test)]
mod tests;
mod time;
mod virtual_memory;
mod windows;

use guest::{
    copy, fill, read_bytes, read_i64, read_u8, read_u64, write_bytes, write_i32, write_u8,
    write_u16, write_u32, write_u64,
};
use symbols::{LibcFn, SandboxFn};

use super::errno::{EBADF, EEXIST, EINVAL, EISDIR, ENOENT, ENOTDIR, ENOTEMPTY, EPERM};

const O_ACCMODE: u64 = 3;
pub(super) const O_WRONLY: u64 = 1;
pub(super) const O_RDWR: u64 = 2;
pub(super) const O_CREAT: u64 = 0o100;
pub(super) const O_EXCL: u64 = 0o200;
pub(super) const O_TRUNC: u64 = 0o1000;
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
                .is_some_and(|b| b.is_file(&host_path(path)))
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
                .is_some_and(|b| b.is_dir(&host_path(path)))
    }

    /// Bring a base file into the overlay so it can be read and written in place.
    fn load(&mut self, path: &str) -> bool {
        if self.files.contains_key(path) {
            return true;
        }
        if self.removed.contains(path) {
            return false;
        }
        match self.base.as_ref().and_then(|b| b.read(&host_path(path))) {
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
                .and_then(|b| b.read(&host_path(path)))
                .map_or(0, |d| d.len()),
        }
    }

    fn list(&self, path: &str) -> Vec<(String, bool)> {
        let mut out: BTreeMap<String, bool> = BTreeMap::new();
        if let Some(base) = &self.base {
            for (name, dir) in base.list_dir(&host_path(path)) {
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

pub(super) fn join(dir: &str, name: &str) -> String {
    format!("{}/{}", dir.trim_end_matches('/'), name)
}

/// Whether `path` starts with a drive (`C:`, then a separator or nothing).
fn drive_path(path: &str) -> bool {
    match path.as_bytes() {
        [letter, b':', rest @ ..] => {
            letter.is_ascii_alphabetic() && matches!(rest.first(), None | Some(b'/' | b'\\'))
        }
        _ => false,
    }
}

/// The sandbox's name for a Windows host path, with `\` or `/` separators: `C:\x` is `/C:/x`,
/// the share path `\\server\share\x` is `/UNC/server/share/x`, and their `\\?\` forms name the
/// same files. A device path (`\\.\pipe\x`, `\\?\Volume{...}\x`) is `/DEVICE/...`, which
/// names no file. `None` when `path` is none of these.
fn from_windows(path: &str) -> Option<String> {
    let path = path.replace('\\', "/");
    let Some(rest) = path.strip_prefix("//") else {
        return drive_path(&path).then(|| format!("/{path}"));
    };
    if let Some(rest) = rest.strip_prefix("?/") {
        if drive_path(rest) {
            return Some(format!("/{rest}"));
        }
        if let Some(share) = rest.strip_prefix("UNC/") {
            return Some(format!("/UNC/{share}"));
        }
        return Some(format!("/DEVICE/{rest}"));
    }
    if let Some(device) = rest.strip_prefix("./") {
        return Some(format!("/DEVICE/{device}"));
    }
    Some(format!("/UNC/{rest}"))
}

/// The Windows name of the sandbox path `path` when it names a drive's or a share's files: the
/// reverse of `from_windows`.
fn to_windows(path: &str) -> Option<String> {
    let rest = path.strip_prefix('/')?;
    if drive_path(rest) {
        return Some(rest.to_string());
    }
    let share = rest.strip_prefix("UNC/")?;
    Some(format!(r"\\{}", share.replace('/', "\\")))
}

/// How many leading components of the sandbox path `parts` are the root of a Windows host's
/// file system, which `..` cannot climb out of: a drive (`C:`), a share (`UNC/server/share`)
/// or a device. None on other hosts.
fn windows_root(parts: &[&str]) -> usize {
    if !cfg!(windows) {
        return 0;
    }
    let root = match parts.first() {
        Some(&"UNC") => 3,
        Some(&"DEVICE") => 2,
        Some(first) if drive_path(first) => 1,
        _ => 0,
    };
    root.min(parts.len())
}

/// The host's name for the sandbox path `path`. The sandbox's paths start at `/`, so on a
/// Windows host it names a drive's files `/C:/...` and a share's `/UNC/server/share/...`;
/// elsewhere the two names are the same.
pub(super) fn host_path(path: &str) -> Cow<'_, Path> {
    match to_windows(path).filter(|_| cfg!(windows)) {
        Some(name) => Cow::Owned(name.into()),
        None => Cow::Borrowed(Path::new(path)),
    }
}

/// Resolve `path` against `cwd` and drop `.`/`..` components. On a Windows host a path that
/// starts with a drive, a share or a device is absolute too (see `from_windows`), and `..`
/// stops at its root.
pub(super) fn absolute(cwd: &str, path: &str) -> String {
    let full = if let Some(name) = from_windows(path).filter(|_| cfg!(windows)) {
        name
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        format!("{cwd}/{path}")
    };
    let all: Vec<&str> = full
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect();
    let mut parts: Vec<&str> = Vec::new();
    let mut floor = 0;
    for part in all {
        if part == ".." {
            if parts.len() > floor {
                parts.pop();
            }
            continue;
        }
        parts.push(part);
        floor = floor.max(windows_root(&parts));
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
    darwin: bool,
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
    /// The program's window, when its pointers are narrower than the host's: where everything
    /// the stand-ins hand the program comes from.
    window: Option<std::sync::Arc<super::Window>>,
    mappings: BTreeMap<u64, virtual_memory::Mapping>,
    /// Bytes held by `mappings`, capped at `virtual_memory::MAX_MAPPED_BYTES`.
    mapped_bytes: u64,
    /// Virtual clock ticks (nanoseconds) handed out by `clock_gettime`; the sandbox has no real
    /// time source on wasm32, so every query advances this by one microsecond.
    clock_ns: u64,
    fs: SandboxFs,
    cwd: String,
    fds: HashMap<i64, OpenFile>,
    /// Pre-opened listeners and the connections they accepted, by descriptor (`-os wasm` runs).
    sockets: HashMap<i64, sockets::Socket>,
    /// Variables the program sees whatever `real_environment` says (`LISTEN_FDS`).
    extra_environment: HashMap<String, String>,
    next_fd: i64,
    /// `FILE*` handle address -> descriptor.
    streams: HashMap<u64, i64>,
    dirs: HashMap<u64, DirStream>,
    /// Darwin POSIX layouts and configuration query numbers for host-target metaprograms.
    darwin_files: bool,
    /// The CPU whose C library records the program uses; `None` is Linux x86-64's, which the
    /// stdlib's wasm records share.
    cpu: Option<crate::target::TargetCpu>,
    errno_cell: u64,
    /// Windows' `GetLastError`.
    windows: super::windows::WindowsCalls,
    /// The "C" `setlocale` returns, in program memory; 0 until first asked for.
    locale_name: u64,
    /// `strerror` results by error number; they stay valid for the whole run, like libc's.
    error_texts: HashMap<u64, u64>,
    stdin: stdin::Stdin,
    objc: objc::Objc,
    /// Compile-time services, when the embedder offers them (the language server in a trusted
    /// workspace). `None` answers every call as unavailable.
    pub services: Option<Rc<dyn crate::service::ServiceProvider>>,
    /// `getenv` reads the process's real environment (the language server in a trusted
    /// workspace); otherwise every variable is unset.
    pub real_environment: bool,
    /// `getenv` results by (name, value); they stay valid for the whole run, like libc's.
    environment_texts: HashMap<(String, String), u64>,
    /// When the wall-clock limit started and how long it is (`stop_after`).
    time_limit: Option<(crate::platform::Stopwatch, std::time::Duration)>,
    /// The limit passed: every interpreter on this host stops (`Host::stop_requested`).
    stopped: bool,
    /// Times the host was asked whether to stop (`has_stopped`).
    polls: u64,
    /// Stop once asked this many times (`stop_after_polls`).
    poll_limit: Option<u64>,
}

impl SandboxHost {
    /// Select the POSIX ABI of code running on this host, including file records and
    /// `sysconf` query numbers. Wasm uses the library's Linux-compatible ABI.
    /// Call before running a different target's code.
    pub fn set_file_target(&mut self, os: crate::target::TargetOs) {
        self.darwin_files = os == crate::target::TargetOs::MacOS;
    }

    /// Select the CPU whose C library records (`struct stat`) the guest uses.
    pub fn set_cpu(&mut self, cpu: crate::target::TargetCpu) {
        self.cpu = Some(cpu);
    }

    /// The bytes of a C `long` in the records the program's C library fills: the target's, but
    /// 8 for wasm, whose library (`internal/files/wasi-libc`) keeps 64-bit fields on wasm32.
    fn long_bytes(&self) -> u8 {
        if self.cpu == Some(crate::target::TargetCpu::Wasm) {
            8
        } else {
            guest::layout().c.long
        }
    }

    /// Stop all code running on this host `limit` from now: the check it belongs to has a
    /// wall-clock limit. Does nothing on a host without a clock (the browser's).
    pub fn stop_after(&mut self, limit: std::time::Duration) {
        if crate::platform::HAS_CLOCK {
            self.time_limit = Some((crate::platform::Stopwatch::start(), limit));
        }
    }

    /// Ends the wall-clock limit of `stop_after` without stopping anything: the check it bounded
    /// is over, and what runs on this host later (a hover finishing an entity the check left
    /// unevaluated) must not fail for the time that has passed since. A host that already
    /// stopped stays stopped.
    pub fn end_time_limit(&mut self) {
        self.time_limit = None;
    }

    /// Stop all code running on this host once it was asked `polls` times whether to stop.
    /// Replaces the clock with a count, so a test of where the compiler asks can say exactly
    /// when the answer turns to yes.
    pub fn stop_after_polls(&mut self, polls: u64) {
        self.poll_limit = Some(polls);
    }

    /// How many times the host was asked whether to stop.
    pub fn polls(&self) -> u64 {
        self.polls
    }

    /// Whether the limit set by `stop_after` or `stop_after_polls` has passed.
    pub fn has_stopped(&mut self) -> bool {
        self.polls += 1;
        if self.poll_limit.is_some_and(|limit| self.polls > limit) {
            self.stopped = true;
        }
        if !self.stopped
            && let Some((since, limit)) = self.time_limit
        {
            self.stopped = since.elapsed() >= limit;
        }
        self.stopped
    }

    /// Free the blocks compile-time code allocated and never freed. Only once no code runs on
    /// this host again (`Interp::release_code` on every interpreter using it): pointers to
    /// them are left dangling in program memory.
    pub fn release_memory(&mut self) {
        self.allocations = HashMap::default();
        self.windows = super::windows::WindowsCalls::default();
    }

    /// Back the sandbox's file system with `base` (read-only) and start in `cwd`.
    pub fn with_files(base: Rc<dyn FileSystem>, cwd: &str) -> Self {
        let mut host = SandboxHost::default();
        host.fs.base = Some(base);
        // A Windows host's directory (`D:\a\b`) is named as its other paths are: `/D:/a/b`.
        if !cwd.is_empty() {
            host.cwd = absolute("/", cwd);
        }
        // Programs expect a scratch directory.
        host.fs.dirs.insert("/tmp".to_string());
        host
    }

    /// A zeroed 16-byte aligned block, or null when the request cannot be met (like `malloc`;
    /// an infallible allocation would abort the whole compiler on `alloc(1 << 60)`).
    fn alloc(&mut self, size: usize) -> u64 {
        let Some(block) = ZeroedBlock::in_window(self.window.as_ref(), size, 16) else {
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
        let inode = self.fs.inode(path);
        let links = 1 + self
            .fs
            .inodes
            .iter()
            .filter(|(p, i)| **i == inode && p.as_str() != path)
            .count() as u64;
        let (mode, size) = if self.fs.is_dir(path) {
            (S_IFDIR | 0o755, 4096)
        } else if self.fs.exists_file(path) {
            (S_IFREG | 0o644, self.fs.file_size(path) as u64)
        } else {
            return Err(ENOENT);
        };
        let os = if self.darwin_files {
            crate::target::TargetOs::MacOS
        } else {
            crate::target::TargetOs::Linux
        };
        let file = records::FileStat {
            inode,
            mode,
            links,
            size,
            mtime: self.fs.mtimes.get(path).copied().unwrap_or(BASE_MTIME),
        };
        records::fill_stat(os, self.cpu, out, &file);
        Ok(())
    }

    fn libc_result(&mut self, result: Result<u64, u64>) -> u64 {
        match result {
            Ok(v) => v,
            Err(code) => self.set_errno(code),
        }
    }
}

/// The stat call a Linux x86-64 program's `syscall(number, ...)` makes, if it is one; the
/// numbers are the target's, so `x64_linux` says whether the program is one.
fn stat_syscall(x64_linux: bool, symbol: &str, args: &[u64]) -> Option<LibcFn> {
    if symbol != "syscall" || !x64_linux {
        return None;
    }
    match word_or_zero(args, 0) {
        4 => Some(LibcFn::Stat),
        5 => Some(LibcFn::Fstat),
        6 => Some(LibcFn::Lstat),
        _ => None,
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
    fn attach_window(&mut self, window: &std::sync::Arc<super::Window>) {
        self.window = Some(window.clone());
    }

    fn stop_requested(&mut self) -> bool {
        self.has_stopped()
    }

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
        sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>> {
        // Linux x86-64 reaches the stat family through syscall(2) (stdlib/POSIX/wrappers.jai).
        let x64_linux =
            !self.darwin_files && matches!(self.cpu, None | Some(crate::target::TargetCpu::X64));
        let (fun, args) = match stat_syscall(x64_linux, symbol, args) {
            Some(fun) => (fun, args.get(1..).unwrap_or_default()),
            None => match SandboxFn::lookup(symbol)? {
                SandboxFn::Windows(fun) => {
                    return self.windows_foreign(fun, args, sig).map(|r| Ok(vec![r]));
                }
                SandboxFn::Objc(fun) => {
                    return Some(Ok(vec![self.objc_foreign(fun, args)]));
                }
                // Each area answers the procedures it implements and passes on the rest.
                SandboxFn::Libc(fun) => (fun, args),
            },
        };
        let arg = |i: usize| word_or_zero(args, i);
        let served = self
            .stdin_foreign(fun, args)
            .or_else(|| self.memory_call(fun, arg))
            .or_else(|| self.virtual_memory_call(fun, arg))
            .or_else(|| self.time_call(fun, arg))
            .or_else(|| self.numeric_call(fun, arg))
            .or_else(|| self.socket_call(fun, arg))
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
        (!self.cwd.is_empty()).then(|| host_path(&self.cwd).to_string_lossy().into_owned())
    }

    fn service(&mut self, service: &str, request: &[u8]) -> crate::service::ServiceReply {
        let Some(services) = &self.services else {
            return crate::service::ServiceReply::unavailable(
                "compile-time services are not available here",
            );
        };
        let base = (!self.cwd.is_empty()).then(|| host_path(&self.cwd));
        services.ask(&crate::service::resolve(service, base.as_deref()), request)
    }
}

/// Shares a [`SandboxHost`] with the embedder so output can be read after the run (and so
/// metaprogram workspaces write to the same buffers).
pub struct SharedHost(pub Rc<RefCell<SandboxHost>>);

impl Host for SharedHost {
    fn attach_window(&mut self, window: &std::sync::Arc<super::Window>) {
        self.0.borrow_mut().attach_window(window);
    }

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

    fn stop_requested(&mut self) -> bool {
        self.0.borrow_mut().has_stopped()
    }

    fn service(&mut self, service: &str, request: &[u8]) -> crate::service::ServiceReply {
        // The provider may run for a while (a short wait in the editor): don't hold the borrow.
        let (services, base) = {
            let host = self.0.borrow();
            (host.services.clone(), host.working_directory())
        };
        let Some(services) = services else {
            return crate::service::ServiceReply::unavailable(
                "compile-time services are not available here",
            );
        };
        services.ask(
            &crate::service::resolve(service, base.as_deref().map(std::path::Path::new)),
            request,
        )
    }
}
