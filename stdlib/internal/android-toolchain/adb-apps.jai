stop_app :: (package_name: string) -> bool {
    quoted := quote_remote(package_name);
    defer free(quoted);
    return shell_status("am", "force-stop", quoted);
}

android_is_app_installed :: (package_name: string) -> bool {
    quoted := quote_remote(package_name);

    defer free(quoted);
    success, output, errors := adb("shell", "pm", "path", quoted, try_and_escape_spaces_in_shell_arguments = false);

    defer free(output);
    defer free(errors);
    if !success return false;

    remaining := output;
    found := false;

    while remaining.count > 0 {
        line := take_line(*remaining);
        if line.count == 0 continue;
        if !starts(line, "package:/") || line.count <= 9 return false;
        found = true;
    }

    return found;
}

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
