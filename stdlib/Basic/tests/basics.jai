#import "Basic";

main :: () {
    assert(type_info(String_Builder).name == "String_Builder");
    b: String_Builder;

    init_string_builder(*b, 8);
    append(*b, "small ");
    append(*b, "buffer growth");
    assert(builder_string_length(*b) == 19);
    text := builder_to_string(*b);

    assert(text == "small buffer growth");
    free(text);
    assert(builder_string_length(*b) == 0);
    free_buffers(*b);
    a: [..] s64;

    array_reserve(*a, 16);
    original := a.data;

    array_resize(*a, 3);
    assert(a[0] == 0 && a[2] == 0 && a.data == original);
    a[0] = 7;
    a[1] = 8;
    a[2] = 9;
    view := array_view(a, 1, 99);
    view[0] = 12;
    assert(a[1] == 12 && view.count == 2);
    assert(array_view(a, 99).count == 0);
    array_insert_at(*a, 4, 0);
    array_insert_at(*a, 5, a.count);
    assert(a.count == 5 && a[0] == 4 && a[4] == 5);
    array_reset(*a);
    assert(a.count == 0 && a.data == null && a.allocated == 0);
    print("% % % %\n", -42, true, "text", cast(u32) 55);
}
