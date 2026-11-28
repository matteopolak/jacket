// Symbol lists already read from library files, keyed by OS, CPU and path. Filled by
// [os_get_symbol_names_from_file_fixed_and_cached].
library_symbol_cache: [..] Library_Symbols;

// Returns the library of the active generator that exports the symbol `name`, or null when none does.
//
// Logs an error and returns null when no generator is active.
find_lib_for_foreign_name :: (name: string) -> *Library_Info {
    return gp_find_library(name);
}

// Returns the exported symbol names of the object or library file at `full_path`, reading each file once.
//
// The first read for an OS, CPU and path is kept, and later calls return that list. On macOS and iOS targets
// one leading `_` is removed from every name. `success` is false, with an error logged, when the file cannot
// be read.
os_get_symbol_names_from_file_fixed_and_cached :: (full_path: string) -> success: bool, [] string {
    return gp_cached_symbols(full_path);
}

// Returns the generic and the target-specific libc include directories for `os` and `cpu`, and whether they
// were found.
//
// - Linux: `usr/include` under the directory in the `DEVELOPER_TOOLS_LINUX_SYSROOT` environment variable (the
//   root directory when it is unset), and its multiarch subdirectory as named by clang.
// - Android: the sysroot of the NDK in `ANDROID_NDK_HOME` or `ANDROID_NDK_ROOT`, and its multiarch
//   subdirectory.
// - macOS and iOS: `usr/include` of the SDK in `DEVELOPER_TOOLS_APPLE_SDK_ROOT`, or on a macOS host the SDK
//   reported by `xcrun`. Both results are the same directory.
//
// Any other OS gives empty strings and `false`. So does any failure: an unknown target, a missing Android NDK
// variable or non-macOS Apple SDK variable, a multiarch directory clang cannot name, or a missing directory.
// All of these log an error. The returned strings are copies the caller frees.
get_libc_paths :: (os := OS, cpu := CPU) -> libc_path: string, libc_platform_path: string, success: bool {
    return gp_libc_paths(os, cpu);
}

// Returns the exported symbol names of the file at `full_path`, as listed by `llvm-nm`.
//
// Reads the file on every call and keeps names exactly as `llvm-nm` prints them. `success` is false, with an
// error logged, when `llvm-nm` is not found, fails, or runs longer than 30 seconds. The names and the array
// are copies the caller frees.
os_get_symbol_names_from_file :: (full_path: string) -> success: bool, [] string {
    return gp_symbols_file(full_path);
}

// Returns the exported symbol names of the object file held in `file_content`.
//
// Writes the bytes to a temporary file, lists it like [os_get_symbol_names_from_file] and deletes it.
// `full_path` only names the file in error messages.
os_get_symbol_names_from_file_content :: (file_content: string, full_path: string) -> success: bool, [] string {
    return gp_symbols_content(file_content, full_path);
}

// Finds and loads the library `library_path` and returns its description, or null with an error logged.
//
// Tries `library_path` as given first, then relative to each of `paths`. For each location it tries the path
// as written, then with `shared_library_suffix`, then with `static_library_suffix`; on POSIX each of these is
// also tried with `lib` put before the file name when it has none. Empty suffixes default to `.dll` and
// `.lib` on Windows, `.dylib` and `.a` on macOS, and `.so` and `.a` elsewhere.
//
// A shared library is opened and queried through its handle. A static library is not opened; its symbol names
// are read from the file. `identifier` names the library in the output and defaults to the lowercased file
// name with `-` and `.` replaced by `_`. With `treat_as_system_library` the library's `full_path` is its bare
// name, without directory or suffix. The result is allocated with the context allocator.
os_load_library :: (paths: [] string, identifier: string, library_path: string, treat_as_system_library: bool, static_library_suffix := "", shared_library_suffix := "") -> *Library_Info {
    return gp_load(paths, identifier, library_path, treat_as_system_library, static_library_suffix, shared_library_suffix);
}

// Returns whether `lib` exports a symbol called `name`.
//
// Asks the loaded library when it has a handle, and otherwise searches the symbol names read from the file. A
// null `lib` or an empty `name` gives false.
os_symbol_exists_in_library :: (lib: *Library_Info, name: string) -> bool {
    return gp_has_symbol(lib, name);
}

// Removes one leading `_` from `name` in place when `TARGET_OS` is macOS or iOS.
//
// Does nothing for a null pointer, an empty name or any other target. The string is not copied; its start
// moves.
fix_macos_foreign_name :: (name: *string) {
    if !name return;

    if (TARGET_OS == .MACOS || TARGET_OS == .IOS) && name.count > 0 && name.data[0] == #char "_" {
        name.data += 1;
        name.count -= 1;
    }
}

// Returns true when `func` is null or no library was found for it.
function_is_missing_foreign_library :: (func: *Function) -> bool {
    return !func || !func.library;
}

// Copies the libclang bundled with the generator into the working directory when it is missing there.
//
// The copy is also made when the local file has a different size or is older than the bundled one. Asserts on
// an OS other than macOS, Windows and Linux, or when the bundled file is missing. Logs an error when the copy
// fails.
ensure_local_libclang_exists :: () {
    #if OS == .MACOS || OS == .WINDOWS || OS == .LINUX {
        name := ifx OS == .WINDOWS then "libclang.dll" else ifx OS == .MACOS then "libclang.dylib" else "libclang.so";
        gp_ensure_clang(tprint("%../internal/developer-tools/clang-bindings/%", #filepath, name));
    } else assert(false, "Bindings_Generator: no bundled libclang for this OS");
}

// Copies the file at `source_path` into the working directory under its own file name when that is missing.
//
// The copy is also made when the local file has a different size or is older. Asserts when `source_path` is
// empty or does not exist. Logs an error when the copy fails.
ensure_local_libclang_exists :: (source_path: string) {
    gp_ensure_clang(source_path);
}

#load "../internal/developer-tools/generator-platform.jai";

#scope_file
#import "Basic";
