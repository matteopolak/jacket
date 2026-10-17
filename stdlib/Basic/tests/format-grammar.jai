#import "Basic";

main :: () {
    errors: int;

    push_context {
        context.logger = count_error;
        context.logger_data = *errors;
        assert(sprint("%2 %1 %", "A", "B") == "B A B");
        assert(sprint("left\x1fright") == "left%right");
        assert(sprint("[%00] %0 %-", "A", "B") == "[] A B-");
        assert(sprint("%%", "A", "B") == "AB");
        extra := "extra";

        assert(sprint(extra, 1, 2) == "extra");
        builder: String_Builder;

        defer free_buffers(*builder);
        dynamic := "%4/%/%";

        assert(print_to_builder(*builder, dynamic, "A"));
        result := builder_to_string(*builder);

        defer free(result);
        assert(result == "/A/");
        assert(errors == 2);
        context.print_style.log_runtime_errors = false;
        assert(sprint(dynamic, "A") == "/A/");
        assert(errors == 2);
        context.print_style.default_format_int.base = 16;
        assert(sprint("%", 31) == "1f");
        assert(sprint("%", formatInt(31)) == "31");
    }

    assert(sprint("%", 31) == "31");
    assert(my_is_digit(48) && my_is_digit(57) && !my_is_digit(-1) && !my_is_digit(58));
    end, value := parse_number("x123!", 1);

    assert(end == 4 && value == 123);
    print("ok\n");
}

count_error :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
    count := cast(*int) data;
    count.* += 1;
}
