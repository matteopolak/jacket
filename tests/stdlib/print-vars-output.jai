// Print_Vars at run time: print_vars_to's aligned "name = value (type)" lines for locals and
// expressions, a custom Print_Vars_Config, and print_vars / log_vars, called more than once with
// different arguments.
#import "Basic";
#import "Print_Vars";

main :: () {
    alpha := 3;
    longer_name := "hi";
    builder: String_Builder;
    print_vars_to(*builder, alpha, longer_name, alpha + 4);
    text := builder_to_string(*builder);
    expected := "alpha       = 3  (s64)\nlonger_name = hi (string)\nalpha + 4   = 7  (s64)\n";
    assert(text == expected, "print_vars_to:\n%\nexpected:\n%", text, expected);
    free(text);

    // Another call with as many arguments prints its own names and values.
    print_vars_to(*builder, longer_name);
    text = builder_to_string(*builder);
    assert(text == "longer_name = hi (string)\n", "second call: '%'", text);
    free(text);
    print_vars_to(*builder, alpha);
    text = builder_to_string(*builder);
    assert(text == "alpha = 3 (s64)\n", "third call: '%'", text);
    free(text);

    config: Print_Vars_Config;
    config.control_flags = .PRINT_NAME | .PRINT_VALUE;
    config.after_value = "";
    config.value_indentation = .RIGHT_JUSTIFIED;
    config.line_separator = "; ";
    print_helper(*builder, config, 4, 3, "x", "7", int);
    print_helper(*builder, config, 4, 3, "name", "100", int);
    text = builder_to_string(*builder);
    assert(text == "x    =   7; name = 100; ", "custom config: '%'", text);
    free(text);

    print_vars(alpha, longer_name);
    log_vars(alpha * 2);
    print("ok\n");
}
