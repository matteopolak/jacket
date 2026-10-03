#scope_module

gp_symbols_file :: (full_path: string, diagnostic_path: string = "") -> bool, [] string {
    label := ifx diagnostic_path then diagnostic_path else full_path;

    if !gp_valid_text(full_path) {
        log_error("Bindings_Generator: invalid symbol file path\n");
        return false, .{};
    }

    absolute, ok := get_absolute_path(full_path);

    defer free(absolute);

    if !ok {
        log_error("Bindings_Generator: cannot resolve symbol file %\n", label);
        return false, .{};
    }

    tool, tool_ok := read_entire_file(tprint("%generator-platform-nm.txt", #filepath));

    defer free(tool);

    if !tool_ok || !gp_valid_text(tool) {
        log_error("Bindings_Generator: missing configured official llvm-nm\n");
        return false, .{};
    }

    status, output, errors, timed_out := GP_Process.run_command(tool, "--export-symbols", "--no-demangle", absolute, capture_and_return_output = true, timeout_ms = 30000);

    defer free(output);
    defer free(errors);

    if status.type != .EXITED || status.exit_code != 0 || timed_out || errors.count {
        log_error("Bindings_Generator: symbol enumeration failed for %\n%", label, errors);
        return false, .{};
    }

    names: [..] string;
    lines := GP_String.split(output, "\n");

    defer free(lines.data);

    for line: lines {
        if OS == .WINDOWS && line.count > 0 && line[line.count - 1] == #char "\r" line = GP_String.slice(line, 0, line.count - 1);
        if !line continue;

        if GP_String.contains(line, "\0") {
            for name: names free(name);
            array_free(names);
            log_error("Bindings_Generator: invalid native symbol output\n");
            return false, .{};
        }

        array_add(*names, copy_string(line));
    }

    return true, .{ names.count, names.data };
}

gp_symbols_content :: (content: string, diagnostic_path: string) -> bool, [] string {
    if content.count < 0 || (content.count > 0 && !content.data) {
        log_error("Bindings_Generator: invalid object content view for %\n", diagnostic_path);
        return false, .{};
    }

    path: [4096] u8;
    if !gp_write_temp(content.data, cast(u64) content.count, path.data, path.count) {
        log_error("Bindings_Generator: cannot stage object content for %\n", diagnostic_path);
        return false, .{};
    }

    filename := to_string(path.data);

    ok, names := gp_symbols_file(filename, diagnostic_path);

    if !file_delete(filename) {
        for name: names free(name);
        array_free(names);
        log_error("Bindings_Generator: cannot remove object staging file for %\n", diagnostic_path);
        return false, .{};
    }

    return ok, names;
}

gp_has_symbol :: (lib: *Library_Info, name: string) -> bool {
    if !lib || !gp_valid_text(name) return false;

    if lib.handle {
        cname := to_c_string(name);
        defer free(cname);
        return gp_library_symbol(cast(*void) lib.handle, cname) != 0;
    }

    for symbol: lib.symbol_names if symbol == name return true;

    return false;
}

gp_load :: (paths: [] string, identifier: string, library_path: string, system: bool, static_suffix: string, shared_suffix: string) -> *Library_Info {
    if !gp_valid_optional_text(identifier) || !gp_valid_text(library_path) {
        log_error("Bindings_Generator: invalid library identifier/path\n");
        return null;
    }

    if paths.count < 0 || paths.count > 0x7fffffffffffffff / size_of(string) || (paths.count > 0 && !paths.data) {
        log_error("Bindings_Generator: invalid library search path view\n");
        return null;
    }

    for path: paths if !gp_valid_text(path) {
        log_error("Bindings_Generator: invalid library search path\n");
        return null;
    }

    if !gp_valid_optional_text(static_suffix) || !gp_valid_optional_text(shared_suffix) {
        log_error("Bindings_Generator: invalid library suffix\n");
        return null;
    }

    if !static_suffix static_suffix = ifx OS == .WINDOWS then ".lib" else ".a";
    if !shared_suffix shared_suffix = ifx OS == .WINDOWS then ".dll" else ifx OS == .MACOS then ".dylib" else ".so";

    for root_index: 0..paths.count {
        root := ifx root_index == 0 then "" else paths[root_index - 1];

        for variant: 0..2 {
            suffix := ifx variant == 0 then "" else ifx variant == 1 then shared_suffix else static_suffix;
            candidate := ifx !root then tprint("%%", library_path, suffix) else tprint("%/%%", root, library_path, suffix);
            result := gp_load_candidate(candidate, identifier, system, static_suffix, shared_suffix);

            if result return result;

            // POSIX library names conventionally omit the physical lib prefix.
            // Keep literal spelling first, then prefix only the final component.
            // Both shared spellings precede the static fallback in this root.
            if OS != .WINDOWS {
                filename_start: int;
                for byte, index: library_path if byte == #char "/" filename_start = index + 1;

                filename := GP_String.slice(library_path, filename_start, library_path.count - filename_start);
                if !GP_String.starts_with(filename, "lib") {
                    directory := GP_String.slice(library_path, 0, filename_start);
                    prefixed := tprint("%lib%%", directory, filename, suffix);

                    candidate = ifx !root then prefixed else tprint("%/%", root, prefixed);
                    result = gp_load_candidate(candidate, identifier, system, static_suffix, shared_suffix);
                    if result return result;
                }
            }
        }
    }

    log_error("Bindings_Generator: unable to load library %\n", library_path);

    return null;
}

gp_load_candidate :: (path: string, identifier: string, system: bool, static_suffix: string, shared_suffix: string) -> *Library_Info {
    is_static := path.count >= static_suffix.count && GP_String.slice(path, path.count - static_suffix.count, static_suffix.count) == static_suffix;
    directory_count, stem_count := gp_library_spelling(path, is_static, static_suffix, shared_suffix);

    if stem_count == 0 return null;

    names: [] string;
    handle: *void;

    if is_static {
        if !GP_FS.file_exists(path) return null;

        ok: bool;
        ok, names = gp_symbols_file(path);
        if !ok return null;

        gp_fix_symbol_names(names);
    } else {
        cpath := to_c_string(path);
        defer free(cpath);
        handle = gp_library_open(cpath);
        if !handle return null;
    }

    result := New(Library_Info);
    stem := GP_String.slice(path, directory_count, stem_count);

    result.identifier = ifx identifier.count then copy_string(identifier) else gp_derived_identifier(stem);
    // Physical path was consumed unchanged above. The live native handle or
    // owned static symbol view remains authoritative for subsequent queries.
    result.full_path = copy_string(ifx system then stem else GP_String.slice(path, 0, directory_count + stem_count));
    result.handle = cast(type_of(result.handle)) handle;
    result.is_shared_library = !is_static;
    result.is_system_library = system;
    result.symbol_names = names;

    return result;
}

gp_valid_text :: (text: string) -> bool {
    return text.count > 0 && gp_valid_optional_text(text);
}

gp_valid_optional_text :: (text: string) -> bool {
    return text.count >= 0 && (text.count == 0 || text.data != null) && !GP_String.contains(text, "\0");
}

gp_derived_identifier :: (stem: string) -> string {
    result := GP_String.to_lower_copy(stem);
    for *byte: result if byte.* == #char "-" || byte.* == #char "." byte.* = #char "_";
    return result;
}

gp_library_spelling :: (path: string, is_static: bool, static_suffix: string, shared_suffix: string) -> int, int {
    directory: int;
    for byte, index: path if byte == #char "/" || (OS == .WINDOWS && byte == #char "\\") directory = index + 1;

    filename := GP_String.slice(path, directory, path.count - directory);
    count := filename.count;
    suffix := ifx is_static then static_suffix else shared_suffix;

    if count >= suffix.count && GP_String.slice(filename, count - suffix.count, suffix.count) == suffix {
        count -= suffix.count;

        // Darwin's numeric versions precede the dylib suffix.
        if !is_static && suffix == ".dylib" {
            while count > 0 {
                first := count;
                while first > 0 && filename[first - 1] >= #char "0" && filename[first - 1] <= #char "9" first -= 1;
                if first == count || first == 0 || filename[first - 1] != #char "." break;
                count = first - 1;
            }
        }
    } else if !is_static && shared_suffix == ".so" {
        // ELF's numeric versions follow .so. Validate the whole version tail;
        // unrelated dotted names are never shortened by a substring match.
        for index: 0..count - 5 {
            if GP_String.slice(filename, index, 4) != ".so." continue;

            numeric := true;
            previous_dot := true;

            for byte: GP_String.slice(filename, index + 4, count - index - 4) {
                if byte == #char "." {
                    if previous_dot numeric = false;
                    previous_dot = true;
                } else {
                    if byte < #char "0" || byte > #char "9" numeric = false;
                    previous_dot = false;
                }
            }

            if numeric && !previous_dot count = index;

            break;
        }
    }

    return directory, count;
}

gp_find_library :: (name: string) -> *Library_Info {
    active := gp_current_generator();
    if !active {
        log_error("Bindings_Generator: no active Generator\n");
        return null;
    }

    for library: active.libraries {
        if gp_has_symbol(library, name) return library;
    }

    return null;
}

gp_fix_symbol_names :: (names: [] string) {
    for *name: names {
        if (TARGET_OS == .MACOS || TARGET_OS == .IOS) && name.count > 0 && name.data[0] == #char "_" {
            normalized := copy_string(GP_String.slice(name.*, 1, name.count - 1));
            free(name.*);
            name.* = normalized;
        }
    }
}

