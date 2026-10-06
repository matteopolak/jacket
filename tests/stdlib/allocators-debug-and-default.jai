// Default_Allocator at run time (heaps, ownership, resizes across the cache-line boundary, the
// startup/shutdown modes) and Overwriting_Allocator (freed memory is overwritten, resizes copy,
// a double free is caught), the latter over a base allocator that never reuses memory so the
// overwritten bytes can be inspected.
#import "Basic";
Default :: #import "Default_Allocator";
#import "Overwriting_Allocator";

main :: () {
    check_default_allocator();
    check_overwriting_allocator();
    check_default_allocator_shutdown();
    print("ok\n");
}

check_default_allocator :: () {
    a := Default.allocator;
    caps, name := get_capabilities(a);
    assert((caps & .CREATE_HEAP) != 0 && (caps & .IS_THIS_YOURS) != 0 && (caps & .ACTUALLY_RESIZE) != 0, "caps %", caps);
    assert(name.count > 0);

    // Startup is idempotent; the default heap exists afterwards.
    a.proc(.STARTUP, 0, 0, null, null);
    assert(!Default.initialize_if_needed(), "the default heap already exists");
    a.proc(.THREAD_START, 0, 0, null, null);
    a.proc(.THREAD_STOP, 0, 0, null, null);

    // Sizes on both sides of a cache line; blocks of 64 bytes or more are 64-aligned.
    for size: s64.[0, 1, 15, 16, 63, 64, 65, 200, 4096, 100000] {
        p := cast(*u8) alloc(size, a);
        assert(p != null || size == 0, "alloc %", size);
        if !p continue;
        if size >= 64 assert(cast(u64) p % 64 == 0, "a % byte block at %", size, p);
        else assert(cast(u64) p % 16 == 0, "a % byte block at %", size, p);
        for i: 0..size - 1 p[i] = cast(u8) (i * 7);
        assert(a.proc(.IS_THIS_YOURS, 0, 0, p, null) != null);
        // Grow across the boundary and shrink back: contents survive each step.
        sizes := s64.[size + 1, size * 3 + 70, 40, 8];
        kept := size;
        for new_size: sizes {
            q := cast(*u8) realloc(p, new_size, kept, a);
            assert(q != null, "resize % -> %", kept, new_size);
            if new_size >= 64 assert(cast(u64) q % 64 == 0, "a resized % byte block at %", new_size, q);
            check := min(kept, new_size);
            for i: 0..check - 1 assert(q[i] == cast(u8) (i * 7), "byte % lost resizing % -> %", i, kept, new_size);
            if new_size > kept for i: kept..new_size - 1 q[i] = cast(u8) (i * 7);
            p = q;
            kept = new_size;
        }
        free(p, a);
        assert(a.proc(.IS_THIS_YOURS, 0, 0, p, null) == null, "a freed block is no longer owned");
    }
    assert(alloc(-1, a) == null);
    assert(a.proc(.ALLOCATE, 0x7fff_ffff_ffff_fff0, 0, null, null) == null, "an impossible size");

    // Separate heaps: ownership is per heap, destroying a heap frees its blocks.
    heap := create_heap(a);
    other := create_heap(a);
    blocks: [100] *void;
    for *blocks it.* = alloc(16 + it_index * 13, heap);
    foreign := alloc(32, other);
    for blocks {
        assert(heap.proc(.IS_THIS_YOURS, 0, 0, it, heap.data) != null);
        assert(other.proc(.IS_THIS_YOURS, 0, 0, it, other.data) == null);
        assert(a.proc(.IS_THIS_YOURS, 0, 0, it, null) == null, "the default heap does not own another heap's block");
    }
    for blocks if it_index % 2 free(it, heap);
    destroy_heap(heap);
    assert(other.proc(.IS_THIS_YOURS, 0, 0, foreign, other.data) != null, "destroying one heap leaves another alone");
    free(foreign, other);
    destroy_heap(other);

    // unsafe_reset only clears the ledger lock (for a child after fork); the allocator keeps working.
    Default.unsafe_reset();
    p := alloc(10, a);
    free(p, a);
}

