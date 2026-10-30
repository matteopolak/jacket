// test: only(.LINUX,.MACOS)
#import "Basic";
#import "Extensions/Testing";

the_user_has_a_name_and_a_home_directory :: () {
    #if OS != .WASM {
        user, ok := S.get_username();

        expect(ok && user.count > 0);
        home, found := S.get_home_directory(user);

        expect(found && home.count > 0);
        home, found = S.get_home_directory("_portable_stdlib_missing_account_751ad90");
        expect(!found && home.count == 0);

        #if OS == .LINUX || OS == .ANDROID {
            account_ok, account := S.get_passwd_for_user(user);
            expect(account_ok && to_string(account.pw_name) == user);
        }
    }
} @test

machine_queries_return_something :: () {
    #if OS != .WASM {
        expect(S.get_machinename().count > 0);
        expect(S.get_path_of_running_executable().count > 0);
    }
} @test

processor_counts_are_positive_and_ordered :: () {
    #if OS != .WASM {
        expect(S.get_number_of_processors(.ALL_LOGICAL) > 0);
        expect(S.get_number_of_processors(.ALL_PHYSICAL) > 0);
        expect(S.get_number_of_processors(.PERFORMANCE_LOGICAL) > 0);
        expect(S.get_number_of_processors(.PERFORMANCE_PHYSICAL) > 0);
        expect(S.get_number_of_processors(.ALL_PHYSICAL) <= S.get_number_of_processors(.ALL_LOGICAL));
        expect(S.get_number_of_processors(.PERFORMANCE_LOGICAL) <= S.get_number_of_processors(.ALL_LOGICAL));
    }
} @test

error_values_and_strings_follow_errno :: () {
    #if OS != .WASM {
        #if OS != .WINDOWS {
            P.set_errno(P.ENOENT);
            expect(S.get_error_value() == P.ENOENT);
            code, description := S.get_error_value_and_string();

            expect(code == P.ENOENT && description.count > 0);
            expect(S.get_error_string(P.ENOENT).count > 0);
        }
    }
} @test

// System does not build for wasm: there the tests are skipped and their bodies left out.
#scope_file

#if OS != .WASM {
    S :: #import "System";
    #if OS != .WINDOWS P :: #import "POSIX";
}
