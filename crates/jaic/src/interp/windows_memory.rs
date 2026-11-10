//! `VirtualAlloc`, `VirtualProtect`, `VirtualFree` and `GetSystemInfo` for compile-time code
//! that targets Windows but does not call the host's own: the sandbox, and another OS's host.
//!
//! Reservations are real host pages (`host_pages`), so access the program did not ask for still
//! faults. Each reservation remembers every page's state for `VirtualProtect`'s old protection
//! and for refusing to protect pages that were never committed. Only whole reservations are
//! freed; decommitting a range is refused.
use super::host_pages::{self, Access};
use super::sandbox::guest::{fill, write_u16, write_u32, write_u64};
use std::collections::BTreeMap;

pub(super) const MEM_COMMIT: u64 = 0x1000;
pub(super) const MEM_RESERVE: u64 = 0x2000;
pub(super) const MEM_RELEASE: u64 = 0x8000;
pub(super) const PAGE_NOACCESS: u32 = 0x01;
pub(super) const PAGE_READONLY: u32 = 0x02;
pub(super) const PAGE_READWRITE: u32 = 0x04;

const ERROR_NOT_ENOUGH_MEMORY: u64 = 8;
const ERROR_INVALID_PARAMETER: u64 = 87;
const ERROR_INVALID_ADDRESS: u64 = 487;
const ERROR_NOACCESS: u64 = 998;

/// The most address space one program reserves in total, as for the sandbox's `mmap`.
pub(super) const MAX_RESERVED_BYTES: u64 = 1 << 30;

/// `SYSTEM_INFO` on 64-bit Windows.
const SYSTEM_INFO_SIZE: usize = 48;

/// `PROCESSOR_ARCHITECTURE_UNKNOWN`: the program's CPU is not the host's business here.
const ARCHITECTURE_UNKNOWN: u16 = 0xFFFF;

/// One `VirtualAlloc` reservation and the state of each of its pages.
struct Reservation {
    base: u64,
    len: u64,
    /// `None` for reserved pages, else the committed page's `PAGE_*` protection.
    pages: Vec<Option<u32>>,
}

impl Drop for Reservation {
    fn drop(&mut self) {
        host_pages::release(self.base as usize, self.len as usize);
    }
}

#[derive(Default)]
pub(super) struct VirtualMemory {
    reservations: BTreeMap<u64, Reservation>,
    reserved_bytes: u64,
}

fn access(protection: u64) -> Result<Access, u64> {
    match protection as u32 {
        PAGE_NOACCESS => Ok(Access::None),
        PAGE_READONLY => Ok(Access::Read),
        PAGE_READWRITE => Ok(Access::ReadWrite),
        // Executable, guard and caching variants.
        _ => Err(ERROR_INVALID_PARAMETER),
    }
}

fn page() -> u64 {
    host_pages::page_size() as u64
}

impl VirtualMemory {
    /// `GetSystemInfo`: the host's page size, one processor (as `sysconf` says in the sandbox).
    pub(super) fn system_info(out: u64) {
        fill(out, 0, SYSTEM_INFO_SIZE);
        write_u16(out, ARCHITECTURE_UNKNOWN);
        write_u32(out + 4, page() as u32);
        write_u64(out + 8, host_pages::granularity() as u64);
        write_u64(out + 16, 0x7FFF_FFFE_FFFF);
        write_u64(out + 24, 1);
        write_u32(out + 32, 1);
        write_u32(out + 40, host_pages::granularity() as u32);
    }

    /// `VirtualAlloc(address, size, kind, protection)`: the start of the pages, or an error.
    pub(super) fn alloc(
        &mut self,
        address: u64,
        size: u64,
        kind: u64,
        protection: u64,
    ) -> Result<u64, u64> {
        let access = access(protection)?;
        if size == 0 || kind & !(MEM_COMMIT | MEM_RESERVE) != 0 || kind == 0 {
            return Err(ERROR_INVALID_PARAMETER);
        }
        // Committing with no address reserves too, as on Windows.
        if kind & MEM_RESERVE != 0 || address == 0 {
            if address != 0 {
                // Reserving at a chosen address is not offered.
                return Err(ERROR_INVALID_ADDRESS);
            }
            return self.reserve(size, (kind & MEM_COMMIT != 0).then_some(access));
        }
        let (base, start, end) = self.pages(address, size)?;
        let reservation = self.reservations.get_mut(&base).expect("found");
        let from = base + start as u64 * page();
        let len = (end - start) as u64 * page();
        if !host_pages::commit(from as usize, len as usize, access) {
            return Err(ERROR_NOT_ENOUGH_MEMORY);
        }
        reservation.pages[start..end].fill(Some(protection as u32));
        Ok(from)
    }

