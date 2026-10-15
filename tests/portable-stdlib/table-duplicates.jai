#import "Basic";
H :: #import "Hash_Table";
Counted :: #import "Hash_Table"(COUNT_COLLISIONS = true);

collision_hash :: (key: s64) -> u32 {
    return 2;
}

Probe :: struct {
    base: Allocator;
    live: s64;
}

tracked :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    probe := cast(*Probe) allocator_data;
    result := probe.base.proc(mode, requested_size, old_size, old_memory, probe.base.data);

    if mode == .ALLOCATE && result probe.live += 1;
    if mode == .FREE && old_memory probe.live -= 1;

    return result;
}

main :: () {
    probe := Probe.{ base = context.allocator };
    table: H.Table(s64, s64, collision_hash);
    table.allocator = .{ tracked, *probe };
    assert(H.table_find_multiple(*table, 42).count == 0);
    assert(H.table_remove_multiple(*table, 42) == 0);

    for i: 0..127 {
        H.table_add(*table, 42, i);
        H.table_add(*table, i + 1000, -i);
    }

    mark := get_temporary_storage_mark();
    copies := H.table_find_multiple(*table, 42);

    assert(copies.count == 128);
    seen: [128] bool;

    for value: copies {
        assert(value >= 0 && value < 128 && !seen[value]);
        seen[value] = true;
    }

    first := H.table_find_pointer(*table, 42);

    assert(first != null);
    H.table_set(*table, 42, 999);
    assert(H.table_find_pointer(*table, 42) == first && first.* == 999);
    for value: copies assert(value >= 0 && value < 128);

    copies[0] = -777;
    assert(first.* == 999);
    H.table_ensure_space(*table, 2048);
    assert(H.table_find_multiple(*table, 42).count == 128);
    assert(copies[0] == -777);
    assert(H.table_remove_multiple(*table, 42) == 128);
    assert(!H.table_contains(*table, 42) && H.table_find_multiple(*table, 42).count == 0);

    for i: 0..127 {
        found, value := H.table_find(*table, i + 1000);
        assert(found && value == -i);
    }

    pointer, added := H.find_or_add(*table, 42);

    assert(added);
    pointer.* = 73;
    repeated, added_again := H.find_or_add(*table, 42);

    assert(!added_again && repeated == pointer && repeated.* == 73);
    set_temporary_storage_mark(mark);
    H.table_resize(*table, 1);
    assert(table.count == 129 && H.table_find_pointer(*table, 42).* == 73);
    H.deinit(*table);
    assert(probe.live == 0);

    no_refill: Counted.Table(s64, s64, collision_hash, REFILL_REMOVED = false);
    for i: 0..19 Counted.table_add(*no_refill, i, i);

    assert(Counted.table_remove_multiple(*no_refill, 0) == 1);
    slots := no_refill.slots_filled;

    Counted.table_add(*no_refill, 100, 100);
    assert(no_refill.slots_filled == slots + 1);
    assert(no_refill.add_collisions > 0);
    assert(Counted.table_find_pointer(*no_refill, 100).* == 100);
    assert(no_refill.find_collisions > 0);
    for i: 200..399 Counted.table_add(*no_refill, i, i);

    assert(no_refill.count == 220);
    Counted.deinit(*no_refill);

    // Default scalar hashing is supplied by the NEW foundation producer.
    defaults: H.Table(s64, s64);

    H.table_add(*defaults, -7, 81);
    assert(H.table_find_pointer(*defaults, -7).* == 81);
    H.deinit(*defaults);
    print("ok\n");
}
