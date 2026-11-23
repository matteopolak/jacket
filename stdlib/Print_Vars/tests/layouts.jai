// print_vars output: default layout, an expression as the name, right-justified names, custom separators, and a type-only line.
#import "Basic";
P :: #import "Print_Vars";

main :: () {
    builder: String_Builder;

    defer free_buffers(*builder);

    count: s32 = 7;
    big: s32 = 1234;
    small: u8 = 3;

    P.print_vars_to(*builder, count, big, small);
    default_text := builder_to_string(*builder,, temp);

    print("default:\n%", default_text);
    assert(default_text == "count = 7    (s32)\nbig   = 1234 (s32)\nsmall = 3    (u8)\n");

    // The name is the expression text in parentheses, and the value is its result.
    reset(*builder);
    P.print_vars_to(*builder, count + 1);
    expression_text := builder_to_string(*builder,, temp);

    print("expression: %", expression_text);
    assert(expression_text == "(count + 1) = 8 (s32)\n");

    // Right-justified names, no value indentation, and custom separators.
    reset(*builder);
    config: P.Print_Vars_Config;
    config.control_flags = .PRINT_NAME | .PRINT_VALUE;
    config.before_name = "<";
    config.after_name = "> ";
    config.after_value = ";";
    config.line_separator = " | ";
    config.name_indentation = .RIGHT_JUSTIFIED;
    config.value_indentation = .NONE;
    P.print_helper(*builder, config, 5, 4, "ab", "7", s32);
    P.print_helper(*builder, config, 5, 4, "longer", "42", s32);
    custom_text := builder_to_string(*builder,, temp);

    print("custom: [%]\n", custom_text);
    assert(custom_text == "<   ab> 7; | <longer> 42; | ");

    // Type only: the line is the type alone, wrapped in its own text.
    reset(*builder);
    type_only: P.Print_Vars_Config;
    type_only.control_flags = .PRINT_TYPE;
    type_only.before_type = "type=";
    type_only.after_type = "";
    type_only.line_separator = "\n";
    P.print_helper(*builder, type_only, 0, 0, "ignored", "ignored", u8);
    type_text := builder_to_string(*builder,, temp);

    print("type only: %", type_text);
    assert(type_text == "type=u8\n");

    print("ok\n");
}
