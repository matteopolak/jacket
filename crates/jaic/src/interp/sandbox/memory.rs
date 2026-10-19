//! Heap memory, byte and C string procedures.

use super::{
    EBADF, EEXIST, EINVAL, EISDIR, ENOENT, ENOTDIR, ENOTEMPTY, LibcFn, SandboxHost, copy, cstr,
    fill, read_bytes, read_u8, write_bytes,
};

impl SandboxHost {
    /// Heap memory, byte and C string procedures, and `errno`.
    pub(super) fn memory_call(&mut self, fun: LibcFn, arg: impl Fn(usize) -> u64) -> Option<u64> {
        Some(match fun {
            LibcFn::Malloc => self.alloc(arg(0) as usize),
            LibcFn::Calloc => (arg(0) as usize)
                .checked_mul(arg(1) as usize)
                .map_or(0, |size| self.alloc(size)),
            LibcFn::Realloc => {
                let (old, size) = (arg(0), arg(1) as usize);
                let new = self.alloc(size);
                if new == 0 {
                    // The old block stays valid when growing fails.
                    return Some(0);
                }
                if let Some((_, old_size)) = self.allocations.get(&old) {
                    copy(new, old, (*old_size).min(size));
                    self.allocations.remove(&old);
                }
                new
            }
            LibcFn::Free => {
                self.allocations.remove(&arg(0));
                0
            }
            LibcFn::Memcpy | LibcFn::Memmove => {
                copy(arg(0), arg(1), arg(2) as usize);
                arg(0)
            }
            LibcFn::Memset => {
                fill(arg(0), arg(1) as u8, arg(2) as usize);
                arg(0)
            }
            LibcFn::Memcmp => {
                let (a, b) = (
                    read_bytes(arg(0), arg(2) as usize),
                    read_bytes(arg(1), arg(2) as usize),
                );
                match a.cmp(&b) {
                    std::cmp::Ordering::Less => -1i64 as u64,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }
            }
            LibcFn::Strlen => {
                let mut n = 0;
                while read_u8(arg(0) + n) != 0 {
                    n += 1;
                }
                n
            }
            LibcFn::Strcmp | LibcFn::Strncmp => {
                let limit = if fun == LibcFn::Strncmp {
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
                    let (x, y) = (read_u8(a), read_u8(b));
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
            LibcFn::Strerror => {
                let code = arg(0) as u32 as u64;
                if let Some(&text) = self.error_texts.get(&code) {
                    return Some(text);
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
                    write_bytes(text, message.as_bytes());
                    self.error_texts.insert(code, text);
                }
                text
            }
            LibcFn::Setlocale => {
                if arg(0) > 6 {
                    0
                } else if arg(1) == 0 || matches!(cstr(arg(1)).as_str(), "" | "C" | "POSIX") {
                    c"C".as_ptr() as u64
                } else {
                    0
                }
            }
            LibcFn::ErrnoLocation | LibcFn::DarwinError => {
                // The cell keeps the last error: reading it must not clear it.
                if self.errno_cell == 0 {
                    self.errno_cell = self.alloc(8);
                }
                self.errno_cell
            }
            _ => return None,
        })
    }
}
