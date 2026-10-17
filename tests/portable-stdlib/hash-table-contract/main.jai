// Reviewed public contract: specs/stdlib/modules/hash-table.md.
#import "Basic";
H :: #import "Hash_Table";
Tracked :: #import "Hash_Table"(COUNT_COLLISIONS = true);

collision :: (key: s64) -> u32 {
    return 0;
}

equal :: (a: s64, b: s64) -> bool {
    return a == b;
}

Allocation_Counts :: struct {
    base: Allocator;
    allocated: s64;
    freed: s64;
}

count_allocations :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    counts := cast(*Allocation_Counts) allocator_data;
    result := counts.base.proc(mode, requested_size, old_size, old_memory, counts.base.data);

    if mode == .ALLOCATE && result counts.allocated += 1;
    if mode == .FREE && old_memory counts.freed += 1;

    return result;
}

Defaulted :: struct {
    count := 7;
    link: *Defaulted;
    inner: H.Table(s64, s64);
}

new_slot_holds_default_value :: () {
    table: H.Table(s64, Defaulted);
    for key: 0..99 {
        slot, added := H.find_or_add(*table, key);

        assert(added && slot.count == 7 && slot.link == null && slot.inner.count == 0);
        slot.count = 8;
    }
}

allocation_ownership :: () {
    saved := context.allocator;

    defer context.allocator = saved;
    original := Allocation_Counts.{ base = saved };
    later := Allocation_Counts.{ base = saved };

    context.allocator = .{ count_allocations, *original };
    table: H.Table(s64, s64);

    H.table_add(*table, 1, 11);
    assert(table.allocator.data == *original && original.allocated == 1);
    context.allocator = .{ count_allocations, *later };
    for key: 2..100 H.table_add(*table, key, key);

    assert(table.allocator.data == *original && original.allocated > 1 && later.allocated == 0);
    H.deinit(*table);
    assert(original.freed == original.allocated && later.freed == 0);
}

string_keys :: () {
    key := copy_string("same key");
    table: H.Table(string, s64);

    H.table_add(*table, key, 71);
    pointer := H.table_find_pointer(*table, "same key");

    assert(pointer != null && pointer.* == 71);
    H.table_set(*table, "same key", 99);
    assert(table.count == 1 && pointer.* == 99);
    H.deinit(*table);
    free(key);
}

tombstone_flags :: () {
    reuse: H.Table(s64, s64, collision, equal, REFILL_REMOVED = true);
    retain: H.Table(s64, s64, collision, equal, REFILL_REMOVED = false);

    defer H.deinit(*reuse);
    defer H.deinit(*retain);
    H.table_add(*reuse, 1, 10);
    H.table_add(*retain, 1, 10);
    H.table_remove(*reuse, 1);
    H.table_remove(*retain, 1);
    H.table_add(*reuse, 2, 20);
    H.table_add(*retain, 2, 20);
    assert(reuse.slots_filled == 1 && retain.slots_filled == 2);
    assert(reuse.count == 1 && retain.count == 1);
}

