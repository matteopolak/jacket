//! Native semaphore waits whose host identity is known before yielding the interpreter.
#![allow(unsafe_code)]

mod provider;

#[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
unsafe extern "C" {
    fn sem_wait(semaphore: *mut std::ffi::c_void) -> i32;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn sem_timedwait(semaphore: *mut std::ffi::c_void, deadline: *const std::ffi::c_void) -> i32;

    #[cfg(target_os = "macos")]
    fn semaphore_wait(semaphore: u32) -> i32;

    #[cfg(target_os = "macos")]
    fn semaphore_timedwait(semaphore: u32, interval: MachTimespec) -> i32;
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct MachTimespec {
    seconds: u32,
    nanoseconds: i32,
}

fn host_address(symbol: &str) -> Option<u64> {
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
    if symbol == "sem_wait" {
        return Some(sem_wait as *const () as usize as u64);
    }
    #[cfg(any(target_os = "linux", target_os = "android"))]
    if symbol == "sem_timedwait" {
        return Some(sem_timedwait as *const () as usize as u64);
    }
    #[cfg(target_os = "macos")]
    match symbol {
        "semaphore_wait" => return Some(semaphore_wait as *const () as usize as u64),
        "semaphore_timedwait" => return Some(semaphore_timedwait as *const () as usize as u64),
        _ => {}
    }
    let _ = symbol;
    None
}

#[derive(Clone, Copy)]
struct ProviderAddress(u64);

impl ProviderAddress {
    fn verify(address: u64) -> Option<Self> {
        provider::trusted(address).then_some(Self(address))
    }
}

const SYMBOLS: [&str; 4] = [
    "sem_wait",
    "sem_timedwait",
    "semaphore_wait",
    "semaphore_timedwait",
];

static VERIFIED: [std::sync::OnceLock<Option<ProviderAddress>>; 4] =
    [const { std::sync::OnceLock::new() }; 4];

/// Prepare native provider proofs before guest code can observe loader error state.
/// Once complete, both classification and later interpreter construction are read-only queries.
pub(in crate::interp) fn prepare() {
    static PREPARED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    PREPARED.get_or_init(|| {
        let _error = crate::interp::last_error::keep();
        for (index, symbol) in SYMBOLS.iter().enumerate() {
            VERIFIED[index].get_or_init(|| host_address(symbol).and_then(ProviderAddress::verify));
        }
        provider::clear_inspection_error();
    });
}

/// Only host waits from the loader's actual core-C provider qualify. Imported function pointers
/// and `RTLD_DEFAULT` can both be interposed, so their equality alone does not establish origin.
pub(in crate::interp) fn known(
    library: Option<&crate::ir::Library>,
    symbol: &str,
    address: u64,
) -> bool {
    let Some(index) = SYMBOLS.iter().position(|name| *name == symbol) else {
        return false;
    };
    library.is_some_and(|library| {
        library.system
            && matches!(library.name.as_str(), "c" | "libc" | "pthread")
            && accepted_binding(address, VERIFIED[index].get().copied().flatten())
    })
}

fn accepted_binding(address: u64, sdk: Option<ProviderAddress>) -> bool {
    sdk.is_some_and(|sdk| sdk.0 == address)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library(name: &str, system: bool) -> crate::ir::Library {
        crate::ir::Library {
            name: name.into(),
            system,
            link_always: false,
            base_dir: String::new(),
        }
    }

    #[test]
    fn equal_interposed_addresses_do_not_establish_a_system_provider() {
        extern "C" fn user_wait(_: *mut std::ffi::c_void) -> i32 {
            0
        }
        let interposed = user_wait as *const () as usize as u64;
        // Model both imports resolving to the same interposer, then query its real image origin.
        let imported_address = interposed;
        let resolved_address = interposed;
        assert!(!accepted_binding(
            resolved_address,
            ProviderAddress::verify(imported_address)
        ));
    }

    #[cfg(any(target_os = "macos", all(target_os = "linux", target_env = "gnu")))]
    #[test]
    fn classification_does_not_replace_a_guests_dynamic_loader_error() {
        unsafe extern "C" {
            fn dlerror() -> *const std::ffi::c_char;
        }
        let _interp = crate::interp::Interp::default();
        let system = library("libc", true);
        let symbol = "sem_wait";
        let address = host_address(symbol).unwrap();
        // SAFETY: the fixed symbol deliberately does not exist; the loader owns error storage.
        let missing = unsafe {
            super::super::sys::dlsym(
                super::super::sys::RTLD_DEFAULT,
                c"jaic_missing_wait_provenance_probe".as_ptr(),
            )
        };
        assert!(missing.is_null());
        let _ = known(Some(&system), symbol, address);
        // SAFETY: read this calling thread's pending loader error before any further query.
        let error = unsafe { dlerror() };
        assert!(
            !error.is_null(),
            "classification cleared the guest's dlerror state"
        );
        // SAFETY: a non-null dlerror result is a loader-owned NUL-terminated string.
        let message = unsafe { std::ffi::CStr::from_ptr(error) };
        assert!(
            message
                .to_string_lossy()
                .contains("jaic_missing_wait_provenance_probe")
        );
    }

    #[test]
    fn only_the_host_sdk_address_and_system_binding_select_a_wait() {
        prepare();
        let system = library("libc", true);
        for symbol in [
            "sem_wait",
            "sem_timedwait",
            "semaphore_wait",
            "semaphore_timedwait",
        ] {
            let Some(address) = host_address(symbol) else {
                continue;
            };
            assert_eq!(
                known(Some(&system), symbol, address),
                provider::trusted(address)
            );
            assert!(!known(Some(&system), symbol, address ^ 1));
            assert!(!known(Some(&library("libc", false)), symbol, address));
            assert!(!known(Some(&library("user", true)), symbol, address));
            assert!(!known(None, symbol, address));
            // Native loading must resolve the same implementation whose address the SDK names.
            let loaded = super::super::Library::open("libc", true, "").unwrap();
            assert_eq!(super::super::lookup(Some(&loaded), symbol), Some(address));
        }
        assert!(!known(Some(&system), "sem_trywait", 17));
        assert!(!known(Some(&system), "opaque_user_wait", 17));
    }
}
