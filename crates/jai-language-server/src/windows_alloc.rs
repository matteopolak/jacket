//! The Windows allocator: mimalloc, set to hand freed pages back to the system at once.
//!
//! The default process heap keeps freed pages committed, so jailsp held its compile peaks. mimalloc
//! alone purges lazily and ends no lower; with the purge delay at 0 and a collect after each
//! compile (`release_free_memory`), both the idle working set and the peak drop. Unix does the
//! same job with `large_alloc`.

/// The allocator `main.rs` installs on Windows.
pub type Allocator = mimalloc::MiMalloc;

/// `mi_option_purge_delay` in the bundled `mimalloc.h` (v2 and v3): the 16th `mi_option_e`
/// entry. `libmimalloc-sys` does not export it.
const PURGE_DELAY: libmimalloc_sys::mi_option_t = 15;

/// Purge freed pages immediately instead of after mimalloc's default 10 ms delay. Call before the
/// first compile.
#[allow(unsafe_code)]
pub fn purge_immediately() {
    // SAFETY: setting an option is allowed at any time; it only changes later purges.
    unsafe { libmimalloc_sys::mi_option_set(PURGE_DELAY, 0) }
}

/// Return the pages that compiles freed to the system.
#[allow(unsafe_code)]
pub fn release_free_memory() {
    // SAFETY: `mi_collect` only walks mimalloc's own heaps.
    unsafe { libmimalloc_sys::mi_collect(true) }
}
