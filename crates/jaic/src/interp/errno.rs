//! The `errno` values the interpreter reports to Jai programs.
//!
//! The low numbers agree on Linux and macOS. `ETIMEDOUT` does not: threads running natively
//! report the host's, while the sandbox emulates a Linux guest and always reports Linux's.

pub(super) const EPERM: u64 = 1;
pub(super) const ENOENT: u64 = 2;
pub(super) const EBADF: u64 = 9;
pub(super) const EBUSY: u64 = 16;
pub(super) const EEXIST: u64 = 17;
pub(super) const ENOTDIR: u64 = 20;
pub(super) const EISDIR: u64 = 21;
pub(super) const EINVAL: u64 = 22;
pub(super) const ENOTEMPTY: u64 = 39;

/// `ETIMEDOUT` as the sandbox's Linux guest sees it.
pub(super) const ETIMEDOUT_LINUX: u64 = 110;

/// `ETIMEDOUT` as the host the interpreter runs on defines it.
pub(super) const ETIMEDOUT_HOST: u64 = if cfg!(target_os = "macos") {
    60
} else {
    ETIMEDOUT_LINUX
};
