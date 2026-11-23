// log_vars sends the same lines that print_vars_to builds to the logger, once per call, with the caller's location.
#import "Basic";
P :: #import "Print_Vars";

logged: string;
logged_line: s64;
log_count: s64;

capture_log :: (message: string, _data: *void, info: Log_Info) {
    log_count += 1;
    logged = copy_string(message);
    logged_line = info.location.line_number;
}

main :: () {
    builder: String_Builder;

    defer free_buffers(*builder);

    value: s64 = -42;
    flag := true;

    push_context {
        context.logger = capture_log;
        P.log_vars(value);
    }

    print("logged once=% text=%", log_count == 1, logged);
    assert(log_count == 1 && logged == "value = -42 (s64)\n");
    assert(logged_line == 24);

    // The same call through the builder for comparison.
    P.print_vars_to(*builder, value, flag);
    text := builder_to_string(*builder,, temp);

    print("builder text: %", text);
    assert(text == "value = -42  (s64)\nflag  = true (bool)\n");
    print("ok\n");
}
