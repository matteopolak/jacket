// Reviewed builder operation + independent failure propagation. Allocation
// pressure from diagnostic text must preserve existing bytes and failed state.
#import "Basic";
RPM :: #import "rpmalloc";

deny :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    return null;
}

main :: () {
    assert(RPM.initialize() == 0);
    memory := RPM.rpmalloc(301);

    assert(memory != null);
    builder: String_Builder;

    append(*builder, "prefix");
    room := ensure_contiguous_space_and_return_available_range(*builder, 1);

    assert(room.count > 0);
    for 0..room.count - 2 room[it] = 65;

    advance_through_ensured_space(*builder, room.count - 1);
    builder.allocator = .{ deny, null };
    RPM.dump_statistics(*builder);
    assert(builder.failed);
    // Public buffer access verifies existing text survived the denied growth.
    bytes := get_buffer_data(get_base_buffer(*builder));
    for 0..5 assert(bytes[it] == "prefix"[it]);

    RPM.rpfree(memory);
    RPM.finalize();
}
