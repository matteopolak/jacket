// Insert, find, get, entry, delete, growth, string keys, custom hash and equal, iteration and
// removal while iterating.
//
//     jaic stdlib/Extensions/Hash_Map/tests/basics.jai -interp
#import "Basic";
Map :: #import "Extensions/Hash_Map";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    if failures < 20 print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

basic_operations :: () {
    map: Map.Map(s64, s64);

    defer Map.deinit(*map);

    check(map.count == 0, "starts empty");
    check(!Map.contains(*map, 1), "empty map has no key");
    check(Map.find(*map, 1) == null, "find on empty");
    check(Map.get(*map, 1) == 0, "get on empty");

    gone, was_there := Map.delete(*map, 1);

    check(!was_there && gone == 0, "delete on empty");

    previous, replaced := Map.insert(*map, 7, 70);

    check(!replaced && previous == 0, "first insert replaces nothing");
    check(Map.find(*map, 7).* == 70, "insert stores the value");

    Map.find(*map, 7).* = 71;
    check(Map.get(*map, 7) == 71, "write through the found pointer");

    previous, replaced = Map.insert(*map, 7, 72);
    check(replaced && previous == 71, "insert returns the replaced value");
    check(map.count == 1, "overwrite keeps the count");
    check(Map.find(*map, 7).* == 72, "overwrite replaces the value");

    check(Map.get(*map, 7, default = -5) == 72, "get ignores the default when present");
    check(Map.get(*map, 8, default = -5) == -5, "get returns the default when missing");
    check(Map.get(*map, 8) == 0, "get defaults to zero");
    check(map.count == 1, "get does not add");

    slot, added := Map.entry(*map, 7);

    check(!added && slot.* == 72 && map.count == 1, "entry finds an existing key");

    slot, added = Map.entry(*map, 9);
    check(added && slot.* == 0 && map.count == 2, "entry adds a zeroed slot");
    slot.* = 90;
    check(Map.get(*map, 9) == 90, "entry slot is writable");

    slot, added = Map.entry(*map, 9);
    check(!added && slot.* == 90 && map.count == 2, "entry finds what it added");

    // A slot that held a value, was removed and is claimed again starts from zero.
    Map.delete(*map, 9);
    slot, added = Map.entry(*map, 9);
    check(added && slot.* == 0, "entry slot is zero after a delete");

    old, removed := Map.delete(*map, 7);

    check(removed && old == 72 && map.count == 1, "delete returns the value");
    check(!Map.contains(*map, 7), "deleted key is gone");
    check(Map.find(*map, 7) == null, "deleted key has no pointer");

    old, removed = Map.delete(*map, 7);
    check(!removed && old == 0, "second delete fails");

    Map.insert(*map, 7, 1);
    check(Map.find(*map, 7).* == 1, "re-add after delete");
}

growth :: () {
    COUNT :: 50000;
    map: Map.Map(s64, s64);

    defer Map.deinit(*map);

    for 0..COUNT - 1 {
        _, replaced := Map.insert(*map, it * 3, it);
        check(!replaced, "unique keys add");
    }

    check(map.count == COUNT, "count after growth");
    check(map.capacity >= COUNT && map.capacity * 7 / 8 >= COUNT, "capacity covers the load limit");
    check(map.capacity / 2 * 7 / 8 < COUNT, "capacity is not larger than needed by more than one doubling");

    for 0..COUNT - 1 {
        value := Map.find(*map, it * 3);
        if !value || value.* != it {
            check(false, "every key survives growth");
            break;
        }
    }

    for 0..COUNT - 1 {
        if Map.contains(*map, it * 3 + 1) {
            check(false, "no false hits");
            break;
        }
    }

    reserved: Map.Map(s64, s64);

    defer Map.deinit(*reserved);
    Map.reserve(*reserved, 1000);
    capacity := reserved.capacity;

    for 0..999 Map.insert(*reserved, it, it);

    check(reserved.capacity == capacity && capacity >= 1000, "reserve means no rebuild");

    Map.clear(*reserved);
    check(reserved.count == 0 && reserved.capacity == capacity, "clear keeps memory");
    check(!Map.contains(*reserved, 5), "clear empties");
    for 0..999 Map.insert(*reserved, it, it);

    check(reserved.capacity == capacity && reserved.count == 1000, "reuse after clear");

    Map.deinit(*reserved);
    check(reserved.count == 0 && reserved.capacity == 0 && reserved.control == null, "deinit frees");
    Map.insert(*reserved, 1, 1);
    check(Map.contains(*reserved, 1), "usable after deinit");
}

