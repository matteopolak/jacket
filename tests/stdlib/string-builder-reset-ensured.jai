// Space ensured in a builder's first buffer stays usable after builder_to_string resets
// the builder (hyperserve advances its receive buffer after taking the header out).
#import "Basic";

main :: () {
    builder: String_Builder;
    space := ensure_contiguous_space_and_return_available_range(*builder);
    assert(space.count >= 4);
    memcpy(space.data, "abcd".data, 4);
    advance_through_ensured_space(*builder, 4);
    assert(builder_to_string(*builder) == "abcd");

    // Reset, but the base buffer's ensured space is still there.
    advance_through_ensured_space(*builder, 2);
    assert(builder_string_length(*builder) == 2);
    reset(*builder);
    assert(builder_string_length(*builder) == 0);
    print("ok\n");
}
