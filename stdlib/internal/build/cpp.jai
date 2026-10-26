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
    intermediates, created := create_intermediate_directory(basename);
    if !created return remember_failure("BuildCpp intermediate directory creation failed", failure);

    defer free(intermediates);
    // Preserve the final filename: Windows import libraries record the DLL name.
    staged := sprint("%/%", intermediates, S.path_filename(output));

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
            if cpp compiler = environment_tool("CXX", default_compiler(true));
            else compiler = environment_tool("CC", default_compiler(false));
        }

        object := sprint("%/%%", intermediates, index, output_suffix(.OBJ_FILE));

        array_add(*objects, object);
        args: [..] string;

        defer array_free(args);
        array_add(*args, compiler);

        if !append_compile_arguments(*args, compiler, file, object, type, debug, extra)
            return remember_failure("BuildCpp Windows SDK discovery failed", failure);
        if !execute(args, directory, failure) return false;
        if !matches_target_object(object, target) return remember_failure("BuildCpp object target validation failed", failure);
    }

    success := true;

    if type == .OBJ_FILE {
        success = U.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(objects[0], output);
        if !success remember_failure("BuildCpp object output move failed", failure);
    } else {
        args: [..] string;

        defer array_free(args);

        if type == .STATIC_LIBRARY {
            archiver := ar_executable_path;
            if archiver.count == 0 archiver = environment_tool("AR", default_archiver());

            array_add(*args, archiver);

            #if OS == .WINDOWS array_add(*args, tprint("/OUT:%", staged));
            else {
                array_add(*args, "rcs");
                array_add(*args, staged);
            }
        } else {
            compiler := compiler_executable_path;
            if compiler.count == 0 {
                if use_cpp compiler = environment_tool("CXX", default_compiler(true));
                else compiler = environment_tool("CC", default_compiler(false));
            }

            array_add(*args, compiler);

            if !append_link_arguments(*args, compiler, staged, intermediates, type, debug, extra, objects, library_files)
                return remember_failure("BuildCpp Windows SDK discovery failed", failure);

        }

        if type == .STATIC_LIBRARY {
            for object: objects array_add(*args, object);
            for library: library_files array_add(*args, library);
        }

        success = execute(args, directory, failure);
        if success {
            success = U.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(staged, output);
            if !success remember_failure("BuildCpp linked output move failed", failure);

            #if OS == .WINDOWS {
                if success && debug {
                    symbols := sprint("%.pdb", S.path_strip_extension(staged));

                    defer free(symbols);

                    if U.file_exists(symbols) {
                        destination := sprint("%.pdb", basename);
                        defer free(destination);
                        success = U.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(symbols, destination);
                        if !success remember_failure("BuildCpp debug symbols output move failed", failure);
                    }
                }

                if success && type == .DYNAMIC_LIBRARY {
                    import_library := sprint("%/import.lib", intermediates);

                    defer free(import_library);

                    if U.file_exists(import_library) {
                        destination := sprint("%.lib", basename);
                        defer free(destination);
                        success = U.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(import_library, destination);
                        if !success remember_failure("BuildCpp import library output move failed", failure);
                    }
                }
            }
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
} @FreesArgument

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
        if !U.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(staged, lib_path) return remember_failure("BuildCpp universal output move failed", failure);

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

#if OS == .WINDOWS {
    Native :: #import "internal/bindings/windows-api";
    #load "cpp-windows.jai";
} else {
    Native :: #import "internal/bindings/posix-api";
    #load "cpp-unix.jai";
}

#scope_file
absolute_path :: (directory: string, path: string) -> string {
    if S.is_absolute_path(path) return copy_string(path);
    return sprint("%/%", directory, path);
}

matches_target_object :: (path: string, target: Operating_System_Tag) -> bool {
    file, opened := F.file_open(path, log_errors = false);
    if !opened return false;

    defer F.file_close(*file);
    header: [32] u8;
    read, count := F.file_read(file, header.data, header.count);

    if !read || count < COFF_HEADER_SIZE return false;
    if target == .LINUX return header[0] == 0x7f && header[1] == #char "E" && header[2] == #char "L" && header[3] == #char "F";

    if target == .WINDOWS {
        // COFF object headers, not PE images: reject wrong-machine objects before linking.
        #if CPU == .ARM64 machine :: 0xaa64;
        else machine :: 0x8664;

        view: [] u8;
        view.data = header.data;
        view.count = count;

        return coff_object_matches(view, machine);
    }

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
    #if OS == .WINDOWS {
        if type == .STATIC_LIBRARY return ".lib";
        if type == .OBJ_FILE return ".obj";
        if type == .EXECUTABLE return ".exe";
        return ".dll";
    }

    if type == .STATIC_LIBRARY return ".a";
    if type == .OBJ_FILE return ".o";
    if type == .EXECUTABLE return "";

    #if OS == .MACOS return ".dylib";
    else return ".so";
}

#scope_module
// Whether `header` (the first bytes of an object file) is a COFF object for `machine`. Handles ordinary
// headers and the `/bigobj` form, an anonymous object header that carries the machine at offset 6.
coff_object_matches :: (header: [] u8, machine: u32) -> bool {
    word :: (bytes: [] u8, offset: s64) -> u32 {
        return (cast(u32) bytes[offset]) | ((cast(u32) bytes[offset + 1]) << 8);
    }

    // ANON_OBJECT_HEADER_BIGOBJ class id {D1BAA1C7-BAEE-4BA9-AF20-FAF66AA4DCB8}, little-endian.
    bigobj_class :: u8.[0xc7, 0xa1, 0xba, 0xd1, 0xee, 0xba, 0xa9, 0x4b, 0xaf, 0x20, 0xfa, 0xf6, 0x6a, 0xa4, 0xdc, 0xb8];

    if header.count >= 32 && word(header, 0) == 0 && word(header, 2) == 0xffff && word(header, 4) >= 2 {
        for bigobj_class if header[BIGOBJ_CLASS_OFFSET + it_index] != it return false;
        return word(header, 6) == machine;
    }

    if header.count < COFF_HEADER_SIZE return false;

    return word(header, 0) == machine && (header[2] != 0 || header[3] != 0) && header[16] == 0 && header[17] == 0;
}

#scope_module
is_cpp :: (file: string) -> bool {
    return S.ends_with(file, ".cpp") || S.ends_with(file, ".cc") ||
           S.ends_with(file, ".cxx") || S.ends_with(file, ".c++") || S.ends_with(file, ".C");
}

#scope_module
remember_failure :: (message: string, failure: *string) -> bool {
    next: string;
    if failure.count > 0 next = sprint("%\n%", failure.*, message);
    else next = copy_string(message);

    free(failure.*);
    failure.* = next;

    return false;
}

#scope_file

COFF_HEADER_SIZE :: 20;
BIGOBJ_CLASS_OFFSET :: 12;

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

cleanup_intermediates :: (directory: string, objects: [] string, staged: string, failure: *string) -> bool {
    success := true;
    for object: objects if !cleanup_file(object, failure) success = false;
    if !cleanup_file(staged, failure) success = false;

    #if OS == .WINDOWS {
        for name: string.["import.lib", "import.exp"] {
            path := sprint("%/%", directory, name);
            defer free(path);
            if !cleanup_file(path, failure) success = false;
        }
    }

    if !F.delete_directory(directory) {
        remember_failure("BuildCpp intermediate directory cleanup failed", failure);
        success = false;
    }

    return success;
}
