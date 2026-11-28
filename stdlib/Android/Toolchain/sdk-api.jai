// Holds the folders of an NDK install; all three are empty when none was found.
NDK_Paths :: struct {
    root: string;
    toolchain: string;
    sysroot: string;
}

// Finds the Android NDK and returns `true` and its root folder, which the caller frees.
//
// Looks at `ANDROID_NDK_HOME`, `ANDROID_NDK_ROOT`, `NDK_HOME` and `NDK_ROOT` in that order, then the newest
// numbered version under `<sdk>/ndk`, then `<sdk>/ndk-bundle`. A candidate counts only when it has an LLVM
// toolchain with a sysroot and `bin/clang` for the host. Returns `false` and an empty string when none does;
// `log_errors` logs that.
android_find_ndk_path :: (log_errors := false) -> bool, string {
    for variable: string.["ANDROID_NDK_HOME", "ANDROID_NDK_ROOT", "NDK_HOME", "NDK_ROOT"] {
        path := tool_environment(variable);
        if android_ndk_valid(path) return true, path;
        free(path);
    }

    sdk_found, sdk := android_find_sdk_path();

    defer free(sdk);

    if sdk_found {
        selected: Android_Ndk_Selection;
        ndks := tool_path_join(sdk, "ndk");

        defer free(ndks);
        Android_Sdk_Files.visit_files(ndks, false, *selected, android_select_ndk,
            visit_files = false, visit_directories = true, follow_directory_symlinks = false);
        if selected.path.count > 0 return true, selected.path;

        legacy := tool_path_join(sdk, "ndk-bundle");
        if android_ndk_valid(legacy) return true, legacy;

        free(legacy);
    }

    if log_errors log_error("No Android NDK with an LLVM toolchain was found\n");

    return false, "";
}

// Returns the Android target triple for `cpu`, and the same triple with [ANDROID_SDK_LEVEL] appended.
//
// Logs an error and returns two empty strings when the CPU has no Android triple.
get_android_target_triple :: (cpu: CPU_Tag) -> triple: string, triple_with_sdk: string {
    triple = Target_Triple.default_target_triple(.ANDROID, cpu);

    if !triple {
        log_error("Unsupported Android target CPU\n");
        return "", "";
    }

    return triple, tprint("%%", triple, ANDROID_SDK_LEVEL);
}

// Returns the NDK's root, LLVM toolchain and sysroot folders, found with [android_find_ndk_path].
//
// All three are empty when no usable NDK is found. They are allocated; the caller frees them.
get_ndk_paths :: () -> NDK_Paths {
    found, root := android_find_ndk_path();
    if !found return .{};

    prebuilt := tool_path_join(root, "toolchains/llvm/prebuilt");

    defer free(prebuilt);
    host := android_ndk_host_tag();
    toolchain := tool_path_join(prebuilt, host);

    if !android_ndk_directory(toolchain) {
        free(root);
        free(toolchain);
        return .{};
    }

    return .{ root, toolchain, tool_path_join(toolchain, "sysroot") };
}

// Returns the NDK's C include folder and its target-specific subfolder for `cpu`.
//
// Both are empty when no NDK is found or `cpu` has no Android triple. The caller frees them.
get_ndk_libc_paths :: (cpu: CPU_Tag) -> libc_path: string, libc_platform_path: string {
    paths := get_ndk_paths();

    defer android_release_ndk(paths);
    triple := get_android_target_triple(cpu);
    if paths.root.count == 0 || triple.count == 0 return "", "";

    include := tool_path_join(paths.sysroot, "usr/include");

    return include, tool_path_join(include, triple);
}

// Returns the paths of the `libclang` library files that exist in the NDK toolchain.
//
// Checks the Linux, macOS and Windows names. The result is empty when no NDK is found; the caller frees the
// array and its strings.
get_ndk_libclang_paths :: () -> [] string {
    paths := get_ndk_paths();

    defer android_release_ndk(paths);
    if paths.root.count == 0 return .{};

    result: [..] string;

    for suffix: string.["lib64/libclang.so", "lib/libclang.so", "lib64/libclang.dylib", "lib/libclang.dylib", "bin/libclang.dll"] {
        path := tool_path_join(paths.toolchain, suffix);
        if Android_Sdk_Files.file_exists(path) array_add(*result, path);
        else free(path);
    }

    return result;
}

