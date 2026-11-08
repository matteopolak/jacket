//! Anonymous private mappings owned by one sandbox; no file mappings or executable pages.
#[cfg(unix)]
use super::{EINVAL, EPERM};
use super::{LibcFn, SandboxHost};

/// The most address space one sandbox host maps in total. Compile-time code that maps without
/// freeing would otherwise reserve memory for as long as the compiler runs.
#[cfg(unix)]
pub(super) const MAX_MAPPED_BYTES: u64 = 1 << 30;

pub(super) fn page_size() -> u64 {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn getpagesize() -> i32;
        }
        // SAFETY: a process-local configuration query with no arguments.
        (unsafe { getpagesize() }) as u64
    }
    #[cfg(not(unix))]
    {
        4096
    }
}

#[cfg(unix)]
mod native {
    use std::ffi::c_void;

    unsafe extern "C" {
        fn mmap(
            addr: *mut c_void,
            len: usize,
            prot: i32,
            flags: i32,
            fd: i32,
            offset: i64,
        ) -> *mut c_void;

        fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
        fn munmap(addr: *mut c_void, len: usize) -> i32;
    }

    pub(in crate::interp::sandbox) struct Mapping {
        pub base: u64,
        pub len: u64,
    }

    impl Mapping {
        pub fn reserve(len: u64, prot: i32) -> Option<Self> {
            let anon = if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
                0x1000
            } else {
                0x20
            };
            // SAFETY: the kernel chooses a fresh anonymous private range; no existing mapping can be replaced.
            let base =
                unsafe { mmap(std::ptr::null_mut(), len as usize, prot, 2 | anon, -1, 0) } as u64;
            (base != u64::MAX).then_some(Self {
                base,
                len,
            })
        }

        pub fn protect(&self, base: u64, len: u64, prot: i32) -> bool {
            // SAFETY: the caller checked that the complete range belongs to this live mapping.
            unsafe { mprotect(base as *mut c_void, len as usize, prot) == 0 }
        }

        pub fn release(&mut self, base: u64, len: u64) -> bool {
            // SAFETY: the caller checked ownership; successful release transfers the remaining ranges to new owners.
            unsafe { munmap(base as *mut c_void, len as usize) == 0 }
        }
    }

    impl Drop for Mapping {
        fn drop(&mut self) {
            if self.len != 0 {
                crate::interp::probe::invalidate();
                // SAFETY: this object exclusively owns the mapping that remains.
                unsafe {
                    munmap(self.base as *mut c_void, self.len as usize);
                }
            }
        }
    }
}

#[cfg(unix)]
pub(super) use native::Mapping;

impl SandboxHost {
    pub(super) fn virtual_memory_call(
        &mut self,
        fun: LibcFn,
        arg: impl Fn(usize) -> u64,
    ) -> Option<u64> {
        if !matches!(fun, LibcFn::Mmap | LibcFn::Mprotect | LibcFn::Munmap) {
            return None;
        }
        #[cfg(not(unix))]
        {
            let _ = arg;
            None
        }
        #[cfg(unix)]
        {
            let page = page_size();
            let length = arg(1);
            let Some(length) = length
                .checked_add(page - 1)
                .map(|n| n / page * page)
                .filter(|&n| n > 0 && isize::try_from(n).is_ok())
            else {
                return Some(self.set_errno(EINVAL));
            };
            if fun == LibcFn::Mmap {
                let anonymous = if self.darwin_files {
                    0x1000
                } else {
                    0x20
                };
                let flags = arg(3);
                if flags != 2 | anonymous
                    || arg(4) as u32 != u32::MAX
                    || arg(5) != 0
                    || arg(2) & !3 != 0
                {
                    return Some(self.set_errno(EPERM));
                }
                if self.mapped_bytes.saturating_add(length) > MAX_MAPPED_BYTES {
                    return Some(self.set_errno(super::super::errno::ENOMEM));
                }
                let Some(mapping) = Mapping::reserve(length, arg(2) as i32) else {
                    return Some(self.set_errno(super::super::errno::ENOMEM));
                };
                self.mapped_bytes += length;
                let base = mapping.base;
                self.mappings.insert(base, mapping);
                return Some(base);
            }
            let base = arg(0);
            if !base.is_multiple_of(page) {
                return Some(self.set_errno(EINVAL));
            }
            let Some(end) = base.checked_add(length) else {
                return Some(self.set_errno(EINVAL));
            };
            let key = self
                .mappings
                .range(..=base)
                .next_back()
                .map(|(&key, _)| key);
            let Some(key) = key else {
                return Some(self.set_errno(EINVAL));
            };
            let mapping = &self.mappings[&key];
            if end > mapping.base + mapping.len {
                return Some(self.set_errno(EINVAL));
            }
            if fun == LibcFn::Mprotect {
                if arg(2) & !3 != 0 {
                    return Some(self.set_errno(EPERM));
                }
                return Some(if mapping.protect(base, length, arg(2) as i32) {
                    0
                } else {
                    self.set_errno(EINVAL)
                });
            }
            let mut mapping = self.mappings.remove(&key).expect("owned mapping");
            if !mapping.release(base, length) {
                self.mappings.insert(key, mapping);
                return Some(self.set_errno(EINVAL));
            }
            self.mapped_bytes -= length;
            let old_end = mapping.base + mapping.len;
            mapping.len = 0;
            if key < base {
                self.mappings.insert(
                    key,
                    Mapping {
                        base: key,
                        len: base - key,
                    },
                );
            }
            if end < old_end {
                self.mappings.insert(
                    end,
                    Mapping {
                        base: end,
                        len: old_end - end,
                    },
                );
            }
            Some(0)
        }
    }
}
