// Insert, find, overwrite, remove, add-refusal, growth, string keys, custom hash and compare,
// iteration and removal while iterating.
//
//     jaic stdlib/Extensions/Swiss_Table/tests/basics.jai -interp
#import "Basic";
Swiss :: #import "Extensions/Swiss_Table";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    if failures < 20 print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

basic_operations :: () {
    table: Swiss.Table(s64, s64);

    defer Swiss.deinit(*table);

    check(table.count == 0, "starts empty");
    check(!Swiss.table_contains(*table, 1), "empty table has no key");
    found_empty, _ := Swiss.table_find(*table, 1);

    check(!found_empty, "find on empty");
    removed, _ := Swiss.table_remove(*table, 1);

    check(!removed, "remove on empty");

    pointer := Swiss.table_set(*table, 7, 70);

    check(pointer.* == 70, "set returns the value");
    pointer.* = 71;

    ok, value := Swiss.table_find(*table, 7);

    check(ok && value == 71, "write through the returned pointer");

    Swiss.table_set(*table, 7, 72);
    check(table.count == 1, "overwrite keeps the count");
    check(Swiss.table_find_pointer(*table, 7).* == 72, "overwrite replaces the value");

    added, existing := Swiss.table_add(*table, 7, 99);

    check(!added && existing.* == 72, "add refuses an existing key and leaves its value");
    check(table.count == 1, "refused add keeps the count");

    added_new, fresh := Swiss.table_add(*table, 8, 80);

    check(added_new && fresh.* == 80 && table.count == 2, "add of a new key");

    entry, newly := Swiss.find_or_add(*table, 9);

    check(newly && entry.* == 0 && table.count == 3, "find_or_add adds a zero");
    entry, newly = Swiss.find_or_add(*table, 9);
    check(!newly && table.count == 3, "find_or_add finds");

    removed_old, old := Swiss.table_remove(*table, 7);

    check(removed_old && old == 72 && table.count == 2, "remove returns the value");
    check(!Swiss.table_contains(*table, 7), "removed key is gone");
    check(Swiss.table_find_pointer(*table, 7) == null, "removed key has no pointer");
    removed, _ = Swiss.table_remove(*table, 7);
    check(!removed, "second remove fails");

    Swiss.table_set(*table, 7, 1);
    check(Swiss.table_find_pointer(*table, 7).* == 1, "re-add after remove");
}

growth :: () {
    COUNT :: 50000;
    table: Swiss.Table(s64, s64);

    defer Swiss.deinit(*table);

    for 0..COUNT - 1 {
        added, _ := Swiss.table_add(*table, it * 3, it);
        check(added, "unique keys add");
    }

    check(table.count == COUNT, "count after growth");
    check(table.capacity >= COUNT && table.capacity * 7 / 8 >= COUNT, "capacity covers the load limit");
    check(table.capacity / 2 * 7 / 8 < COUNT, "capacity is not larger than needed by more than one doubling");

    for 0..COUNT - 1 {
        ok, value := Swiss.table_find(*table, it * 3);
        if !ok || value != it {
            check(false, "every key survives growth");
            break;
        }
    }

    for 0..COUNT - 1 {
        if Swiss.table_contains(*table, it * 3 + 1) {
            check(false, "no false hits");
            break;
        }
    }

    reserved: Swiss.Table(s64, s64);

    defer Swiss.deinit(*reserved);
    Swiss.init(*reserved, 1000);
    capacity := reserved.capacity;

    for 0..999 Swiss.table_set(*reserved, it, it);

    check(reserved.capacity == capacity && capacity >= 1000, "reserve means no rebuild");

    Swiss.table_reset_keeping_memory(*reserved);
    check(reserved.count == 0 && reserved.capacity == capacity, "reset keeping memory");
    check(!Swiss.table_contains(*reserved, 5), "reset empties");
    for 0..999 Swiss.table_set(*reserved, it, it);

    check(reserved.capacity == capacity && reserved.count == 1000, "reuse after reset");

    Swiss.table_reset(*reserved);
    check(reserved.count == 0 && reserved.capacity == 0 && reserved.control == null, "reset frees");
    Swiss.table_set(*reserved, 1, 1);
    check(Swiss.table_contains(*reserved, 1), "usable after reset");
}