// Copies the NDK's `libc++_shared.so` for `cpu_target` into `output_dir`, creating the folder.
//
// Returns `false` when no NDK is found, the CPU has no Android triple, or the folder or copy fails.
copy_android_libcpp :: (cpu_target: CPU_Tag, output_dir: string) -> bool {
    paths := get_ndk_paths();

    defer android_release_ndk(paths);
    triple := get_android_target_triple(cpu_target);
    if paths.root.count == 0 || triple.count == 0 return false;

    source := tool_path_join(paths.sysroot, tprint("usr/lib/%/libc++_shared.so", triple));

    defer free(source);
    target := tool_path_join(output_dir, "libc++_shared.so");

    defer free(target);
    if !Android_Sdk_File.make_directory_if_it_does_not_exist(output_dir, true) return false;

    return Android_Sdk_Files.copy_file(source, target);
}

// Links the compiler's objects with the NDK's clang as a custom link step.
//
// Sets `message.linker_exit_code` and `message.executable_write_failed` from the result, and completes the
// custom link command when `message` is in that phase. The failure values are set before anything else, so
// every early return reports a failed link. Supports executable and shared-library output only. Passes
// `extra_args` after `options.additional_linker_arguments`.
run_android_link_command :: (message: *Message_Phase, options: Build_Options, extra_args: [] string = .[]) {
    paths := get_ndk_paths();

    defer android_release_ndk(paths);
    message.executable_write_failed = true;
    message.linker_exit_code = -1;

    defer {
        if message.phase == .READY_FOR_CUSTOM_LINK_COMMAND compiler_custom_link_command_is_complete(message.workspace);
    }

    if paths.root.count == 0 return;

    triple, with_sdk := get_android_target_triple(options.cpu_target);
    if triple.count == 0 return;

    compiler := tool_path_join(paths.toolchain, android_ndk_compiler_suffix());

    defer free(compiler);
    arguments: [..] string;

    defer array_free(arguments);
    array_add(*arguments, compiler);
    array_add(*arguments, "--target");
    array_add(*arguments, with_sdk);
    array_add(*arguments, "--sysroot");
    array_add(*arguments, paths.sysroot);

    if options.output_type == .DYNAMIC_LIBRARY array_add(*arguments, "-shared");
    else if options.output_type != .EXECUTABLE {
        log_error("Android custom linker requires executable or shared-library output\n");
        return;
    }

    array_add(*arguments, "-o");
    array_add(*arguments, message.executable_name);
    for object: message.compiler_generated_object_files array_add(*arguments, object);
    for object: message.support_object_files array_add(*arguments, object);
    for library: message.user_libraries array_add(*arguments, library);
    for library: message.system_libraries array_add(*arguments, tprint("-l%", library));
    for argument: options.additional_linker_arguments array_add(*arguments, argument);
    for argument: extra_args array_add(*arguments, argument);

    outcome, output, errors, timed_out := Android_Sdk_Process.run_command(..arguments,
        capture_and_return_output = true, print_captured_output = true);

    defer free(output);
    defer free(errors);
    message.linker_exit_code = ifx outcome.type == .EXITED then outcome.exit_code else -1;
    message.executable_write_failed = timed_out || outcome.type != .EXITED || outcome.exit_code != 0 || !Android_Sdk_Files.file_exists(message.executable_name);
}

#scope_file
Target_Triple :: #import "internal/target-triple";
#import "Basic";
#import "Compiler";
Android_Sdk_Files :: #import "File_Utilities";
Android_Sdk_File :: #import "File";
Android_Sdk_Process :: #import "Process";
#load "../../internal/android-toolchain/android-sdk.jai";
