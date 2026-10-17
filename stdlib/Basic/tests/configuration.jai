B :: #import "Basic";
On :: #import "Basic";

ENABLE_ASSERT :: false; // A local constant does not configure Basic.

main :: () {
    B.assert(!ENABLE_ASSERT);
    B.assert(type_info(B.String_Builder) == type_info(On.String_Builder));
    B.assert(size_of(B.String_Builder.Buffer) == 32);
    B.assert(B.STRING_BUILDER_BUFFER_SIZE == 4096 - size_of(B.String_Builder.Buffer));
    builder: B.String_Builder;

    defer B.free_buffers(*builder);
    buffer := B.get_current_buffer(*builder);

    B.assert(buffer.ensured_count == 0);
    memory := B.ensure_contiguous_space(*builder, 7);

    B.assert(memory != null && buffer.ensured_count == 7);
    B.advance_through_ensured_space(*builder, 3);
    B.assert(buffer.ensured_count == 4 && buffer.count == 3);
    check_on_capacity();
    B.print("ok\n");
}

Guard_On :: struct {
    builder: B.String_Builder;
    canary: [16] u8;
}

check_on_capacity :: () {
    guarded: Guard_On;

    defer B.free_buffers(*guarded.builder);
    for *byte: guarded.canary byte.* = 206;

    range := B.ensure_contiguous_space_and_return_available_range(*guarded.builder);
    capacity := range.count;

    for *byte: range byte.* = 42;

    B.advance_through_ensured_space(*guarded.builder, capacity);
    B.assert(B.builder_string_length(*guarded.builder) == capacity);
    B.assert(B.get_current_buffer(*guarded.builder) == B.get_base_buffer(*guarded.builder));
    for byte: guarded.canary B.assert(byte == 206);

    B.append(*guarded.builder, "x");
    B.assert(B.builder_string_length(*guarded.builder) == capacity + 1);
    B.assert(B.get_current_buffer(*guarded.builder) != B.get_base_buffer(*guarded.builder));
    for byte: guarded.canary B.assert(byte == 206);
}
