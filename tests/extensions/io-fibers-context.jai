// Each fiber gets its own copy of the spawning context and its own temporary storage, and an
// allocator or context value set in one fiber does not leak into another.
#import "Basic";
Io :: #import "Extensions/Io";

#add_context marker: int;

Seen :: struct {
    marker: int;
    temporary: *Temporary_Storage;
    used_before: s64;
    used_after: s64;
}

seen: [4] Seen;
group: Io.Wait_Group;

observer :: (index: int) {
    seen[index].marker = context.marker;
    seen[index].temporary = context.temporary_storage;
    seen[index].used_before = context.temporary_storage.total_bytes_occupied;

    // Change this fiber's context, switch away, and come back: only this fiber sees the change.
    context.marker = 1000 + index;
    text := tprint("fiber % made this %", index, index * index);

    Io.yield();
    Io.yield();

    assert(context.marker == 1000 + index, "fiber % lost its context value, got %", index, context.marker);
    seen[index].used_after = context.temporary_storage.total_bytes_occupied;
    expected := sprint("fiber % made this %", index, index * index);

    assert(text == expected, "temporary text was overwritten: '%'", text);

    Io.done(*group);
}

// A pushed allocator is inherited by the fibers spawned inside it, and only by them.
counting_allocations: int;

counting_allocator :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .ALLOCATE counting_allocations += 1;

    return context.default_allocator.proc(mode, requested_size, old_size, old_memory, allocator_data);
}

allocating_fiber :: () {
    before := counting_allocations;
    item := New(int);

    free(item);

    assert(counting_allocations == before + 1, "this fiber uses the allocator it was spawned with");
    Io.done(*group);
}

plain_fiber :: () {
    before := counting_allocations;
    item := New(int);

    free(item);

    assert(counting_allocations == before, "this fiber must not use the counting allocator");
    Io.done(*group);
}

// Temporary storage is reset when the fiber ends, so a fiber that allocates a lot does not grow
// the next one that reuses its record.
bulk :: () {
    for 1..200 {
        text := tprint("%, %, %", it, it * 2, it * 3);
        assert(text.count > 0);
    }

    Io.done(*group);
}

root :: () {
    context.marker = 7;
    Io.add(*group, seen.count);
    for 0..seen.count - 1 Io.spawn(observer, it);

    // Changing the context after spawning does not reach the fibers that already exist.
    context.marker = 99;
    Io.wait(*group);

    for seen {
        assert(it.marker == 7, "fiber % started with marker %", it_index, it.marker);
        assert(it.temporary != context.temporary_storage, "fiber % shares the root's temporary storage", it_index);
        assert(it.used_before == 0, "fiber % started with %b of temporary storage in use", it_index, it.used_before);
        assert(it.used_after > 0);

        for other: it_index + 1..seen.count - 1 {
            assert(it.temporary != seen[other].temporary, "fibers % and % share temporary storage", it_index, other);
        }
    }

    assert(context.marker == 99);

    Io.add(*group, 1);
    counting: Allocator;
    counting.proc = counting_allocator;

    {
        push_allocator(counting);
        Io.spawn(allocating_fiber);
    }

    Io.wait(*group);

    Io.add(*group, 1);
    Io.spawn(plain_fiber);
    Io.wait(*group);

    // The root's own temporary storage is untouched by the fibers' use of theirs.
    mark := context.temporary_storage.total_bytes_occupied;

    Io.add(*group, 20);
    for 1..20 Io.spawn(bulk);

    Io.wait(*group);
    Io.yield();
    assert(context.temporary_storage.total_bytes_occupied == mark, "the root's temporary storage changed");
}

main :: () {
    Io.io_run(root);
    print("ok\n");
}
