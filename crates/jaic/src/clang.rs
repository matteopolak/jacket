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

use std::cell::RefCell;
use std::collections::HashMap;
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
    unsafe { LoadLibraryA(filename) }
}
#[cfg(windows)]
unsafe fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
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

/// The libclang entry points the bridge calls.
struct Api {
    create_index: unsafe extern "C" fn(c_int, c_int) -> *mut c_void,
    dispose_index: unsafe extern "C" fn(*mut c_void),
    parse: unsafe extern "C" fn(
        *mut c_void,
        *const c_char,
        *const *const c_char,
        c_int,
        *mut CxUnsaved,
        c_uint,
        c_uint,
        *mut *mut c_void,
    ) -> c_int,
    dispose_tu: unsafe extern "C" fn(*mut c_void),
    num_diagnostics: unsafe extern "C" fn(*mut c_void) -> c_uint,
    get_diagnostic: unsafe extern "C" fn(*mut c_void, c_uint) -> *mut c_void,
    diagnostic_severity: unsafe extern "C" fn(*mut c_void) -> c_int,
    format_diagnostic: unsafe extern "C" fn(*mut c_void, c_uint) -> CxString,
    default_diagnostic_options: unsafe extern "C" fn() -> c_uint,
    dispose_diagnostic: unsafe extern "C" fn(*mut c_void),
    get_cstring: unsafe extern "C" fn(CxString) -> *const c_char,
    dispose_string: unsafe extern "C" fn(CxString),
    tu_cursor: unsafe extern "C" fn(*mut c_void) -> CxCursor,
    visit_children: unsafe extern "C" fn(CxCursor, Visitor, *mut c_void) -> c_uint,
    cursor_kind: unsafe extern "C" fn(CxCursor) -> c_int,
    cursor_spelling: unsafe extern "C" fn(CxCursor) -> CxString,
    cursor_type: unsafe extern "C" fn(CxCursor) -> CxType,
    cursor_location: unsafe extern "C" fn(CxCursor) -> CxLocation,
    cursor_extent: unsafe extern "C" fn(CxCursor) -> CxRange,
    spelling_location:
        unsafe extern "C" fn(CxLocation, *mut *mut c_void, *mut c_uint, *mut c_uint, *mut c_uint),
    file_name: unsafe extern "C" fn(*mut c_void) -> CxString,
    in_system_header: unsafe extern "C" fn(CxLocation) -> c_int,
    cursor_linkage: unsafe extern "C" fn(CxCursor) -> c_int,
    storage_class: unsafe extern "C" fn(CxCursor) -> c_int,
    cursor_definition: unsafe extern "C" fn(CxCursor) -> CxCursor,
    canonical_cursor: unsafe extern "C" fn(CxCursor) -> CxCursor,
    cursor_referenced: unsafe extern "C" fn(CxCursor) -> CxCursor,
    semantic_parent: unsafe extern "C" fn(CxCursor) -> CxCursor,
    is_definition: unsafe extern "C" fn(CxCursor) -> c_uint,
    enum_value: unsafe extern "C" fn(CxCursor) -> i64,
    enum_uvalue: unsafe extern "C" fn(CxCursor) -> u64,
    enum_int_type: unsafe extern "C" fn(CxCursor) -> CxType,
    typedef_underlying: unsafe extern "C" fn(CxCursor) -> CxType,
    is_bitfield: unsafe extern "C" fn(CxCursor) -> c_uint,
    bit_width: unsafe extern "C" fn(CxCursor) -> c_int,
    is_anonymous: unsafe extern "C" fn(CxCursor) -> c_uint,
    is_anonymous_record: unsafe extern "C" fn(CxCursor) -> c_uint,
    num_arguments: unsafe extern "C" fn(CxCursor) -> c_int,
    argument: unsafe extern "C" fn(CxCursor, c_uint) -> CxCursor,
    cursor_variadic: unsafe extern "C" fn(CxCursor) -> c_uint,
    raw_comment: unsafe extern "C" fn(CxCursor) -> CxString,
    comment_range: unsafe extern "C" fn(CxCursor) -> CxRange,
    range_start: unsafe extern "C" fn(CxRange) -> CxLocation,
    macro_function_like: unsafe extern "C" fn(CxCursor) -> c_uint,
    macro_builtin: unsafe extern "C" fn(CxCursor) -> c_uint,
    mangling: unsafe extern "C" fn(CxCursor) -> CxString,
    field_offset: unsafe extern "C" fn(CxCursor) -> i64,
    tokenize: unsafe extern "C" fn(*mut c_void, CxRange, *mut *mut CxToken, *mut c_uint),
    dispose_tokens: unsafe extern "C" fn(*mut c_void, *mut CxToken, c_uint),
    token_spelling: unsafe extern "C" fn(*mut c_void, CxToken) -> CxString,
    token_kind: unsafe extern "C" fn(CxToken) -> c_int,
    evaluate: unsafe extern "C" fn(CxCursor) -> *mut c_void,
    eval_kind: unsafe extern "C" fn(*mut c_void) -> c_int,
    eval_int: unsafe extern "C" fn(*mut c_void) -> i64,
    eval_unsigned: unsafe extern "C" fn(*mut c_void) -> c_uint,
    eval_unsigned_value: unsafe extern "C" fn(*mut c_void) -> u64,
    eval_double: unsafe extern "C" fn(*mut c_void) -> f64,
    eval_str: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    eval_dispose: unsafe extern "C" fn(*mut c_void),
    type_spelling: unsafe extern "C" fn(CxType) -> CxString,
    pointee: unsafe extern "C" fn(CxType) -> CxType,
    result_type: unsafe extern "C" fn(CxType) -> CxType,
    num_arg_types: unsafe extern "C" fn(CxType) -> c_int,
    arg_type: unsafe extern "C" fn(CxType, c_uint) -> CxType,
    type_variadic: unsafe extern "C" fn(CxType) -> c_uint,
    array_element: unsafe extern "C" fn(CxType) -> CxType,
    array_size: unsafe extern "C" fn(CxType) -> i64,
    canonical_type: unsafe extern "C" fn(CxType) -> CxType,
    type_declaration: unsafe extern "C" fn(CxType) -> CxCursor,
    size_of: unsafe extern "C" fn(CxType) -> i64,
    align_of: unsafe extern "C" fn(CxType) -> i64,
    is_const: unsafe extern "C" fn(CxType) -> c_uint,
    named_type: unsafe extern "C" fn(CxType) -> CxType,
    type_offset_of: unsafe extern "C" fn(CxType, *const c_char) -> i64,
    num_template_args: unsafe extern "C" fn(CxType) -> c_int,
    cxx_method_static: unsafe extern "C" fn(CxCursor) -> c_uint,
    cxx_method_virtual: unsafe extern "C" fn(CxCursor) -> c_uint,
    cxx_method_pure_virtual: unsafe extern "C" fn(CxCursor) -> c_uint,
    cxx_method_const: unsafe extern "C" fn(CxCursor) -> c_uint,
    ctor_copy: unsafe extern "C" fn(CxCursor) -> c_uint,
    ctor_move: unsafe extern "C" fn(CxCursor) -> c_uint,
    cxx_manglings: unsafe extern "C" fn(CxCursor) -> *mut CxStringSet,
    dispose_string_set: unsafe extern "C" fn(*mut CxStringSet),
    specialized_template: unsafe extern "C" fn(CxCursor) -> CxCursor,
    template_arg_type: unsafe extern "C" fn(CxType, c_uint) -> CxType,
    access_specifier: unsafe extern "C" fn(CxCursor) -> c_int,
    function_inlined: unsafe extern "C" fn(CxCursor) -> c_uint,
    cursor_result_type: unsafe extern "C" fn(CxCursor) -> CxType,
    objc_base_type: unsafe extern "C" fn(CxType) -> CxType,
    objc_num_protocols: unsafe extern "C" fn(CxType) -> c_uint,
    objc_protocol: unsafe extern "C" fn(CxType, c_uint) -> CxCursor,
    objc_num_type_args: unsafe extern "C" fn(CxType) -> c_uint,
    objc_type_arg: unsafe extern "C" fn(CxType, c_uint) -> CxType,
    modified_type: unsafe extern "C" fn(CxType) -> CxType,
    is_virtual_base: unsafe extern "C" fn(CxCursor) -> c_uint,
    /// libclang 20+; older ones leave virtual base offsets unknown.
    offset_of_base: Option<unsafe extern "C" fn(CxCursor, CxCursor) -> i64>,
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
    let mut fallback = None;
    for path in candidates() {
        match load_api(&path) {
            Ok(Some(api)) if api.offset_of_base.is_some() => return Ok((api, path)),
            Ok(Some(api)) => {
                if fallback.is_none() {
                    fallback = Some((api, path));
                }
            }
            Ok(None) => {}
            Err(e) => error = error.or(Some(e)),
        }
    }
    if let Some(found) = fallback {
        return Ok(found);
    }
    Err(error.unwrap_or_else(|| "could not find a libclang (set JAI_LIBCLANG to its path)".into()))
}