exercise :: ($T: Type) {
    table: T;

    defer H.deinit(*table);
    assert(table.count == 0 && table.allocated == 0 && table.entries.data == null);
    assert(!H.table_contains(*table, 4) && H.table_find_pointer(*table, 4) == null);
    found := H.table_find(*table, 4);

    assert(!found);
    missing_removed := H.table_remove(*table, 4);

    assert(!missing_removed && H.table_remove_multiple(*table, 4) == 0);
    assert(H.table_find_multiple(*table, 4).count == 0);
    first := H.table_add(*table, 4, 11);
    second := H.table_add(*table, 4, 22);

    H.table_add(*table, 4, 33);
    assert(first != second && table.count == 3);
    selected := H.table_find_pointer(*table, 4);

    assert(selected != null);
    other_sum := 66 - selected.*;
    replacement := H.table_set(*table, 4, 90);

    assert(replacement == selected && table.count == 3 && replacement.* == 90);
    values := H.table_find_multiple(*table, 4);
    sum: s64;

    for value: values sum += value;

    assert(values.count == 3 && sum == other_sum + 90);
    removed, old := H.table_remove(*table, 4);

    assert(removed && old == 90 && table.count == 2);
    assert(H.table_contains(*table, 4));
    assert(H.table_remove_multiple(*table, 4) == 2 && table.count == 0);
    assert(!H.table_contains(*table, 4));
    slot, added := H.find_or_add(*table, 5);

    assert(added && slot != null);
    slot.* = 100;
    again, added_again := H.find_or_add(*table, 5);

    assert(!added_again && again == slot && again.* == 100);
    for key: 0..199 H.table_set(*table, key, key * 17 + 3);

    assert(table.count == 200 && table.allocated >= table.count);

    for key: 0..199 {
        exists, value := H.table_find(*table, key);
        assert(exists && value == key * 17 + 3);
    }

    for key: 0..99 {
        success, value := H.table_remove(*table, key * 2);
        assert(success && value == key * 34 + 3);
    }

    for key: 0..99 assert(H.table_contains(*table, key * 2 + 1));

    H.table_ensure_space(*table, 200);
    for key: 200..399 H.table_add(*table, key, key * 17 + 3);

    assert(table.count == 300);
    H.table_resize(*table, 1);
    assert(table.allocated >= table.count && table.count == 300);
    seen: s64;
    forward_sum: s64;
    reverse_sum: s64;

    for value, key: *table {
        assert(value == key * 17 + 3);
        seen += 1;
        forward_sum += value;
    }

    assert(seen == table.count);

    for < value, key: *table {
        assert(value == key * 17 + 3);
        reverse_sum += value;
    }

    assert(reverse_sum == forward_sum);
    for *value, key: *table value.* = key * 23;
    for value, key: *table assert(value == key * 23);

    storage := table.entries.data;
    allocated := table.allocated;

    H.table_reset_keeping_memory(*table);
    assert(table.count == 0 && table.slots_filled == 0 && table.entries.data == storage && table.allocated == allocated);
    H.table_set(*table, 8, 99);
    assert(H.table_contains(*table, 8));
    H.table_reset(*table);
    assert(table.count == 0 && table.allocated == 0 && table.slots_filled == 0 && table.entries.data == null);
    H.expand(*table, 128);
    assert(table.allocated >= 128 && table.count == 0);
    H.table_add(*table, 99, 88);
    assert(H.table_find_pointer(*table, 99).* == 88);
}

main :: () {
    #assert H.NEVER_OCCUPIED_HASH == 0 && H.REMOVED_HASH == 1 && H.FIRST_VALID_HASH == 2;
    #assert !#exists(H.Table(s64, s64).add_collisions);
    #assert #exists(Tracked.Table(s64, s64).add_collisions);
    assert(type_info(H.Table(s64, s64)).name == "Table");

    for n: 1..4096 {
        power := H.next_power_of_two(n);
        assert(power >= n && (power & (power - 1)) == 0 && (power == 1 || power / 2 < n));
    }

    new_slot_holds_default_value();
    exercise(H.Table(s64, s64));
    exercise(H.Table(s64, s64, collision, equal, LOAD_FACTOR_PERCENT = 30, REFILL_REMOVED = true));
    exercise(H.Table(s64, s64, collision, equal, LOAD_FACTOR_PERCENT = 90, REFILL_REMOVED = false));
    allocation_ownership();
    string_keys();
    tombstone_flags();
    preallocated: H.Table(s64, s64);

    H.init(*preallocated, slots_to_allocate = 64);
    assert(preallocated.count == 0 && preallocated.allocated >= 64);
    H.deinit(*preallocated);
    tracked: Tracked.Table(s64, s64, collision, equal);

    defer Tracked.deinit(*tracked);
    Tracked.table_add(*tracked, 7, 1);
    Tracked.table_add(*tracked, 8, 2);
    assert(tracked.add_collisions > 0);
    Tracked.table_find_pointer(*tracked, 8);
    assert(tracked.find_collisions > 0);
    print("ok\n");
}
