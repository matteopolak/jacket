#import "Basic";
X :: #import "Xar";

Probe :: struct {
    base: Allocator;
    allocations: s64;
    frees: s64;
    bytes: s64;
}

forward :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    probe := cast(*Probe) data;

    if mode == .ALLOCATE {
        probe.allocations += 1;
        probe.bytes += size;
    }

    if mode == .FREE {
        probe.frees += 1;
        probe.bytes -= old_size;
    }

    return probe.base.proc(mode, size, old_size, memory, probe.base.data);
}

compare :: (a: s64, b: s64) -> s32 {
    if a < b return -1;
    if a > b return 1;
    return 0;
}

main :: () {
    probe: Probe;
    probe.base = context.allocator;
    array: X.Xar(s64, 2, 6);
    array.allocator = .{ forward, *probe };
    first := X.xar_add(*array, cast(s64) 99);

    for i: 1..99 assert(X.xar_add(*array, 99 - i) != null);

    assert(array.count == 100 && first == *array[0] && first.* == 99 && probe.allocations == 6);
    assert(array.chunks[0] != null && array.chunks[5] != null);
    iterator := X.xar_iterator(*array, 7);

    assert(iterator.index_in_chunk == 3 && iterator.chunk_cap == 4 && iterator.chunk_pointer == *array.chunks[1]);
    assert(X.xar_get(*array, iterator).* == 92);
    assert(X.xar_next(*array, *iterator).* == 91 && iterator.index_in_chunk == 0 && iterator.chunk_cap == 8);
    assert(X.xar_prev(*array, *iterator).* == 92 && iterator.index_in_chunk == 3);
    for *array it.* += 1;

    assert(array[0] == 100 && array[99] == 1);
    X.quick_sort(array, compare);
    for array assert(it == it_index + 1);

    X.quick_sort_simple(array, (a, b) => compare(b, a));
    assert(array[0] == 100 && array[99] == 1);
    X.xar_unordered_remove(*array, 4);
    assert(array.count == 99 && array[4] == 1);
    iterator = X.xar_iterator(*array, array.count - 1);
    assert(X.xar_next(*array, *iterator) == null);

    for step: 0..98 {
        assert(iterator.it_index == 98 - step && X.xar_get(*array, iterator) == *array[98 - step]);
        previous := X.xar_prev(*array, *iterator);
        if step == 98 assert(previous == null);
        else assert(previous != null);
    }

    iterator = X.xar_iterator(*array, 0);
    assert(X.xar_prev(*array, *iterator) == null);
    saved := array.allocator;

    X.xar_reset(*array);
    assert(array.count == 0 && probe.allocations == probe.frees && probe.bytes == 0 && array.allocator.proc == saved.proc);
    for chunk: array.chunks assert(chunk == null);

    captured: X.Xar(s32, 1, 4);

    {
        push_allocator(Allocator.{ forward, *probe });
        assert(X.xar_add(*captured, cast(s32) 0) != null);
    }

    for i: 1..7 assert(X.xar_add(*captured, cast(s32) i) != null);

    assert(captured.allocator.proc == forward);
    X.xar_reset(*captured);
    assert(probe.allocations == probe.frees && probe.bytes == 0);
    print("ok\n");
}