// Last, since it releases every block of every heap: SHUTDOWN (finalize) drops the ledger, and the
// next allocation starts a fresh default heap.
check_default_allocator_shutdown :: () {
    a := Default.allocator;
    live := alloc(24, a);
    assert(a.proc(.IS_THIS_YOURS, 0, 0, live, null) != null);
    a.proc(.SHUTDOWN, 0, 0, null, null);
    assert(Default.initialize_if_needed(), "a new default heap after finalize");
    assert(a.proc(.IS_THIS_YOURS, 0, 0, live, null) == null, "finalize released the block");
    again := alloc(24, a);
    assert(again != null && a.proc(.IS_THIS_YOURS, 0, 0, again, null) != null);
    free(again, a);
}

// Hands out memory and never reuses it, so freed blocks can be read afterwards.
Arena :: struct {
    memory: [65536] u8;
    used: s64;
    frees: s64;
}
arena_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    arena := cast(*Arena) data;
    if mode == .ALLOCATE || mode == .RESIZE {
        if arena.used + size + 16 > arena.memory.count return null;
        result := arena.memory.data + arena.used;
        arena.used += (size + 15) & ~15;
        if mode == .RESIZE && old_memory memcpy(result, old_memory, min(size, old_size));
        return result;
    }
    if mode == .FREE arena.frees += 1;
    return null;
}

Assertion :: struct {
    count: s64;
    last: string;
}
note_assertion :: (location: Source_Code_Location, message: string) -> bool {
    assertions.count += 1;
    assertions.last = copy_string(message);
    return false;
}
assertions: Assertion;

check_overwriting_allocator :: () {
    arena := New(Arena);
    defer free(arena);
    base := Allocator.{ arena_proc, arena };
    debug := get_overwriting_allocator(base, 0xab);
    caps := get_capabilities(debug);
    assert((caps & .HINT_I_AM_A_DEBUG_ALLOCATOR) != 0);

    p := cast(*u8) alloc(40, debug);
    for 0..39 p[it] = cast(u8) it;
    // A resize copies, then overwrites and frees the old block.
    q := cast(*u8) realloc(p, 100, 40, debug);
    assert(q != p, "a resize moves");
    for 0..39 assert(q[it] == it, "byte % copied", it);
    for 0..39 assert(p[it] == 0xab, "the old block is overwritten: byte % is %", it, p[it]);
    assert(arena.frees == 1, "the old block went back to the base allocator");
    // A free overwrites the whole block.
    q[99] = 1;
    free(q, debug);
    for 0..99 assert(q[it] == 0xab, "freed byte % is %", it, q[it]);
    assert(arena.frees == 2);
    // Shrinking keeps the front.
    r := cast(*u8) alloc(16, debug);
    for 0..15 r[it] = 9;
    s := cast(*u8) realloc(r, 4, 16, debug);
    for 0..3 assert(s[it] == 9);
    for 0..15 assert(r[it] == 0xab);
    free(s, debug);
    free(null, debug);

    // A double free and a pointer it never handed out are reported, not passed on.
    {
        scoped := context;
        scoped.assertion_failed = note_assertion;
        push_context, defer_pop scoped;
        frees := arena.frees;
        free(s, debug);
        assert(assertions.count == 1, "the double free was reported");
        local: s64;
        free(*local, debug);
        assert(assertions.count == 2, "a foreign pointer was reported: %", assertions.last);
        assert(arena.frees == frees, "neither reached the base allocator");
    }

    // The default overwrite byte, and the context allocator as the default base.
    plain := get_overwriting_allocator();
    t := cast(*u8) alloc(8, plain);
    t[0] = 1;
    u := cast(*u8) realloc(t, 8000, 8, plain);
    assert(u[0] == 1);
    free(u, plain);
    deinit_overwriting_allocator(plain);
    deinit_overwriting_allocator(debug);
}