// Inserting and removing the same few keys forever must not grow the map.
churn :: () {
    map: Map.Map(s64, s64);

    defer Map.deinit(*map);

    for 0..99 Map.insert(*map, 1000000 + it, it);

    capacity_before := map.capacity;

    for round: 0..49999 {
        for i: 0..9 Map.insert(*map, round * 10 + i, i);

        for i: 0..9 {
            value, removed := Map.delete(*map, round * 10 + i);
            if !removed || value != i {
                check(false, "churn remove");
                return;
            }
        }
    }

    check(map.count == 100, "churn leaves the original entries");
    check(map.capacity <= capacity_before * 2, "churn does not grow without bound");

    for 0..99 {
        value := Map.find(*map, 1000000 + it);
        if !value || value.* != it {
            check(false, "originals survive churn");
            break;
        }
    }

    // A hash that sends everything to one group keeps deleted slots in the probe path.
    clumped: Map.Map(s64, s64, (key: s64) -> u32 {
        return 0;
    });

    defer Map.deinit(*clumped);

    for round: 0..299 {
        for i: 0..19 Map.insert(*clumped, round * 20 + i, i);

        for i: 0..19 {
            _, removed := Map.delete(*clumped, round * 20 + i);
            if !removed {
                check(false, "clumped remove");
                return;
            }
        }
    }

    check(clumped.count == 0 && clumped.capacity <= 64, "clumped churn stays small");
}

string_keys :: () {
    map: Map.Map(string, s64);

    defer Map.deinit(*map);

    for 0..9999 {
        key := tprint("key-%", it);
        Map.insert(*map, copy_string(key), it);
    }

    defer for map free(it_index);

    check(map.count == 10000, "string count");

    for 0..9999 {
        value := Map.find(*map, tprint("key-%", it));
        if !value || value.* != it {
            check(false, "string lookup");
            break;
        }
    }

    check(!Map.contains(*map, "key-10000"), "missing string");
    check(!Map.contains(*map, ""), "empty string is missing");
    Map.insert(*map, copy_string(""), -1);
    check(Map.find(*map, "").* == -1, "empty string key");
}

