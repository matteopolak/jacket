//! The code generation backend as a library opened on first use.
//!
//! A release `jaic` does not link LLVM. Code generation (`jaic-llvm`) lives in `libjaic_backend`
//! beside the executable, and [`load`] opens it when a build first needs native code, so `-check`,
//! `-interp`, the language server and the linter never pay for loading LLVM.
//!
//! The two sides share an `ir::Program` by pointer, with no serialization. That is only sound when
//! both were built from the same sources by the same compiler, so the library exports a stamp
//! ([`stamp`]: crate version, `rustc` release, a hash of the IR and option sources, the sizes of
//! the shared types) and [`load`] refuses a library whose stamp differs.
//!
//! Nothing that owns memory crosses back: the library reports object paths and errors through the
//! driver's closure, which copies them, so each side frees only what its own allocator made.
#![allow(unsafe_code)]

use crate::options::Options;
use jaic::ir::Program;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// What the library exports. The stamp is read first; only a matching library is asked to emit.
pub const STAMP_SYMBOL: &str = "jaic_backend_stamp";

pub const EMIT_SYMBOL: &str = "jaic_backend_emit";

/// `unsafe extern "C" fn(len: *mut usize) -> *const u8`: the stamp's bytes.
type StampFn = unsafe extern "C" fn(*mut usize) -> *const u8;

/// `unsafe extern "C" fn(request: *mut EmitRequest) -> bool`: whether it wrote the objects.
type EmitFn = unsafe extern "C" fn(*mut c_void) -> bool;

/// What the driver and the library must agree on, as one string.
pub fn stamp() -> &'static str {
    static STAMP: OnceLock<String> = OnceLock::new();
    STAMP.get_or_init(|| {
        format!(
            "{}/{}-{}-{}",
            env!("BACKEND_STAMP_AT_BUILD"),
            size_of::<Program>(),
            size_of::<Options>(),
            size_of::<EmitRequest<'_>>(),
        )
    })
}

/// One request for object files.
pub struct EmitRequest<'a> {
    pub program: &'a Program,
    pub options: &'a Options,
    pub path: &'a Path,
    /// One object (`emit_object`) or as many as the program splits into (`emit_objects`).
    pub single: bool,
    /// The driver's thread pool size, for the library's own copy of the pool.
    pub threads: usize,
    pub reply: &'a mut dyn FnMut(Reply<'_>),
}

/// What the library tells the driver.
pub enum Reply<'a> {
    Object(&'a Path),
    Failed(&'a str),
}

/// The library's side of [`EmitFn`]: run the request with the two emitters, turning a panic into
/// a failure so it does not unwind through the driver's frames.
///
/// # Safety
/// `request` is the `EmitRequest` the driver passed to the exported entry point.
pub unsafe fn serve(
    request: *mut c_void,
    one: fn(&Program, &Options, &Path) -> Result<(), String>,
    many: fn(&Program, &Options, &Path) -> Result<Vec<PathBuf>, String>,
) -> bool {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::Mutex;
    static PANIC: Mutex<String> = Mutex::new(String::new());
    // SAFETY: the caller passes the driver's request, alive for this call.
    let request = unsafe { &mut *request.cast::<EmitRequest<'_>>() };
    std::panic::set_hook(Box::new(|info| {
        if let Ok(mut text) = PANIC.lock() {
            *text = info.to_string();
        }
    }));
    jaic::pool::configure(request.threads);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        if request.single {
            one(request.program, request.options, request.path).map(|()| vec![request.path.into()])
        } else {
            many(request.program, request.options, request.path)
        }
    }));
    let failure = match outcome {
        Ok(Ok(objects)) => {
            for object in &objects {
                (request.reply)(Reply::Object(object));
            }
            return true;
        }
        Ok(Err(message)) => message,
        Err(_) => {
            let text = PANIC.lock().map(|t| t.clone()).unwrap_or_default();
            format!("the code generation backend panicked: {text}")
        }
    };
    (request.reply)(Reply::Failed(&failure));
    false
}

