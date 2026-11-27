//! Anonymous private mappings owned by one sandbox; no file mappings or executable pages.
use super::{EINVAL, EPERM};
use super::{LibcFn, SandboxHost};
use crate::interp::host_pages::{self, Access};
use std::sync::Arc;

/// The most address space one sandbox host maps in total. Compile-time code that maps without
/// freeing would otherwise reserve memory for as long as the compiler runs.
pub(super) const MAX_MAPPED_BYTES: u64 = 1 << 30;

pub(super) fn page_size() -> u64 {
    host_pages::page_size() as u64
}

/// The host pages under a mapping: one reservation, released once every part of the mapping
/// is unmapped.
struct Reservation {
    base: usize,
    len: usize,
}

impl Drop for Reservation {
    fn drop(&mut self) {
        host_pages::release(self.base, self.len);
    }
}

#[derive(Clone)]
enum Backing {
    /// Host pages: the mapping holds its reservation until the last part is unmapped.
    Host {
        _pages: Arc<Reservation>,
    },
    Window(Arc<crate::interp::Window>),
}

/// A private anonymous mapping, or what `munmap` left of one: host address space, or a page
/// range of the program's window, where `base` is the program address.
pub(super) struct Mapping {
    pub base: u64,
    pub len: u64,
    backing: Backing,
}

/// The access C's `PROT_READ` (1) and `PROT_WRITE` (2) bits ask for; write implies read, as on
/// the hardware the sandbox runs on.
fn access(prot: u64) -> Access {
    match prot & 3 {
        0 => Access::None,
        1 => Access::Read,
        _ => Access::ReadWrite,
    }
}

impl Mapping {
    /// `len` bytes of fresh zeroed pages with `prot`, in the program's window when it has one.
    fn reserve(len: u64, prot: u64, window: Option<&Arc<crate::interp::Window>>) -> Option<Self> {
        let mapping = if let Some(window) = window {
            Self {
                base: window.alloc_pages(len as usize)?,
                len,
                backing: Backing::Window(window.clone()),
            }
        } else {
            let base = host_pages::reserve(len as usize)?;
            Self {
                base: base as u64,
                len,
                backing: Backing::Host {
                    _pages: Arc::new(Reservation {
                        base,
                        len: len as usize,
                    }),
                },
            }
        };
        // A window's pages are committed already; a fresh reservation's are not.
        let ok = match &mapping.backing {
            Backing::Window(_) => mapping.protect(mapping.base, len, prot),
            Backing::Host {
                ..
            } => host_pages::commit(mapping.base as usize, len as usize, access(prot)),
        };
        ok.then_some(mapping)
    }

    /// The host address of program address `addr` in this mapping.
    fn host(&self, addr: u64) -> usize {
        match &self.backing {
            Backing::Host {
                ..
            } => addr as usize,
            Backing::Window(window) => (addr + window.base()) as usize,
        }
    }

    fn protect(&self, base: u64, len: u64, prot: u64) -> bool {
        host_pages::protect(self.host(base), len as usize, access(prot))
    }

    /// Give back `base..base + len`, a page range of this mapping.
    fn unmap(&self, base: u64, len: u64) -> bool {
        match &self.backing {
            Backing::Window(window) => {
                window.free_pages(base, len as usize);
                true
            }
            Backing::Host {
                ..
            } => host_pages::decommit(base as usize, len as usize),
        }
    }

    /// The part `base..base + len` of this mapping, sharing its pages.
    fn part(&self, base: u64, len: u64) -> Self {
        Self {
            base,
            len,
            backing: self.backing.clone(),
        }
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        if self.len != 0 {
            crate::interp::probe::invalidate();
            self.unmap(self.base, self.len);
        }
    }
}

impl SandboxHost {
    pub(super) fn virtual_memory_call(
        &mut self,
        fun: LibcFn,
        arg: impl Fn(usize) -> u64,
    ) -> Option<u64> {
        if !matches!(
            fun,
            LibcFn::Mmap | LibcFn::Mmap64 | LibcFn::Mprotect | LibcFn::Munmap
        ) {
            return None;
        }
        let page = page_size();
        let length = arg(1);
        let Some(length) = length
            .checked_add(page - 1)
            .map(|n| n / page * page)
            .filter(|&n| n > 0 && isize::try_from(n).is_ok())
        else {
            return Some(self.set_errno(EINVAL));
        };
        if matches!(fun, LibcFn::Mmap | LibcFn::Mmap64) {
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
            if self.mapped_bytes.saturating_add(length) > self.map_limit.unwrap_or(MAX_MAPPED_BYTES)
            {
                return Some(self.set_errno(super::super::errno::ENOMEM));
            }
            let Some(mapping) = Mapping::reserve(length, arg(2), self.window.as_ref()) else {
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
            return Some(if mapping.protect(base, length, arg(2)) {
                0
            } else {
                self.set_errno(EINVAL)
            });
        }
        let mut mapping = self.mappings.remove(&key).expect("owned mapping");
        if !mapping.unmap(base, length) {
            self.mappings.insert(key, mapping);
            return Some(self.set_errno(EINVAL));
        }
        self.mapped_bytes -= length;
        let old_end = mapping.base + mapping.len;
        mapping.len = 0;
        if key < base {
            self.mappings.insert(key, mapping.part(key, base - key));
        }
        if end < old_end {
            self.mappings.insert(end, mapping.part(end, old_end - end));
        }
        Some(0)
    }
}
