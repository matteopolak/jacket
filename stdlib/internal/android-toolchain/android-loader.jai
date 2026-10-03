#scope_module
android_dynamic_loader_path :: (config: Android_Project_Config) -> string {
    return tool_path_join(config.project_path, ".build_android_loader/libloader.so");
}

android_dynamic_loader_source :: (config: Android_Project_Config, options: Android_Build_Options) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    print(*builder, "// Independent Android dynamic loader version %, SDK %, CPU %, mode %\n", ANDROID_DYNAMIC_LOADER_VERSION, options.sdk_level, options.cpu_target, options.build_type);
    append(*builder, "#include <android/native_activity.h>\n#include <android/log.h>\n#include <dlfcn.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\nstatic void *payload;\n__attribute__((visibility(\"default\")))\nvoid ANativeActivity_onCreate(ANativeActivity *activity, void *state, size_t size) {\n const char *library = ");
    android_host_literal(*builder, config.lib_name);
    append(*builder, ";\n const char *root = activity->externalDataPath;\n if (!root) { ANativeActivity_finish(activity); return; }\n size_t capacity = strlen(root) + strlen(library) + 8;\n char *path = malloc(capacity);\n if (!path) { ANativeActivity_finish(activity); return; }\n snprintf(path, capacity, \"%s/lib%s.so\", root, library);\n payload = dlopen(path, RTLD_NOW | RTLD_LOCAL);\n free(path);\n if (!payload) { __android_log_print(ANDROID_LOG_ERROR, \"JaiLoader\", \"%s\", dlerror()); ANativeActivity_finish(activity); return; }\n void (*entry)(ANativeActivity*,void*,size_t) = (void (*)(ANativeActivity*,void*,size_t))dlsym(payload, \"ANativeActivity_onCreate\");\n if (!entry) { __android_log_print(ANDROID_LOG_ERROR, \"JaiLoader\", \"Native activity entry not found\"); ANativeActivity_finish(activity); return; }\n entry(activity, state, size);\n}\n");

    return builder_to_string(*builder);
}

android_dynamic_loader_build :: (config: Android_Project_Config, native: Android_Loader_Compiler.Build_Options, options: Android_Build_Options, force: bool) -> bool {
    if native.cpu_target != options.cpu_target || config.lib_name.count == 0 return false;

    paths := get_ndk_paths();

    defer android_release_ndk(paths);
    if paths.root.count == 0 return false;

    triple := get_android_target_triple(options.cpu_target);
    if triple.count == 0 return false;

    directory := tool_path_join(config.project_path, ".build_android_loader");

    defer free(directory);
    if !Android_Loader_File.make_directory_if_it_does_not_exist(directory, true) return false;

    source_path := tool_path_join(directory, "loader.c");

    defer free(source_path);
    library := android_dynamic_loader_path(config);

    defer free(library);
    source := android_dynamic_loader_source(config, options);

    defer free(source);

    if !force && Android_Loader_Files.file_exists(library) {
        old, read := Android_Loader_File.read_entire_file(source_path, log_errors = false);
        defer free(old);
        if read && old == source return true;
    }

    if !Android_Loader_File.write_entire_file(source_path, source) return false;
    if Android_Loader_Files.file_exists(library) && !Android_Loader_File.file_delete(library) return false;

    compiler := tool_path_join(paths.toolchain, android_ndk_compiler_suffix());

    defer free(compiler);
    target := sprint("%%", triple, options.sdk_level);

    defer free(target);
    result, output, errors, timed_out := Android_Loader_Process.run_command(compiler,
        "--target", target, "--sysroot", paths.sysroot, "-shared", "-fPIC",
        ifx options.build_type == .DEBUG then "-O0" else "-O2", source_path,
        "-o", library, "-landroid", "-llog", "-ldl", capture_and_return_output = true, print_captured_output = true);

    defer free(output);
    defer free(errors);

    return result.type == .EXITED && result.exit_code == 0 && !timed_out && Android_Loader_Files.file_exists(library);
}

#scope_file
#import "Basic";
Android_Loader_Compiler :: #import "Compiler";
Android_Loader_File :: #import "File";
Android_Loader_Files :: #import "File_Utilities";
Android_Loader_Process :: #import "Process";
