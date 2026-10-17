#scope_file
perform_build :: (output_basename: string, files: ..string, type: Build_Type, debug := false, extra: [] string = .[], library_files: [] string = .[], target := OS, compiler_executable_path := "", ar_executable_path := "", working_directory := "", failure: *string) -> bool {
    if !native_text(output_basename) || !native_text(working_directory) || !native_text(compiler_executable_path) || !native_text(ar_executable_path)
        return remember_failure("BuildCpp paths cannot contain NUL", failure);

    for group: ([] string).[files, extra, library_files] {
        for value: group if !native_text(value) return remember_failure("BuildCpp arguments cannot contain NUL", failure);
    }

    if target != OS return remember_failure("BuildCpp cross-OS toolchains are unavailable", failure);
    if files.count == 0 || output_basename.count == 0 return remember_failure("BuildCpp needs source files and an output basename", failure);
    if type == .OBJ_FILE && files.count != 1 return remember_failure("BuildCpp OBJ_FILE currently requires one source", failure);
    if type == .STATIC_LIBRARY && library_files.count != 0 return remember_failure("BuildCpp static archive dependencies are unavailable", failure);

    requested_directory := working_directory;
    if requested_directory.count == 0 requested_directory = ".";

    directory, resolved := F.get_absolute_path(requested_directory);
    if !resolved return remember_failure("BuildCpp working directory cannot be resolved", failure);

    defer free(directory);
    basename := absolute_path(directory, output_basename);

    defer free(basename);
    output := sprint("%%", basename, output_suffix(type));

    defer free(output);
    template := sprint("%.build-XXXXXX", basename);

    defer free(template);
    native_template := to_c_string(template);

    defer free(native_template);
    if !Native.mkdtemp(native_template) return remember_failure("BuildCpp intermediate directory creation failed", failure);

    intermediates := to_string(native_template);
    staged := sprint("%/result%", intermediates, output_suffix(type));

    defer free(staged);
    objects: [..] string;

    defer {
        for object: objects free(object);
        array_free(objects);
    }

    cleanup_done := false;

    defer if !cleanup_done cleanup_intermediates(intermediates, objects, staged, failure);
    use_cpp := false;

    for file, index: files {
        cpp := is_cpp(file);

        use_cpp = use_cpp || cpp;
        compiler := compiler_executable_path;
        if compiler.count == 0 {
            if cpp compiler = environment_tool("CXX", "c++");
            else compiler = environment_tool("CC", "cc");
        }

        object := sprint("%/%.o", intermediates, index);

        array_add(*objects, object);
        args: [..] string;

        defer array_free(args);
        array_add(*args, compiler);

        if debug {
            array_add(*args, "-O0");
            array_add(*args, "-g");
        } else array_add(*args, "-O2");

        if type == .DYNAMIC_LIBRARY array_add(*args, "-fPIC");
        for arg: extra array_add(*args, arg);

        array_add(*args, "-c");
        array_add(*args, file);
        array_add(*args, "-o");
        array_add(*args, object);
        if !execute(args, directory, failure) return false;
        if !matches_target_object(object, target) return remember_failure("BuildCpp object target validation failed", failure);
    }

    success := true;

    if type == .OBJ_FILE {
        success = F.file_move(objects[0], output);
        if !success remember_failure("BuildCpp object output move failed", failure);
    } else {
        args: [..] string;

        defer array_free(args);

        if type == .STATIC_LIBRARY {
            archiver := ar_executable_path;
            if archiver.count == 0 archiver = environment_tool("AR", "ar");

            array_add(*args, archiver);
            array_add(*args, "rcs");
            array_add(*args, staged);
        } else {
            compiler := compiler_executable_path;
            if compiler.count == 0 {
                if use_cpp compiler = environment_tool("CXX", "c++");
                else compiler = environment_tool("CC", "cc");
            }

            array_add(*args, compiler);

            if type == .DYNAMIC_LIBRARY {
                #if OS == .MACOS array_add(*args, "-dynamiclib");
                else array_add(*args, "-shared");
            }

            for arg: extra array_add(*args, arg);

            array_add(*args, "-o");
            array_add(*args, staged);
        }

        for object: objects array_add(*args, object);
        for library: library_files array_add(*args, library);

        success = execute(args, directory, failure);
        if success {
            success = F.file_move(staged, output);
            if !success remember_failure("BuildCpp linked output move failed", failure);
        }
    }

    cleaned := cleanup_intermediates(intermediates, objects, staged, failure);

    cleanup_done = true;

    return success && cleaned;
}

