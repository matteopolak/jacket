// test: skip_on(.WASM)
// Native account and machine queries.
#import "Basic";
#import "Extensions/Testing";

account_queries_find_the_user_and_reject_a_missing_one :: () {
    #if OS != .WASM {
        username, known := S.get_username();

        expect(known && username.count > 0);
        home, found := S.get_home_directory();

        expect(found && home.count > 0);
        home, found = S.get_home_directory(username);
        expect(found && home.count > 0);
        home, found = S.get_home_directory("_platform_io_missing_account_31eea0");
        expect(!found && home.count == 0);
    }
} @test

the_machine_name_and_executable_path_are_known :: () {
    #if OS != .WASM {
        expect(S.get_machinename().count > 0);
        expect(S.get_path_of_running_executable().count > 0);
    }
} @test

processor_counts_are_consistent :: () {
    #if OS != .WASM {
        logical := S.get_number_of_processors(.ALL_LOGICAL);
        physical := S.get_number_of_processors(.ALL_PHYSICAL);
        fast_logical := S.get_number_of_processors(.PERFORMANCE_LOGICAL);
        fast_physical := S.get_number_of_processors(.PERFORMANCE_PHYSICAL);

        expect(physical > 0 && logical >= physical);
        expect(fast_physical > 0 && fast_logical >= fast_physical);
        expect(logical >= fast_logical && physical >= fast_physical);
    }
} @test

the_last_error_is_reported_with_a_description :: () {
    #if OS != .WASM {
        #if OS == .WINDOWS {
            counter := S.read_cpu_counter();

            W.Sleep(1);
            expect(S.read_cpu_counter() > counter);
            set_last_error(2);
            expect_equal(S.get_error_value(), 2);
            code, description := S.get_error_value_and_string();

            expect(type_of(code) == W.OS_Error_Code);
            expect(code == 2 && description.count > 0);
        } else {
            P.set_errno(P.ENOENT);
            expect(S.get_error_value() == P.ENOENT);
            code, description := S.get_error_value_and_string();
            expect(code == P.ENOENT && description.count > 0);
        }
    }
} @test

// System does not build for wasm: there the tests are skipped and their bodies left out.
#scope_file

#if OS != .WASM {
    S :: #import "System";

    #if OS == .WINDOWS W :: #import "Windows";
    else P :: #import "POSIX";

    #if OS == .WINDOWS {
        kernel32 :: #system_library "kernel32";
        set_last_error :: (code: u32) #foreign kernel32 "SetLastError";
    }
}
