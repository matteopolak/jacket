// Reviewed rewrite dump appends diagnostic counters; disabled form has no side
// effect. Formatting itself is not a stable serialization contract.
#import "Basic";
RPM :: #import "rpmalloc";
Small :: #import "rpmalloc"(ENABLE_STATISTICS = false);

main :: () {
    assert(RPM.initialize() == 0);
    memory := RPM.rpmalloc(1024);

    assert(memory != null);
    builder: String_Builder;

    append(*builder, "retained prefix\n");
    RPM.dump_statistics(*builder);
    assert(!builder.failed);
    text := builder_to_string(*builder, do_reset = false);

    assert(text.count > "retained prefix\n".count);
    for 0.."retained prefix\n".count - 1 assert(text[it] == "retained prefix\n"[it]);

    count := builder_string_length(*builder);

    Small.dump_statistics(*builder);
    Small.dump_statistics(null);
    assert(builder_string_length(*builder) == count);
    RPM.dump_statistics(*builder);
    assert(builder_string_length(*builder) > count && !builder.failed);
    free_buffers(*builder);
    // Formatting grows this builder on the same allocator being captured.
    // The counter capture must remain valid after native leases are released.
    heap := RPM.rpmalloc_allocator_proc(.CREATE_HEAP, 0, 0, null, null);

    assert(heap != null);
    recursive: String_Builder;

    init_string_builder(*recursive);
    recursive.allocator = .{ RPM.rpmalloc_allocator_proc, heap };
    range := ensure_contiguous_space_and_return_available_range(*recursive, 1);
    for 0..range.count - 2 range[it] = 65;

    advance_through_ensured_space(*recursive, range.count - 1);
    before := builder_string_length(*recursive);

    RPM.dump_statistics(*recursive);
    assert(!recursive.failed && builder_string_length(*recursive) > before);
    free_buffers(*recursive);
    RPM.rpmalloc_allocator_proc(.DESTROY_HEAP, 0, 0, null, heap);
    RPM.rpfree(memory);
    RPM.finalize();
}
