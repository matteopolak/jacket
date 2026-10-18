//! `FILE*` streams.

use super::*;

impl SandboxHost {
    /// `FILE*` streams over the descriptors.
    pub(super) fn stream_call(&mut self, fun: LibcFn, arg: impl Fn(usize) -> u64) -> Option<u64> {
        Some(match fun {
            LibcFn::Fopen | LibcFn::Fopen64 => {
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
            LibcFn::Fclose => match self.streams.remove(&arg(0)) {
                Some(fd) => {
                    self.fds.remove(&fd);
                    self.allocations.remove(&arg(0));
                    0
                }
                None => self.set_errno(EBADF),
            },
            LibcFn::Fflush => 0,
            LibcFn::Fread => {
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
            LibcFn::Fwrite => {
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
            LibcFn::Fseek | LibcFn::Fseeko | LibcFn::Fseeko64 => {
                match self.streams.get(&arg(0)).copied() {
                    Some(fd) => match self.seek(fd, arg(1) as i64, arg(2) as u32 as u64) {
                        Some(_) => 0,
                        None => self.set_errno(EINVAL),
                    },
                    None => self.set_errno(EBADF),
                }
            }
            LibcFn::Ftell | LibcFn::Ftello | LibcFn::Ftello64 => {
                match self.streams.get(&arg(0)).copied() {
                    Some(fd) => self.fds.get(&fd).map_or(u64::MAX, |f| f.pos as u64),
                    None => self.set_errno(EBADF),
                }
            }
            LibcFn::Rewind => {
                if let Some(fd) = self.streams.get(&arg(0)).copied() {
                    self.seek(fd, 0, 0);
                }
                0
            }
            LibcFn::Feof => match self.streams.get(&arg(0)).copied() {
                Some(fd) => u64::from(self.fds.get(&fd).is_some_and(|f| f.eof)),
                None => 0,
            },
            LibcFn::Ferror => 0,
            LibcFn::Fileno => self.streams.get(&arg(0)).map_or(u64::MAX, |fd| *fd as u64),
            _ => return None,
        })
    }
}
