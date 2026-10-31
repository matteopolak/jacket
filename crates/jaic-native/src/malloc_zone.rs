//! Makes `fork` safe in a process whose C allocator is mimalloc's macOS malloc zone.
//!
//! The zone declares itself version 10, and libmalloc calls the version-9 `reinit_lock` hook of
//! every zone in the child after `fork`, but the zone leaves that slot null: the child crashes
//! before it can `exec`. A compile-time `run_command` forks, so this fills the slot with a no-op.
//! (mimalloc keeps no zone lock to reinitialise.)

#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_void};

/// Index of `reinit_lock` among the pointer-sized fields of `malloc_introspection_t`
/// (enumerator, `good_size`, check, print, log, `force_lock`, `force_unlock`, statistics,
/// `zone_locked`, then the four discharge-checking hooks).
const REINIT_LOCK: usize = 13;

/// Indices among the pointer-sized fields of `malloc_zone_t` (the version follows `introspect`).
const ZONE_NAME: usize = 9;

const INTROSPECT: usize = 12;
const VERSION: usize = 13;

unsafe extern "C" {
    fn malloc_get_all_zones(
        task: u32,
        reader: *const c_void,
        addresses: *mut *const usize,
        count: *mut u32,
    ) -> i32;
}

extern "C" fn no_reinit(_zone: *mut c_void) {
}

/// Fill `reinit_lock` of mimalloc's zone, when one is registered and lacks the hook.
pub fn fix_fork() {
    let mut zones: *const usize = std::ptr::null();
    let mut count = 0u32;
    // SAFETY: with a null task and reader, `malloc_get_all_zones` lists this process's live
    // zones; each is the frozen `malloc_zone_t` ABI, whose name is checked before the zone is
    // touched, and mimalloc's introspection table is a writable static.
    unsafe {
        if malloc_get_all_zones(0, std::ptr::null(), &raw mut zones, &raw mut count) != 0
            || zones.is_null()
        {
            return;
        }
        for i in 0..count as usize {
            let zone = *zones.add(i) as *mut *mut c_void;
            let name = *zone.add(ZONE_NAME) as *const c_char;
            if name.is_null() || CStr::from_ptr(name).to_bytes() != b"mimalloc" {
                continue;
            }
            let version = *(zone.add(VERSION) as *const u32);
            let introspect = (*zone.add(INTROSPECT)).cast::<*mut c_void>();
            if version >= 9 && !introspect.is_null() && (*introspect.add(REINIT_LOCK)).is_null() {
                *introspect.add(REINIT_LOCK) = no_reinit as *mut c_void;
            }
        }
    }
}
