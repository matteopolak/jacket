//! In-memory backing for the shared Windows call layer.
use super::symbols::WinFn;
use super::{BASE_MTIME, EEXIST, ENOENT, Host, SandboxHost};
use crate::interp::windows::{FileDetails, WindowsBackend};

impl SandboxHost {
    pub(super) fn windows_foreign(
        &mut self,
        fun: WinFn,
        args: &[u64],
        sig: &crate::ir::Sig,
    ) -> Option<u64> {
        let mut calls = std::mem::take(&mut self.windows);
        let result = calls.foreign(self, fun, args, sig);
        self.windows = calls;
        result
    }
}

impl WindowsBackend for SandboxHost {
    fn details(&self, path: &str) -> Option<FileDetails> {
        let dir = self.fs.is_dir(path);
        if !dir && !self.fs.exists_file(path) {
            return None;
        }
        let time = BASE_MTIME * 10_000_000 + 116_444_736_000_000_000;
        let written = self.fs.mtimes.get(path).copied().unwrap_or(BASE_MTIME) * 10_000_000
            + 116_444_736_000_000_000;
        Some(FileDetails {
            attributes: if dir {
                0x10
            } else {
                0x80
            },
            size: self.fs.file_size(path) as u64,
            created: time,
            accessed: time,
            written,
            identity: [1, self.fs.inode(path), 0],
        })
    }
    fn fd_details(&self, fd: i64) -> Option<FileDetails> {
        self.details(&self.fds.get(&fd)?.path)
    }
    fn canonical(&self, fd: i64) -> Option<String> {
        Some(self.fds.get(&fd)?.path.clone())
    }
    fn contents(&self, fd: i64) -> Option<Vec<u8>> {
        self.fs.files.get(&self.fds.get(&fd)?.path).cloned()
    }
    fn flush(&mut self, fd: i64) -> Result<(), u64> {
        if self.fds.contains_key(&fd) {
            Ok(())
        } else {
            Err(super::EBADF)
        }
    }
    fn copy_file(&mut self, from: &str, to: &str) -> Result<(), u64> {
        if !self.fs.load(from) {
            return Err(ENOENT);
        }
        if !self.parent_exists(to) {
            return Err(ENOENT);
        }
        let bytes = self.fs.files[from].clone();
        self.fs.removed.remove(to);
        self.fs.files.insert(to.into(), bytes);
        self.fs.changed(to);
        Ok(())
    }
    fn hard_link(&mut self, from: &str, to: &str) -> Result<(), u64> {
        if self.fs.exists_file(to) || self.fs.is_dir(to) {
            return Err(EEXIST);
        }
        self.copy_file(from, to)?;
        let inode = self.fs.inode(from);
        self.fs.inodes.insert(from.into(), inode);
        self.fs.inodes.insert(to.into(), inode);
        Ok(())
    }
    fn cwd(&self) -> String {
        if self.cwd.is_empty() {
            "/".into()
        } else {
            self.cwd.clone()
        }
    }
    fn set_cwd(&mut self, path: String) -> Result<(), u64> {
        self.cwd = path;
        Ok(())
    }
    fn is_dir(&self, path: &str) -> bool {
        self.fs.is_dir(path)
    }
    fn exists_file(&self, path: &str) -> bool {
        self.fs.exists_file(path)
    }
    fn list(&self, path: &str) -> Vec<(String, bool)> {
        self.fs.list(path)
    }
    fn create_dir(&mut self, path: &str) -> Result<(), u64> {
        if self.fs.is_dir(path) || self.fs.exists_file(path) {
            return Err(EEXIST);
        }
        if !self.parent_exists(path) {
            return Err(ENOENT);
        }
        self.fs.removed.remove(path);
        self.fs.dirs.insert(path.into());
        Ok(())
    }
    fn delete_file(&mut self, path: &str) -> Result<(), u64> {
        if !self.fs.exists_file(path) {
            return Err(ENOENT);
        }
        self.fs.files.remove(path);
        self.fs.forget(path);
        self.fs.removed.insert(path.into());
        Ok(())
    }
    fn remove_dir(&mut self, path: &str) -> Result<(), u64> {
        self.remove_dir(path)
    }
    fn rename_file(&mut self, from: &str, to: &str) -> Result<(), u64> {
        self.rename_file(from, to)
    }
    fn open_file(&mut self, path: String, flags: u64) -> Result<i64, u64> {
        self.open_file(path, flags)
    }
    fn has_fd(&self, fd: i64) -> bool {
        self.fds.contains_key(&fd)
    }
    fn close_fd(&mut self, fd: i64) {
        self.fds.remove(&fd);
    }
    fn read_fd(&mut self, fd: i64, out: u64, count: usize) -> Option<usize> {
        self.read_fd(fd, out, count)
    }
    fn write_fd(&mut self, fd: i64, bytes: &[u8]) -> Option<usize> {
        self.write_fd(fd, bytes)
    }
    fn file_len(&self, fd: i64) -> Option<usize> {
        self.file_len(fd)
    }
    fn seek(&mut self, fd: i64, offset: i64, whence: u64) -> Option<i64> {
        self.seek(fd, offset, whence)
    }
    fn write(&mut self, bytes: &[u8], stderr: bool) {
        Host::write(self, bytes, stderr);
    }
    fn wall_ns(&mut self) -> u64 {
        self.clock_ns += 1_000;
        self.clock_ns + BASE_MTIME * 1_000_000_000
    }
    fn monotonic_ns(&mut self) -> u64 {
        self.clock_ns += 1_000;
        self.clock_ns
    }
}