gp_cached_symbols :: (path: string) -> bool, [] string {
    for entry: library_symbol_cache {
        if entry.os == OS && entry.cpu == CPU && entry.full_path == path return true, entry.symbol_names;
    }

    ok, names := gp_symbols_file(path);
    if !ok return false, .{};

    gp_fix_symbol_names(names);

    if !gp_cache_allocator.proc gp_cache_allocator = context.default_allocator;

    push_allocator(gp_cache_allocator);
    cached: [..] string;
    for name: names array_add(*cached, copy_string(name));

    entry: Library_Symbols;
    entry.os = OS;
    entry.cpu = CPU;
    entry.full_path = copy_string(path);
    entry.symbol_names = .{ cached.count, cached.data };
    array_add(*library_symbol_cache, entry);

    return true, names;
}

gp_ensure_clang :: (source: string) {
    assert(gp_valid_text(source), "Bindings_Generator: invalid libclang source path");
    source_time, source_size, exists := GP_FS.file_modtime_and_size(source);

    assert(exists, "Bindings_Generator: missing libclang source");
    start: s64;

    for c, index: source {
        if c == #char "/" || (OS == .WINDOWS && c == #char "\\") start = index + 1;
    }

    basename := GP_String.slice(source, start, source.count - start);

    assert(basename.count > 0, "Bindings_Generator: empty libclang basename");
    local_time, local_size, local_exists := GP_FS.file_modtime_and_size(basename);
    if local_exists && source_size == local_size && compare_apollo_times(source_time, local_time) <= 0 return;
    if !GP_FS.copy_file(source, basename) log_error("Bindings_Generator: failed to copy libclang from % to %\n", source, basename);
}

