// Real capture OOM and recursive formatting after releasing its native scope.
B :: #import "Basic";
RPM :: #import "rpmalloc";

probe :: #library,system "rpmalloc_metadata_fault";
metadata_fail_begin :: (ordinal: u32) #foreign probe;
metadata_fail_end :: () -> u32 #foreign probe;
metadata_fail_abort :: (line: u64) #foreign probe;

check :: (condition: bool, location: Source_Code_Location = #caller_location) {
    if !condition metadata_fail_abort(cast(u64) location.line_number);
}

main :: () {
    check(RPM.initialize() == 0);
    memory := RPM.rpmalloc(4096);

    check(memory != null);
    (cast(*u8) memory)[0] = 71;
    builder: B.String_Builder;

    B.append(*builder, "prefix");
    before_length := B.builder_string_length(*builder);

    check(before_length == 6);
    metadata_fail_begin(1);
    RPM.dump_statistics(*builder);
    denied := metadata_fail_end();

    check(denied == 1 && builder.failed);
    check(B.builder_string_length(*builder) == before_length);
    bytes := B.get_buffer_data(B.get_base_buffer(*builder));
    for 0..5 check(bytes[it] == "prefix"[it]);

    B.free_buffers(*builder);

    heap := RPM.heap_acquire();

    check(heap != null);
    recursive: B.String_Builder;

    B.init_string_builder(*recursive);
    recursive.allocator = .{ RPM.rpmalloc_allocator_proc, heap };
    room := B.ensure_contiguous_space_and_return_available_range(*recursive, 1);

    check(room.count > 1);
    for 0..room.count - 2 room[it] = 65;

    B.advance_through_ensured_space(*recursive, room.count - 1);
    before := B.builder_string_length(*recursive);

    RPM.dump_statistics(*recursive);
    check(!recursive.failed && B.builder_string_length(*recursive) > before);
    check((cast(*u8) memory)[0] == 71);
    B.free_buffers(*recursive);
    RPM.heap_release(heap);
    RPM.rpfree(memory);
    RPM.finalize();
}