/// The entry points of the libclang at `path` (`None` when it does not exist or will not load).
fn load_api(path: &str) -> Result<Option<Api>, String> {
    {
        let Ok(c) = CString::new(path) else {
            return Ok(None);
        };
        if std::path::Path::new(path).is_absolute() && !std::path::Path::new(path).exists() {
            return Ok(None);
        }
        let handle = unsafe { dlopen(c.as_ptr(), 2) };
        if handle.is_null() {
            return Ok(None);
        }
        macro_rules! sym {
            ($name:literal) => {{
                let n = CString::new($name).unwrap();
                let p = unsafe { dlsym(handle, n.as_ptr()) };
                if p.is_null() {
                    return Err(format!("{path}: missing symbol {}", $name));
                }
                // The target is the `Api` field's function-pointer type, inferred per use.
                #[allow(clippy::missing_transmute_annotations)]
                let f = unsafe { std::mem::transmute(p) };
                f
            }};
        }
        let api = Api {
            create_index: sym!("clang_createIndex"),
            dispose_index: sym!("clang_disposeIndex"),
            parse: sym!("clang_parseTranslationUnit2"),
            dispose_tu: sym!("clang_disposeTranslationUnit"),
            num_diagnostics: sym!("clang_getNumDiagnostics"),
            get_diagnostic: sym!("clang_getDiagnostic"),
            diagnostic_severity: sym!("clang_getDiagnosticSeverity"),
            format_diagnostic: sym!("clang_formatDiagnostic"),
            default_diagnostic_options: sym!("clang_defaultDiagnosticDisplayOptions"),
            dispose_diagnostic: sym!("clang_disposeDiagnostic"),
            get_cstring: sym!("clang_getCString"),
            dispose_string: sym!("clang_disposeString"),
            tu_cursor: sym!("clang_getTranslationUnitCursor"),
            visit_children: sym!("clang_visitChildren"),
            cursor_kind: sym!("clang_getCursorKind"),
            cursor_spelling: sym!("clang_getCursorSpelling"),
            cursor_type: sym!("clang_getCursorType"),
            cursor_location: sym!("clang_getCursorLocation"),
            cursor_extent: sym!("clang_getCursorExtent"),
            spelling_location: sym!("clang_getSpellingLocation"),
            file_name: sym!("clang_getFileName"),
            in_system_header: sym!("clang_Location_isInSystemHeader"),
            cursor_linkage: sym!("clang_getCursorLinkage"),
            storage_class: sym!("clang_Cursor_getStorageClass"),
            cursor_definition: sym!("clang_getCursorDefinition"),
            canonical_cursor: sym!("clang_getCanonicalCursor"),
            cursor_referenced: sym!("clang_getCursorReferenced"),
            semantic_parent: sym!("clang_getCursorSemanticParent"),
            is_definition: sym!("clang_isCursorDefinition"),
            enum_value: sym!("clang_getEnumConstantDeclValue"),
            enum_uvalue: sym!("clang_getEnumConstantDeclUnsignedValue"),
            enum_int_type: sym!("clang_getEnumDeclIntegerType"),
            typedef_underlying: sym!("clang_getTypedefDeclUnderlyingType"),
            is_bitfield: sym!("clang_Cursor_isBitField"),
            bit_width: sym!("clang_getFieldDeclBitWidth"),
            is_anonymous: sym!("clang_Cursor_isAnonymous"),
            is_anonymous_record: sym!("clang_Cursor_isAnonymousRecordDecl"),
            num_arguments: sym!("clang_Cursor_getNumArguments"),
            argument: sym!("clang_Cursor_getArgument"),
            cursor_variadic: sym!("clang_Cursor_isVariadic"),
            raw_comment: sym!("clang_Cursor_getRawCommentText"),
            comment_range: sym!("clang_Cursor_getCommentRange"),
            range_start: sym!("clang_getRangeStart"),
            macro_function_like: sym!("clang_Cursor_isMacroFunctionLike"),
            macro_builtin: sym!("clang_Cursor_isMacroBuiltin"),
            mangling: sym!("clang_Cursor_getMangling"),
            field_offset: sym!("clang_Cursor_getOffsetOfField"),
            tokenize: sym!("clang_tokenize"),
            dispose_tokens: sym!("clang_disposeTokens"),
            token_spelling: sym!("clang_getTokenSpelling"),
            token_kind: sym!("clang_getTokenKind"),
            evaluate: sym!("clang_Cursor_Evaluate"),
            eval_kind: sym!("clang_EvalResult_getKind"),
            eval_int: sym!("clang_EvalResult_getAsLongLong"),
            eval_unsigned: sym!("clang_EvalResult_isUnsignedInt"),
            eval_unsigned_value: sym!("clang_EvalResult_getAsUnsigned"),
            eval_double: sym!("clang_EvalResult_getAsDouble"),
            eval_str: sym!("clang_EvalResult_getAsStr"),
            eval_dispose: sym!("clang_EvalResult_dispose"),
            type_spelling: sym!("clang_getTypeSpelling"),
            pointee: sym!("clang_getPointeeType"),
            result_type: sym!("clang_getResultType"),
            num_arg_types: sym!("clang_getNumArgTypes"),
            arg_type: sym!("clang_getArgType"),
            type_variadic: sym!("clang_isFunctionTypeVariadic"),
            array_element: sym!("clang_getArrayElementType"),
            array_size: sym!("clang_getArraySize"),
            canonical_type: sym!("clang_getCanonicalType"),
            type_declaration: sym!("clang_getTypeDeclaration"),
            size_of: sym!("clang_Type_getSizeOf"),
            align_of: sym!("clang_Type_getAlignOf"),
            is_const: sym!("clang_isConstQualifiedType"),
            named_type: sym!("clang_Type_getNamedType"),
            type_offset_of: sym!("clang_Type_getOffsetOf"),
            num_template_args: sym!("clang_Type_getNumTemplateArguments"),
            cxx_method_static: sym!("clang_CXXMethod_isStatic"),
            cxx_method_virtual: sym!("clang_CXXMethod_isVirtual"),
            cxx_method_pure_virtual: sym!("clang_CXXMethod_isPureVirtual"),
            cxx_method_const: sym!("clang_CXXMethod_isConst"),
            ctor_copy: sym!("clang_CXXConstructor_isCopyConstructor"),
            ctor_move: sym!("clang_CXXConstructor_isMoveConstructor"),
            cxx_manglings: sym!("clang_Cursor_getCXXManglings"),
            dispose_string_set: sym!("clang_disposeStringSet"),
            specialized_template: sym!("clang_getSpecializedCursorTemplate"),
            template_arg_type: sym!("clang_Type_getTemplateArgumentAsType"),
            access_specifier: sym!("clang_getCXXAccessSpecifier"),
            function_inlined: sym!("clang_Cursor_isFunctionInlined"),
            cursor_result_type: sym!("clang_getCursorResultType"),
            objc_base_type: sym!("clang_Type_getObjCObjectBaseType"),
            objc_num_protocols: sym!("clang_Type_getNumObjCProtocolRefs"),
            objc_protocol: sym!("clang_Type_getObjCProtocolDecl"),
            objc_num_type_args: sym!("clang_Type_getNumObjCTypeArgs"),
            objc_type_arg: sym!("clang_Type_getObjCTypeArg"),
            modified_type: sym!("clang_Type_getModifiedType"),
            is_virtual_base: sym!("clang_isVirtualBase"),
            offset_of_base: {
                let n = CString::new("clang_getOffsetOfBase").unwrap();
                let p = unsafe { dlsym(handle, n.as_ptr()) };
                if p.is_null() {
                    None
                } else {
                    #[allow(clippy::missing_transmute_annotations)]
                    Some(unsafe { std::mem::transmute(p) })
                }
            },
        };
        Ok(Some(api))
    }
}

