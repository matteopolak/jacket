// Independent public allocator ownership/resize/lifecycle behavior shared by cache profiles.
cache_probe :: #library,system "rpmalloc_callback_probe";
cache_parallel :: (worker: #type () #c_call) -> s32 #foreign cache_probe "binding_rpmalloc_parallel";
cache_claim :: () -> u32 #foreign cache_probe "binding_rpmalloc_handoff_claim";
cache_reset_claims :: () #foreign cache_probe "binding_rpmalloc_handoff_reset";
cache_selected: Allocator_Proc;
cache_shared_heap: *void;
cache_handoff: [4] *void;

cache_free_worker :: () #c_call {
    push_context, defer_pop;
    assert(cache_selected(.THREAD_START, 0, 0, null, null) == null);
    index := cache_claim();

    assert(index < 4 && cache_handoff[index] != null);
    assert((cast(*u8) cache_handoff[index])[0] == cast(u8) (71 + index));
    assert(cache_selected(.FREE, 0, 0, cache_handoff[index], cache_shared_heap) == null);
    cache_handoff[index] = null;
    assert(cache_selected(.THREAD_STOP, 0, 0, null, null) == null);
}

cache_exercise :: (proc: Allocator_Proc) {
    assert(proc(.STARTUP, 0, 0, null, null) == null);
    heap := proc(.CREATE_HEAP, 0, 0, null, null);

    assert(heap != null);
    slots: [256] *void;
    // Overflow each native cache batch rather than relying on small frees.
    for *slots {
        it.* = proc(.ALLOCATE, 131072, 0, null, heap);
        assert(it.* != null);
        (cast(*u8) it.*)[0] = 39;
        assert(cast(u64) proc(.IS_THIS_YOURS, 0, 0, it.*, heap) == 1);
    }

    for slots {
        assert((cast(*u8) it)[0] == 39);
        assert(proc(.FREE, 0, 0, it, heap) == null);
    }

    cache_selected = proc;
    cache_shared_heap = heap;

    for 0..3 {
        cache_handoff[it] = proc(.ALLOCATE, 512, 0, null, heap);
        assert(cache_handoff[it] != null);
        (cast(*u8) cache_handoff[it])[0] = cast(u8) (71 + it);
    }

    cache_reset_claims();
    assert(cache_parallel(cache_free_worker) == 0);
    for cache_handoff assert(it == null);

    assert(proc(.DESTROY_HEAP, 0, 0, null, heap) == null);
    assert(proc(.THREAD_STOP, 0, 0, null, null) == null);
    assert(proc(.THREAD_START, 0, 0, null, null) == null);
    memory := proc(.ALLOCATE, 71, 0, null, null);

    assert(memory != null);
    (cast(*u8) memory)[0] = 81;
    replacement := proc(.RESIZE, 8192, 0, memory, null);

    assert(replacement != null && (cast(*u8) replacement)[0] == 81);
    assert(proc(.FREE, 0, 0, replacement, null) == null);
}
