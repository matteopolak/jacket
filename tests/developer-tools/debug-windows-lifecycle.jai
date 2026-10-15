// Original public behavior fixture for future Windows native execution.
// Mac cross-checking establishes type/ABI validity, not execution evidence.
#import "Basic";
D :: #import "Debug";
R :: #import "Runtime_Support";

custom_logger :: (message: string, data: *void, info: Log_Info) {
    assert(message == "original logger" && data == null);
}

custom_assert :: (loc: Source_Code_Location, message: string) -> bool {
    return false;
}

main :: () {
    #assert OS == .WINDOWS;
    old_logger := context.logger;
    old_assert := context.assertion_failed;

    defer context.logger = old_logger;
    defer context.assertion_failed = old_assert;
    context.logger = custom_logger;
    context.assertion_failed = custom_assert;
    D.init();
    assert(context.logger == custom_logger);
    assert(context.assertion_failed == custom_assert);
    D.disable_signal_handler();
    D.disable_signal_handler();
    context.logger = R.runtime_support_default_logger;
    context.assertion_failed = R.runtime_support_assertion_failed;
    D.init();
    assert(context.logger == D.os_logger);
    assert(context.assertion_failed != R.runtime_support_assertion_failed);
    installed := context.assertion_failed;

    D.init();
    assert(context.logger == D.os_logger && context.assertion_failed == installed);
    D.enable_signal_handler(interactive = true);
    D.set_report_mode(false);
    D.disable_signal_handler();
    D.os_logger("original Windows logger\n", null, Log_Info.{ common_flags = .ERROR });
    print("ok\n");
}