/// The opened library.
pub struct Backend {
    emit: EmitFn,
}

static BACKEND: OnceLock<Result<Backend, String>> = OnceLock::new();

/// Open the library once; every later call returns the same result.
///
/// # Errors
/// When no library is found, it cannot be opened, or its stamp differs from this driver's.
pub fn load() -> Result<&'static Backend, String> {
    BACKEND.get_or_init(open).as_ref().map_err(Clone::clone)
}

/// The file name of the library on this platform.
pub fn library_name() -> &'static str {
    if cfg!(windows) {
        "jaic_backend.dll"
    } else if cfg!(target_os = "macos") {
        "libjaic_backend.dylib"
    } else {
        "libjaic_backend.so"
    }
}

/// `JAIC_BACKEND`, else the library beside the executable.
fn candidates() -> Vec<PathBuf> {
    if let Some(path) = jacket_config::env::BACKEND.non_empty() {
        return vec![PathBuf::from(path)];
    }
    let exe = std::env::current_exe().ok();
    let exe = exe
        .as_deref()
        .and_then(|e| e.canonicalize().ok().or(Some(e.to_path_buf())));
    exe.and_then(|e| e.parent().map(|d| d.join(library_name())))
        .into_iter()
        .collect()
}

fn open() -> Result<Backend, String> {
    let candidates = candidates();
    let Some(path) = candidates.iter().find(|p| p.is_file()) else {
        let looked: Vec<String> = candidates
            .iter()
            .map(|p| format!("`{}`", p.display()))
            .collect();
        return Err(format!(
            "this jaic cannot write native code: the code generation library `{}` was not found (looked at {})\nhelp: reinstall jaic, or set JAIC_BACKEND to the library's path",
            library_name(),
            looked.join(", ")
        ));
    };
    load_from(path)
}

/// Open the library at `path`, refusing one built for another jaic.
///
/// # Errors
/// When the file cannot be opened, is no code generation library, or has a different stamp.
pub fn load_from(path: &Path) -> Result<Backend, String> {
    let library = sys::open(path).map_err(|e| {
        format!(
            "could not open the code generation library `{}`: {e}",
            path.display()
        )
    })?;
    let not_ours = |e: String| {
        format!(
            "`{}` is not a jaic code generation library: {e}",
            path.display()
        )
    };
    // SAFETY: the symbols have the signatures declared above in every library built from these
    // sources; the stamp is checked before the emit function is used.
    let (theirs, emit) = unsafe {
        let stamp: StampFn =
            std::mem::transmute(sys::symbol(library, STAMP_SYMBOL).map_err(not_ours)?);
        let emit: EmitFn =
            std::mem::transmute(sys::symbol(library, EMIT_SYMBOL).map_err(not_ours)?);
        let mut len = 0;
        let bytes = stamp(&raw mut len);
        (
            String::from_utf8_lossy(std::slice::from_raw_parts(bytes, len)).into_owned(),
            emit,
        )
    };
    if theirs != stamp() {
        return Err(format!(
            "the code generation library `{}` was built for a different jaic\nhelp: it is `{theirs}`, this jaic is `{}`; install the library that came with this jaic",
            path.display(),
            stamp()
        ));
    }
    Ok(Backend {
        emit,
    })
}

