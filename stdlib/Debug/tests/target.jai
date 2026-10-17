#import "Basic";
D :: #import "Debug";
N :: #import "Debug"(USE_GRAPHICS = false);

main :: () {
    #assert D.Exit_Code == N.Exit_Code;
    #assert D.Assert_Response == N.Assert_Response;
    #assert D.Minidump_State == N.Minidump_State;
    #assert !D.CRASH_HANDLER_USE_SEPARATE_THREAD;
    #assert !#exists(D.init_crash_handler_thread);
    #assert !#exists(N.init_crash_handler_thread);
    #assert !#exists(N.interactive_assert);
    #assert !#exists(N.os_show_assert_dialog);
    assert(*D.crash_handler == *N.crash_handler);
    assert(*D.ignored_asserts == *N.ignored_asserts);
    assert(type_info(D.Exit_Code).runtime_size == size_of(s32));
    assert(cast(s32) D.Exit_Code.Exception == 4);
    assert(cast(s32) D.Assert_Response.Ignore == 0);
    assert(cast(s32) D.Assert_Response.Debug == 1);
    assert(cast(s32) D.Assert_Response.Abort == 2);

    #if OS == .ANDROID || OS == .WASM {
        assert(!D.os_is_debugger_present() && !N.os_is_debugger_present());
        assert(!D.is_debugger_present() && !N.is_debugger_present());
    }

    #if OS != .WINDOWS {
        assert(D.write_minidump(null).count == 0);
        assert(N.write_minidump(null, large_dump = true).count == 0);
        assert(D.get_written_minidump_filename().count == 0);
        assert(N.get_written_minidump_filename().count == 0);
    }

    print("ok\n");
}
