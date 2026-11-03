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

// Prepares `builder` for use. Every other builder procedure does this for you, so the only reason to call it is to pick
// `buffer_size`, the size of the buffers added after the first (4 KiB by default). The builder takes the current context
// allocator. Calling it on a prepared builder does nothing.
//
// ```jai
// builder: String_Builder;
// init_string_builder(*builder, 64 * 1024);
// defer free_buffers(*builder);
// append(*builder, "ready");
// assert(builder_string_length(*builder) == 5);
// ```
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

// Returns a pointer to `bytes` of free space at the end of `builder`, or null if the builder failed. Write there, then call
// `advance_through_ensured_space`. See `ensure_contiguous_space_and_return_available_range` for the slice form.
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

// Makes sure `builder` has at least `minimum_bytes` of free space in one piece and returns all of it as a slice. Write into
// the slice, then call `advance_through_ensured_space` with the number of bytes you used. Use it to format straight into the
// builder instead of building a temporary string first. An empty slice means the builder failed.
//
// ```jai
// builder: String_Builder;
// defer free_buffers(*builder);
// space := ensure_contiguous_space_and_return_available_range(*builder, 8);
// space[0] = #char "h";
// space[1] = #char "i";
// advance_through_ensured_space(*builder, 2);
// text := builder_to_string(*builder);
// defer free(text);
// assert(text == "hi");
// ```
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

// Appends everything in `steal_from` to `builder` and leaves `steal_from` empty. When both use the same allocator the
// buffers are moved instead of copied. Handy when threads fill their own builders and one of them joins the results.
//
// ```jai
// a: String_Builder;
// b: String_Builder;
// defer free_buffers(*a);
// defer free_buffers(*b);
// append(*a, "one ");
// append(*b, "two");
// append_and_steal_buffers(*a, *b);
// assert(builder_string_length(*b) == 0);
// text := builder_to_string(*a);
// defer free(text);
// assert(text == "one two");
// ```
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

// Counts the first `bytes` of the space from `ensure_contiguous_space` as written. `bytes` must not be more than the space
// that was ensured.
advance_through_ensured_space :: (using builder: *String_Builder, bytes: s64) {
    assert(initialized && bytes >= 0 && bytes <= current_buffer.allocated - current_buffer.count);

    #if ENABLE_ASSERT {
        assert(bytes <= current_buffer.ensured_count);
        current_buffer.ensured_count -= bytes;
    }

    current_buffer.count += bytes;
}

// Appends `length` bytes at `s` to `builder`. A negative `length` marks the builder as failed.
//
// ```jai
// builder: String_Builder;
// defer free_buffers(*builder);
// data := u8.[1, 2, 3];
// append(*builder, data.data, 3);
// assert(builder_string_length(*builder) == 3);
// ```
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
//
// ```jai
// builder: String_Builder;
// defer free_buffers(*builder);
// append(*builder, "hello, ");
// append(*builder, "world");
// text := builder_to_string(*builder);
// defer free(text);
// assert(text == "hello, world");
// ```
append :: inline (builder: *String_Builder, s: string) {
    append(builder, s.data, s.count);
} @AppendsString @Opens

// Appends one byte to `builder`.
//
// ```jai
// builder: String_Builder;
// defer free_buffers(*builder);
// append(*builder, #char "A");
// append(*builder, #char "B");
// text := builder_to_string(*builder);
// defer free(text);
// assert(text == "AB");
// ```
append :: inline (using builder: *String_Builder, byte: u8) {
    if initialized && !failed && current_buffer.allocated > current_buffer.count {
        get_buffer_data(current_buffer)[current_buffer.count] = byte;
        current_buffer.count += 1;
        #if ENABLE_ASSERT current_buffer.ensured_count = 0;
    } else {
        append(builder, *byte, 1);
    }
} @Opens

// Appends the raw bytes of the value `pointer` points to, as they sit in memory.
//
// ```jai
// builder: String_Builder;
// defer free_buffers(*builder);
// value: u32 = 0x01020304;
// append_by_pointer(*builder, *value);
// assert(builder_string_length(*builder) == 4);
// ```
append_by_pointer :: inline (builder: *String_Builder, pointer: *$T) {
    append(builder, cast(*u8) pointer, size_of(T));
}

simple_memcpy :: inline (using builder: *String_Builder, x: $T) {
    append_by_pointer(builder, *x);
}

// The number of bytes in `builder`.
//
// ```jai
// builder: String_Builder;
// defer free_buffers(*builder);
// assert(builder_string_length(*builder) == 0);
// append(*builder, "abc");
// assert(builder_string_length(*builder) == 3);
// ```
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

// Frees the memory `builder` took from its allocator and puts it back in its empty state, clearing a failure too. The
// builder can be used again afterwards. `builder_to_string` and `write_builder` already do this, so you only need it for a
// builder you abandon.
//
// ```jai
// builder: String_Builder;
// append(*builder, "scratch");
// free_buffers(*builder);
// assert(builder_string_length(*builder) == 0);
// ```
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
}

// Empties `builder` and frees its extra buffers so it can be used again.
//
// ```jai
// builder: String_Builder;
// defer free_buffers(*builder);
// append(*builder, "old");
// reset(*builder);
// append(*builder, "new");
// text := builder_to_string(*builder);
// defer free(text);
// assert(text == "new");
// ```
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
}

// The text in `builder` as a new string, allocated with the context allocator (not the builder's). You free it. The builder
// is emptied afterwards unless `do_reset` is `false`. `extra_bytes_to_prepend` leaves that many unwritten bytes in front of
// the text. A failed builder gives an empty string.
//
// ```jai
// builder: String_Builder;
// append(*builder, "total: ");
// print_to_builder(*builder, "%", 42);
// text := builder_to_string(*builder);
// defer free(text);
// assert(text == "total: 42");
// ```
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
} @Allocates

// Writes the text in `builder` to standard output, or standard error with `to_standard_error`, and returns the byte count.
// The builder is emptied afterwards unless `do_reset` is `false`.
//
// ```jai
// builder: String_Builder;
// append(*builder, "to the console\n");
// written := write_builder(*builder);
// assert(written == 15);
// ```
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
} @WritesOutput

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
