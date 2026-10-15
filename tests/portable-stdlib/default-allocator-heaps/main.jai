// Reviewed optimized allocator capabilities/lifecycle, allocation and ownership.
using #import "Basic";
D :: #import "Default_Allocator";
Quiet :: #import "Default_Allocator"(ENABLE_ASSERTS = false, ENABLE_VALIDATE_ARGS = false);
T :: #import "Thread";

main :: () {
    assert(CPU == .X64);
    D.finalize();
    assert(!D.is_initialized());
    assert(D.initialize_if_needed());
    assert(!D.initialize_if_needed());
    assert(D.is_initialized() && D.allocator_default_heap != null);
    name: string;
    caps := cast(Allocator_Caps) cast(u64) D.allocator_proc(.CAPS, 0, 0, *name, null);

    assert((caps & (.MULTIPLE_THREADS | .FREE | .ACTUALLY_RESIZE | .CREATE_HEAP | .IS_THIS_YOURS)) == (.MULTIPLE_THREADS | .FREE | .ACTUALLY_RESIZE | .CREATE_HEAP | .IS_THIS_YOURS));
    assert(name.count > 0);
    one := cast(*D.Heap) D.allocator_proc(.CREATE_HEAP, 0, 0, null, null);
    two := cast(*D.Heap) D.allocator_proc(.CREATE_HEAP, 0, 0, null, null);

    assert(one != null && two != null && one != two);
    args := get_command_line_arguments();
    if args.count > 1 {
        allocation := D.allocator_proc(.ALLOCATE, 4, 0, null, one);

        assert(allocation != null);
        if args[1] == "wrong-heap" D.allocator_proc(.FREE, 0, 4, allocation, two);
        if args[1] == "wrong-size" D.allocator_proc(.RESIZE, 8, 5, allocation, one);
        if args[1] == "unknown-pointer" D.usable_size(*args);

        return;
    }

    assert(type_info(D.Heap).name == "Heap");
    assert(type_info(D.Span).name == "Span");
    assert(type_info(D.Heap_Size_Class).name == "Heap_Size_Class");

    for size: 0..129 {
        allocation := D.allocator_proc(.ALLOCATE, size, 0, null, one);

        assert(allocation != null && (cast(u64) allocation & 15) == 0);
        assert(D.usable_size(allocation) >= size);
        assert(D.is_this_yours(one, allocation) && !D.is_this_yours(two, allocation));
        assert(D.allocator_proc(.IS_THIS_YOURS, 0, 0, allocation, one) != null);
        assert(D.allocator_proc(.IS_THIS_YOURS, 0, 0, allocation, two) == null);
        bytes := cast(*u8) allocation;
        for i: 0..size - 1 bytes[i] = cast(u8) (i % 251);

        grown := D.allocator_proc(.RESIZE, size + 257, size, allocation, one);

        assert(grown != null && D.usable_size(grown) >= size + 257);
        assert(D.is_this_yours(one, grown) && !D.is_this_yours(two, grown));
        bytes = cast(*u8) grown;
        for i: 0..size - 1 assert(bytes[i] == cast(u8) (i % 251));

        shrunk := D.allocator_proc(.RESIZE, size, size + 257, grown, one);

        assert(shrunk != null);
        bytes = cast(*u8) shrunk;
        for i: 0..size - 1 assert(bytes[i] == cast(u8) (i % 251));

        D.allocator_proc(.FREE, 0, size, shrunk, one);
        assert(!D.is_this_yours(one, shrunk));
    }

    live := D.allocator_proc(.ALLOCATE, 8193, 0, null, one);
    other := D.allocator_proc(.ALLOCATE, 257, 0, null, two);

    assert(live != null && other != null);
    (cast(*u8) live)[0] = 99;
    assert(D.allocator_proc(.RESIZE, 0x7fffffffffffffff, 8193, live, one) == null);
    assert(D.is_this_yours(one, live) && (cast(*u8) live)[0] == 99);
    D.heap_free_all(one);
    assert(!D.is_this_yours(one, live) && D.is_this_yours(two, other));
    threads: [4] T.Thread;
    for *thread: threads {
        thread.data = one;
        prepared := T.thread_init(thread, concurrent_allocations);
        assert(prepared);
        T.thread_start(thread);
    }

    for *thread: threads {
        done := T.thread_is_done(thread, 10000);
        assert(done);
        T.thread_deinit(thread);
    }

    fresh := D.allocator_proc(.ALLOCATE, 128, 0, null, one);

    assert(fresh != null && D.is_this_yours(one, fresh));
    stats := D.heap_statistics(one);

    assert(stats.sizecache >= 0 && stats.spancache >= 0);
    global := D.global_statistics();

    assert(global.mapped == 0 && global.mapped_peak == 0 && global.cached == 0);
    assert(global.huge_alloc == 0 && global.huge_alloc_peak == 0);
    assert(global.mapped_total == 0 && global.unmapped_total == 0);
    D.allocator_proc(.DESTROY_HEAP, 0, 0, null, one);
    assert(D.is_this_yours(two, other));
    D.allocator_proc(.DESTROY_HEAP, 0, 0, null, two);
    D.unsafe_reset();
    D.finalize();
    assert(!D.is_initialized() && D.allocator_default_heap == null);
    assert(D.initialize_if_needed());
    default_memory := D.allocator.proc(.ALLOCATE, 31, 0, null, D.allocator.data);

    assert(default_memory != null && D.is_this_yours(D.allocator_default_heap, default_memory));
    D.allocator.proc(.FREE, 0, 31, default_memory, null);
    D.finalize();
    quiet_heap := Quiet.allocator_proc(.CREATE_HEAP, 0, 0, null, null);
    quiet_memory := Quiet.allocator_proc(.ALLOCATE, 23, 0, null, quiet_heap);

    assert(quiet_memory != null && Quiet.is_this_yours(cast(*Quiet.Heap) quiet_heap, quiet_memory));
    quiet_grown := Quiet.allocator_proc(.RESIZE, 71, 23, quiet_memory, quiet_heap);

    assert(quiet_grown != null && Quiet.usable_size(quiet_grown) >= 71);
    Quiet.allocator_proc(.FREE, 0, 71, quiet_grown, quiet_heap);
    assert(!Quiet.is_this_yours(cast(*Quiet.Heap) quiet_heap, quiet_grown));
    Quiet.allocator_proc(.DESTROY_HEAP, 0, 0, null, quiet_heap);
    Quiet.finalize();
    print("ok\n");
}

concurrent_allocations :: (thread: *T.Thread) -> s64 {
    heap := cast(*D.Heap) thread.data;

    for iteration: 0..511 {
        pointer := D.allocator_proc(.ALLOCATE, 37, 0, null, heap);

        assert(pointer != null && D.is_this_yours(heap, pointer));
        bytes := cast(*u8) pointer;
        for i: 0..36 bytes[i] = cast(u8) (iteration % 251);

        grown := D.allocator_proc(.RESIZE, 129, 37, pointer, heap);

        assert(grown != null && D.is_this_yours(heap, grown));
        bytes = cast(*u8) grown;
        for i: 0..36 assert(bytes[i] == cast(u8) (iteration % 251));

        D.allocator_proc(.FREE, 0, 129, grown, heap);
    }

    return 0;
}
