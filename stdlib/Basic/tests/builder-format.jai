#import "Basic";

main :: () {
    builder_printer: (builder: *String_Builder, format_string: string, args: ..Any) -> bool = print;

    assert(builder_printer == print_to_builder);
    builder: String_Builder;

    defer free_buffers(*builder);
    buffer: *String_Builder.Buffer = get_current_buffer(*builder);

    assert(buffer.count == 0);
    assert(print(*builder, "% %", -42, true));
    text := builder_to_string(*builder, do_reset = false);

    defer free(text);
    assert(text == "-42 true");
    assert(builder_string_length(*builder) == text.count);
    reset(*builder);
    missing := "% %";
    extra := "";

    errors: int;

    {
        replacement := context;
        replacement.logger = count_error;
        replacement.logger_data = *errors;
        push_context replacement {
            assert(print(*builder, missing, 1));
            assert(print(*builder, extra, 1));
        }
    }

    assert(errors == 1);
    partial := builder_to_string(*builder);

    defer free(partial);
    assert(partial == "1 ");
    assert(tprint("%:%", "temporary", 7) == "temporary:7");
    assert(tprint("%2 %1 %", "one", "two") == "two one two");
    assert(tprint("\% %0", 5) == "% 5");
    reset(*builder);
    bytes: [5000] u8;
    for *byte: bytes byte.* = cast(u8) 120;

    append(*builder, bytes.data, bytes.count);
    assert(builder_string_length(*builder) == bytes.count);
    grown := builder_to_string(*builder);

    defer free(grown);
    assert(grown.count == 5000 && grown[0] == 120 && grown[4999] == 120);
    range := ensure_contiguous_space_and_return_available_range(*builder, 3);

    assert(range.count >= 3);
    range[0] = 97;
    range[1] = 98;
    range[2] = 99;
    advance_through_ensured_space(*builder, 3);
    ensured := builder_to_string(*builder);

    defer free(ensured);
    assert(ensured == "abc");
    append(*builder, "prefix");
    assert(expand(*builder));
    assert(builder_string_length(*builder) == 6);
    source: String_Builder;

    defer free_buffers(*source);
    append(*source, "inline");
    append(*source, bytes.data, bytes.count);
    append_and_steal_buffers(*builder, *source);
    assert(builder_string_length(*source) == 0);
    transferred := builder_to_string(*builder);

    defer free(transferred);
    assert(transferred.count == 5012);
    assert(transferred[0] == 112 && transferred[6] == 105 && transferred[12] == 120 && transferred[5011] == 120);
    first := Allocations.{ underlying = context.allocator };
    second := Allocations.{ underlying = context.allocator };
    separate_source: String_Builder;
    separate_dest: String_Builder;

    separate_source.allocator = .{ track_allocation, *first };
    separate_dest.allocator = .{ track_allocation, *second };
    append(*separate_source, bytes.data, bytes.count);
    assert(first.allocations == 1);
    append_and_steal_buffers(*separate_dest, *separate_source);
    assert(builder_string_length(*separate_source) == 0);
    assert(builder_string_length(*separate_dest) == 5000);
    assert(first.frees == 1 && second.allocations == 1 && second.frees == 0);
    free_buffers(*separate_source);
    free_buffers(*separate_dest);
    assert(first.frees == first.allocations && second.frees == second.allocations);
    failure: String_Builder;
    failure.allocator = .{ deny_allocation, null };
    append(*failure, bytes.data, bytes.count);
    assert(failure.failed);
    assert(!print(*failure, ""));
    assert(builder_to_string(*failure) == "");
    free_buffers(*failure);
    print("ok\n");
}

Allocations :: struct {
    underlying: Allocator;
    allocations: int;
    frees: int;
}

track_allocation :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Allocations) allocator_data;
    if mode == .ALLOCATE state.allocations += 1;
    if mode == .FREE state.frees += 1;
    return state.underlying.proc(mode, requested_size, old_size, old_memory, state.underlying.data);
}

deny_allocation :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    return null;
}

count_error :: (message: string, data: *void, info: Log_Info) {
    assert((info.common_flags & .ERROR) != 0);
    (cast(*int) data).* += 1;
}
