//! Handle-based bridge to the system `libclang`, used by `stdlib/Bindings_Generator`.
//!
//! The interpreter's foreign calls pass scalars only: libclang's cursors and
//! types are structs passed and returned by value, and `clang_visitChildren`
//! needs a native callback. This module loads `libclang` itself, keeps every
//! cursor and type it hands out in an arena, and exposes them to Jai code as
//! small integer handles through one primitive (`__jaic_clang`). Equal cursors
//! (and equal types) always get the same handle, so handles can be compared and
//! used as hash keys. Handles are valid until the translation unit is disposed.
//!
//! Only the libclang found on the host is used (`JAI_LIBCLANG`, the Xcode and
//! Command Line Tools copies, Homebrew LLVM, then the system loader), preferring
//! one new enough to report base class offsets (see `open_api`).
#![allow(unsafe_code)]

use crate::fxhash::HashMap;
use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_ulong, c_void};

#[repr(C)]
#[derive(Clone, Copy)]
struct CxString {
    data: *const c_void,
    flags: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct CxCursor {
    kind: c_int,
    xdata: c_int,
    data: [*const c_void; 3],
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct CxType {
    kind: c_int,
    data: [*const c_void; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CxLocation {
    ptr: [*const c_void; 2],
    int_data: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CxRange {
    ptr: [*const c_void; 2],
    begin: c_uint,
    end: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CxToken {
    int_data: [c_uint; 4],
    ptr: *mut c_void,
}

#[repr(C)]
struct CxUnsaved {
    filename: *const c_char,
    contents: *const c_char,
    length: c_ulong,
}

type Visitor = extern "C" fn(CxCursor, CxCursor, *mut c_void) -> c_int;

#[cfg(unix)]
unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

#[cfg(windows)]
unsafe extern "system" {
    fn LoadLibraryA(name: *const c_char) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
}

#[cfg(windows)]
unsafe fn dlopen(filename: *const c_char, _flags: c_int) -> *mut c_void {
    // SAFETY: the caller passes a NUL-terminated path, as `dlopen` requires.
    unsafe { LoadLibraryA(filename) }
}

#[cfg(windows)]
unsafe fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
    // SAFETY: the caller passes a live module handle and a NUL-terminated symbol name, as
    // `dlsym` requires.
    unsafe { GetProcAddress(handle, symbol.cast()) }
}

/// No dynamic loading in the browser build: libclang is never found.
#[cfg(not(any(unix, windows)))]
unsafe fn dlopen(_filename: *const c_char, _flags: c_int) -> *mut c_void {
    std::ptr::null_mut()
}

#[cfg(not(any(unix, windows)))]
unsafe fn dlsym(_handle: *mut c_void, _symbol: *const c_char) -> *mut c_void {
    std::ptr::null_mut()
}

/// Declares `Api` from a table of `field: "symbol" => signature` rows: the struct of function
/// pointers and the loader that resolves them, so a field and its symbol cannot drift apart.
macro_rules! libclang_api {
    ($($field:ident: $symbol:literal => fn($($arg:ty),* $(,)?) $(-> $ret:ty)?,)*) => {
        /// The libclang entry points the bridge calls.
        struct Api {
            $($field: unsafe extern "C" fn($($arg),*) $(-> $ret)?,)*
            /// `clang_getOffsetOfBase`: libclang 20+; older ones leave virtual base offsets
            /// unknown.
            offset_of_base: Option<unsafe extern "C" fn(CxCursor, CxCursor) -> i64>,
        }

        impl Api {
            /// Resolves every entry point in `library`; the error names the first missing symbol.
            fn resolve(library: &Library, path: &str) -> Result<Api, String> {
                Ok(Api {
                    $($field: {
                        let address = library.symbol($symbol);
                        if address.is_null() {
                            return Err(format!("{path}: missing symbol {}", $symbol));
                        }
                        // SAFETY: `$symbol` is the libclang C function whose signature is the
                        // field type declared in this table, and a function pointer is the same
                        // size as the `*mut c_void` that `dlsym` returned.
                        unsafe {
                            std::mem::transmute::<*mut c_void, unsafe extern "C" fn($($arg),*) $(-> $ret)?>(
                                address,
                            )
                        }
                    },)*
                    offset_of_base: {
                        let address = library.symbol("clang_getOffsetOfBase");
                        if address.is_null() {
                            None
                        } else {
                            // SAFETY: as above, for `clang_getOffsetOfBase(CXCursor, CXCursor)`.
                            Some(unsafe {
                                std::mem::transmute::<
                                    *mut c_void,
                                    unsafe extern "C" fn(CxCursor, CxCursor) -> i64,
                                >(address)
                            })
                        }
                    },
                })
            }
        }
    };
}

libclang_api! {
    create_index: "clang_createIndex" => fn(c_int, c_int) -> *mut c_void,
    dispose_index: "clang_disposeIndex" => fn(*mut c_void),
    parse: "clang_parseTranslationUnit2" => fn(
        *mut c_void,
        *const c_char,
        *const *const c_char,
        c_int,
        *mut CxUnsaved,
        c_uint,
        c_uint,
        *mut *mut c_void,
    ) -> c_int,
    dispose_tu: "clang_disposeTranslationUnit" => fn(*mut c_void),
    num_diagnostics: "clang_getNumDiagnostics" => fn(*mut c_void) -> c_uint,
    get_diagnostic: "clang_getDiagnostic" => fn(*mut c_void, c_uint) -> *mut c_void,
    diagnostic_severity: "clang_getDiagnosticSeverity" => fn(*mut c_void) -> c_int,
    format_diagnostic: "clang_formatDiagnostic" => fn(*mut c_void, c_uint) -> CxString,
    default_diagnostic_options: "clang_defaultDiagnosticDisplayOptions" => fn() -> c_uint,
    dispose_diagnostic: "clang_disposeDiagnostic" => fn(*mut c_void),
    get_cstring: "clang_getCString" => fn(CxString) -> *const c_char,
    dispose_string: "clang_disposeString" => fn(CxString),
    tu_cursor: "clang_getTranslationUnitCursor" => fn(*mut c_void) -> CxCursor,
    visit_children: "clang_visitChildren" => fn(CxCursor, Visitor, *mut c_void) -> c_uint,
    cursor_kind: "clang_getCursorKind" => fn(CxCursor) -> c_int,
    cursor_spelling: "clang_getCursorSpelling" => fn(CxCursor) -> CxString,
    cursor_type: "clang_getCursorType" => fn(CxCursor) -> CxType,
    cursor_location: "clang_getCursorLocation" => fn(CxCursor) -> CxLocation,
    cursor_extent: "clang_getCursorExtent" => fn(CxCursor) -> CxRange,
    spelling_location: "clang_getSpellingLocation" => fn(
        CxLocation,
        *mut *mut c_void,
        *mut c_uint,
        *mut c_uint,
        *mut c_uint,
    ),
    file_name: "clang_getFileName" => fn(*mut c_void) -> CxString,
    in_system_header: "clang_Location_isInSystemHeader" => fn(CxLocation) -> c_int,
    cursor_linkage: "clang_getCursorLinkage" => fn(CxCursor) -> c_int,
    storage_class: "clang_Cursor_getStorageClass" => fn(CxCursor) -> c_int,
    cursor_definition: "clang_getCursorDefinition" => fn(CxCursor) -> CxCursor,
    canonical_cursor: "clang_getCanonicalCursor" => fn(CxCursor) -> CxCursor,
    cursor_referenced: "clang_getCursorReferenced" => fn(CxCursor) -> CxCursor,
    semantic_parent: "clang_getCursorSemanticParent" => fn(CxCursor) -> CxCursor,
    is_definition: "clang_isCursorDefinition" => fn(CxCursor) -> c_uint,
    enum_value: "clang_getEnumConstantDeclValue" => fn(CxCursor) -> i64,
    enum_uvalue: "clang_getEnumConstantDeclUnsignedValue" => fn(CxCursor) -> u64,
    enum_int_type: "clang_getEnumDeclIntegerType" => fn(CxCursor) -> CxType,
    typedef_underlying: "clang_getTypedefDeclUnderlyingType" => fn(CxCursor) -> CxType,
    is_bitfield: "clang_Cursor_isBitField" => fn(CxCursor) -> c_uint,
    bit_width: "clang_getFieldDeclBitWidth" => fn(CxCursor) -> c_int,
    is_anonymous: "clang_Cursor_isAnonymous" => fn(CxCursor) -> c_uint,
    is_anonymous_record: "clang_Cursor_isAnonymousRecordDecl" => fn(CxCursor) -> c_uint,
    num_arguments: "clang_Cursor_getNumArguments" => fn(CxCursor) -> c_int,
    argument: "clang_Cursor_getArgument" => fn(CxCursor, c_uint) -> CxCursor,
    cursor_variadic: "clang_Cursor_isVariadic" => fn(CxCursor) -> c_uint,
    raw_comment: "clang_Cursor_getRawCommentText" => fn(CxCursor) -> CxString,
    comment_range: "clang_Cursor_getCommentRange" => fn(CxCursor) -> CxRange,
    range_start: "clang_getRangeStart" => fn(CxRange) -> CxLocation,
    macro_function_like: "clang_Cursor_isMacroFunctionLike" => fn(CxCursor) -> c_uint,
    macro_builtin: "clang_Cursor_isMacroBuiltin" => fn(CxCursor) -> c_uint,
    mangling: "clang_Cursor_getMangling" => fn(CxCursor) -> CxString,
    field_offset: "clang_Cursor_getOffsetOfField" => fn(CxCursor) -> i64,
    tokenize: "clang_tokenize" => fn(*mut c_void, CxRange, *mut *mut CxToken, *mut c_uint),
    dispose_tokens: "clang_disposeTokens" => fn(*mut c_void, *mut CxToken, c_uint),
    token_spelling: "clang_getTokenSpelling" => fn(*mut c_void, CxToken) -> CxString,
    token_kind: "clang_getTokenKind" => fn(CxToken) -> c_int,
    evaluate: "clang_Cursor_Evaluate" => fn(CxCursor) -> *mut c_void,
    eval_kind: "clang_EvalResult_getKind" => fn(*mut c_void) -> c_int,
    eval_int: "clang_EvalResult_getAsLongLong" => fn(*mut c_void) -> i64,
    eval_unsigned: "clang_EvalResult_isUnsignedInt" => fn(*mut c_void) -> c_uint,
    eval_unsigned_value: "clang_EvalResult_getAsUnsigned" => fn(*mut c_void) -> u64,
    eval_double: "clang_EvalResult_getAsDouble" => fn(*mut c_void) -> f64,
    eval_str: "clang_EvalResult_getAsStr" => fn(*mut c_void) -> *const c_char,
    eval_dispose: "clang_EvalResult_dispose" => fn(*mut c_void),
    type_spelling: "clang_getTypeSpelling" => fn(CxType) -> CxString,
    pointee: "clang_getPointeeType" => fn(CxType) -> CxType,
    result_type: "clang_getResultType" => fn(CxType) -> CxType,
    num_arg_types: "clang_getNumArgTypes" => fn(CxType) -> c_int,
    arg_type: "clang_getArgType" => fn(CxType, c_uint) -> CxType,
    type_variadic: "clang_isFunctionTypeVariadic" => fn(CxType) -> c_uint,
    array_element: "clang_getArrayElementType" => fn(CxType) -> CxType,
    array_size: "clang_getArraySize" => fn(CxType) -> i64,
    canonical_type: "clang_getCanonicalType" => fn(CxType) -> CxType,
    type_declaration: "clang_getTypeDeclaration" => fn(CxType) -> CxCursor,
    size_of: "clang_Type_getSizeOf" => fn(CxType) -> i64,
    align_of: "clang_Type_getAlignOf" => fn(CxType) -> i64,
    is_const: "clang_isConstQualifiedType" => fn(CxType) -> c_uint,
    named_type: "clang_Type_getNamedType" => fn(CxType) -> CxType,
    type_offset_of: "clang_Type_getOffsetOf" => fn(CxType, *const c_char) -> i64,
    num_template_args: "clang_Type_getNumTemplateArguments" => fn(CxType) -> c_int,
    cxx_method_static: "clang_CXXMethod_isStatic" => fn(CxCursor) -> c_uint,
    cxx_method_virtual: "clang_CXXMethod_isVirtual" => fn(CxCursor) -> c_uint,
    cxx_method_pure_virtual: "clang_CXXMethod_isPureVirtual" => fn(CxCursor) -> c_uint,
    cxx_method_const: "clang_CXXMethod_isConst" => fn(CxCursor) -> c_uint,
    ctor_copy: "clang_CXXConstructor_isCopyConstructor" => fn(CxCursor) -> c_uint,
    ctor_move: "clang_CXXConstructor_isMoveConstructor" => fn(CxCursor) -> c_uint,
    cxx_manglings: "clang_Cursor_getCXXManglings" => fn(CxCursor) -> *mut CxStringSet,
    dispose_string_set: "clang_disposeStringSet" => fn(*mut CxStringSet),
    specialized_template: "clang_getSpecializedCursorTemplate" => fn(CxCursor) -> CxCursor,
    template_arg_type: "clang_Type_getTemplateArgumentAsType" => fn(CxType, c_uint) -> CxType,
    access_specifier: "clang_getCXXAccessSpecifier" => fn(CxCursor) -> c_int,
    function_inlined: "clang_Cursor_isFunctionInlined" => fn(CxCursor) -> c_uint,
    cursor_result_type: "clang_getCursorResultType" => fn(CxCursor) -> CxType,
    objc_base_type: "clang_Type_getObjCObjectBaseType" => fn(CxType) -> CxType,
    objc_num_protocols: "clang_Type_getNumObjCProtocolRefs" => fn(CxType) -> c_uint,
    objc_protocol: "clang_Type_getObjCProtocolDecl" => fn(CxType, c_uint) -> CxCursor,
    objc_num_type_args: "clang_Type_getNumObjCTypeArgs" => fn(CxType) -> c_uint,
    objc_type_arg: "clang_Type_getObjCTypeArg" => fn(CxType, c_uint) -> CxType,
    modified_type: "clang_Type_getModifiedType" => fn(CxType) -> CxType,
    is_virtual_base: "clang_isVirtualBase" => fn(CxCursor) -> c_uint,
}

/// A loaded dynamic library. It is never unloaded, so addresses resolved from it stay valid for
/// the life of the process.
struct Library(*mut c_void);

impl Library {
    /// Loads the library at `path` (`None` when it does not load).
    fn open(path: &CStr) -> Option<Library> {
        // SAFETY: `path` is NUL-terminated. Loading libclang runs its initializers, which is the
        // point of this module; 2 is `RTLD_NOW`.
        let handle = unsafe { dlopen(path.as_ptr(), 2) };
        (!handle.is_null()).then_some(Library(handle))
    }

    /// The address of the exported `symbol`, or null.
    fn symbol(&self, symbol: &str) -> *mut c_void {
        let name = CString::new(symbol).expect("libclang symbol names contain no NUL");
        // SAFETY: `self.0` is a live handle from `dlopen` (see `Library`) and `name` is
        // NUL-terminated.
        unsafe { dlsym(self.0, name.as_ptr()) }
    }
}

#[repr(C)]
struct CxStringSet {
    strings: *mut CxString,
    count: c_uint,
}

struct State {
    api: Api,
    path: String,
    index: *mut c_void,
    tu: *mut c_void,
    args: Vec<CString>,
    file_name: CString,
    file_contents: Vec<u8>,
    cursors: Vec<CxCursor>,
    cursor_ids: HashMap<(c_int, c_int, usize, usize, usize), u32>,
    types: Vec<CxType>,
    type_ids: HashMap<(c_int, usize, usize), u32>,
    last_text: Vec<u8>,
    last_count: i64,
    tokens: Vec<(i32, Vec<u8>)>,
    manglings: Vec<Vec<u8>>,
    lists: Vec<Vec<i64>>,
    eval: *mut c_void,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

/// Directories under `parent` named `{prefix}{version}` (Homebrew's `llvm@22`, Debian's
/// `llvm-18`), newest version first.
fn versioned_dirs(parent: &str, prefix: &str) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return Vec::new();
    };
    let mut dirs: Vec<(u32, std::path::PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let version = name.strip_prefix(prefix)?.split('.').next()?.parse().ok()?;
            Some((version, e.path()))
        })
        .collect();
    dirs.sort_by(|a, b| b.cmp(a));
    dirs.into_iter().map(|(_, path)| path).collect()
}

/// Where libclang is looked for when no path is given (`JAI_LIBCLANG` comes first).
fn candidates() -> Vec<String> {
    let mut list = Vec::new();
    let mut dirs: Vec<String> = [
        "/Library/Developer/CommandLineTools/usr/lib",
        "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib",
        "/opt/homebrew/opt/llvm/lib",
        "/usr/local/opt/llvm/lib",
    ]
    .map(String::from)
    .to_vec();
    // Keg-only Homebrew versions (`brew install llvm@22`) are not linked to `opt/llvm`.
    for parent in ["/opt/homebrew/opt", "/usr/local/opt"] {
        for dir in versioned_dirs(parent, "llvm@") {
            dirs.push(format!("{}/lib", dir.display()));
        }
    }
    dirs.extend(
        [
            "/usr/lib",
            "/usr/lib64",
            "/usr/lib/x86_64-linux-gnu",
            "/usr/lib/aarch64-linux-gnu",
        ]
        .map(String::from),
    );
    for dir in dirs {
        list.push(format!("{dir}/libclang.dylib"));
        list.push(format!("{dir}/libclang.so"));
    }
    for dir in versioned_dirs("/usr/lib", "llvm-") {
        // `libclang.so` comes with the -dev package; the runtime package has only `.so.1`.
        list.push(format!("{}/lib/libclang.so", dir.display()));
        list.push(format!("{}/lib/libclang.so.1", dir.display()));
    }
    list.push("libclang.dylib".into());
    list.push("libclang.so".into());
    if cfg!(windows) {
        // The LLVM installer's default location, then the DLL search path (`PATH`).
        list.push(r"C:\Program Files\LLVM\bin\libclang.dll".into());
        list.push("libclang.dll".into());
    }
    list
}

/// Loads libclang. An explicit path (the argument, then `JAI_LIBCLANG`) is used as is. Of the
/// usual install locations, the first libclang that reports base class offsets
/// (`clang_getOffsetOfBase`, libclang 20+) wins, so the generated bindings do not depend on
/// which toolchain happens to come first (Xcode 16's libclang predates it); without one, the
/// first that loads is used.
fn open_api(explicit: &str) -> Result<(Api, String), String> {
    let mut chosen: Vec<String> = Vec::new();
    if !explicit.is_empty() {
        chosen.push(explicit.to_string());
    }
    if let Ok(path) = std::env::var("JAI_LIBCLANG")
        && !path.is_empty()
    {
        chosen.push(path);
    }
    let mut error = None;
    for path in chosen {
        match load_api(&path) {
            Ok(Some(api)) => return Ok((api, path)),
            Ok(None) => {}
            Err(e) => error = error.or(Some(e)),
        }
    }
    // The first libclang that loads but predates `clang_getOffsetOfBase`: used only when no
    // installed one has it.
    let mut without_base_offsets = None;
    for path in candidates() {
        match load_api(&path) {
            Ok(Some(api)) if api.offset_of_base.is_some() => return Ok((api, path)),
            Ok(Some(api)) => {
                if without_base_offsets.is_none() {
                    without_base_offsets = Some((api, path));
                }
            }
            Ok(None) => {}
            Err(e) => error = error.or(Some(e)),
        }
    }
    if let Some(found) = without_base_offsets {
        return Ok(found);
    }
    Err(error.unwrap_or_else(|| "could not find a libclang (set JAI_LIBCLANG to its path)".into()))
}

/// The compiler's own headers (`stdarg.h`, ...) shipped beside a libclang at `lib/clang/<version>/include`.
/// A libclang that cannot place its resource directory (a Homebrew or conda LLVM loaded from another
/// prefix) would otherwise stop on the first `#include <stdarg.h>`.
fn builtin_include_dir(libclang: &str) -> Option<String> {
    let lib = crate::canonicalize(libclang).ok()?;
    let clang = lib.parent()?.join("clang");
    let mut found: Vec<_> = std::fs::read_dir(clang)
        .ok()?
        .flatten()
        .map(|e| e.path().join("include"))
        .filter(|p| p.join("stdarg.h").exists())
        .collect();
    found.sort();
    Some(found.pop()?.to_string_lossy().into_owned())
}

/// The entry points of the libclang at `path` (`None` when it does not exist or will not load).
fn load_api(path: &str) -> Result<Option<Api>, String> {
    let Ok(c) = CString::new(path) else {
        return Ok(None);
    };
    if std::path::Path::new(path).is_absolute() && !std::path::Path::new(path).exists() {
        return Ok(None);
    }
    let Some(library) = Library::open(&c) else {
        return Ok(None);
    };
    Api::resolve(&library, path).map(Some)
}

/// Cursor kinds `CXCursor_InvalidFile` through `CXCursor_InvalidCode`: what libclang answers
/// with when there is no such cursor.
fn is_invalid_cursor(c: &CxCursor) -> bool {
    (70..=73).contains(&c.kind)
}

impl State {
    fn new(api: Api, path: String) -> State {
        State {
            api,
            path,
            index: std::ptr::null_mut(),
            tu: std::ptr::null_mut(),
            args: Vec::new(),
            file_name: CString::default(),
            file_contents: Vec::new(),
            cursors: Vec::new(),
            cursor_ids: HashMap::default(),
            types: Vec::new(),
            type_ids: HashMap::default(),
            last_text: Vec::new(),
            last_count: 0,
            tokens: Vec::new(),
            manglings: Vec::new(),
            lists: Vec::new(),
            eval: std::ptr::null_mut(),
        }
    }

    /// The bytes of `s`, which is disposed.
    fn string(&self, s: CxString) -> Vec<u8> {
        // SAFETY: `s` was just returned by a libclang call, so `get_cstring` gives null or a
        // NUL-terminated string that lives until `dispose_string`, which consumes `s`.
        unsafe {
            let p = (self.api.get_cstring)(s);
            let out = if p.is_null() {
                Vec::new()
            } else {
                CStr::from_ptr(p).to_bytes().to_vec()
            };
            (self.api.dispose_string)(s);
            out
        }
    }

    fn cursor_id(&mut self, c: CxCursor) -> i64 {
        let key = (
            c.kind,
            c.xdata,
            c.data[0] as usize,
            c.data[1] as usize,
            c.data[2] as usize,
        );
        if let Some(&id) = self.cursor_ids.get(&key) {
            return i64::from(id);
        }
        self.cursors.push(c);
        let id = self.cursors.len() as u32;
        self.cursor_ids.insert(key, id);
        i64::from(id)
    }

    /// The handle for `c`, or 0 when libclang answered with "no such cursor".
    fn cursor_handle(&mut self, c: CxCursor) -> i64 {
        if is_invalid_cursor(&c) {
            0
        } else {
            self.cursor_id(c)
        }
    }

    fn type_id(&mut self, t: CxType) -> i64 {
        let key = (t.kind, t.data[0] as usize, t.data[1] as usize);
        if let Some(&id) = self.type_ids.get(&key) {
            return i64::from(id);
        }
        self.types.push(t);
        let id = self.types.len() as u32;
        self.type_ids.insert(key, id);
        i64::from(id)
    }

    fn cursor(&self, h: i64) -> CxCursor {
        self.cursors
            .get((h as usize).wrapping_sub(1))
            .copied()
            .unwrap_or(CxCursor {
                kind: 70, // CXCursor_InvalidFile
                xdata: 0,
                data: [std::ptr::null(); 3],
            })
    }

    fn ty(&self, h: i64) -> CxType {
        self.types
            .get((h as usize).wrapping_sub(1))
            .copied()
            .unwrap_or(CxType {
                kind: 0,
                data: [std::ptr::null(); 2],
            })
    }

    // The four `on_*` methods are where libclang queries on handles are made. They are safe to
    // call because `cursor`/`ty` only give values recorded since the last `clear_handles`, which
    // runs before the translation unit those values point into is disposed, or an invalid
    // placeholder that libclang answers with its "invalid" result. `query` is always one of
    // `self.api`'s entry points, and `arg` a plain value (an index or another handle's value).

    /// Asks libclang `query` about the cursor behind handle `h`.
    fn on_cursor<R>(&self, h: i64, query: unsafe extern "C" fn(CxCursor) -> R) -> R {
        // SAFETY: see the comment above `on_cursor`.
        unsafe { query(self.cursor(h)) }
    }

    /// Like `on_cursor`, with one more argument.
    fn on_cursor_with<A, R>(
        &self,
        h: i64,
        arg: A,
        query: unsafe extern "C" fn(CxCursor, A) -> R,
    ) -> R {
        // SAFETY: see the comment above `on_cursor`; `arg` is a plain value or a pointer the
        // caller keeps alive across the call.
        unsafe { query(self.cursor(h), arg) }
    }

    /// Asks libclang `query` about the type behind handle `h`.
    fn on_type<R>(&self, h: i64, query: unsafe extern "C" fn(CxType) -> R) -> R {
        // SAFETY: see the comment above `on_cursor`.
        unsafe { query(self.ty(h)) }
    }

    /// Like `on_type`, with one more argument.
    fn on_type_with<A, R>(&self, h: i64, arg: A, query: unsafe extern "C" fn(CxType, A) -> R) -> R {
        // SAFETY: see the comment above `on_cursor`; `arg` is a plain value or a pointer the
        // caller keeps alive across the call.
        unsafe { query(self.ty(h), arg) }
    }

    /// Asks libclang `query` about the current evaluation result, if there is one.
    fn on_eval<R>(&self, query: unsafe extern "C" fn(*mut c_void) -> R) -> Option<R> {
        // SAFETY: a non-null `eval` came from `evaluate` and is nulled when disposed.
        (!self.eval.is_null()).then(|| unsafe { query(self.eval) })
    }

    fn clear_handles(&mut self) {
        self.cursors.clear();
        self.cursor_ids.clear();
        self.types.clear();
        self.type_ids.clear();
        self.tokens.clear();
        self.lists.clear();
        self.release_eval();
    }

    fn release_eval(&mut self) {
        if !self.eval.is_null() {
            // SAFETY: `eval` came from `evaluate` and has not been disposed (it is nulled here).
            unsafe { (self.api.eval_dispose)(self.eval) };
            self.eval = std::ptr::null_mut();
        }
    }

    fn dispose(&mut self) {
        self.clear_handles();
        // SAFETY: `tu` and `index` came from `parse` and are each disposed once (nulled here);
        // the handles that pointed into the translation unit were just cleared.
        unsafe {
            if !self.tu.is_null() {
                (self.api.dispose_tu)(self.tu);
                self.tu = std::ptr::null_mut();
            }
            if !self.index.is_null() {
                (self.api.dispose_index)(self.index);
                self.index = std::ptr::null_mut();
            }
        }
    }

    /// Executes one bridge operation: the integer result, with any text result left in
    /// `last_text`.
    fn run(&mut self, op: &str, a: i64, b: i64, text: &[u8]) -> Result<i64, String> {
        if let Some(r) = self.session_op(op, a, b, text)? {
            return Ok(r);
        }
        if let Some(r) = self.cursor_op(op, a, b) {
            return Ok(r);
        }
        if let Some(r) = self.cursor_link_op(op, a, b) {
            return Ok(r);
        }
        if let Some(r) = self.source_op(op, a) {
            return Ok(r);
        }
        if let Some(r) = self.eval_op(op, a) {
            return Ok(r);
        }
        if let Some(r) = self.type_op(op, a, b, text)? {
            return Ok(r);
        }
        Err(format!("unknown clang operation `{op}`"))
    }

    /// Translation unit lifecycle, diagnostics and child lists.
    fn session_op(&mut self, op: &str, a: i64, b: i64, text: &[u8]) -> Result<Option<i64>, String> {
        let r = match op {
            "path" => {
                self.last_text = self.path.clone().into_bytes();
                0
            }
            "reset" => {
                self.dispose();
                self.args.clear();
                self.file_name = CString::default();
                self.file_contents.clear();
                0
            }
            "add_arg" => {
                self.args
                    .push(CString::new(text).map_err(|_| "argument contains NUL")?);
                0
            }
            "file" => {
                self.file_name = CString::new(text).map_err(|_| "file name contains NUL")?;
                0
            }
            "contents" => {
                self.file_contents = text.to_vec();
                0
            }
            "parse" => self.parse(),
            "diag_count" => self.diagnostic_count(),
            "diag" => self.diagnostic(a),
            "dispose" => {
                self.dispose();
                0
            }
            "last_count" => self.last_count,
            "children" => {
                let children = self.children(a);
                self.last_count = children.len() as i64;
                // Handles of consecutive new cursors are not consecutive when some
                // already existed, so hand them out through `child`.
                let ids: Vec<i64> = children.into_iter().map(|c| self.cursor_id(c)).collect();
                self.lists.push(ids);
                self.lists.len() as i64
            }
            "list_item" => self
                .lists
                .get((a as usize).wrapping_sub(1))
                .and_then(|l| l.get(b as usize))
                .copied()
                .unwrap_or(0),
            _ => return Ok(None),
        };
        Ok(Some(r))
    }

    /// Parses the file set up by `file`/`contents`/`add_arg`: the root cursor's handle, or 0.
    fn parse(&mut self) -> i64 {
        self.dispose();
        // SAFETY: `create_index` takes plain ints.
        self.index = unsafe { (self.api.create_index)(0, 0) };
        let mut args = self.args.clone();
        if let Some(dir) = builtin_include_dir(&self.path)
            && !args.iter().any(|a| a.to_bytes() == b"-resource-dir")
        {
            args.push(CString::new("-isystem").unwrap());
            args.push(CString::new(dir).unwrap());
        }
        let mut argv: Vec<*const c_char> = args.iter().map(|c| c.as_ptr()).collect();
        argv.push(self.file_name.as_ptr());
        let mut contents = self.file_contents.clone();
        contents.push(0);
        let mut unsaved = CxUnsaved {
            filename: self.file_name.as_ptr(),
            contents: contents.as_ptr().cast::<c_char>(),
            length: self.file_contents.len() as c_ulong,
        };
        let mut tu: *mut c_void = std::ptr::null_mut();
        // DetailedPreprocessingRecord (1) | KeepGoing (0x200)
        // SAFETY: `index` was just created; `argv` holds NUL-terminated strings owned by `args`
        // and `file_name`, and `unsaved` one NUL-terminated buffer, all alive until the call
        // returns; the out pointer is a valid local.
        let err = unsafe {
            (self.api.parse)(
                self.index,
                std::ptr::null(),
                argv.as_ptr(),
                argv.len() as c_int,
                &raw mut unsaved,
                1,
                1,
                &raw mut tu,
            )
        };
        self.tu = tu;
        if err != 0 || tu.is_null() {
            return 0;
        }
        // SAFETY: `tu` is the translation unit just parsed.
        let root = unsafe { (self.api.tu_cursor)(tu) };
        self.cursor_id(root)
    }

    fn diagnostic_count(&self) -> i64 {
        if self.tu.is_null() {
            return 0;
        }
        // SAFETY: `tu` is the live translation unit from the last `parse`.
        i64::from(unsafe { (self.api.num_diagnostics)(self.tu) })
    }

    /// Diagnostic `index`: its severity, with the formatted text left in `last_text`.
    fn diagnostic(&mut self, index: i64) -> i64 {
        if self.tu.is_null() {
            return 0;
        }
        // SAFETY: `tu` is the live translation unit from the last `parse`; the diagnostic is
        // disposed after its text is copied out.
        let (severity, message) = unsafe {
            let d = (self.api.get_diagnostic)(self.tu, index as c_uint);
            let severity = (self.api.diagnostic_severity)(d);
            let message = (self.api.format_diagnostic)(d, (self.api.default_diagnostic_options)());
            let text = self.string(message);
            (self.api.dispose_diagnostic)(d);
            (severity, text)
        };
        self.last_text = message;
        i64::from(severity)
    }

    /// The child cursors of cursor `h`.
    fn children(&self, h: i64) -> Vec<CxCursor> {
        let mut list: Vec<CxCursor> = Vec::new();
        // SAFETY: the cursor is valid (see `on_cursor`); `collect` is the visitor ABI and only
        // casts `data` back to the `list` that outlives this call.
        unsafe {
            (self.api.visit_children)(self.cursor(h), collect, (&raw mut list).cast::<c_void>());
        }
        list
    }

    /// The file name, line and column of `loc` (an empty name when it has no file).
    fn spelling_location(&self, loc: CxLocation) -> (Vec<u8>, c_uint, c_uint) {
        let (mut file, mut line, mut col, mut off) = (std::ptr::null_mut(), 0, 0, 0);
        // SAFETY: `loc` comes from a live cursor and the out pointers are valid locals; the
        // file, when there is one, belongs to the live translation unit.
        unsafe {
            (self.api.spelling_location)(
                loc,
                &raw mut file,
                &raw mut line,
                &raw mut col,
                &raw mut off,
            );
            let name = if file.is_null() {
                Vec::new()
            } else {
                self.string((self.api.file_name)(file))
            };
            (name, line, col)
        }
    }

    /// Cursor properties that are one libclang call on the cursor.
    fn cursor_op(&mut self, op: &str, a: i64, b: i64) -> Option<i64> {
        let r = match op {
            "kind" => i64::from(self.on_cursor(a, self.api.cursor_kind)),
            "linkage" => i64::from(self.on_cursor(a, self.api.cursor_linkage)),
            "storage_class" => i64::from(self.on_cursor(a, self.api.storage_class)),
            "is_definition" => i64::from(self.on_cursor(a, self.api.is_definition)),
            "enum_value" => self.on_cursor(a, self.api.enum_value),
            "enum_uvalue" => self.on_cursor(a, self.api.enum_uvalue) as i64,
            "is_bitfield" => i64::from(self.on_cursor(a, self.api.is_bitfield)),
            "bit_width" => i64::from(self.on_cursor(a, self.api.bit_width)),
            "is_anonymous" => i64::from(self.on_cursor(a, self.api.is_anonymous)),
            "is_anonymous_record" => i64::from(self.on_cursor(a, self.api.is_anonymous_record)),
            "num_args" => i64::from(self.on_cursor(a, self.api.num_arguments)),
            "is_variadic" => i64::from(self.on_cursor(a, self.api.cursor_variadic)),
            "macro_function_like" => i64::from(self.on_cursor(a, self.api.macro_function_like)),
            "macro_builtin" => i64::from(self.on_cursor(a, self.api.macro_builtin)),
            "field_offset" => self.on_cursor(a, self.api.field_offset),
            "is_virtual_base" => i64::from(self.on_cursor(a, self.api.is_virtual_base)),
            // Offset in bits of base specifier `b` in a complete object of record `a` (-1 unknown).
            "base_offset" => match self.api.offset_of_base {
                Some(f) => self.on_cursor_with(a, self.cursor(b), f),
                None => -1,
            },
            "is_static_method" => i64::from(self.on_cursor(a, self.api.cxx_method_static)),
            "is_virtual" => i64::from(self.on_cursor(a, self.api.cxx_method_virtual)),
            "is_pure_virtual" => i64::from(self.on_cursor(a, self.api.cxx_method_pure_virtual)),
            "is_const_method" => i64::from(self.on_cursor(a, self.api.cxx_method_const)),
            "is_copy_ctor" => i64::from(self.on_cursor(a, self.api.ctor_copy)),
            "is_move_ctor" => i64::from(self.on_cursor(a, self.api.ctor_move)),
            "is_inlined" => i64::from(self.on_cursor(a, self.api.function_inlined)),
            "access" => i64::from(self.on_cursor(a, self.api.access_specifier)),
            "spelling" => {
                let spelling = self.on_cursor(a, self.api.cursor_spelling);
                self.last_text = self.string(spelling);
                0
            }
            "comment" => {
                let comment = self.on_cursor(a, self.api.raw_comment);
                self.last_text = self.string(comment);
                0
            }
            "mangling" => {
                let mangling = self.on_cursor(a, self.api.mangling);
                self.last_text = self.string(mangling);
                0
            }
            "manglings" => self.cxx_manglings(a),
            "system_header" => {
                let loc = self.on_cursor(a, self.api.cursor_location);
                // SAFETY: `loc` is a location of a live cursor.
                i64::from(unsafe { (self.api.in_system_header)(loc) })
            }
            "location" => {
                let loc = self.on_cursor(a, self.api.cursor_location);
                let (file, line, col) = self.spelling_location(loc);
                if !file.is_empty() {
                    self.last_text = file;
                }
                self.last_count = i64::from(col);
                i64::from(line)
            }
            // Line where the cursor's attached comment starts (0 without one).
            "comment_line" => {
                let range = self.on_cursor(a, self.api.comment_range);
                // SAFETY: `range` is the comment range of a live cursor.
                let loc = unsafe { (self.api.range_start)(range) };
                i64::from(self.spelling_location(loc).1)
            }
            "tokenize" => self.tokenize(a),
            _ => return None,
        };
        Some(r)
    }

    /// Operations whose result is another cursor (0 when there is none), or its type.
    fn cursor_link_op(&mut self, op: &str, a: i64, b: i64) -> Option<i64> {
        let r = match op {
            "type" => {
                let t = self.on_cursor(a, self.api.cursor_type);
                self.type_id(t)
            }
            "definition" => {
                let c = self.on_cursor(a, self.api.cursor_definition);
                self.cursor_handle(c)
            }
            "canonical" => {
                let c = self.on_cursor(a, self.api.canonical_cursor);
                self.cursor_id(c)
            }
            "referenced" => {
                let c = self.on_cursor(a, self.api.cursor_referenced);
                self.cursor_handle(c)
            }
            "semantic_parent" => {
                let c = self.on_cursor(a, self.api.semantic_parent);
                self.cursor_handle(c)
            }
            // The template a specialization or instantiation was made from (0 if none).
            "specialized_template" => {
                let c = self.on_cursor(a, self.api.specialized_template);
                self.cursor_handle(c)
            }
            "enum_int_type" => {
                let t = self.on_cursor(a, self.api.enum_int_type);
                self.type_id(t)
            }
            "typedef_underlying" => {
                let t = self.on_cursor(a, self.api.typedef_underlying);
                self.type_id(t)
            }
            "arg" => {
                let c = self.on_cursor_with(a, b as c_uint, self.api.argument);
                self.cursor_id(c)
            }
            // Objective-C: the declared result type of a method cursor.
            "cursor_result_type" => {
                let t = self.on_cursor(a, self.api.cursor_result_type);
                self.type_id(t)
            }
            _ => return None,
        };
        Some(r)
    }

    /// All C++ manglings of a constructor/destructor (Itanium: C1/C2, D0/D1/D2): their count,
    /// to be read with `mangling_item`.
    fn cxx_manglings(&mut self, h: i64) -> i64 {
        self.manglings.clear();
        let set = self.on_cursor(h, self.api.cxx_manglings);
        if !set.is_null() {
            // SAFETY: a non-null set from libclang holds `count` strings, which it owns until
            // `dispose_string_set`; they are read without disposing and copied out first.
            unsafe {
                for i in 0..(*set).count as usize {
                    let p = (self.api.get_cstring)(*(*set).strings.add(i));
                    self.manglings.push(if p.is_null() {
                        Vec::new()
                    } else {
                        CStr::from_ptr(p).to_bytes().to_vec()
                    });
                }
                (self.api.dispose_string_set)(set);
            }
        }
        self.manglings.len() as i64
    }

    /// Tokenizes the extent of cursor `h`: the token count, with kinds and spellings kept for
    /// `token_kind`/`token`.
    fn tokenize(&mut self, h: i64) -> i64 {
        let range = self.on_cursor(h, self.api.cursor_extent);
        let mut tokens: *mut CxToken = std::ptr::null_mut();
        let mut count: c_uint = 0;
        self.tokens.clear();
        // SAFETY: `tu` is the live translation unit the cursor belongs to; libclang gives back
        // `count` tokens that stay valid until `dispose_tokens`, after they are copied out.
        unsafe {
            (self.api.tokenize)(self.tu, range, &raw mut tokens, &raw mut count);
            for i in 0..count as usize {
                let t = *tokens.add(i);
                let spelling = self.string((self.api.token_spelling)(self.tu, t));
                self.tokens.push(((self.api.token_kind)(t), spelling));
            }
            if !tokens.is_null() {
                (self.api.dispose_tokens)(self.tu, tokens, count);
            }
        }
        i64::from(count)
    }

    /// The tokens and source text produced by `tokenize`.
    fn source_op(&mut self, op: &str, a: i64) -> Option<i64> {
        let r = match op {
            "token_kind" => self.tokens.get(a as usize).map_or(0, |t| i64::from(t.0)),
            "token" => {
                self.last_text = self
                    .tokens
                    .get(a as usize)
                    .map(|t| t.1.clone())
                    .unwrap_or_default();
                0
            }
            "mangling_item" => {
                self.last_text = self.manglings.get(a as usize).cloned().unwrap_or_default();
                0
            }
            _ => return None,
        };
        Some(r)
    }

    /// Constant evaluation of a cursor.
    fn eval_op(&mut self, op: &str, a: i64) -> Option<i64> {
        let r = match op {
            "evaluate" => {
                self.release_eval();
                // SAFETY: the cursor is valid (see `on_cursor`).
                self.eval = unsafe { (self.api.evaluate)(self.cursor(a)) };
                self.on_eval(self.api.eval_kind).map_or(0, i64::from)
            }
            "eval_int" => {
                if self.on_eval(self.api.eval_unsigned).unwrap_or(0) != 0 {
                    self.on_eval(self.api.eval_unsigned_value).unwrap_or(0) as i64
                } else {
                    self.on_eval(self.api.eval_int).unwrap_or(0)
                }
            }
            "eval_is_unsigned" => self.on_eval(self.api.eval_unsigned).map_or(0, i64::from),
            "eval_double" => self
                .on_eval(self.api.eval_double)
                .map_or(0, |d| d.to_bits() as i64),
            "eval_string" => {
                if let Some(p) = self.on_eval(self.api.eval_str)
                    && !p.is_null()
                {
                    // SAFETY: libclang returns a NUL-terminated string owned by the evaluation
                    // result, which is alive here.
                    self.last_text = unsafe { CStr::from_ptr(p) }.to_bytes().to_vec();
                }
                0
            }
            _ => return None,
        };
        Some(r)
    }

    /// Type queries.
    fn type_op(&mut self, op: &str, a: i64, b: i64, text: &[u8]) -> Result<Option<i64>, String> {
        let r = match op {
            "t_kind" => i64::from(self.ty(a).kind),
            "t_spelling" => {
                let spelling = self.on_type(a, self.api.type_spelling);
                self.last_text = self.string(spelling);
                0
            }
            "t_pointee" => {
                let t = self.on_type(a, self.api.pointee);
                self.type_id(t)
            }
            "t_result" => {
                let t = self.on_type(a, self.api.result_type);
                self.type_id(t)
            }
            "t_num_args" => i64::from(self.on_type(a, self.api.num_arg_types)),
            "t_arg" => {
                let t = self.on_type_with(a, b as c_uint, self.api.arg_type);
                self.type_id(t)
            }
            "t_variadic" => i64::from(self.on_type(a, self.api.type_variadic)),
            "t_element" => {
                let t = self.on_type(a, self.api.array_element);
                self.type_id(t)
            }
            "t_array_size" => self.on_type(a, self.api.array_size),
            "t_canonical" => {
                let t = self.on_type(a, self.api.canonical_type);
                self.type_id(t)
            }
            "t_decl" => {
                let c = self.on_type(a, self.api.type_declaration);
                self.cursor_handle(c)
            }
            "t_sizeof" => self.on_type(a, self.api.size_of),
            "t_alignof" => self.on_type(a, self.api.align_of),
            "t_const" => i64::from(self.on_type(a, self.api.is_const)),
            "t_named" => {
                let t = self.on_type(a, self.api.named_type);
                self.type_id(t)
            }
            "t_offsetof" => {
                let name = CString::new(text).map_err(|_| "name contains NUL")?;
                self.on_type_with(a, name.as_ptr(), self.api.type_offset_of)
            }
            "t_template_arg" => {
                let t = self.on_type_with(a, b as c_uint, self.api.template_arg_type);
                self.type_id(t)
            }
            "t_num_template_args" => i64::from(self.on_type(a, self.api.num_template_args)),
            // Objective-C: the pieces of `id<P>` / `Base<Arg>` object types.
            "t_objc_base" => {
                let t = self.on_type(a, self.api.objc_base_type);
                self.type_id(t)
            }
            "t_objc_num_protocols" => i64::from(self.on_type(a, self.api.objc_num_protocols)),
            "t_objc_protocol" => {
                let c = self.on_type_with(a, b as c_uint, self.api.objc_protocol);
                self.cursor_handle(c)
            }
            "t_objc_num_type_args" => i64::from(self.on_type(a, self.api.objc_num_type_args)),
            "t_objc_type_arg" => {
                let t = self.on_type_with(a, b as c_uint, self.api.objc_type_arg);
                self.type_id(t)
            }
            // The type an attributed type (`T _Nonnull`) was made from.
            "t_modified" => {
                let t = self.on_type(a, self.api.modified_type);
                self.type_id(t)
            }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }
}

extern "C" fn collect(cursor: CxCursor, _parent: CxCursor, data: *mut c_void) -> c_int {
    // SAFETY: `data` is the `&mut Vec<CxCursor>` that `State::children` passed to
    // `visit_children`; it is alive and used by nobody else for the whole visit.
    let list = unsafe { &mut *data.cast::<Vec<CxCursor>>() };

    list.push(cursor);
    1 // CXChildVisit_Continue
}

/// Execute one bridge operation. Returns the integer result; text results are
/// kept for `last_text`.
pub fn call(op: &str, a: i64, b: i64, text: &[u8]) -> Result<i64, String> {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        if op == "load" {
            if slot.is_none() {
                let (api, path) = open_api(&String::from_utf8_lossy(text))?;
                let mut state = State::new(api, path);
                state.last_text = state.path.clone().into_bytes();
                *slot = Some(state);
            }
            return Ok(1);
        }
        let Some(s) = slot.as_mut() else {
            return Err("Bindings_Generator: libclang is not loaded".into());
        };
        s.last_text.clear();
        s.run(op, a, b, text)
    })
}

/// The text produced by the last operation.
pub fn last_text() -> Vec<u8> {
    STATE.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|s| s.last_text.clone())
            .unwrap_or_default()
    })
}