    fn reserve(&mut self, size: u64, commit: Option<Access>) -> Result<u64, u64> {
        let granule = host_pages::granularity() as u64;
        let len = size
            .checked_next_multiple_of(granule)
            .filter(|&len| self.reserved_bytes.saturating_add(len) <= MAX_RESERVED_BYTES)
            .ok_or(ERROR_NOT_ENOUGH_MEMORY)?;
        let base = host_pages::reserve(len as usize).ok_or(ERROR_NOT_ENOUGH_MEMORY)? as u64;
        let mut reservation = Reservation {
            base,
            len,
            pages: vec![None; (len / page()) as usize],
        };
        if let Some(access) = commit {
            if !host_pages::commit(base as usize, len as usize, access) {
                return Err(ERROR_NOT_ENOUGH_MEMORY);
            }
            let protection = match access {
                Access::None => PAGE_NOACCESS,
                Access::Read => PAGE_READONLY,
                Access::ReadWrite => PAGE_READWRITE,
            };
            reservation.pages.fill(Some(protection));
        }
        self.reserved_bytes += len;
        self.reservations.insert(base, reservation);
        Ok(base)
    }

    /// `VirtualProtect(address, size, protection, old)`: every page must be committed.
    pub(super) fn protect(
        &mut self,
        address: u64,
        size: u64,
        protection: u64,
        old: u64,
    ) -> Result<u64, u64> {
        let access = access(protection)?;
        if old == 0 {
            return Err(ERROR_NOACCESS);
        }
        let (base, start, end) = self.pages(address, size)?;
        let reservation = self.reservations.get_mut(&base).expect("found");
        let pages = &mut reservation.pages[start..end];
        let Some(Some(previous)) = pages.first().copied() else {
            return Err(ERROR_INVALID_ADDRESS);
        };
        if pages.contains(&None) {
            return Err(ERROR_INVALID_ADDRESS);
        }
        let from = base + start as u64 * page();
        let len = (end - start) as u64 * page();
        if !host_pages::protect(from as usize, len as usize, access) {
            return Err(ERROR_INVALID_ADDRESS);
        }
        pages.fill(Some(protection as u32));
        write_u32(old, previous);
        Ok(1)
    }

    /// `VirtualFree(address, size, kind)`: only `MEM_RELEASE` of a whole reservation.
    pub(super) fn free(&mut self, address: u64, size: u64, kind: u64) -> Result<u64, u64> {
        if kind != MEM_RELEASE || size != 0 {
            return Err(ERROR_INVALID_PARAMETER);
        }
        let reservation = self
            .reservations
            .remove(&address)
            .ok_or(ERROR_INVALID_ADDRESS)?;
        self.reserved_bytes -= reservation.len;
        Ok(1)
    }

    /// The reservation holding `address..address + size` and the page indexes it covers.
    fn pages(&self, address: u64, size: u64) -> Result<(u64, usize, usize), u64> {
        let page = page();
        let end = address
            .checked_add(size.max(1))
            .and_then(|end| end.checked_next_multiple_of(page))
            .ok_or(ERROR_INVALID_ADDRESS)?;
        let start = address / page * page;
        let (&base, reservation) = self
            .reservations
            .range(..=start)
            .next_back()
            .ok_or(ERROR_INVALID_ADDRESS)?;
        if end > base + reservation.len {
            return Err(ERROR_INVALID_ADDRESS);
        }
        Ok((
            base,
            ((start - base) / page) as usize,
            ((end - base) / page) as usize,
        ))
    }
}