#scope_export
build_cpp :: (output_basename: string, files: ..string, type: Build_Type, debug := false, extra: [] string = .[], library_files: [] string = .[], target := OS, compiler_executable_path := "", ar_executable_path := "", working_directory := "", loc := #caller_location) -> bool #compile_time {
    failure: string;
    success := perform_build(output_basename, ..files, type = type, debug = debug, extra = extra, library_files = library_files, target = target, compiler_executable_path = compiler_executable_path, ar_executable_path = ar_executable_path, working_directory = working_directory, failure = *failure);
    success = success && failure.count == 0;
    publish_failure(success, failure, loc);

    return success;
}

build_universal_macos_binary :: (lib_path: string, files: ..string, type: Build_Type, debug := false, extra: [] string = .[]) -> bool #compile_time {
    failure: string;
    success := perform_universal(lib_path, ..files, type = type, debug = debug, extra = extra, failure = *failure);
    success = success && failure.count == 0;
    publish_failure(success, failure, #caller_location);

    return success;
}

enum_cpp_files :: (path: string, recursive := false) -> [..] string {
    listed := U.file_list(path, recursive);

    defer {
        for file: listed free(file);
        array_free(listed);
    }

    result: [..] string;
    for file: listed if is_cpp(file) || S.ends_with(file, ".c") array_add(*result, copy_string(file));

    return result;
}

free_cpp_files :: (files: [] string) {
    for file: files free(file);
    array_free(files);
} @Releases

#scope_file
perform_universal :: (lib_path: string, files: ..string, type: Build_Type, debug := false, extra: [] string = .[], failure: *string) -> bool {
    #if OS != .MACOS return remember_failure("universal macOS builds require a macOS host", failure);
    else {
        template := sprint("%.universal-XXXXXX", lib_path);

        defer free(template);
        native := to_c_string(template);

        defer free(native);
        if !Native.mkdtemp(native) return remember_failure("BuildCpp universal directory creation failed", failure);

        directory := to_string(native);

        defer if !F.delete_directory(directory) remember_failure("BuildCpp universal directory cleanup failed", failure);
        arm := sprint("%/arm64", directory);
        x64 := sprint("%/x86_64", directory);

        defer free(arm);
        defer free(x64);
        arm_output := sprint("%%", arm, output_suffix(type));
        x64_output := sprint("%%", x64, output_suffix(type));

        defer free(arm_output);
        defer free(x64_output);

        defer {
            cleanup_file(arm_output, failure);
            cleanup_file(x64_output, failure);
        }

        staged := sprint("%/universal", directory);

        defer free(staged);
        defer cleanup_file(staged, failure);
        flags: [..] string;

        defer array_free(flags);
        for flag: extra array_add(*flags, flag);

        array_add(*flags, "-arch");
        array_add(*flags, "arm64");
        if !perform_build(arm, ..files, type = type, debug = debug, extra = flags, failure = failure) return false;

        flags[flags.count - 1] = "x86_64";
        if !perform_build(x64, ..files, type = type, debug = debug, extra = flags, failure = failure) return false;

        args := string.["lipo", "-create", arm_output, x64_output, "-output", staged];
        if !execute(args, "", failure) return false;
        if !F.file_move(staged, lib_path) return remember_failure("BuildCpp universal output move failed", failure);

        return true;
    }
}

