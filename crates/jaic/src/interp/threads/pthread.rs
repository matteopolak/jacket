//! Host SDK calls, reached only after scheduler identity and signature validation.
use super::Worker;

#[cfg(target_os = "macos")]
type Handle = *mut std::ffi::c_void;
#[cfg(any(target_os = "linux", target_os = "android"))]
type Handle = usize;

#[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
unsafe extern "C" {
    fn pthread_self() -> Handle;
    fn pthread_getname_np(thread: Handle, name: *mut u8, len: usize) -> i32;
    #[cfg(target_os = "macos")]
    fn pthread_setname_np(name: *const u8) -> i32;
    #[cfg(not(target_os = "macos"))]
    fn pthread_setname_np(thread: Handle, name: *const u8) -> i32;
}

pub(super) fn current() -> Option<u64> {
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
    // SAFETY: the SDK function has no arguments and returns this live OS thread.
    return Some(unsafe { pthread_self() } as u64);
    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos")))]
    None
}

pub(super) fn worker_identity(worker: &Worker) -> Option<u64> {
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
    {
        use std::os::unix::thread::JoinHandleExt;
        Some(worker.as_pthread_t() as u64)
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos")))]
    {
        let _ = worker;
        None
    }
}

pub(super) fn set_arity() -> Option<usize> {
    if cfg!(target_os = "macos") {
        Some(1)
    } else if cfg!(any(target_os = "linux", target_os = "android")) {
        Some(2)
    } else {
        None
    }
}

pub(super) fn name_capacity() -> usize {
    if cfg!(target_os = "macos") {
        64
    } else {
        16
    }
}

pub(super) fn set_name(handle: u64, name: &[u8]) -> u64 {
    debug_assert_eq!(name.last(), Some(&0));
    #[cfg(any(target_os = "linux", target_os = "android"))]
    // SAFETY: caller retains a validated live pthread and a terminated byte buffer.
    return unsafe { pthread_setname_np(handle as Handle, name.as_ptr()) } as u32 as u64;
    #[cfg(target_os = "macos")]
    {
        let _ = handle; // Darwin's public setter names only the current OS thread.
        // SAFETY: the caller validated the one-argument API and terminated buffer.
        unsafe { pthread_setname_np(name.as_ptr()) as u32 as u64 }
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos")))]
    unreachable!("unsupported host must be rejected before a naming call: {handle}")
}

pub(super) fn get_name(handle: u64, capacity: u64) -> (u64, Vec<u8>) {
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
    {
        // More space than the maximum host name needs cannot change a successful query.
        let len = capacity.min(name_capacity() as u64) as usize;
        let mut bytes = vec![0; len];
        // SAFETY: the caller retains the native thread; `bytes` owns `len` bytes.
        let status = unsafe { pthread_getname_np(handle as Handle, bytes.as_mut_ptr(), len) };
        if status == 0 {
            let end = bytes
                .iter()
                .position(|&byte| byte == 0)
                .expect("SDK names are terminated");
            bytes.truncate(end + 1);
        }
        (status as u32 as u64, bytes)
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos")))]
    unreachable!("unsupported host must be rejected before a naming call: {handle}, {capacity}")
}
