#import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
R :: #import "Runtime_Support";
D :: #import "Default_Allocator";

main :: () {
    base := context.allocator;

    assert(base.proc == R.Context_Base.default_allocator.proc);
    assert(base.proc == D.allocator_proc);
    assert(R.runtime_support_default_allocator_proc == D.allocator_proc);
    caps, name := get_capabilities(base);

    #if CPU == .X64 && OS != .IOS {
        assert((caps & .CREATE_HEAP) != 0 && (caps & .IS_THIS_YOURS) != 0 && name.count > 0);
        allocations_before_this_call_will_not_be_considered_leaks();
        first := create_heap(base);
        second := create_heap(first); // CREATE_HEAP ignores incoming data; independent heap.

        assert(first.proc == base.proc && second.proc == base.proc);
        assert(first.data != null && second.data != null && first.data != second.data);
        pointer: *u8;

        push_context {
            context.allocator = first;
            pointer = cast(*u8) alloc(64);
            assert(pointer != null && cast(u64) pointer % align_of(u64) == 0);
            assert(first.proc(.IS_THIS_YOURS, 0, 0, pointer, first.data) != null);
            assert(second.proc(.IS_THIS_YOURS, 0, 0, pointer, second.data) == null);
            assert(base.proc(.IS_THIS_YOURS, 0, 0, pointer, base.data) == null);
            for i: 0..63 pointer[i] = cast(u8) i;

            pointer = cast(*u8) realloc(pointer, 96, 64);
            assert(pointer != null && cast(u64) pointer % align_of(u64) == 0);
            for i: 0..63 assert(pointer[i] == cast(u8) i);

            failed := first.proc(.RESIZE, -1, 96, pointer, first.data);

            assert(failed == null);
            for i: 0..63 assert(pointer[i] == cast(u8) i);

            pointer = cast(*u8) realloc(pointer, 17, 96);
            assert(pointer != null && cast(u64) pointer % align_of(u64) == 0);
            for i: 0..16 assert(pointer[i] == cast(u8) i);

            free(pointer);
            retained := cast(*u8) alloc(128);

            assert(retained != null && cast(u64) retained % align_of(u64) == 0);
            retained[0] = 91;
        }

        push_context {
            context.allocator = second;
            bytes := cast(*u8) alloc(512);

            assert(bytes != null && cast(u64) bytes % align_of(u64) == 0);
            bytes[511] = 71;
            // Destruction must release this live independent-heap allocation.
        }

        destroy_heap(second);
        destroy_heap(first);
        report := make_leak_report(.{ scan_global_memory_to_remove_leaks = false, scan_leaks_to_combine = false });

        assert(report.sorted_summaries.count == 0);
        deinit(*report);
    } else {
        #if OS == .MACOS {
            assert((caps & .CREATE_HEAP) == 0);
            // Calling CREATE_HEAP without its capability is an allocator error.
        }
    }

    print("ok\n");
}
