//! The working directory and directory streams.

use super::{
    DirStream, EBADF, EEXIST, ENOENT, ENOTDIR, ENOTEMPTY, LibcFn, SandboxHost, records,
    write_bytes, write_u8,
};

impl SandboxHost {
    /// The working directory and directories: `getcwd`, `mkdir`, `opendir`, ...
    pub(super) fn dir_call(&mut self, fun: LibcFn, arg: impl Fn(usize) -> u64) -> Option<u64> {
        Some(match fun {
            LibcFn::Getcwd => {
                let cwd = if self.cwd.is_empty() {
                    "/".to_string()
                } else {
                    self.cwd.clone()
                };
                if (cwd.len() as u64) + 1 > arg(1) {
                    self.set_errno(34); // ERANGE
                    0
                } else {
                    write_bytes(arg(0), cwd.as_bytes());
                    write_u8(arg(0) + cwd.len() as u64, 0);
                    arg(0)
                }
            }
            LibcFn::Chdir => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) {
                    self.cwd = path;
                    0
                } else {
                    self.set_errno(ENOENT)
                }
            }
            LibcFn::Realpath | LibcFn::RealpathDarwin => {
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
                    write_bytes(out, path.as_bytes());
                    write_u8(out + path.len() as u64, 0);
                    out
                }
            }
            LibcFn::Mkdir => {
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
            LibcFn::Rmdir => {
                let path = self.path(arg(0));
                return Some(self.rmdir(&path));
            }
            LibcFn::Opendir => {
                let path = self.path(arg(0));
                if self.fs.is_dir(&path) {
                    let mut entries = vec![(".".to_string(), true), ("..".to_string(), true)];
                    entries.extend(self.fs.list(&path));
                    let handle = self.alloc(16);
                    let abi = self.abi();
                    let entry = self.alloc(records::dirent_size(abi));
                    self.dirs.insert(
                        handle,
                        DirStream {
                            abi,
                            entries,
                            next: 0,
                            entry,
                        },
                    );
                    handle
                } else {
                    self.set_errno(if self.fs.exists_file(&path) {
                        ENOTDIR
                    } else {
                        ENOENT
                    });
                    0
                }
            }
            LibcFn::Readdir | LibcFn::Readdir64 => match self.dirs.get_mut(&arg(0)) {
                Some(stream) if stream.next < stream.entries.len() => {
                    let (name, dir) = stream.entries[stream.next].clone();
                    stream.next += 1;
                    let entry = stream.entry;
                    let kind = if dir {
                        records::DT_DIR
                    } else {
                        records::DT_REG
                    };
                    let record = records::DirEntry {
                        inode: stream.next as u64,
                        next: stream.next as u64,
                        kind,
                        name: name.as_bytes(),
                    };
                    records::fill_dirent(stream.abi, entry, &record);
                    entry
                }
                _ => 0,
            },
            LibcFn::Closedir => match self.dirs.remove(&arg(0)) {
                Some(stream) => {
                    self.allocations.remove(&stream.entry);
                    self.allocations.remove(&arg(0));
                    0
                }
                None => self.set_errno(EBADF),
            },
            _ => return None,
        })
    }

    /// `rmdir`: `0`, or `-1` with `errno` set.
    pub(super) fn rmdir(&mut self, path: &str) -> u64 {
        let result = self.remove_dir(path).map(|()| 0);
        self.libc_result(result)
    }

    /// Remove an empty directory; the error is an errno.
    pub(super) fn remove_dir(&mut self, path: &str) -> Result<(), u64> {
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
}
