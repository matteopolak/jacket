#scope_module
android_host_literal :: (builder: *String_Builder, value: string) {
    append(builder, #char "\"");

    for byte: value {
        if byte == #char "\"" append(builder, "\\\"");
        else if byte == #char "\\" append(builder, "\\\\");
        else if byte == 10 append(builder, "\\n");
        else if byte == 13 append(builder, "\\r");
        else if byte == 9 append(builder, "\\t");
        else if byte == 0 append(builder, "\\0");
        else append(builder, byte);
    }

    append(builder, #char "\"");
}

android_host_helper_source :: (app_name: string, package_name: string, activity: string, debug: bool) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    print(*builder, "// Android helper version %, debug=%\n", ANDROID_HELPER_VERSION, debug);
    append(*builder, "#import \"Basic\";\nAT :: #import \"Android/Toolchain\";\nmain :: () {\n print(\"Starting %\\n\", ");
    android_host_literal(*builder, app_name);
    append(*builder, ");\n success, output, errors := AT.adb(\"shell\", \"am\", \"start\", \"-n\", ");
    component := sprint("%/%", package_name, activity);

    defer free(component);
    quoted_component := quote_remote(component);

    defer free(quoted_component);
    android_host_literal(*builder, quoted_component);
    append(*builder, ", print_captured_output = true, try_and_escape_spaces_in_shell_arguments = false);\n free(output);\n free(errors);\n if !success exit(1);\n}\n");

    return builder_to_string(*builder);
}

android_host_helper_compile :: (app_name: string, package_name: string, output_path: string, activity: string, debug: bool, force: bool) -> bool #compile_time {
    if !android_device_package_valid(package_name) || !android_host_activity_valid(activity) return false;

    directory := ifx output_path.count > 0 then output_path else ".";
    if !Android_Helper_File.make_directory_if_it_does_not_exist(directory, true) return false;

    source_name := tool_path_join(directory, ".jai-android-helper-v2.jai");

    defer free(source_name);
    executable := tool_path_join(directory, android_get_helper_program_executable_name());

    defer free(executable);
    source := android_host_helper_source(app_name, package_name, activity, debug);

    defer free(source);

    if !force && Android_Helper_Files.file_exists(executable) {
        previous, read := Android_Helper_File.read_entire_file(source_name, log_errors = false);
        defer free(previous);
        if read && previous == source return true;
    }

    if !Android_Helper_File.write_entire_file(source_name, source) return false;

    configured_compiler := tool_environment("JAIC");

    defer free(configured_compiler);
    compiler := ifx configured_compiler.count > 0 then configured_compiler else "jaic";
    optimization := ifx debug then "-O0" else "-O2";
    outcome, output, errors, timed_out := Android_Helper_Process.run_command(compiler, source_name, optimization, "-exe", executable,
        capture_and_return_output = true, print_captured_output = true);

    defer free(output);
    defer free(errors);

    return outcome.type == .EXITED && outcome.exit_code == 0 && !timed_out && Android_Helper_Files.file_exists(executable);
}

android_host_activity_valid :: (activity: string) -> bool {
    if activity.count == 0 return false;

    if activity[0] == #char "." {
        activity.data += 1;
        activity.count -= 1;
    }

    if activity.count == 0 return false;

    for byte: activity {
        if (byte >= #char "a" && byte <= #char "z") || (byte >= #char "A" && byte <= #char "Z") || (byte >= #char "0" && byte <= #char "9") || byte == #char "_" || byte == #char "." || byte == #char "$" continue;
        return false;
    }

    return true;
}

#scope_file
#import "Basic";
Android_Helper_File :: #import "File";
Android_Helper_Files :: #import "File_Utilities";
Android_Helper_Process :: #import "Process";
