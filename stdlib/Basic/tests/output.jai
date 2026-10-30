#import "Basic";

main :: () {
    assert(print("out %", 7) == 5);
    assert(print("err %", true, to_standard_error = true) == 8);
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "++");
    assert(write_builder(*builder) == 2);
    assert(builder_string_length(*builder) == 0);
    mark := get_temporary_storage_mark();
    text := tprint("temporary %", 9);

    assert(text == "temporary 9");
    copied := copy_string(text);

    set_temporary_storage_mark(mark);
    assert(copied == "temporary 9");
    free(copied);
    normal := sprint("owned %", 5);

    reset_temporary_storage();
    assert(normal == "owned 5");
    free(normal);
    exit(17);
}
