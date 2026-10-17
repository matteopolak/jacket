#import "Basic";
S :: #import "System";
#if OS == .WINDOWS W :: #import "Windows";
else P :: #import "POSIX";

main :: () {
    mark := get_temporary_storage_mark();

    defer set_temporary_storage_mark(mark);
    username, known := S.get_username();

    assert(known && username.count > 0);
    home, found := S.get_home_directory();

    assert(found && home.count > 0);
    home, found = S.get_home_directory(username);
    assert(found && home.count > 0);
    home, found = S.get_home_directory("_platform_io_missing_account_31eea0");
    assert(!found && home.count == 0);
    assert(S.get_machinename().count > 0);
    assert(S.get_path_of_running_executable().count > 0);
    logical := S.get_number_of_processors(.ALL_LOGICAL);
    physical := S.get_number_of_processors(.ALL_PHYSICAL);
    fast_logical := S.get_number_of_processors(.PERFORMANCE_LOGICAL);
    fast_physical := S.get_number_of_processors(.PERFORMANCE_PHYSICAL);

    assert(physical > 0 && logical >= physical);
    assert(fast_physical > 0 && fast_logical >= fast_physical);
    assert(logical >= fast_logical && physical >= fast_physical);

    #if OS == .WINDOWS {
        counter := S.read_cpu_counter();

        W.Sleep(1);
        assert(S.read_cpu_counter() > counter);
        set_last_error(2);
        assert(S.get_error_value() == 2);
        code, description := S.get_error_value_and_string();

        assert(type_of(code) == W.OS_Error_Code);
        assert(code == 2 && description.count > 0);
    } else {
        P.set_errno(P.ENOENT);
        assert(S.get_error_value() == P.ENOENT);
        code, description := S.get_error_value_and_string();
        assert(code == P.ENOENT && description.count > 0);
    }

    print("ok\n");
}

#scope_file
#if OS == .WINDOWS {
    kernel32 :: #system_library "kernel32";
    set_last_error :: (code: u32) #foreign kernel32 "SetLastError";
}
