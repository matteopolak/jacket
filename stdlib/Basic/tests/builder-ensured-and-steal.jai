// Behaviour of this Basic that goes beyond the portable builder contract: the ensured count shrinks as
// space is used, stealing buffers between builders with different allocators copies them, and a builder
// whose allocation failed turns into an empty string.
#import "Basic";

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

main :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    init_string_builder(*builder);
    buffer := get_current_buffer(*builder);
    memory := ensure_contiguous_space(*builder, 7);

    assert(memory != null && buffer.ensured_count == 7);
    advance_through_ensured_space(*builder, 3);
    assert(buffer.ensured_count == 4 && buffer.count == 3);
    bytes: [5000] u8;
    for *byte: bytes byte.* = cast(u8) 120;

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
    assert(builder_to_string(*failure) == "");
    free_buffers(*failure);
    print("ok\n");
}