// Strings of every length up to 40 that differ in one byte: the default string hash reads
// them in words, with a different tail for each length class.
string_lengths :: () {
    map: Map.Map(string, s64);

    defer Map.deinit(*map);
    defer for map free(it_index);

    expected := 0;

    for length: 0..40 {
        for position: -1..length - 1 {
            text := alloc_string(length);

            for 0..length - 1 text[it] = cast(u8) (#char "a" + (it * 7 + length) % 26);

            if position >= 0 text[position] = #char "Z";

            // Position -1 is the unchanged text; a length-1 text with its only byte changed is
            // a different key, as is every other single change.
            Map.insert(*map, text, length * 100 + position);
            expected += 1;
        }
    }

    check(map.count == expected, "every length and byte change is its own key");

    for length: 0..40 {
        for position: -1..length - 1 {
            text := alloc_string(length);

            for 0..length - 1 text[it] = cast(u8) (#char "a" + (it * 7 + length) % 26);

            if position >= 0 text[position] = #char "Z";

            value := Map.find(*map, text);

            if !value || value.* != length * 100 + position {
                check(false, "string of varied length found");
                return;
            }

            free(text);
        }
    }
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
    map: Map.Map(string, s64, case_insensitive_hash, case_insensitive_equal);

    defer Map.deinit(*map);

    Map.insert(*map, "Hello", 1);
    Map.insert(*map, "HELLO", 2);
    Map.insert(*map, "world", 3);

    check(map.count == 2, "custom compare merges keys");
    check(Map.find(*map, "hello").* == 2, "custom lookup");
    _, replaced := Map.insert(*map, "WORLD", 4);

    check(replaced, "custom equal finds the existing key");

    // Every key collides, so lookups lean on the compare procedure alone.
    all_same: Map.Map(s64, s64, (key: s64) -> u32 {
        return 5;
    });

    defer Map.deinit(*all_same);

    for 0..499 Map.insert(*all_same, it, it * 2);

    ok := true;

    for 0..499 {
        value := Map.find(*all_same, it);
        if !value || value.* != it * 2 ok = false;
    }

    check(ok && all_same.count == 500, "total collisions still find everything");
    for 0..499 if it % 2 == 0 Map.delete(*all_same, it);

    for 0..499 {
        if Map.contains(*all_same, it) != (it % 2 == 1) ok = false;
    }

    check(ok && all_same.count == 250, "removal under total collisions");
}

iteration :: () {
    COUNT :: 5000;
    map: Map.Map(s64, s64);

    defer Map.deinit(*map);

    for 0..COUNT - 1 Map.insert(*map, it, it * 10);
    for 0..COUNT - 1 if it % 3 == 0 Map.delete(*map, it);

    seen := NewArray(COUNT, s64);

    defer array_free(seen);
    visited := 0;

    for value, key: map {
        seen[key] += 1;
        visited += 1;
        if value != key * 10 check(false, "value matches key");
    }

    ok := true;

    for 0..COUNT - 1 {
        expected := ifx it % 3 == 0 then 0 else 1;
        if seen[it] != expected ok = false;
    }

    check(ok && visited == map.count, "forward visits each entry once");

    visited = 0;

    for < map {
        visited += 1;
        seen[it_index] -= 1;
    }

    check(visited == map.count, "reverse visits each entry once");

    for *map it.* += 1;

    check(Map.find(*map, 1).* == 11, "pointer iteration writes through");

    first := true;

    for map {
        if first {
            first = false;
            break;
        }
    }

    check(!first, "break leaves the loop");

    steps := 0;

    for map {
        steps += 1;
        if steps == 3 break;
    }

    check(steps == 3, "break stops the forward loop");

    steps = 0;

    for < map {
        steps += 1;
        if steps == 3 break;
    }

    check(steps == 3, "break stops the reverse loop");

    kept := 0;

    for map {
        if it_index % 2 == 0 continue;
        kept += 1;
    }

    skipped := 0;

    for < map {
        if it_index % 2 != 0 continue;
        skipped += 1;
    }

    odd := 0;
    even := 0;

    for value, key: map {
        if key % 2 != 0 odd += 1;
        else even += 1;
    }

    check(kept == odd && skipped == even, "continue goes to the next entry in both directions");

    empty: Map.Map(s64, s64);
    for empty check(false, "empty map has nothing to visit");
}

remove_while_iterating :: () {
    COUNT :: 4000;
    map: Map.Map(s64, s64);

    defer Map.deinit(*map);

    for 0..COUNT - 1 Map.insert(*map, it, it);

    visited := 0;

    for map {
        visited += 1;
        if it_index % 2 == 0 remove it;
    }

    check(visited == COUNT, "removing does not skip entries");
    check(map.count == COUNT / 2, "removed half");
    ok := true;

    for 0..COUNT - 1 {
        if Map.contains(*map, it) != (it % 2 == 1) ok = false;
    }

    check(ok, "only odd keys remain");

    // The map is still sound after in-loop removals: add and find as usual.
    for 0..COUNT - 1 Map.insert(*map, it, it + 1);

    check(map.count == COUNT, "re-add after loop removals");

    for < map if it_index % 3 == 0 remove it;
    for map if it_index % 3 == 0 check(false, "reverse removal");

    check(!Map.contains(*map, 3) && Map.contains(*map, 4), "reverse remove keeps the rest");

    for map remove it;

    check(map.count == 0, "remove everything");
    Map.insert(*map, 1, 1);
    check(map.count == 1, "usable after removing everything");
}

struct_values :: () {
    Point :: struct {
        x: s32;
        y: s32;
    }

    map: Map.Map(Point, string, (p: Point) -> u32 {
        return cast(u32) p.x * 73856093 + cast(u32) p.y * 19349663;
    }, (a: Point, b: Point) -> bool {
        return a.x == b.x && a.y == b.y;
    });

    defer Map.deinit(*map);

    Map.insert(*map, .{ 1, 2 }, "a");
    Map.insert(*map, .{ 2, 1 }, "b");
    check(Map.find(*map, .{ 2, 1 }).* == "b", "struct keys");
    check(Map.find(*map, .{ 3, 3 }) == null, "missing struct key");
    check(Map.get(*map, .{ 3, 3 }, default = "none") == "none", "get default for a string value");
    check(Map.get(*map, .{ 3, 3 }) == "", "get zero for a string value");
}

main :: () {
    basic_operations();
    growth();
    churn();
    string_keys();
    string_lengths();
    custom_functions();
    iteration();
    remove_while_iterating();
    struct_values();

    if failures == 0 print("ok\n");
    else exit(1);
}
