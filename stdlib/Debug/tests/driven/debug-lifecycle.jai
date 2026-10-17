// Reviewed Debug init/handler lifecycle and assertion callback ownership.
#import "Basic";
D :: #import "Debug";
R :: #import "Runtime_Support";

oracle :: #foreign_library "../../../../target/developer_tools_debug_lifecycle";
install_prior :: () -> s32 #foreign oracle "original_debug_install_prior";
changed_actions :: () -> s32 #foreign oracle "original_debug_changed_actions";
prior_restored :: () -> s32 #foreign oracle "original_debug_prior_restored";
restore_original :: () -> s32 #foreign oracle "original_debug_restore_original";

custom_assert :: (loc: Source_Code_Location, message: string) -> bool {
    assert(loc.line_number == 29 && message == "original assertion");
    return false;
}

main :: () {
    assert(install_prior() == 1);
    defer assert(restore_original() == 1);
    assert(cast(int) D.Assert_Response.Ignore == 0);
    assert(cast(int) D.Assert_Response.Debug == 1);
    assert(cast(int) D.Assert_Response.Abort == 2);
    old := context.assertion_failed;

    defer context.assertion_failed = old;
    context.assertion_failed = custom_assert;
    D.init();
    assert(changed_actions() == 5);
    assert(context.assertion_failed == custom_assert);
    D.init();
    assert(context.assertion_failed == custom_assert);
    D.disable_signal_handler();
    D.disable_signal_handler();
    assert(prior_restored() == 1);
    context.assertion_failed = R.runtime_support_assertion_failed;
    D.init();
    assert(context.assertion_failed != R.runtime_support_assertion_failed);
    installed := context.assertion_failed;

    D.init();
    assert(context.assertion_failed == installed);
    D.set_report_mode(false);
    D.enable_signal_handler(interactive = true);
    D.enable_signal_handler();
    assert(changed_actions() == 5);
    D.disable_signal_handler();
    assert(prior_restored() == 1);
    location := Source_Code_Location.{ "original-debug.jai", 29, 5 };

    D.no_assert(location, "original assertion");
    D.print_assert(location, "original assertion");
    D.trace_assert(location, "original trace");
    print("ok\n");
}