#scope_file
#import "Basic";
C :: #import "Compiler";
F :: #import "File";
U :: #import "File_Utilities";
S :: #import "String";
Process :: #import "Process";
Native :: #import "POSIX";

absolute_path :: (directory: string, path: string) -> string {
    if S.is_absolute_path(path) return copy_string(path);
    return sprint("%/%", directory, path);
}

matches_target_object :: (path: string, target: Operating_System_Tag) -> bool {
    file, opened := F.file_open(path, log_errors = false);
    if !opened return false;

    defer F.file_close(*file);
    header: [4] u8;
    read, count := F.file_read(file, header.data, 4);

    if !read || count != 4 return false;
    if target == .LINUX return header[0] == 0x7f && header[1] == 69 && header[2] == 76 && header[3] == 70;
    if target != .MACOS return false;

    return ((header[0] == 0xcf || header[0] == 0xce) && header[1] == 0xfa && header[2] == 0xed && header[3] == 0xfe) ||
           (header[0] == 0xfe && header[1] == 0xed && header[2] == 0xfa && (header[3] == 0xcf || header[3] == 0xce)) ||
           (header[0] == 0xca && header[1] == 0xfe && header[2] == 0xba && (header[3] == 0xbe || header[3] == 0xbf));
}

native_text :: (value: string) -> bool {
    for byte: value if byte == 0 return false;
    return true;
}

output_suffix :: (type: Build_Type) -> string {
    if type == .STATIC_LIBRARY return ".a";
    if type == .OBJ_FILE return ".o";
    if type == .EXECUTABLE return "";

    #if OS == .MACOS return ".dylib";
    else return ".so";
}

is_cpp :: (file: string) -> bool {
    return S.ends_with(file, ".cpp") || S.ends_with(file, ".cc") ||
           S.ends_with(file, ".cxx") || S.ends_with(file, ".c++") || S.ends_with(file, ".C");
}

environment_tool :: (name: string, fallback: string) -> string {
    key := to_c_string(name);

    defer free(key);
    found := Native.getenv(key);
    if !found || found[0] == 0 return fallback;

    return to_string(found);
}

remember_failure :: (message: string, failure: *string) -> bool {
    next: string;
    if failure.count > 0 next = sprint("%\n%", failure.*, message);
    else next = copy_string(message);

    free(failure.*);
    failure.* = next;

    return false;
}

publish_failure :: (success: bool, failure: string, loc: Source_Code_Location) {
    if success {
        free(failure);
        return;
    }

    message := copy_temporary_string(failure);

    free(failure);
    C.compiler_report(message, loc);
}

execute :: (args: [] string, directory: string, failure: *string) -> bool {
    result, out, err, timed := Process.run_command(..args, working_directory = directory, capture_and_return_output = true);

    defer free(out);
    defer free(err);

    if result.type == .EXITED && result.exit_code == 0 && !timed {
        write_string(out);
        write_string(err, true);
        return true;
    }

    diagnostic := sprint("BuildCpp tool failed (%): %\n%%", args[0], result.exit_code, out, err);

    defer free(diagnostic);

    return remember_failure(diagnostic, failure);
}

cleanup_file :: (path: string, failure: *string) -> bool {
    native := to_c_string(path);
    defer free(native);
    if Native.unlink(native) == 0 || cast(s32) Native.errno() == Native.ENOENT return true;
    return remember_failure("BuildCpp intermediate file cleanup failed", failure);
}

cleanup_intermediates :: (directory: string, objects: [] string, staged: string, failure: *string) -> bool {
    success := true;
    for object: objects if !cleanup_file(object, failure) success = false;
    if !cleanup_file(staged, failure) success = false;

    if !F.delete_directory(directory) {
        remember_failure("BuildCpp intermediate directory cleanup failed", failure);
        success = false;
    }

    return success;
}
