#import "Basic";

calls: int;
received: string;
received_location: Source_Code_Location;
answer: bool;

capture :: (location: Source_Code_Location, message: string) -> bool {
    calls += 1;
    received = message;
    received_location = location;
    return answer;
}

main :: () {
    successful_calls: int;
    failed_calls: int;
    first_message: string;
    first_location: Source_Code_Location;
    helper_result: bool;

    push_context {
        context.assertion_failed = capture;
        assert(true, "must not dispatch %", 7);
        successful_calls = calls;

        assert(false, "value % / %", 7, "text");
        failed_calls = calls;
        first_message = copy_string(received);
        first_location = received_location;

        answer = true;
        helper_result = assert_helper("answer %", 9);
    }

    defer free(first_message);
    assert(successful_calls == 0);
    assert(failed_calls == 1 && first_message == "value 7 / text");
    assert(first_location.line_number > 0);
    assert(helper_result);
    assert(calls == 2 && received == "answer 9");
    print("ok\n");
}