impl State {
    fn string(&self, s: CxString) -> Vec<u8> {
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
            return id as i64;
        }
        self.cursors.push(c);
        let id = self.cursors.len() as u32;
        self.cursor_ids.insert(key, id);
        id as i64
    }

    fn type_id(&mut self, t: CxType) -> i64 {
        let key = (t.kind, t.data[0] as usize, t.data[1] as usize);
        if let Some(&id) = self.type_ids.get(&key) {
            return id as i64;
        }
        self.types.push(t);
        let id = self.types.len() as u32;
        self.type_ids.insert(key, id);
        id as i64
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
            unsafe { (self.api.eval_dispose)(self.eval) };
            self.eval = std::ptr::null_mut();
        }
    }

    fn dispose(&mut self) {
        self.clear_handles();
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
}

extern "C" fn collect(cursor: CxCursor, _parent: CxCursor, data: *mut c_void) -> c_int {
    let list = unsafe { &mut *(data as *mut Vec<CxCursor>) };
    list.push(cursor);
    1 // CXChildVisit_Continue
}

/// Execute one bridge operation. Returns the integer result; text results are
/// kept for `last_text`.
pub fn call(op: &str, a: i64, b: i64, text: &[u8]) -> Result<i64, String> {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        if op == "load" {
            if slot.is_some() {
                return Ok(1);
            }
            let (api, path) = open_api(&String::from_utf8_lossy(text))?;
            *slot = Some(State {
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
            });
            slot.as_mut().unwrap().last_text = slot.as_ref().unwrap().path.clone().into_bytes();
            return Ok(1);
        }
        let Some(s) = slot.as_mut() else {
            return Err("Bindings_Generator: libclang is not loaded".into());
        };
        s.last_text.clear();
        let api = &s.api as *const Api;
        // SAFETY: `api` points into `s`, which lives as long as this call and
        // is never moved while it runs; it only sidesteps borrowing `s` twice.
        let api = unsafe { &*api };
        let r = match op {
            "path" => {
                s.last_text = s.path.clone().into_bytes();
                0
            }
            "reset" => {
                s.dispose();
                s.args.clear();
                s.file_name = CString::default();
                s.file_contents.clear();
                0
            }
            "add_arg" => {
                s.args
                    .push(CString::new(text).map_err(|_| "argument contains NUL")?);
                0
            }
            "file" => {
                s.file_name = CString::new(text).map_err(|_| "file name contains NUL")?;
                0
            }
            "contents" => {
                s.file_contents = text.to_vec();
                0
            }
            "parse" => {
                s.dispose();
                unsafe {
                    s.index = (api.create_index)(0, 0);
                    let mut argv: Vec<*const c_char> = s.args.iter().map(|c| c.as_ptr()).collect();
                    argv.push(s.file_name.as_ptr());
                    let mut contents = s.file_contents.clone();
                    contents.push(0);
                    let mut unsaved = CxUnsaved {
                        filename: s.file_name.as_ptr(),
                        contents: contents.as_ptr() as *const c_char,
                        length: s.file_contents.len() as c_ulong,
                    };
                    let mut tu: *mut c_void = std::ptr::null_mut();
                    // DetailedPreprocessingRecord (1) | KeepGoing (0x200)
                    let err = (api.parse)(
                        s.index,
                        std::ptr::null(),
                        argv.as_ptr(),
                        argv.len() as c_int,
                        &mut unsaved,
                        1,
                        1,
                        &mut tu,
                    );
                    s.tu = tu;
                    if err != 0 || tu.is_null() {
                        return Ok(0);
                    }
                    let root = (api.tu_cursor)(tu);
                    s.cursor_id(root)
                }
            }
            "diag_count" => {
                if s.tu.is_null() {
                    0
                } else {
                    unsafe { (api.num_diagnostics)(s.tu) as i64 }
                }
            }
            "diag" => unsafe {
                let d = (api.get_diagnostic)(s.tu, a as c_uint);
                let severity = (api.diagnostic_severity)(d);
                let msg = (api.format_diagnostic)(d, (api.default_diagnostic_options)());
                s.last_text = s.string(msg);
                (api.dispose_diagnostic)(d);
                severity as i64
            },
            "dispose" => {
                s.dispose();
                0
            }
            "last_count" => s.last_count,
            "children" => {
                let c = s.cursor(a);
                let mut list: Vec<CxCursor> = Vec::new();
                unsafe { (api.visit_children)(c, collect, &mut list as *mut _ as *mut c_void) };
                s.last_count = list.len() as i64;
                // Handles of consecutive new cursors are not consecutive when some
                // already existed, so hand them out through `child`.
                let ids: Vec<i64> = list.into_iter().map(|c| s.cursor_id(c)).collect();
                s.lists.push(ids);
                s.lists.len() as i64
            }
            "list_item" => s
                .lists
                .get((a as usize).wrapping_sub(1))
                .and_then(|l| l.get(b as usize))
                .copied()
                .unwrap_or(0),
            // ----- cursors
            "kind" => unsafe { (api.cursor_kind)(s.cursor(a)) as i64 },
            "spelling" => unsafe {
                s.last_text = s.string((api.cursor_spelling)(s.cursor(a)));
                0
            },
            "type" => {
                let t = unsafe { (api.cursor_type)(s.cursor(a)) };
                s.type_id(t)
            }
            "location" => unsafe {
                let loc = (api.cursor_location)(s.cursor(a));
                let (mut file, mut line, mut col, mut off) = (std::ptr::null_mut(), 0, 0, 0);
                (api.spelling_location)(loc, &mut file, &mut line, &mut col, &mut off);
                if !file.is_null() {
                    s.last_text = s.string((api.file_name)(file));
                }
                s.last_count = col as i64;
                line as i64
            },
            "system_header" => unsafe {
                (api.in_system_header)((api.cursor_location)(s.cursor(a))) as i64
            },
            "linkage" => unsafe { (api.cursor_linkage)(s.cursor(a)) as i64 },
            "storage_class" => unsafe { (api.storage_class)(s.cursor(a)) as i64 },
            "definition" => {
                let c = unsafe { (api.cursor_definition)(s.cursor(a)) };
                if (70..=73).contains(&c.kind) {
                    0
                } else {
                    s.cursor_id(c)
                }
            }
            "canonical" => {
                let c = unsafe { (api.canonical_cursor)(s.cursor(a)) };
                s.cursor_id(c)
            }
            "referenced" => {
                let c = unsafe { (api.cursor_referenced)(s.cursor(a)) };
                if (70..=73).contains(&c.kind) {
                    0
                } else {
                    s.cursor_id(c)
                }
            }
            "semantic_parent" => {
                let c = unsafe { (api.semantic_parent)(s.cursor(a)) };
                if (70..=73).contains(&c.kind) {
                    0
                } else {
                    s.cursor_id(c)
                }
            }
            "is_definition" => unsafe { (api.is_definition)(s.cursor(a)) as i64 },
            "enum_value" => unsafe { (api.enum_value)(s.cursor(a)) },
            "enum_uvalue" => unsafe { (api.enum_uvalue)(s.cursor(a)) as i64 },
            "enum_int_type" => {
                let t = unsafe { (api.enum_int_type)(s.cursor(a)) };
                s.type_id(t)
            }
            "typedef_underlying" => {
                let t = unsafe { (api.typedef_underlying)(s.cursor(a)) };
                s.type_id(t)
            }
            "is_bitfield" => unsafe { (api.is_bitfield)(s.cursor(a)) as i64 },
            "bit_width" => unsafe { (api.bit_width)(s.cursor(a)) as i64 },
            "is_anonymous" => unsafe { (api.is_anonymous)(s.cursor(a)) as i64 },
            "is_anonymous_record" => unsafe { (api.is_anonymous_record)(s.cursor(a)) as i64 },
            "num_args" => unsafe { (api.num_arguments)(s.cursor(a)) as i64 },
            "arg" => {
                let c = unsafe { (api.argument)(s.cursor(a), b as c_uint) };
                s.cursor_id(c)
            }
            "is_variadic" => unsafe { (api.cursor_variadic)(s.cursor(a)) as i64 },
            "comment" => unsafe {
                s.last_text = s.string((api.raw_comment)(s.cursor(a)));
                0
            },
            // Line where the cursor's attached comment starts (0 without one).
            "comment_line" => unsafe {
                let loc = (api.range_start)((api.comment_range)(s.cursor(a)));
                let (mut file, mut line, mut col, mut off) = (std::ptr::null_mut(), 0, 0, 0);
                (api.spelling_location)(loc, &mut file, &mut line, &mut col, &mut off);
                line as i64
            },
            "macro_function_like" => unsafe { (api.macro_function_like)(s.cursor(a)) as i64 },
            "macro_builtin" => unsafe { (api.macro_builtin)(s.cursor(a)) as i64 },
            "mangling" => unsafe {
                s.last_text = s.string((api.mangling)(s.cursor(a)));
                0
            },
            "field_offset" => unsafe { (api.field_offset)(s.cursor(a)) },
            "is_virtual_base" => unsafe { (api.is_virtual_base)(s.cursor(a)) as i64 },
            // Offset in bits of base specifier `b` in a complete object of record `a` (-1 unknown).
            "base_offset" => match api.offset_of_base {
                Some(f) => unsafe { f(s.cursor(a), s.cursor(b)) },
                None => -1,
            },
            "is_static_method" => unsafe { (api.cxx_method_static)(s.cursor(a)) as i64 },
            "is_virtual" => unsafe { (api.cxx_method_virtual)(s.cursor(a)) as i64 },
            "is_pure_virtual" => unsafe { (api.cxx_method_pure_virtual)(s.cursor(a)) as i64 },
            "is_const_method" => unsafe { (api.cxx_method_const)(s.cursor(a)) as i64 },
            "is_copy_ctor" => unsafe { (api.ctor_copy)(s.cursor(a)) as i64 },
            "is_move_ctor" => unsafe { (api.ctor_move)(s.cursor(a)) as i64 },
            "is_inlined" => unsafe { (api.function_inlined)(s.cursor(a)) as i64 },
            "access" => unsafe { (api.access_specifier)(s.cursor(a)) as i64 },
            // All C++ manglings of a constructor/destructor (Itanium: C1/C2, D0/D1/D2).
            // Returns the count; read them with `mangling_item`.
            "manglings" => unsafe {
                s.manglings.clear();
                let set = (api.cxx_manglings)(s.cursor(a));
                if !set.is_null() {
                    let n = (*set).count as usize;
                    for i in 0..n {
                        let cs = *(*set).strings.add(i);
                        // The set owns its strings: read without disposing.
                        let p = (api.get_cstring)(cs);
                        s.manglings.push(if p.is_null() {
                            Vec::new()
                        } else {
                            CStr::from_ptr(p).to_bytes().to_vec()
                        });
                    }
                    (api.dispose_string_set)(set);
                }
                s.manglings.len() as i64
            },
            "mangling_item" => {
                s.last_text = s.manglings.get(a as usize).cloned().unwrap_or_default();
                0
            }
            // The template a specialization or instantiation was made from (0 if none).
            "specialized_template" => {
                let c = unsafe { (api.specialized_template)(s.cursor(a)) };
                if (70..=73).contains(&c.kind) {
                    0
                } else {
                    s.cursor_id(c)
                }
            }
            "tokenize" => unsafe {
                let c = s.cursor(a);
                let range = (api.cursor_extent)(c);
                let mut tokens: *mut CxToken = std::ptr::null_mut();
                let mut count: c_uint = 0;
                (api.tokenize)(s.tu, range, &mut tokens, &mut count);
                s.tokens.clear();
                for i in 0..count as usize {
                    let t = *tokens.add(i);
                    let spelling = s.string((api.token_spelling)(s.tu, t));
                    s.tokens.push(((api.token_kind)(t), spelling));
                }
                if !tokens.is_null() {
                    (api.dispose_tokens)(s.tu, tokens, count);
                }
                count as i64
            },
            "token_kind" => s.tokens.get(a as usize).map_or(0, |t| t.0 as i64),
            "token" => {
                s.last_text = s
                    .tokens
                    .get(a as usize)
                    .map(|t| t.1.clone())
                    .unwrap_or_default();
                0
            }
            "evaluate" => unsafe {
                s.release_eval();
                s.eval = (api.evaluate)(s.cursor(a));
                if s.eval.is_null() {
                    0
                } else {
                    (api.eval_kind)(s.eval) as i64
                }
            },
            "eval_int" => {
                if s.eval.is_null() {
                    0
                } else {
                    unsafe {
                        if (api.eval_unsigned)(s.eval) != 0 {
                            (api.eval_unsigned_value)(s.eval) as i64
                        } else {
                            (api.eval_int)(s.eval)
                        }
                    }
                }
            }
            "eval_is_unsigned" => {
                if s.eval.is_null() {
                    0
                } else {
                    unsafe { (api.eval_unsigned)(s.eval) as i64 }
                }
            }
            "eval_double" => {
                if s.eval.is_null() {
                    0
                } else {
                    unsafe { (api.eval_double)(s.eval).to_bits() as i64 }
                }
            }
            "eval_string" => {
                if !s.eval.is_null() {
                    unsafe {
                        let p = (api.eval_str)(s.eval);
                        if !p.is_null() {
                            s.last_text = CStr::from_ptr(p).to_bytes().to_vec();
                        }
                    }
                }
                0
            }
            // ----- types
            "t_kind" => s.ty(a).kind as i64,
            "t_spelling" => unsafe {
                s.last_text = s.string((api.type_spelling)(s.ty(a)));
                0
            },
            "t_pointee" => {
                let t = unsafe { (api.pointee)(s.ty(a)) };
                s.type_id(t)
            }
            "t_result" => {
                let t = unsafe { (api.result_type)(s.ty(a)) };
                s.type_id(t)
            }
            "t_num_args" => unsafe { (api.num_arg_types)(s.ty(a)) as i64 },
            "t_arg" => {
                let t = unsafe { (api.arg_type)(s.ty(a), b as c_uint) };
                s.type_id(t)
            }
            "t_variadic" => unsafe { (api.type_variadic)(s.ty(a)) as i64 },
            "t_element" => {
                let t = unsafe { (api.array_element)(s.ty(a)) };
                s.type_id(t)
            }
            "t_array_size" => unsafe { (api.array_size)(s.ty(a)) },
            "t_canonical" => {
                let t = unsafe { (api.canonical_type)(s.ty(a)) };
                s.type_id(t)
            }
            "t_decl" => {
                let c = unsafe { (api.type_declaration)(s.ty(a)) };
                if (70..=73).contains(&c.kind) {
                    0
                } else {
                    s.cursor_id(c)
                }
            }
            "t_sizeof" => unsafe { (api.size_of)(s.ty(a)) },
            "t_alignof" => unsafe { (api.align_of)(s.ty(a)) },
            "t_const" => unsafe { (api.is_const)(s.ty(a)) as i64 },
            "t_named" => {
                let t = unsafe { (api.named_type)(s.ty(a)) };
                s.type_id(t)
            }
            "t_offsetof" => unsafe {
                let name = CString::new(text).map_err(|_| "name contains NUL")?;
                (api.type_offset_of)(s.ty(a), name.as_ptr())
            },
            "t_template_arg" => {
                let t = unsafe { (api.template_arg_type)(s.ty(a), b as c_uint) };
                s.type_id(t)
            }
            // Objective-C: the declared result type of a method cursor, and the pieces of
            // `id<P>` / `Base<Arg>` object types.
            "cursor_result_type" => {
                let t = unsafe { (api.cursor_result_type)(s.cursor(a)) };
                s.type_id(t)
            }
            "t_objc_base" => {
                let t = unsafe { (api.objc_base_type)(s.ty(a)) };
                s.type_id(t)
            }
            "t_objc_num_protocols" => unsafe { (api.objc_num_protocols)(s.ty(a)) as i64 },
            "t_objc_protocol" => {
                let c = unsafe { (api.objc_protocol)(s.ty(a), b as c_uint) };
                if (70..=73).contains(&c.kind) {
                    0
                } else {
                    s.cursor_id(c)
                }
            }
            "t_objc_num_type_args" => unsafe { (api.objc_num_type_args)(s.ty(a)) as i64 },
            "t_objc_type_arg" => {
                let t = unsafe { (api.objc_type_arg)(s.ty(a), b as c_uint) };
                s.type_id(t)
            }
            // The type an attributed type (`T _Nonnull`) was made from.
            "t_modified" => {
                let t = unsafe { (api.modified_type)(s.ty(a)) };
                s.type_id(t)
            }
            "t_num_template_args" => unsafe { (api.num_template_args)(s.ty(a)) as i64 },
            other => return Err(format!("unknown clang operation '{other}'")),
        };
        let _ = b;
        Ok(r)
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
