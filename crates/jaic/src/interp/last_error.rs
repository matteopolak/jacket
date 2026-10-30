//! The program's own `GetLastError` / `errno`, kept through the interpreter's work.
//!
//! The error value is per thread and shared with the interpreter itself: the program calls a
//! foreign procedure that fails, and its next step reads the code. If the interpreter does any
//! system call of its own in between, it overwrites that code. Making a procedure's frame on its
//! first call is such a moment (even reading an environment variable sets `ERROR_ENVVAR_NOT_FOUND` on
//! Windows), so `keep` guards it.

/// Restores the calling thread's last-error value to what it is now when dropped.
pub(super) fn keep() -> impl Drop {
    Saved(get())
}

/// Runs `work` and leaves the calling thread's last-error value as it was before.
#[cfg(test)]
fn preserving<T>(work: impl FnOnce() -> T) -> T {
    let _restore = keep();
    work()
}

struct Saved(u32);

impl Drop for Saved {
    fn drop(&mut self) {
        set(self.0);
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetLastError(code: u32);
    fn GetLastError() -> u32;
}

#[cfg(windows)]
fn get() -> u32 {
    // SAFETY: GetLastError only reads the calling thread's error slot.
    unsafe { GetLastError() }
}

#[cfg(windows)]
fn set(code: u32) {
    // SAFETY: SetLastError only writes the calling thread's error slot.
    unsafe { SetLastError(code) }
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
unsafe extern "C" {
    #[link_name = "__error"]
    fn errno_location() -> *mut i32;
}

#[cfg(any(target_os = "linux", target_os = "android"))]
unsafe extern "C" {
    #[link_name = "__errno_location"]
    fn errno_location() -> *mut i32;
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "android"
))]
fn get() -> u32 {
    // SAFETY: the location is the calling thread's own `errno`.
    unsafe { *errno_location() as u32 }
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "android"
))]
fn set(code: u32) {
    // SAFETY: as in `get`.
    unsafe { *errno_location() = code as i32 }
}

#[cfg(not(any(
    windows,
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "android"
)))]
fn get() -> u32 {
    0
}

#[cfg(not(any(
    windows,
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "android"
)))]
fn set(_code: u32) {
}

#[cfg(all(
    test,
    any(
        windows,
        target_os = "macos",
        target_os = "linux",
        target_os = "freebsd"
    )
))]
mod tests {
    use super::*;

    #[test]
    fn the_error_value_survives_work_that_changes_it() {
        set(2);
        let seen = preserving(|| {
            set(203);
            // The kind of call that reset it: a failed environment lookup.
            let _ = jacket_config::env::for_program("LAST_ERROR_TEST_UNSET");
            get()
        });
        assert_ne!(
            seen, 2,
            "the work must be able to overwrite it for this to test anything"
        );
        assert_eq!(get(), 2);
    }
}
