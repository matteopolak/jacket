get_plugin :: () -> *C.Metaprogram_Plugin {
    state := New(Link_Plugin);
    if !state {
        log_error("linux_build: unable to allocate plugin state.");
        return null;
    }

    state.owner = context.allocator;
    state.base.before_intercept = before_intercept;
    state.base.add_source = add_source;
    state.base.message = message;
    state.base.finish = finish;
    state.base.shutdown = shutdown;

    return *state.base;
}

before_intercept :: (p: *C.Metaprogram_Plugin, flags: *C.Intercept_Flags) {
    options := C.get_build_options(p.workspace);
    options.os_target = .LINUX;
    options.backend = .X64;
    options.use_custom_link_command = true;
    C.set_build_options(options, p.workspace);
}

add_source :: (p: *C.Metaprogram_Plugin) {}
finish :: (p: *C.Metaprogram_Plugin) {}

shutdown :: (p: *C.Metaprogram_Plugin) {
    state := cast(*Link_Plugin) p;
    owner := state.owner;

    push_allocator(owner);
    free(state.output);
    free(state);
}

message :: (p: *C.Metaprogram_Plugin, message: *C.Message) {
    if message.kind != .PHASE return;

    phase := cast(*C.Message_Phase) message;
    if phase.phase == .PRE_WRITE_EXECUTABLE {
        print("linux_build executable: %\n", phase.executable_name);
        return;
    }

    if phase.phase != .READY_FOR_CUSTOM_LINK_COMMAND return;

    state := cast(*Link_Plugin) p;

    push_allocator(state.owner);
    options := C.get_build_options(p.workspace);
    directory := options.output_path;

    if directory.count == 0 directory = ".";

    free(state.output);
    state.output = S.join(directory, "/", options.output_executable_name);
    root := toolchain_root();

    defer free(root);
    args := link_arguments(root, phase, state.output);

    defer release_arguments(*args);

    if !F.file_exists(args[0]) log_error("linux_build: linker is missing: %", args[0]);

    result, output, errors, timed := P.run_command(..args, capture_and_return_output = true);

    defer free(output);
    defer free(errors);
    failed := timed || result.type != .EXITED || result.exit_code != 0;

    phase.linker_exit_code = result.exit_code;

    if failed {
        if phase.linker_exit_code == 0 phase.linker_exit_code = -1;

        phase.executable_write_failed = true;
        log_error("linux_build: linker failed (%, exit %, signal %, timeout %).\n%\n%",
                  result.type, result.exit_code, result.signal, timed, output, errors);
        C.compiler_set_workspace_status(.FAILED, p.workspace);
    }

    phase.executable_name = state.output;
    C.compiler_custom_link_command_is_complete(p.workspace);
}

#scope_module
#import "Basic";
C :: #import "Compiler";
S :: #import "String";
F :: #import "File_Utilities";
P :: #import "Process";
#if OS == .WINDOWS Native :: #import "Windows";
else Native :: #import "POSIX";

Link_Plugin :: struct {
    base: C.Metaprogram_Plugin;
    owner: Allocator;
    output: string;
}

toolchain_root :: () -> string {
    value := Native.getenv("LINUX_MULTIARCH_ROOT");
    if !value || value[0] == 0 return normalize_root("C:/UnrealToolchains/v21_clang-15.0.1-centos7");
    return normalize_root(to_string(value));
}

normalize_root :: (input: string) -> string {
    end := input.count;
    while end > 0 && (input[end - 1] == 47 || input[end - 1] == 92) end -= 1;

    result := copy_string(S.slice(input, 0, end));
    for *byte: result if byte.* == 92 byte.* = 47;

    return result;
}

append_argument :: (args: *[..] string, value: string) {
    array_add(args, copy_string(value));
}

release_arguments :: (args: *[..] string) {
    for arg: args.* free(arg);
    array_free(args.*);
} @Releases

link_arguments :: (root: string, phase: *C.Message_Phase, output: string) -> [..] string {
    args: [..] string;
    sdk := S.join(root, "/x86_64-unknown-linux-gnu");

    defer free(sdk);
    linker := S.join(root, "/bin/ld.lld.exe");

    defer free(linker);
    append_argument(*args, linker);
    append_argument(*args, "--sysroot");
    append_argument(*args, sdk);
    for arg: string.["--build-id", "--gdb-index", "-g", "--nostdlib"] append_argument(*args, arg);

    for suffix: string.["/usr/lib", "/usr/lib64"] {
        path := S.join(sdk, suffix);
        append_argument(*args, "-L");
        append_argument(*args, path);
        free(path);
    }

    for object: string.["crt1.o", "crti.o", "crtn.o"] {
        path := S.join(sdk, "/usr/lib/", object);
        append_argument(*args, path);
        free(path);
    }

    for arg: string.["--dynamic-linker", "/lib64/ld-linux-x86-64.so.2", "--eh-frame-hdr", "-export-dynamic", "-rpath=$ORIGIN"] append_argument(*args, arg);
    for object: phase.compiler_generated_object_files append_argument(*args, object);
    for object: phase.support_object_files append_argument(*args, object);

    append_argument(*args, "--start-group");

    for library: phase.user_libraries {
        _, has_extension := S.path_extension(library);
        if has_extension append_argument(*args, library);
        else {
            archive := S.join(library, ".a");
            append_argument(*args, archive);
            free(archive);
        }
    }

    for library: phase.system_libraries {
        if library == "-lc" || library == "-lpthread" {
            append_argument(*args, library);
            continue;
        }

        extension, present := S.path_extension(library);
        if !present || extension != "dll" continue;

        base := S.basename(library);
        mapped: string;

        if S.begins_with(base, "lib") mapped = S.join("-l", S.slice(base, 3, base.count));
        else mapped = S.join(base, ".a");

        append_argument(*args, mapped);
        free(mapped);
    }

    append_argument(*args, "--end-group");
    append_argument(*args, "-o");
    append_argument(*args, output);

    return args;
}
