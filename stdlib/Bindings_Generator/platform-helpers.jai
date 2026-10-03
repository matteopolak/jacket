// Assigned public facade. Algorithms live in independently organized internals.
library_symbol_cache: [..] Library_Symbols;

find_lib_for_foreign_name :: (name: string) -> *Library_Info {
    return gp_find_library(name);
}

os_get_symbol_names_from_file_fixed_and_cached :: (full_path: string) -> success: bool, [] string {
    return gp_cached_symbols(full_path);
}

get_libc_paths :: (os := OS, cpu := CPU) -> libc_path: string, libc_platform_path: string, success: bool {
    return gp_libc_paths(os, cpu);
}

os_get_symbol_names_from_file :: (full_path: string) -> success: bool, [] string {
    return gp_symbols_file(full_path);
}

os_get_symbol_names_from_file_content :: (file_content: string, full_path: string) -> success: bool, [] string {
    return gp_symbols_content(file_content, full_path);
}

os_load_library :: (paths: [] string, identifier: string, library_path: string, treat_as_system_library: bool, static_library_suffix := "", shared_library_suffix := "") -> *Library_Info {
    return gp_load(paths, identifier, library_path, treat_as_system_library, static_library_suffix, shared_library_suffix);
}

os_symbol_exists_in_library :: (lib: *Library_Info, name: string) -> bool {
    return gp_has_symbol(lib, name);
}

fix_macos_foreign_name :: (name: *string) {
    if !name return;

    if (TARGET_OS == .MACOS || TARGET_OS == .IOS) && name.count > 0 && name.data[0] == #char "_" {
        name.data += 1;
        name.count -= 1;
    }
}

function_is_missing_foreign_library :: (func: *Function) -> bool {
    return !func || !func.library;
}

ensure_local_libclang_exists :: () {
    #if OS == .MACOS || OS == .WINDOWS || OS == .LINUX {
        name := ifx OS == .WINDOWS then "libclang.dll" else ifx OS == .MACOS then "libclang.dylib" else "libclang.so";
        gp_ensure_clang(tprint("%../internal/developer-tools/clang-bindings/%", #filepath, name));
    } else assert(false, "Bindings_Generator: no bundled libclang for this OS");
}

ensure_local_libclang_exists :: (source_path: string) {
    gp_ensure_clang(source_path);
}

#load "../internal/developer-tools/generator-platform.jai";

#scope_file
#import "Basic";
