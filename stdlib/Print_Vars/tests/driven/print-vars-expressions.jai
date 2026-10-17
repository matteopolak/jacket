#import "Basic";
#import "String";
P :: #import "Print_Vars";

Record :: struct {
    value: s32;
}

evaluations: s32;

increment :: () -> s32 {
    evaluations += 1;
    return 9;
}

main :: () {
    record := Record.{ 12 };
    values := s32.[3, 5, 8];
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "prefix\n");
    P.print_vars_to(*builder, record.value, values[1], cast(s64) record.value, (3 + 4), increment());
    // Reviewed expansion permits repeated/eager evaluation of side effects.
    assert(!builder.failed && evaluations >= 1);
    text := builder_to_string(*builder);

    defer free(text);
    assert(begins_with(text, "prefix\n"));
    assert(contains(text, "record.value") && contains(text, "values[1]"));
    assert(contains(text, " = 12") && contains(text, " = 5") && contains(text, " = 7"));
    assert(contains(text, "s64") && contains(text, "increment()"));
    // Every control-flag combination independently selects its own decorations.
    config: P.Print_Vars_Config;
    config.before_name = "<";
    config.after_name = ">";
    config.before_value = "[";
    config.after_value = "]";
    config.before_type = "{";
    config.after_type = "}";
    config.line_separator = "|";
    config.name_indentation = .NONE;
    config.value_indentation = .NONE;

    for bits: 0..7 {
        config.control_flags = cast(type_of(config.control_flags)) bits;
        P.print_helper(*builder, config, 20, 20, "name", "value", s32);
        rendered := builder_to_string(*builder);

        defer free(rendered);
        expected: String_Builder;

        defer free_buffers(*expected);
        if bits & 1 append(*expected, "<name>");
        if bits & 2 append(*expected, "[value]");
        if bits & 4 append(*expected, "{s32}");

        append(*expected, "|");
        expected_text := builder_to_string(*expected);

        defer free(expected_text);
        assert(rendered == expected_text);
    }

    print("ok\n");
}