impl Backend {
    fn run(
        &self,
        program: &Program,
        options: &Options,
        path: &Path,
        single: bool,
    ) -> Result<Vec<PathBuf>, String> {
        let mut objects = Vec::new();
        let mut failure = None;
        let mut reply = |reply: Reply<'_>| match reply {
            Reply::Object(object) => objects.push(object.to_path_buf()),
            Reply::Failed(message) => failure = Some(message.to_owned()),
        };
        let mut request = EmitRequest {
            program,
            options,
            path,
            single,
            threads: jaic::pool::threads(),
            reply: &mut reply,
        };
        // SAFETY: the library passed the stamp check, so it reads the same request layout.
        let ok = unsafe { (self.emit)(std::ptr::from_mut(&mut request).cast()) };
        match (ok, failure) {
            (true, _) => Ok(objects),
            (false, Some(message)) => Err(message),
            (false, None) => Err("the code generation backend failed without a message".into()),
        }
    }

    /// Write `program` as the one object file `path`.
    ///
    /// # Errors
    /// The backend's message when code generation fails.
    pub fn emit_object(
        &self,
        program: &Program,
        options: &Options,
        path: &Path,
    ) -> Result<(), String> {
        self.run(program, options, path, true).map(drop)
    }

    /// Write `program` as `path` and, for a program split into units, `path.1.o`, `path.2.o`...
    ///
    /// # Errors
    /// The backend's message when code generation fails.
    pub fn emit_objects(
        &self,
        program: &Program,
        options: &Options,
        path: &Path,
    ) -> Result<Vec<PathBuf>, String> {
        self.run(program, options, path, false)
    }
}

#[cfg(unix)]
mod sys {
    use std::ffi::{CStr, CString, c_char, c_int, c_void};
    use std::path::Path;

    unsafe extern "C" {
        fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
        fn dlerror() -> *const c_char;
    }

    const RTLD_NOW: c_int = 2;

    #[derive(Clone, Copy)]
    pub struct Library(*mut c_void);

    fn last_error() -> String {
        // SAFETY: `dlerror` returns null or a C string valid until the next dl call.
        let text = unsafe { dlerror() };
        if text.is_null() {
            "unknown error".into()
        } else {
            // SAFETY: non-null, NUL-terminated.
            unsafe { CStr::from_ptr(text) }
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn open(path: &Path) -> Result<Library, String> {
        let name = CString::new(path.as_os_str().as_encoded_bytes()).map_err(|e| e.to_string())?;
        // SAFETY: `name` is a NUL-terminated path.
        let handle = unsafe { dlopen(name.as_ptr(), RTLD_NOW) };
        if handle.is_null() {
            Err(last_error())
        } else {
            Ok(Library(handle))
        }
    }

    pub fn symbol(library: Library, name: &str) -> Result<*mut c_void, String> {
        let name = CString::new(name).map_err(|e| e.to_string())?;
        // SAFETY: a live handle and a NUL-terminated name.
        let address = unsafe { dlsym(library.0, name.as_ptr()) };
        if address.is_null() {
            Err(last_error())
        } else {
            Ok(address)
        }
    }
}

#[cfg(windows)]
mod sys {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    unsafe extern "system" {
        fn LoadLibraryExW(name: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    }

    /// Look for the library's own dependencies beside it.
    const LOAD_WITH_ALTERED_SEARCH_PATH: u32 = 8;

    #[derive(Clone, Copy)]
    pub struct Library(*mut c_void);

    pub fn open(path: &Path) -> Result<Library, String> {
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // SAFETY: `wide` is a NUL-terminated path.
        let handle = unsafe {
            LoadLibraryExW(
                wide.as_ptr(),
                std::ptr::null_mut(),
                LOAD_WITH_ALTERED_SEARCH_PATH,
            )
        };
        if handle.is_null() {
            Err(std::io::Error::last_os_error().to_string())
        } else {
            Ok(Library(handle))
        }
    }

    pub fn symbol(library: Library, name: &str) -> Result<*mut c_void, String> {
        let mut bytes = name.as_bytes().to_vec();
        bytes.push(0);
        // SAFETY: a live module and a NUL-terminated name.
        let address = unsafe { GetProcAddress(library.0, bytes.as_ptr()) };
        if address.is_null() {
            Err(std::io::Error::last_os_error().to_string())
        } else {
            Ok(address)
        }
    }
}
