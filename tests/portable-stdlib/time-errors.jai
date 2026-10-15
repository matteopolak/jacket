#import "Basic";
#load "../../stdlib/internal/basic/time-errors.jai";

reported: s64;

capture :: (loc: Source_Code_Location, message: string) -> bool {
    reported += 1;
    assert(message.count > 0 && context.handling_assertion_failure);
    report_time_failure("nested reporting must not recurse");
    return false;
}

main :: () {
    previous := context.assertion_failed;

    context.assertion_failed = capture;
    defer context.assertion_failed = previous;
    assert(!context.handling_assertion_failure);
    report_time_failure("original diagnostic");
    assert(reported == 1 && !context.handling_assertion_failure);
    value := time_failure_nan("elapsed failure");

    assert(value != value && reported == 2 && !context.handling_assertion_failure);
    context.handling_assertion_failure = true;
    report_time_failure("already handling an error");
    assert(reported == 2 && context.handling_assertion_failure);
    context.handling_assertion_failure = false;
    print("ok\n");
}
