//! The `__jaic_*` names the checker and the runtime support module agree on.
//!
//! Each is declared in `stdlib/internal/runtime` (or synthesised by the compiler) under exactly
//! this spelling; the checker looks it up or defines it by this constant, so a rename is one edit
//! here plus the Jai side.

/// `Runtime_Support`'s failed-check handler, called by bounds, null and overflow checks.
pub(super) const CHECK_FAILED: &str = "__jaic_check_failed";

/// The handler a runtime that opted in calls when an `assert` fails.
pub(super) const INTERPRETER_ASSERTION: &str = "__jaic_interpreter_assertion";

/// The library-less `#foreign` global the compiler defines with the build directory.
pub(super) const BUILD_DIRECTORY: &str = "__jaic_build_directory";

/// The private intrinsic that reads the workspace's temporary-storage size.
pub(super) const RUNTIME_TEMPORARY_STORAGE_SIZE: &str = "__jaic_runtime_temporary_storage_size";

/// Label for the temporary-storage size when a diagnostic names where it was set.
pub(super) const TEMPORARY_STORAGE_SIZE_LABEL: &str =
    "__jaic_runtime_configuration.temporary_storage_size";

/// The global lock behind `lock cmpxchg16b`-style pair exchanges.
pub(super) const ASM_CMPXCHG_LOCK: &str = "__jaic_asm_cmpxchg_lock";

/// The global holding the AES S-box table.
pub(super) const ASM_AES_SBOX: &str = "__jaic_asm_aes_sbox";
