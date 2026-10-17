#import "Basic";
D :: #import "Debug";
N :: #import "Debug"(USE_GRAPHICS = false);
T :: #import "Hash_Table";
R :: #import "Runtime_Support";

main :: () {
    #assert !#exists(N.interactive_assert);
    #assert !#exists(N.os_show_assert_dialog);
    #assert !#exists(D.init_crash_handler_thread);
    #assert !#exists(N.init_crash_handler_thread);
    #assert D.Exit_Code == N.Exit_Code;
    #assert D.Assert_Response == N.Assert_Response;
    assert(*D.crash_handler == *N.crash_handler);
    assert(*D.ignored_asserts == *N.ignored_asserts);
    assert(!D.os_attach_to_debugger() && !N.os_attach_to_debugger());
    assert(D.attach_to_debugger() == D.is_debugger_present());
    location := Source_Code_Location.{ "original-final-debug.jai", 42, 6 };

    assert(D.break_assert(location, "original break assertion") == D.is_debugger_present());
    assert(D.os_show_assert_dialog(location, "original dialog") == .Abort);
    N.set_report_mode(false);
    assert(D.interactive_assert(location, "original noninteractive assertion"));
    T.table_set(*N.ignored_asserts, location, true);
    defer T.table_reset(*N.ignored_asserts);
    D.set_report_mode(true);
    assert(!D.interactive_assert(location, "original ignored assertion"));
    D.set_minidump_state(.Full);
    assert(D.write_minidump(null).count == 0);
    assert(N.write_minidump(null, true).count == 0);
    assert(D.get_written_minidump_filename().count == 0);
    old := context.assertion_failed;

    defer context.assertion_failed = old;
    context.assertion_failed = R.runtime_support_assertion_failed;
    N.init();
    assert(context.assertion_failed == N.break_assert);
    N.disable_signal_handler();
    context.assertion_failed = R.runtime_support_assertion_failed;
    D.init();
    assert(context.assertion_failed == D.interactive_assert);
    D.disable_signal_handler();
    D.set_report_mode(false);
    print("ok\n");
}
