JAVA_ENV_NAME :: "JAVA_HOME";

find_java_bin_path_from_environment :: () -> bool, bin_path: string, exe_path: string {
    C :: #import "Process";
    home := tool_environment(JAVA_ENV_NAME);

    defer free(home);
    if home.count == 0 return false, "", "";

    // Preserve native roots; trailing separators must not duplicate /bin.
    while home.count > 1 && (home[home.count - 1] == 47 || home[home.count - 1] == 92) home.count -= 1;

    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, home);
    append(*builder, "/bin");
    bin := builder_to_string(*builder);

    reset(*builder);
    append(*builder, bin);
    #if OS == .WINDOWS append(*builder, "/java.exe");
    else append(*builder, "/java");
    exe := builder_to_string(*builder);

    result, output, errors, timed_out := C.run_command(exe, "-version", capture_and_return_output = true);

    defer free(output);
    defer free(errors);

    if result.type != .EXITED || result.exit_code != 0 || timed_out {
        free(bin);
        free(exe);
        return false, "", "";
    }

    return true, bin, exe;
}

#scope_module
#import "Basic";

tool_environment :: (variable: string) -> string {
    #if OS == .WINDOWS {
        W :: #import "Windows_Utf8";
        name, converted := W.utf8_to_wide(variable);

        defer free(name);
        if !converted return "";

        capacity := java_get_environment(name, null, 0);
        if capacity == 0 || capacity > 2147483647 return "";

        buffer := cast(*u16) alloc(cast(s64) capacity * 2);
        if !buffer return "";

        defer free(buffer);
        count := java_get_environment(name, buffer, capacity);
        // Environment mutation during the query must not expose partial output.
        if count == 0 || count >= capacity return "";

        value, valid := W.wide_to_utf8(buffer, cast(s32) count);
        if valid return value;

        free(value);

        return "";
    } else {
        name := to_c_string(variable);

        defer free(name);
        value := java_get_environment(name);
        if !value return "";

        count: s64;
        while value[count] != 0 count += 1;

        return copy_string(.{ count, value });
    }
}

#if OS == .WINDOWS {
    java_environment_library :: #system_library "kernel32";
    java_get_environment :: (lpName: *u16, lpBuffer: *u16, nSize: u32) -> u32 #foreign java_environment_library "GetEnvironmentVariableW";
} else {
    java_environment_library :: #system_library "libc";
    java_get_environment :: (name: *u8) -> *u8 #foreign java_environment_library "getenv";
}
