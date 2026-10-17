Builder_Types :: #import "internal/basic-builder-types"(ENABLE_ASSERT = ENABLE_ASSERT);

String_Builder :: Builder_Types.String_Builder;
STRING_BUILDER_BUFFER_SIZE :: Builder_Types.STRING_BUILDER_BUFFER_SIZE;

get_base_buffer :: inline (using builder: *String_Builder) -> *String_Builder.Buffer {
    return cast(*String_Builder.Buffer) initial_bytes.data;
}

get_current_buffer :: inline (using builder: *String_Builder) -> *String_Builder.Buffer {
    if !initialized init_string_builder(builder);
    return current_buffer;
}

get_buffer_data :: inline (using buffer: *String_Builder.Buffer) -> *u8 {
    return cast(*u8) buffer + size_of(String_Builder.Buffer);
}

init_string_builder :: (using builder: *String_Builder, buffer_size := -1) {
    if initialized return;
    if !allocator.proc allocator = context.allocator;

    subsequent_buffer_size = buffer_size;
    if subsequent_buffer_size < 1 subsequent_buffer_size = STRING_BUILDER_BUFFER_SIZE;

    base := get_base_buffer(builder);
    base.* = .{};
    base.allocated = initial_bytes.count - size_of(String_Builder.Buffer);
    current_buffer = base;
    initialized = true;
}

ensure_contiguous_space :: (using builder: *String_Builder, bytes: s64) -> pointer: *void {
    assert(bytes >= 0);
    if failed return null;

    buffer := get_current_buffer(builder);
    if bytes > buffer.allocated - buffer.count {
        if !grow_builder(builder, bytes) return null;
        buffer = current_buffer;
    }

    #if ENABLE_ASSERT buffer.ensured_count = bytes;

    return get_buffer_data(buffer) + buffer.count;
}

ensure_contiguous_space_and_return_available_range :: (using builder: *String_Builder, minimum_bytes := 1) -> [] u8 {
    memory := ensure_contiguous_space(builder, minimum_bytes);
    if !memory return .{};

    available := current_buffer.allocated - current_buffer.count;

    #if ENABLE_ASSERT current_buffer.ensured_count = available;

    return .{ available, cast(*u8) memory };
}

expand :: (using builder: *String_Builder) -> bool {
    if !initialized init_string_builder(builder);
    return grow_builder(builder, 1);
}

append_and_steal_buffers :: (builder: *String_Builder, steal_from: *String_Builder) {
    assert(builder != steal_from);
    if !steal_from.initialized return;

    if builder.failed || steal_from.failed {
        builder.failed = true;
        return;
    }

    if !builder.initialized init_string_builder(builder);

    base := get_base_buffer(steal_from);

    if builder.allocator.proc != steal_from.allocator.proc || builder.allocator.data != steal_from.allocator.data {
        buffer := base;
        while buffer {
            append(builder, get_buffer_data(buffer), buffer.count);
            if builder.failed return;
            buffer = buffer.next;
        }

        reset(steal_from);

        return;
    }

    // Inline storage belongs to the source object; heap buffers can change owners.
    append(builder, get_buffer_data(base), base.count);
    if builder.failed return;

    if base.next {
        builder.current_buffer.next = base.next;
        builder.current_buffer = steal_from.current_buffer;
        base.next = null;
    }

    reset(steal_from);
}

advance_through_ensured_space :: (using builder: *String_Builder, bytes: s64) {
    assert(initialized && bytes >= 0 && bytes <= current_buffer.allocated - current_buffer.count);

    #if ENABLE_ASSERT {
        assert(bytes <= current_buffer.ensured_count);
        current_buffer.ensured_count -= bytes;
    }

    current_buffer.count += bytes;
}

append :: (using builder: *String_Builder, s: *u8, length: s64) {
    memory := ensure_contiguous_space(builder, length);
    if !memory return;
    memcpy(memory, s, length);
    advance_through_ensured_space(builder, length);
}

append :: inline (builder: *String_Builder, s: string) {
    append(builder, s.data, s.count);
}

append :: inline (builder: *String_Builder, byte: u8) {
    append(builder, *byte, 1);
}

append_by_pointer :: inline (builder: *String_Builder, pointer: *$T) {
    append(builder, cast(*u8) pointer, size_of(T));
}

simple_memcpy :: inline (using builder: *String_Builder, x: $T) {
    append_by_pointer(builder, *x);
}

builder_string_length :: (using builder: *String_Builder) -> int {
    if !initialized return 0;

    buffer := get_base_buffer(builder);
    total: s64;

    while buffer {
        total += buffer.count;
        buffer = buffer.next;
    }

    return total;
}

free_buffers :: (using builder: *String_Builder) {
    if !initialized return;

    base := get_base_buffer(builder);
    buffer := base.next;

    while buffer {
        next := buffer.next;
        allocator_call(allocator, .FREE, 0, buffer.allocated + size_of(String_Builder.Buffer), buffer);
        buffer = next;
    }

    base.* = .{};
    current_buffer = null;
    initialized = false;
    failed = false;
} @Releases

reset :: (using builder: *String_Builder) {
    free_buffers(builder);
    init_string_builder(builder, subsequent_buffer_size);
}

builder_to_string :: (builder: *String_Builder, extra_bytes_to_prepend := 0, do_reset := true) -> string {
    assert(extra_bytes_to_prepend >= 0);
    if builder.failed return "";

    result := alloc_string(builder_string_length(builder) + extra_bytes_to_prepend);
    cursor := extra_bytes_to_prepend;

    if builder.initialized {
        buffer := get_base_buffer(builder);
        while buffer {
            memcpy(result.data + cursor, get_buffer_data(buffer), buffer.count);
            cursor += buffer.count;
            buffer = buffer.next;
        }
    }

    if do_reset reset(builder);

    return result;
}

write_builder :: (using builder: *String_Builder, do_reset := true, to_standard_error := false) -> s64 {
    if failed return 0;

    total := builder_string_length(builder);

    if initialized {
        buffer := get_base_buffer(builder);
        while buffer {
            write_string(string.{ buffer.count, get_buffer_data(buffer) }, to_standard_error);
            buffer = buffer.next;
        }
    }

    if do_reset reset(builder);

    return total;
}

#scope_file
grow_builder :: (using builder: *String_Builder, minimum_capacity: s64) -> bool {
    if failed return false;

    capacity := max(minimum_capacity, subsequent_buffer_size);
    if capacity > 0x7fffffffffffffff - size_of(String_Builder.Buffer) {
        failed = true;
        return false;
    }

    buffer := cast(*String_Builder.Buffer) allocator_call(allocator, .ALLOCATE, capacity + size_of(String_Builder.Buffer), 0, null);
    if !buffer {
        failed = true;
        return false;
    }

    buffer.* = .{};
    buffer.allocated = capacity;
    current_buffer.next = buffer;
    current_buffer = buffer;

    return true;
}
