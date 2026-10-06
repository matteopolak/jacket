// System: the running executable's path, user, home directory and machine names, OS error
// codes and their descriptions, and processor counts.
#import "Basic";
#import "File";
#import "File_Utilities";
#import "String";
#import "System";

main :: () {
    // The WASM sandbox has no users, host names or executable files.
    #if OS != .WASM {
        executable := get_path_of_running_executable();
        assert(executable.count > 0, "the running executable has a path");
        if file_exists(executable) {
            directory, found := is_directory(executable);
            assert(found && !directory, "'%' is a file", executable);
        } else {
            // Under `jaic run` it is the executable `jaic build` would write, which may not exist.
            assert(contains(path_filename(executable), "system-info"), "'%' names the program", executable);
        }
        #if OS == .WINDOWS {
            assert(executable.count > 3 && executable[1] == #char ":", "absolute '%'", executable);
        } else {
            assert(executable[0] == #char "/", "absolute '%'", executable);
        }

        user, user_ok := get_username();
        assert(user_ok && user.count > 0, "username");
        home, home_ok := get_home_directory();
        assert(home_ok && home.count > 0, "home directory");
        home_is_directory := is_directory(home);
        assert(home_is_directory, "home '%' is a directory", home);
        #if OS == .WINDOWS {
            // Only the current user's directory is known.
            _, other_ok := get_home_directory("someone-else");
            assert(!other_ok);
        } else {
            named, named_ok := get_home_directory(user);
            assert(named_ok && named == home, "home of % is '%', not '%'", user, named, home);
            _, missing_ok := get_home_directory("no-such-user-jaic-test");
            assert(!missing_ok);
            passwd_ok, entry := get_passwd_for_user();
            assert(passwd_ok && to_string(entry.pw_name) == user && to_string(entry.pw_dir) == home, "passwd entry");
            passwd_ok, entry = get_passwd_for_user(user);
            assert(passwd_ok && to_string(entry.pw_name) == user);
            passwd_ok = get_passwd_for_user("no-such-user-jaic-test");
            assert(!passwd_ok);
        }

        machine := get_machinename();
        assert(machine.count > 0 && machine.count < 256, "machine name '%'", machine);
        for machine assert(it >= 32, "printable machine name '%'", machine);
    }

    {
        logical := get_number_of_processors(.ALL_LOGICAL);
        physical := get_number_of_processors(.ALL_PHYSICAL);
        fast_logical := get_number_of_processors();
        fast_physical := get_number_of_processors(.PERFORMANCE_PHYSICAL);
        assert(logical >= 1 && physical >= 1 && fast_logical >= 1 && fast_physical >= 1, "processors % % % %", logical, physical, fast_logical, fast_physical);
        assert(physical <= logical && fast_logical <= logical && fast_physical <= physical && fast_physical <= fast_logical, "processors % % % %", logical, physical, fast_logical, fast_physical);
        assert(get_number_of_processors_for_type(.ALL_LOGICAL) == logical);
    }

    // The error code a failed call leaves behind, and its description.
    _, opened := file_open("no-such-file-for-system-info.txt", log_errors = false);
    assert(!opened);
    code := get_error_value();
    assert(code == 2, "a missing file is error 2 (ENOENT, ERROR_FILE_NOT_FOUND), not %", code);
    description := get_error_string(code);
    assert(contains(to_lower_copy(description), "file"), "error 2 is '%'", description);
    _, opened = file_open("no-such-file-for-system-info.txt", log_errors = false);
    same_code, same_description := get_error_value_and_string();
    assert(same_code == 2 && same_description == description, "% '%'", same_code, same_description);
    assert(get_error_string(0).count > 0);
    // A code no OS uses still gets some text.
    assert(get_error_string(cast(OS_Error_Code) 987654).count > 0);

    print("ok\n");
}