// Inserting and removing the same few keys forever must not grow the table.
churn :: () {
    table: Swiss.Table(s64, s64);

    defer Swiss.deinit(*table);

    for 0..99 Swiss.table_set(*table, 1000000 + it, it);

    capacity_before := table.capacity;

    for round: 0..49999 {
        for i: 0..9 Swiss.table_set(*table, round * 10 + i, i);

        for i: 0..9 {
            ok, value := Swiss.table_remove(*table, round * 10 + i);
            if !ok || value != i {
                check(false, "churn remove");
                return;
            }
        }
    }

    check(table.count == 100, "churn leaves the original entries");
    check(table.capacity <= capacity_before * 2, "churn does not grow without bound");

    for 0..99 {
        ok, value := Swiss.table_find(*table, 1000000 + it);
        if !ok || value != it {
            check(false, "originals survive churn");
            break;
        }
    }

    // A hash that sends everything to one group keeps deleted slots in the probe path.
    clumped: Swiss.Table(s64, s64, (key: s64) -> u32 {
        return 0;
    });

    defer Swiss.deinit(*clumped);

    for round: 0..299 {
        for i: 0..19 Swiss.table_set(*clumped, round * 20 + i, i);

        for i: 0..19 {
            ok, _ := Swiss.table_remove(*clumped, round * 20 + i);
            if !ok {
                check(false, "clumped remove");
                return;
            }
        }
    }

    check(clumped.count == 0 && clumped.capacity <= 64, "clumped churn stays small");
}

string_keys :: () {
    table: Swiss.Table(string, s64);

    defer Swiss.deinit(*table);

    for 0..9999 {
        key := tprint("key-%", it);
        Swiss.table_set(*table, copy_string(key), it);
    }

    defer for table free(it_index);

    check(table.count == 10000, "string count");

    for 0..9999 {
        ok, value := Swiss.table_find(*table, tprint("key-%", it));
        if !ok || value != it {
            check(false, "string lookup");
            break;
        }
    }

    check(!Swiss.table_contains(*table, "key-10000"), "missing string");
    check(!Swiss.table_contains(*table, ""), "empty string is missing");
    Swiss.table_set(*table, copy_string(""), -1);
    check(Swiss.table_find_pointer(*table, "").* == -1, "empty string key");
}

case_insensitive_hash :: (key: string) -> u32 {
    value: u32 = 17;

    for byte: key {
        lower := byte;
        if lower >= #char "A" && lower <= #char "Z" lower += 32;
        value = value * 31 + lower;
    }

    return value;
}

case_insensitive_equal :: (a: string, b: string) -> bool {
    if a.count != b.count return false;

    for 0..a.count - 1 {
        x := a[it];
        y := b[it];

        if x >= #char "A" && x <= #char "Z" x += 32;
        if y >= #char "A" && y <= #char "Z" y += 32;
        if x != y return false;
    }

    return true;
}

custom_functions :: () {
    table: Swiss.Table(string, s64, case_insensitive_hash, case_insensitive_equal);

    defer Swiss.deinit(*table);

    Swiss.table_set(*table, "Hello", 1);
    Swiss.table_set(*table, "HELLO", 2);
    Swiss.table_set(*table, "world", 3);

    check(table.count == 2, "custom compare merges keys");
    check(Swiss.table_find_pointer(*table, "hello").* == 2, "custom lookup");
    added, _ := Swiss.table_add(*table, "WORLD", 4);

    check(!added, "custom compare refuses duplicates");

    // Every key collides, so lookups lean on the compare procedure alone.
    all_same: Swiss.Table(s64, s64, (key: s64) -> u32 {
        return 5;
    });

    defer Swiss.deinit(*all_same);

    for 0..499 Swiss.table_set(*all_same, it, it * 2);

    ok := true;

    for 0..499 {
        found, value := Swiss.table_find(*all_same, it);
        if !found || value != it * 2 ok = false;
    }

    check(ok && all_same.count == 500, "total collisions still find everything");
    for 0..499 if it % 2 == 0 Swiss.table_remove(*all_same, it);

    for 0..499 {
        if Swiss.table_contains(*all_same, it) != (it % 2 == 1) ok = false;
    }

    check(ok && all_same.count == 250, "removal under total collisions");
}

