// rpmalloc as an Allocator (it links the pinned C library, built for macOS and Linux by
// tools/build_native_libs.py): size classes, 64-byte alignment from one cache line up, resizes that
// keep contents, ownership per heap, separate heaps, and use from several threads.
#import "Basic";
#import "Thread";
#import "rpmalloc";

main :: () {
    a := Allocator.{ rpmalloc_allocator_proc, null };
    a.data = a.proc(.STARTUP, 0, 0, null, null);
    assert(a.data != null, "startup gives the main heap");
    caps, name := get_capabilities(a);
    assert((caps & .CREATE_HEAP) != 0 && (caps & .IS_THIS_YOURS) != 0 && (caps & .ACTUALLY_RESIZE) != 0, "caps %", caps);
    assert(name.count > 0);

    for size: s64.[1, 15, 16, 63, 64, 65, 500, 4096, 70000, 3_000_000] {
        p := cast(*u8) alloc(size, a);
        assert(p != null, "alloc %", size);
        if size >= 64 assert(cast(u64) p % 64 == 0, "a % byte block at %", size, p);
        for i: 0..size - 1 p[i] = cast,trunc(u8) (i * 13);
        assert(a.proc(.IS_THIS_YOURS, 0, 0, p, a.data) != null);

        // Grow across size classes, then shrink: contents survive each step.
        kept := size;
        for new_size: s64.[size + 1, size * 4 + 100, 24] {
            q := cast(*u8) realloc(p, new_size, kept, a);
            assert(q != null, "resize % -> %", kept, new_size);
            check := min(kept, new_size);
            for i: 0..check - 1 assert(q[i] == cast,trunc(u8) (i * 13), "byte % lost resizing % -> %", i, kept, new_size);
            if new_size > kept for i: kept..new_size - 1 q[i] = cast,trunc(u8) (i * 13);
            p = q;
            kept = new_size;
        }
        free(p, a);
    }

    // Many live blocks do not overlap.
    blocks: [200] *u8;
    for *blocks it.* = cast(*u8) alloc(48 + it_index, a);
    for *blocks memset(it.*, cast(u8) (it_index + 1), 48 + it_index);
    for block, number: blocks for i : 0..47 + number assert(block[i] == cast,trunc(u8) (number + 1), "block % was overwritten", number);
    for blocks free(it, a);

    // A separate heap owns its own blocks only, and destroying it releases them.
    heap := create_heap(a);
    other := create_heap(a);
    inside := alloc(100, heap);
    outside := alloc(100, other);
    assert(heap.proc(.IS_THIS_YOURS, 0, 0, inside, heap.data) != null);
    assert(heap.proc(.IS_THIS_YOURS, 0, 0, outside, heap.data) == null, "ownership is per heap");
    destroy_heap(heap);
    destroy_heap(other);

    // Threads each take a heap of their own and may free what another thread allocated.
    shared: [4] *u8;
    for *shared it.* = cast(*u8) alloc(256, a);
    workers: [4] Thread;
    for *workers {
        it.data = *shared[it_index];
        thread_init(it, (t: *Thread) -> s64 {
            mine := Allocator.{ rpmalloc_allocator_proc, null };
            mine.data = mine.proc(.THREAD_START, 0, 0, null, null);
            slot := cast(**u8) t.data;
            for 0..255 slot.*[it] = cast,trunc(u8) it;
            fresh := cast(*u8) alloc(512, mine);
            memset(fresh, 7, 512);
            free(fresh, mine);
            mine.proc(.THREAD_STOP, 0, 0, null, mine.data);
            return 0;
        });
        thread_start(it);
    }
    for *workers {
        thread_is_done(it, -1);
        thread_deinit(it);
    }
    for shared {
        for i: 0..255 assert(it[i] == cast,trunc(u8) i, "thread wrote its slot");
        free(it, a);
    }

    // Shutdown with nothing outstanding is clean.
    a.proc(.SHUTDOWN, 0, 0, null, a.data);
    print("ok\n");
}
