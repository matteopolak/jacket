B :: #import "Basic"()(ENABLE_ASSERT = false);

check :: (condition: bool, loc := #caller_location) {
    if !condition context.assertion_failed(loc, "configuration regression failed");
}

calls: int;

effect :: () -> bool {
    calls += 1;
    return false;
}

message :: () -> string {
    calls += 1;
    return "disabled";
}

location :: () -> Source_Code_Location {
    calls += 1;
    return .{};
}

main :: () {
    #assert !#exists(B.String_Builder.Buffer.ensured_count);
    check(size_of(B.String_Builder.Buffer) == 24);
    check(B.STRING_BUILDER_BUFFER_SIZE == 4096 - size_of(B.String_Builder.Buffer));
    disabled: B.String_Builder;

    defer B.free_buffers(*disabled);
    B.append(*disabled, "works");
    check(B.builder_string_length(*disabled) == 5);
    B.assert(effect(), message(), effect(), loc = location());
    check(calls == 0);
    check_off_capacity();
    B.print("ok\n");
}

Guard_Off :: struct {
    builder: B.String_Builder;
    canary: [16] u8;
}

check_off_capacity :: () {
    guarded: Guard_Off;

    defer B.free_buffers(*guarded.builder);
    for *byte: guarded.canary byte.* = 206;

    range := B.ensure_contiguous_space_and_return_available_range(*guarded.builder);
    capacity := range.count;

    for *byte: range byte.* = 42;

    B.advance_through_ensured_space(*guarded.builder, capacity);
    check(B.builder_string_length(*guarded.builder) == capacity);
    check(B.get_current_buffer(*guarded.builder) == B.get_base_buffer(*guarded.builder));
    for byte: guarded.canary check(byte == 206);

    B.append(*guarded.builder, "x");
    check(B.builder_string_length(*guarded.builder) == capacity + 1);
    check(B.get_current_buffer(*guarded.builder) != B.get_base_buffer(*guarded.builder));
    for byte: guarded.canary check(byte == 206);
}