iteration :: () {
    COUNT :: 5000;
    table: Swiss.Table(s64, s64);

    defer Swiss.deinit(*table);

    for 0..COUNT - 1 Swiss.table_set(*table, it, it * 10);
    for 0..COUNT - 1 if it % 3 == 0 Swiss.table_remove(*table, it);

    seen := NewArray(COUNT, s64);

    defer array_free(seen);
    visited := 0;

    for value, key: table {
        seen[key] += 1;
        visited += 1;
        if value != key * 10 check(false, "value matches key");
    }

    ok := true;

    for 0..COUNT - 1 {
        expected := ifx it % 3 == 0 then 0 else 1;
        if seen[it] != expected ok = false;
    }

    check(ok && visited == table.count, "forward visits each entry once");

    visited = 0;

    for < table {
        visited += 1;
        seen[it_index] -= 1;
    }

    check(visited == table.count, "reverse visits each entry once");

    for *table it.* += 1;

    check(Swiss.table_find_pointer(*table, 1).* == 11, "pointer iteration writes through");

    first := true;

    for table {
        if first {
            first = false;
            break;
        }
    }

    check(!first, "break leaves the loop");

    empty: Swiss.Table(s64, s64);
    for empty check(false, "empty table has nothing to visit");
}

remove_while_iterating :: () {
    COUNT :: 4000;
    table: Swiss.Table(s64, s64);

    defer Swiss.deinit(*table);

    for 0..COUNT - 1 Swiss.table_set(*table, it, it);

    visited := 0;

    for table {
        visited += 1;
        if it_index % 2 == 0 remove it;
    }

    check(visited == COUNT, "removing does not skip entries");
    check(table.count == COUNT / 2, "removed half");
    ok := true;

    for 0..COUNT - 1 {
        if Swiss.table_contains(*table, it) != (it % 2 == 1) ok = false;
    }

    check(ok, "only odd keys remain");

    // The table is still sound after in-loop removals: add and find as usual.
    for 0..COUNT - 1 Swiss.table_set(*table, it, it + 1);

    check(table.count == COUNT, "re-add after loop removals");

    for < table if it_index % 3 == 0 remove it;
    for table if it_index % 3 == 0 check(false, "reverse removal");

    check(!Swiss.table_contains(*table, 3) && Swiss.table_contains(*table, 4), "reverse remove keeps the rest");

    for table remove it;

    check(table.count == 0, "remove everything");
    Swiss.table_set(*table, 1, 1);
    check(table.count == 1, "usable after removing everything");
}

struct_values :: () {
    Point :: struct {
        x: s32;
        y: s32;
    }

    table: Swiss.Table(Point, string, (p: Point) -> u32 {
        return cast(u32) p.x * 73856093 + cast(u32) p.y * 19349663;
    }, (a: Point, b: Point) -> bool {
        return a.x == b.x && a.y == b.y;
    });

    defer Swiss.deinit(*table);

    Swiss.table_set(*table, .{ 1, 2 }, "a");
    Swiss.table_set(*table, .{ 2, 1 }, "b");
    check(Swiss.table_find_pointer(*table, .{ 2, 1 }).* == "b", "struct keys");
    check(Swiss.table_find_pointer(*table, .{ 3, 3 }) == null, "missing struct key");
}

main :: () {
    basic_operations();
    growth();
    churn();
    string_keys();
    custom_functions();
    iteration();
    remove_while_iterating();
    struct_values();

    if failures == 0 print("ok\n");
    else exit(1);
}
