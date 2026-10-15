// Reviewed public heap/config callbacks; independent cross-thread/free safety.
#import "Basic";
RPM :: #import "rpmalloc";

#add_context heap_callback_marker: s64;
probe :: #library,system "rpmalloc_callback_probe";
map_native :: (size: u64) -> *void #foreign probe "binding_rpmalloc_test_map";
release_native :: (memory: *void) #foreign probe "binding_rpmalloc_test_unmap";
parallel :: (worker: #type () #c_call) -> s32 #foreign probe "binding_rpmalloc_parallel";
claim :: () -> u32 #foreign probe "binding_rpmalloc_handoff_claim";
reset_claims :: () #foreign probe "binding_rpmalloc_handoff_reset";
count :: (index: u32, amount: u64) -> u64 #foreign probe "binding_rpmalloc_count";
shared: type_of(RPM.heap_acquire());
handoff: [4] *void;
returned: [4] *void;
trigger: bool;
decline: bool;

free_worker :: () #c_call {
    push_context, defer_pop;
    context.heap_callback_marker = 74;
    RPM.thread_initialize();
    index := claim();

    assert(index < 4 && handoff[index] != null);
    assert((cast(*u8) handoff[index])[0] == cast(u8) (81 + index));
    RPM.heap_free(shared, handoff[index]);
    handoff[index] = null;
    RPM.thread_finalize(true);
}

allocation_worker :: () #c_call {
    push_context, defer_pop;
    context.heap_callback_marker = 74;
    RPM.thread_initialize();
    index := claim();

    assert(index < 4);
    returned[index] = RPM.heap_alloc(shared, 1024);
    assert(returned[index] != null);
    (cast(*u8) returned[index])[0] = cast(u8) (91 + index);
    RPM.thread_finalize(true);
}

map_memory :: (size: s64, offset: *s64) -> *void {
    assert(context.heap_callback_marker == 47 || context.heap_callback_marker == 74);
    count(0, 1);
    if decline return null;

    if trigger {
        trigger = false;
        assert(parallel(free_worker) == 0);
    }

    memory := map_native(cast(u64) size);
    if memory {
        offset.* = 7;
        count(1, 1);
    }

    return memory;
}

unmap_memory :: (address: *void, size: s64, offset: s64, full: s64) {
    assert(size > 0);

    if full {
        assert(offset == 7);
        count(2, 1);
        release_native(address);
    } else assert(offset == 0);
}

main :: () {
    context.heap_callback_marker = 47;
    config: RPM.rpmalloc_config_t;
    config.memory_map = map_memory;
    config.memory_unmap = unmap_memory;
    decline = true;
    assert(RPM.initialize_config(*config) != 0);
    assert(!RPM.is_initialized());
    decline = false;
    assert(RPM.initialize_config(*config) == 0);
    shared = RPM.heap_acquire();
    assert(shared != null);

    for 0..3 {
        handoff[it] = RPM.heap_alloc(shared, 512);
        assert(handoff[it] != null);
        (cast(*u8) handoff[it])[0] = cast(u8) (81 + it);
    }

    reset_claims();
    trigger = true;
    huge := RPM.heap_alloc(shared, 64 * 1024 * 1024);

    assert(huge != null && !trigger);
    for 0..3 assert(handoff[it] == null);

    frees: s64;
    for 0..126 frees += shared.size_class_use[it].free_total;

    assert(frees == 4);
    RPM.heap_free(shared, huge);
    reset_claims();
    assert(parallel(allocation_worker) == 0);

    for 0..3 {
        assert(returned[it] != null && (cast(*u8) returned[it])[0] == cast(u8) (91 + it));
        RPM.heap_free(shared, returned[it]);
    }

    kept := RPM.heap_alloc(shared, 1024);

    assert(kept != null);
    (cast(*u8) kept)[0] = 97;
    decline = true;
    attempts := count(0, 0);

    assert(RPM.heap_realloc(shared, kept, 64 * 1024 * 1024, .{}) == null);
    assert(count(0, 0) > attempts && (cast(*u8) kept)[0] == 97);
    decline = false;
    RPM.heap_free(shared, kept);
    RPM.heap_free_all(shared);
    // Official native heap release preserves allocations; matching global free
    // remains valid after the explicit heap lease ends.
    kept = RPM.heap_alloc(shared, 1024);
    assert(kept != null);
    (cast(*u8) kept)[0] = 67;
    RPM.heap_release(shared);
    assert((cast(*u8) kept)[0] == 67);
    RPM.rpfree(kept);
    RPM.finalize();
    assert(count(1, 0) == count(2, 0));
}
