// Force-stops the app `package_name` on the device; returns `true` when `adb` succeeds.
stop_app :: (package_name: string) -> bool {
    quoted := quote_remote(package_name);
    defer free(quoted);
    return shell_status("am", "force-stop", quoted);
}

// Returns whether `package_name` is installed on the device, by asking `pm path`.
//
// Returns `false` when `adb` fails or prints a line that is not a `package:/` path.
android_is_app_installed :: (package_name: string) -> bool {
    quoted := quote_remote(package_name);

    defer free(quoted);
    success, output, errors := adb("shell", "pm", "path", quoted, try_and_escape_spaces_in_shell_arguments = false);

    defer free(output);
    defer free(errors);
    if !success return false;

    remaining := output;
    PACKAGE_PREFIX :: "package:/";

    found := false;

    while remaining.count > 0 {
        line := take_line(*remaining);
        if line.count == 0 continue;
        if !starts(line, PACKAGE_PREFIX) || line.count <= PACKAGE_PREFIX.count return false;
        found = true;
    }

    return found;
}

// Returns whether the app `package_name` has a running process, and whether the query worked.
//
// `pidof` reporting no process counts as a successful query with `running` false. When `success` is false,
// `running` is false and means nothing.
is_app_running :: (package_name: string) -> running: bool, success: bool {
    quoted := quote_remote(package_name);

    defer free(quoted);
    completed, output, errors, code := adb("shell", "pidof", quoted, try_and_escape_spaces_in_shell_arguments = false);

    defer free(output);
    defer free(errors);

    if !completed {
        // pidof reports no matching process with status 1 and no diagnostic.
        if code == 1 && output.count == 0 && errors.count == 0 return false, true;
        return false, false;
    }

    valid, pid := parse_process_ids(output);

    return valid && pid > 0, valid;
}

#scope_module
parse_process_ids :: (output: string) -> bool, s32 {
    remaining := output;
    first: s32;

    while remaining.count > 0 {
        word := take_word(*remaining);
        if word.count == 0 break;

        value: s64;

        for digit: word {
            if digit < #char "0" || digit > #char "9" return false, 0;
            value = value * 10 + cast(s64) (digit - #char "0");
            if value > 0x7fffffff return false, 0;
        }

        if value == 0 return false, 0;
        if first == 0 first = cast(s32) value;
    }

    return first != 0, first;
}
