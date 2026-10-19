//! File descriptors and path operations over the virtual file system.

use super::{EBADF, EEXIST, EINVAL, ENOENT, Host, LibcFn, SandboxHost, read_bytes};

impl SandboxHost {
    /// File descriptors and paths: `open`, `read`, `write`, `stat`, `rename`, ...
    pub(super) fn file_call(&mut self, fun: LibcFn, arg: impl Fn(usize) -> u64) -> Option<u64> {
        Some(match fun {
            LibcFn::Write => {
                let (fd, ptr, len) = (arg(0) as i32 as i64, arg(1), arg(2) as usize);
                let bytes = read_bytes(ptr, len);
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
            LibcFn::Access => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) || self.fs.exists_file(&path) {
                    0
                } else {
                    self.set_errno(ENOENT)
                }
            }
            // The virtual file system keeps no permission bits (every file is readable and
            // writable), so changing them succeeds on any path that exists.
            LibcFn::Chmod => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) || self.fs.exists_file(&path) {
                    0
                } else {
                    self.set_errno(ENOENT)
                }
            }
            LibcFn::Fchmod => {
                if self.fds.contains_key(&(arg(0) as i32 as i64)) {
                    0
                } else {
                    self.set_errno(EBADF)
                }
            }
            LibcFn::Stat | LibcFn::Lstat => {
                let path = self.path(arg(0));
                let result = self.fill_stat(&path, arg(1)).map(|()| 0);
                self.libc_result(result)
            }
            LibcFn::Fstat => {
                let fd = arg(0) as i32 as i64;
                match self.fds.get(&fd).map(|f| f.path.clone()) {
                    Some(path) => {
                        let result = self.fill_stat(&path, arg(1)).map(|()| 0);
                        self.libc_result(result)
                    }
                    None => self.set_errno(EBADF),
                }
            }
            LibcFn::Open | LibcFn::Open64 => {
                let path = self.path(arg(0));
                let result = self
                    .open_file(path, arg(1) as u32 as u64)
                    .map(|fd| fd as u64);
                self.libc_result(result)
            }
            LibcFn::Close => {
                if self.fds.remove(&(arg(0) as i32 as i64)).is_some() {
                    0
                } else {
                    self.set_errno(EBADF)
                }
            }
            LibcFn::Read => match self.read_fd(arg(0) as i32 as i64, arg(1), arg(2) as usize) {
                Some(n) => n as u64,
                None => self.set_errno(EBADF),
            },
            LibcFn::Lseek | LibcFn::Lseek64 => {
                match self.seek(arg(0) as i32 as i64, arg(1) as i64, arg(2)) {
                    Some(p) => p as u64,
                    None => self.set_errno(EINVAL),
                }
            }
            LibcFn::Unlink | LibcFn::Remove => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) && fun == LibcFn::Remove {
                    return Some(self.rmdir(&path));
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
            LibcFn::Rename => {
                let (from, to) = (self.path(arg(0)), self.path(arg(1)));
                let result = self.rename_file(&from, &to).map(|()| 0);
                self.libc_result(result)
            }
            LibcFn::Link => {
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
            _ => return None,
        })
    }

    /// Move a file to `to`, replacing what is there; the error is an errno.
    pub(super) fn rename_file(&mut self, from: &str, to: &str) -> Result<(), u64> {
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
