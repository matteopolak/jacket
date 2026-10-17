#import "Basic";

Empty :: struct {}
ignore_any :: (value: Any) {}

main :: () {
    empty: Any;
    builder: String_Builder;

    defer free_buffers(*builder);
    print_item_to_builder(*builder, empty);
    empty_text := builder_to_string(*builder);

    defer free(empty_text);
    assert(empty_text == "Any.{}");
    empty_argument := sprint("%", empty);

    defer free(empty_argument);
    assert(empty_argument == "Any.{}");

    k: s64 = 7;
    borrowed := Any.{ type = type_info(s64), value_pointer = *k };

    k = 11;
    alias_text := sprint("%", borrowed);

    defer free(alias_text);
    assert(alias_text == "11");

    p: *s64;
    address := *(p.*);

    assert(address == null);
    unused: Any = p.*; // jailint: allow(unused_variable) the formatted value is what is tested

    ignore_any(p.*);
    zero: *Empty;

    ignore_any(zero.*);
    print("continued\n");
}
