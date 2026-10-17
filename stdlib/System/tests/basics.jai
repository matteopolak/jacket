#import "Basic";
S :: #import "System";
P :: #import "POSIX";

main :: () {
    mark := get_temporary_storage_mark();

    defer set_temporary_storage_mark(mark);
    user, ok := S.get_username();

    assert(ok && user.count > 0);
    home, found := S.get_home_directory(user);

    assert(found && home.count > 0);
    home, found = S.get_home_directory("_portable_stdlib_missing_account_751ad90");
    assert(!found && home.count == 0);
    assert(S.get_machinename().count > 0);
    assert(S.get_path_of_running_executable().count > 0);
    assert(S.get_number_of_processors(.ALL_LOGICAL) > 0);
    assert(S.get_number_of_processors(.ALL_PHYSICAL) > 0);
    assert(S.get_number_of_processors(.PERFORMANCE_LOGICAL) > 0);
    assert(S.get_number_of_processors(.PERFORMANCE_PHYSICAL) > 0);
    assert(S.get_number_of_processors(.ALL_PHYSICAL) <= S.get_number_of_processors(.ALL_LOGICAL));
    assert(S.get_number_of_processors(.PERFORMANCE_LOGICAL) <= S.get_number_of_processors(.ALL_LOGICAL));

    #if OS == .LINUX || OS == .ANDROID {
        account_ok, account := S.get_passwd_for_user(user);
        assert(account_ok && to_string(account.pw_name) == user);
    }

    P.set_errno(P.ENOENT);
    assert(S.get_error_value() == P.ENOENT);
    code, description := S.get_error_value_and_string();

    assert(code == P.ENOENT && description.count > 0);
    assert(S.get_error_string(P.ENOENT).count > 0);
    print("ok\n");
}
