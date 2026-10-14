B :: #import "Basic";
Off :: #import "Basic"(ENABLE_ASSERT = false);
On :: #import "Basic"(ENABLE_ASSERT = true);
Both :: #import "Basic"(MEMORY_DEBUGGER = true, ENABLE_ASSERT = false, TEMP_ALLOCATOR_OVERWRITE_MEMORY = true, VISUALIZE_MEMORY_DEBUGGER = false);

ENABLE_ASSERT :: false; // Root constants do not configure imported module instances.

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
    B.assert(!ENABLE_ASSERT);
    B.assert(type_info(B.String_Builder) == type_info(On.String_Builder));
    B.assert(size_of(B.String_Builder.Buffer) == 32);
    B.assert(size_of(Off.String_Builder.Buffer) == 24);
    B.assert(B.STRING_BUILDER_BUFFER_SIZE == 4096 - size_of(B.String_Builder.Buffer));
    B.assert(Off.STRING_BUILDER_BUFFER_SIZE == 4096 - size_of(Off.String_Builder.Buffer));
    builder: B.String_Builder;

    defer B.free_buffers(*builder);
    buffer := B.get_current_buffer(*builder);

    B.assert(buffer.ensured_count == 0);
    memory := B.ensure_contiguous_space(*builder, 7);

    B.assert(memory != null && buffer.ensured_count == 7);
    B.advance_through_ensured_space(*builder, 3);
    B.assert(buffer.ensured_count == 4 && buffer.count == 3);
    disabled: Off.String_Builder;

    defer Off.free_buffers(*disabled);
    Off.append(*disabled, "works");
    B.assert(Off.builder_string_length(*disabled) == 5);
    Off.assert(effect(), message(), effect(), loc = location());
    B.assert(calls == 0);
    check_on_capacity();
    check_off_capacity();
    check_combined_configuration();
    B.print("ok\n");
}

check_combined_configuration :: () {
    B.assert(size_of(Both.String_Builder.Buffer) == size_of(Off.String_Builder.Buffer));
    builder: Both.String_Builder;
    bytes: [5000] u8;

    Both.append(*builder, bytes.data, bytes.count);
    B.assert(Both.builder_string_length(*builder) == bytes.count);
    Both.free_buffers(*builder);
    initial: [128] u8;
    storage: Both.Temporary_Storage_Paged_Alloc;
    storage.data = initial.data;
    storage.size = initial.count;

    push_context {
        context.temporary_storage = *storage;
        mark := Both.get_temporary_storage_mark();
        memory := cast(*u8) Both.talloc(512);
        memory[0] = 42;
        B.assert(storage.overflow_pages != null);
        Both.set_temporary_storage_mark(mark);
        B.assert(storage.overflow_pages == null && storage.total_bytes_occupied == 0);
        retained := cast(*u8) Both.talloc(16);
        retained[0] = 42;
        Both.reset_temporary_storage();
        B.assert(retained[0] == Both.RESET_STORAGE_STAMP);
    }
}

Guard_On :: struct {
    builder: B.String_Builder;
    canary: [16] u8;
}

Guard_Off :: struct {
    builder: Off.String_Builder;
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

check_off_capacity :: () {
    guarded: Guard_Off;

    defer Off.free_buffers(*guarded.builder);
    for *byte: guarded.canary byte.* = 206;

    range := Off.ensure_contiguous_space_and_return_available_range(*guarded.builder);
    capacity := range.count;

    for *byte: range byte.* = 42;

    Off.advance_through_ensured_space(*guarded.builder, capacity);
    B.assert(Off.builder_string_length(*guarded.builder) == capacity);
    B.assert(Off.get_current_buffer(*guarded.builder) == Off.get_base_buffer(*guarded.builder));
    for byte: guarded.canary B.assert(byte == 206);

    Off.append(*guarded.builder, "x");
    B.assert(Off.builder_string_length(*guarded.builder) == capacity + 1);
    B.assert(Off.get_current_buffer(*guarded.builder) != Off.get_base_buffer(*guarded.builder));
    for byte: guarded.canary B.assert(byte == 206);
}
