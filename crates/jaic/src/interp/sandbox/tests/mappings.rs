//! The sandbox's private anonymous mappings: `mmap`, `mprotect` and `munmap`.
use super::*;

/// A 32-bit Linux program's `mmap` links to `mmap64` (64-bit file offsets); the sandbox answers both.
#[test]
fn mmap64_maps_like_mmap() {
    let mut h = host();
    h.darwin_files = cfg!(target_os = "macos");
    let flags = if h.darwin_files {
        2 | 0x1000
    } else {
        2 | 0x20
    };
    let page = virtual_memory::page_size();
    let at = call(&mut h, "mmap64", &[0, page, 3, flags, u64::MAX, 0]);
    assert_ne!(at, u64::MAX);
    assert_eq!(call(&mut h, "munmap", &[at, page]), 0);
}

/// Unmapping the middle of a mapping keeps both ends and their bytes; the hole is no longer
/// the program's.
#[test]
fn unmapping_part_of_a_mapping_keeps_the_rest() {
    let mut h = host();
    h.darwin_files = cfg!(target_os = "macos");
    let flags = if h.darwin_files {
        2 | 0x1000
    } else {
        2 | 0x20
    };
    let page = virtual_memory::page_size();
    let at = call(&mut h, "mmap", &[0, 3 * page, 3, flags, u64::MAX, 0]);
    assert_ne!(at, u64::MAX);
    let byte = |addr: u64| addr as *mut u8;
    // SAFETY: the three pages were just mapped readable and writable.
    unsafe {
        assert_eq!(*byte(at + page), 0);
        *byte(at) = 7;
        *byte(at + 2 * page) = 9;
    }
    assert_eq!(call(&mut h, "munmap", &[at + page, page]), 0);
    // SAFETY: the first and last pages are still mapped.
    unsafe {
        assert_eq!(*byte(at), 7);
        assert_eq!(*byte(at + 2 * page), 9);
    }
    assert_eq!(call(&mut h, "mprotect", &[at + page, page, 1]), u64::MAX);
    assert_eq!(errno(&h), EINVAL as u32);
    assert_eq!(call(&mut h, "mprotect", &[at + 2 * page, page, 1]), 0);
    assert_eq!(call(&mut h, "munmap", &[at, page]), 0);
    assert_eq!(call(&mut h, "munmap", &[at + 2 * page, page]), 0);
}

#[test]
fn anonymous_mappings_share_one_total_cap_per_host() {
    const ENOMEM: u32 = 12;
    let mut h = host();
    h.darwin_files = cfg!(target_os = "macos");
    let flags = if h.darwin_files {
        2 | 0x1000
    } else {
        2 | 0x20
    };
    let map = |h: &mut SandboxHost, len: u64| call(h, "mmap", &[0, len, 3, flags, u64::MAX, 0]);
    let page = virtual_memory::page_size();
    // Without a lower limit the cap is `MAX_MAPPED_BYTES`; a refused request maps nothing.
    assert_eq!(
        map(&mut h, virtual_memory::MAX_MAPPED_BYTES + page),
        u64::MAX
    );
    assert_eq!(errno(&h), ENOMEM);
    // A cap of a few pages: Windows commits what is mapped, and a test should not commit the
    // real 1 GiB cap.
    let half = 2 * page;
    h.map_limit = Some(2 * half);

    // One request over the cap is refused outright.
    assert_eq!(map(&mut h, 2 * half + page), u64::MAX);
    assert_eq!(errno(&h), ENOMEM);

    let first = map(&mut h, half);
    let second = map(&mut h, half);
    assert_ne!(first, u64::MAX);
    assert_ne!(second, u64::MAX);
    // The total, not each request, is capped.
    assert_eq!(map(&mut h, page), u64::MAX);
    assert_eq!(errno(&h), ENOMEM);

    // Unmapping gives the bytes back, whole or in part.
    assert_eq!(call(&mut h, "munmap", &[first, half]), 0);
    let again = map(&mut h, page);
    assert_ne!(again, u64::MAX);
    assert_eq!(call(&mut h, "munmap", &[second, page]), 0);
    assert_eq!(call(&mut h, "munmap", &[again, page]), 0);
}
