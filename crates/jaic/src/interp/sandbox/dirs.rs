//! The working directory and directory streams.

use super::*;

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
                    unsafe {
                        std::ptr::copy_nonoverlapping(cwd.as_ptr(), arg(0) as *mut u8, cwd.len());
                        *((arg(0) + cwd.len() as u64) as *mut u8) = 0;
                    }
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
                    unsafe {
                        std::ptr::copy_nonoverlapping(path.as_ptr(), out as *mut u8, path.len());
                        *((out + path.len() as u64) as *mut u8) = 0;
                    }
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
            LibcFn::Readdir | LibcFn::Readdir64 => match self.dirs.get_mut(&arg(0)) {
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
        let result = self.remove_dir(path).map(|_| 0);
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
