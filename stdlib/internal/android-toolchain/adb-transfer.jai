adb_push :: (local_source: string, remote_destination: string, only_update_newer_files := false, dry_run := false, compression_algorithm := Adb_Compression_Algorithm.UNSPECIFIED, disable_compression := false) -> bool {
    arguments: [..] string;

    defer array_free(arguments);
    array_add(*arguments, "push");
    if only_update_newer_files array_add(*arguments, "--sync");
    if dry_run array_add(*arguments, "-n");
    if !compression_arguments(*arguments, compression_algorithm, disable_compression) return false;

    array_add(*arguments, local_source);
    array_add(*arguments, remote_destination);

    return discard_capture(arguments);
}

adb_pull :: (remote_source: string, local_destination := "", preserve_timestamps := true, disable_compression := false, compression_algorithm := Adb_Compression_Algorithm.UNSPECIFIED) -> bool {
    arguments: [..] string;

    defer array_free(arguments);
    array_add(*arguments, "pull");
    if preserve_timestamps array_add(*arguments, "-a");
    if !compression_arguments(*arguments, compression_algorithm, disable_compression) return false;

    array_add(*arguments, remote_source);
    if local_destination.count > 0 array_add(*arguments, local_destination);

    return discard_capture(arguments);
}

adb_shell_mkdir :: (path: string) -> bool {
    quoted := quote_remote(path);
    defer free(quoted);
    return shell_status("mkdir", "-p", quoted);
}

adb_shell_pidof :: (package_name: string) -> bool, s32 {
    quoted := quote_remote(package_name);

    defer free(quoted);
    success, output, errors, code := adb("shell", "pidof", quoted, try_and_escape_spaces_in_shell_arguments = false);

    defer free(output);
    defer free(errors);

    if !success {
        if code != 1 context.logger("ADB PID query failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false, -1;
    }

    remaining := output;
    first := take_word(*remaining);
    valid, pid := parse_process_ids(first);

    if !valid {
        context.logger("invalid ADB process ID\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false, -1;
    }

    if take_word(*remaining).count > 0 context.logger("ADB PID query ignored additional process IDs\n", context.logger_data, Log_Info.{ common_flags = .WARNING });

    return true, pid;
}

adb_uninstall :: (package_name: string) -> bool {
    success, output, errors := adb("uninstall", package_name);
    defer free(output);
    defer free(errors);
    return success;
}

#scope_module
discard_capture :: (arguments: [] string) -> bool {
    success, output, errors := adb(..arguments);
    defer free(output);
    defer free(errors);
    return success;
}

shell_status :: (arguments: ..string) -> bool {
    all: [..] string;

    defer array_free(all);
    array_add(*all, "shell");
    for argument: arguments array_add(*all, argument);

    success, output, errors := adb(..all, try_and_escape_spaces_in_shell_arguments = false);

    defer free(output);
    defer free(errors);

    return success;
}

compression_arguments :: (arguments: *[..] string, algorithm: Adb_Compression_Algorithm, disabled: bool) -> bool {
    if disabled {
        if algorithm != .UNSPECIFIED {
            adb_failure("ADB compression algorithm conflicts with disabled compression");
            return false;
        }

        array_add(arguments, "-Z");

        return true;
    }

    if algorithm == .UNSPECIFIED return true;

    name: string;
    if algorithm == .ANY name = "any";
    else if algorithm == .NONE name = "none";
    else if algorithm == .BROTLI name = "brotli";
    else if algorithm == .LZ4 name = "lz4";
    else if algorithm == .ZSTD name = "zstd";
    else {
        context.logger("unsupported ADB compression algorithm\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false;
    }

    array_add(arguments, "-z");
    array_add(arguments, name);

    return true;
}
