#import "Basic";
P :: #import "Print_Vars";

evaluations: s32;
logged: string;
logged_location: Source_Code_Location;

increment :: () -> s32 {
    evaluations += 1;
    return 9;
}

capture_log :: (message: string, _data: *void, info: Log_Info) {
    logged = copy_string(message);
    logged_location = info.location;
}

main :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    x: s32 = 7;
    y: s32 = 22;

    P.print_vars_to(*builder, x, y);
    text := builder_to_string(*builder);

    defer free(text);
    assert(text == "x = 7  (s32)\ny = 22 (s32)\n");
    reset(*builder);
    P.print_vars_to(*builder, increment());
    assert(evaluations == 1);
    side_effect := builder_to_string(*builder);

    defer free(side_effect);
    assert(side_effect == "increment() = 9 (s32)\n");
    reset(*builder);
    P.print_vars_to(*builder);
    assert(builder_string_length(*builder) == 0);
    config: P.Print_Vars_Config;
    config.control_flags = .PRINT_NAME | .PRINT_VALUE;
    config.before_name = "[";
    config.after_name = "]: ";
    config.after_value = "!";
    config.name_indentation = .RIGHT_JUSTIFIED;
    config.value_indentation = .NONE;
    P.print_helper(*builder, config, 3, 9, "a", "5", s32);
    custom := builder_to_string(*builder);

    defer free(custom);
    assert(custom == "[  a]: 5!\n");

    push_context {
        context.logger = capture_log;
        P.log_vars(x);
    }

    defer free(logged);
    assert(logged == "x = 7 (s32)\n");
    assert(logged_location.line_number > 0);
    P.print_vars(x);
    print("ok\n");
}
