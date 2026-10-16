#import "Basic";

main :: () {
    empty: Any;
    missing_value := Any.{ type_info(s64), null };
    builder: String_Builder;

    defer free_buffers(*builder);

    print_item_to_builder(*builder, empty);
    empty_text := builder_to_string(*builder);

    defer free(empty_text);
    assert(empty_text == "Any.{}");
    reset(*builder);
    print_item_to_builder(*builder, missing_value);
    missing_text := builder_to_string(*builder);

    defer free(missing_text);
    assert(missing_text == "(Any with null 'type' field)");
    empty_argument := sprint("%", empty);

    defer free(empty_argument);
    assert(empty_argument == "Any.{}");
    missing_argument := sprint("%", missing_value);

    defer free(missing_argument);
    assert(missing_argument == "(Any with null 'type' field)");
    // Continuation is intentional: the reviewed fallback is a printable value.
    continued := sprint("%|%", missing_value, 7);

    defer free(continued);
    assert(continued == "(Any with null 'type' field)|7");
    print("ok\n");
}
