Builder_Types :: #import "internal/basic-builder-types"(ENABLE_ASSERT = ENABLE_ASSERT);

// Builds a string from many pieces without copying it each time. Declare a `String_Builder`, `append` or
// `print_to_builder` to it, then take the text with `builder_to_string` or print it with `write_builder`.
//
// A builder starts with 4 KiB of room inside it and takes more from the context allocator in effect
// when it is first used. It must not be copied while in use. Call `free_buffers` when done with it,
// unless `builder_to_string` or `write_builder` has just emptied it. If memory runs out the builder is
// marked failed and ignores further appends.
//
// ```jai
// builder: String_Builder;
// defer free_buffers(*builder);
// append(*builder, "a");
// append(*builder, #char "b");
// print_to_builder(*builder, "%", 3);
// text := builder_to_string(*builder);
// defer free(text);
// assert(text == "ab3");
// ```
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

// Prepares `builder` for use. Called by the other procedures when needed, so you rarely call it. `buffer_size`
// sets the size of the buffers allocated after the first (the default is 4 KiB); the builder takes the
// current context allocator. Does nothing when already prepared.
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
} @Opens

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

// Appends `length` bytes at `s` to `builder`. A negative `length` is an error and sets the builder to
// failed.
append :: (using builder: *String_Builder, s: *u8, length: s64) {
    // The common case: the text fits in the buffer being filled.
    if initialized && !failed && length >= 0 && length <= current_buffer.allocated - current_buffer.count {
        memcpy(get_buffer_data(current_buffer) + current_buffer.count, s, length);
        current_buffer.count += length;
        #if ENABLE_ASSERT current_buffer.ensured_count = 0;
        return;
    }

    memory := ensure_contiguous_space(builder, length);
    if !memory return;

    memcpy(memory, s, length);
    advance_through_ensured_space(builder, length);
} @Opens

// Appends the string `s` to `builder`.
append :: inline (builder: *String_Builder, s: string) {
    append(builder, s.data, s.count);
} @AppendsString @Opens

// Appends one byte to `builder`.
append :: inline (using builder: *String_Builder, byte: u8) {
    if initialized && !failed && current_buffer.allocated > current_buffer.count {
        get_buffer_data(current_buffer)[current_buffer.count] = byte;
        current_buffer.count += 1;
        #if ENABLE_ASSERT current_buffer.ensured_count = 0;
    } else {
        append(builder, *byte, 1);
    }
} @Opens

// Appends the bytes of the value `pointer` points to, as they are in memory.
append_by_pointer :: inline (builder: *String_Builder, pointer: *$T) {
    append(builder, cast(*u8) pointer, size_of(T));
}

simple_memcpy :: inline (using builder: *String_Builder, x: $T) {
    append_by_pointer(builder, *x);
}

// The number of bytes appended to `builder` since it was last reset.
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

// Frees the memory `builder` took from its allocator and returns it to its empty state, also clearing
// the failed state. The builder can be used again afterwards.
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
} @FreesArgument @Closes

// Empties `builder`, freeing its extra memory, so it can be used again.
reset :: (using builder: *String_Builder) {
    #if ENABLE_ASSERT {
        // Space ensured in the inline buffer survives a reset, so code that ensures space, takes
        // the string and then advances over what it wrote is not rejected.
        ensured: s64;
        if initialized ensured = get_base_buffer(builder).ensured_count;
    }

    free_buffers(builder);
    init_string_builder(builder, subsequent_buffer_size);

    #if ENABLE_ASSERT get_base_buffer(builder).ensured_count = ensured;
} @Closes

// The text in `builder`, in a new string; the string is allocated with the context allocator, not the
// builder's, and the caller frees it. With `do_reset` (the default) the builder is emptied afterwards,
// otherwise it keeps its text. `extra_bytes_to_prepend` leaves that many unwritten bytes in front of the text.
// A builder that has failed gives an empty string.
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
} @Allocates @Closes

// Writes the text in `builder` to standard output, or standard error with `to_standard_error`, and returns the
// number of bytes. With `do_reset` (the default) the builder is emptied afterwards.
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
} @WritesOutput @Closes

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