gp_env :: (name: string) -> string {
    cname := to_c_string(name);
    defer free(cname);
    return to_string(gp_environment(cname));
}

gp_directory :: (path: string) -> bool {
    if !gp_valid_text(path) return false;
    directory, ok := GP_FS.is_directory(path);
    return directory && ok;
}

gp_libc_paths :: (os: Operating_System_Tag, cpu: CPU_Tag) -> string, string, bool {
    if cpu != .X64 && cpu != .ARM64 return "", "", false;

    generic: string;
    platform: string;

    if os == .LINUX {
        root := gp_env("DEVELOPER_TOOLS_LINUX_SYSROOT");
        generic = tprint("%/usr/include", root);
        triple := ifx cpu == .X64 then "x86_64-linux-gnu" else "aarch64-linux-gnu";
        platform = tprint("%/%", generic, triple);
    } else if os == .ANDROID {
        ndk := gp_env("ANDROID_NDK_HOME");
        if !ndk ndk = gp_env("ANDROID_NDK_ROOT");

        if !ndk {
            log_error("Bindings_Generator: configure ANDROID_NDK_HOME for Android libc headers\n");
            return "", "", false;
        }

        host := ifx OS == .MACOS then "darwin-x86_64" else ifx OS == .WINDOWS then "windows-x86_64" else "linux-x86_64";

        generic = tprint("%/toolchains/llvm/prebuilt/%/sysroot/usr/include", ndk, host);
        triple := ifx cpu == .X64 then "x86_64-linux-android" else "aarch64-linux-android";

        platform = tprint("%/%", generic, triple);
    } else if os == .MACOS || os == .IOS {
        sdk := gp_env("DEVELOPER_TOOLS_APPLE_SDK_ROOT");
        if !sdk {
            if OS != .MACOS {
                log_error("Bindings_Generator: configure DEVELOPER_TOOLS_APPLE_SDK_ROOT for Apple libc headers\n");
                return "", "", false;
            }

            selector := ifx os == .MACOS then "macosx" else "iphoneos";
            status, output, errors, timed_out := GP_Process.run_command("/usr/bin/xcrun", "--sdk", selector, "--show-sdk-path", capture_and_return_output = true, timeout_ms = 30000);

            defer free(output);
            defer free(errors);

            if status.type != .EXITED || status.exit_code != 0 || timed_out || errors.count {
                log_error("Bindings_Generator: cannot discover Apple SDK\n%", errors);
                return "", "", false;
            }

            while output.count > 0 && (output[output.count - 1] == #char "\n" || output[output.count - 1] == #char "\r") output = GP_String.slice(output, 0, output.count - 1);

            // Materialize under the caller temporary arena before releasing probe output.
            sdk = tprint("%", output);
        }

        generic = tprint("%/usr/include", sdk);
        platform = generic;
    } else return "", "", false;

    if !gp_directory(generic) || !gp_directory(platform) {
        log_error("Bindings_Generator: libc include directories unavailable for OS % CPU %\n", os, cpu);
        return "", "", false;
    }

    return copy_string(generic), copy_string(platform), true;
}

#scope_file
gp_cache_allocator: Allocator;

#import "Basic";
#import "File";
GP_Process :: #import "Process";
GP_String :: #import "String";
GP_FS :: #import "File_Utilities";

gp_native :: #foreign_library "generator_platform";
gp_write_temp :: (data: *void, size: u64, path: *u8, capacity: u64) -> s32 #foreign gp_native;
gp_library_open :: (path: *u8) -> *void #foreign gp_native;
gp_library_symbol :: (handle: *void, name: *u8) -> s32 #foreign gp_native;
gp_library_close :: (handle: *void) #foreign gp_native;
gp_environment :: (name: *u8) -> *u8 #foreign gp_native;
