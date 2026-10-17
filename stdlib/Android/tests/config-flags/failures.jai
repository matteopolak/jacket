// Independent validation policy; fatal verification remains enabled after call.
#import "Basic";
A :: #import "Android/Toolchain";

errors: s64;
assertions: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if message.count > 0 && (info.common_flags & .ERROR) != 0 errors += 1;
}

continue_failure :: (location: Source_Code_Location, message: string) -> bool {
    assertions += 1;
    return false;
}

main :: () {
    original_logger := context.logger;
    original_assertion := context.assertion_failed;

    for text, index: string.["unknown", "mcc|unknown", "mcc||mnc", "mcc|", "|mcc", "Orientation", "mcc mnc"] {
        context.logger = capture;
        context.assertion_failed = continue_failure;
        result := A.android_config_changes_from_string(text);

        context.logger = original_logger;
        context.assertion_failed = original_assertion;
        assert(cast(s32) result == 0);
        assert(errors == index + 1 && assertions == index + 1);
    }

    print("ok\n");
}
