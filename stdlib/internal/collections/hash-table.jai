#module_parameters(COUNT_COLLISIONS := false);
NEVER_OCCUPIED_HASH :: 0;
REMOVED_HASH :: 1;
FIRST_VALID_HASH :: 2;

// A hash table from `Key_Type` to `Value_Type`: open addressing with linear probing, in one array of entries.
//
// A zero-initialized `Table` is ready to use and owns no memory until the first add. The first growth takes
// the allocator from `context.allocator` and the table keeps it; free everything with `deinit` (or
// `table_reset`). The slot count is a power of two, at least 32, and doubles when the table is more than
// `LOAD_FACTOR_PERCENT` full counting removed entries, so adds are amortized constant time.
//
// Keys are hashed with `Hash.get_hash` and compared with `==`, which suits integers, enums, pointers and
// strings (by content). Pass `given_hash_function` and `given_compare_function` for anything else. Adding a
// key that is already present does not replace it: `table_add` keeps both entries, `table_set` replaces the
// value. The first is how to make a multi-map, see `table_find_multiple`.
//
// `count` is the number of entries. Removed entries leave a marker that a later add reuses when
// `REFILL_REMOVED` is `true` (the default); with `false` markers stay until the table grows. Pointers to values
// and keys are stable only until the table grows, i.e. until the next add that fills it, `expand`,
// `table_resize` or `table_ensure_space`.
//
// The order of iteration is by slot, which depends on the hashes, and is not the insertion order. Do not
// rely on it, nor on it staying the same after the table grows.
//
// ```jai
// table: Table(string, int);
// table_add(*table, "one", 1);
// table_set(*table, "two", 2);
// table_set(*table, "two", 22); // replaces
//
// found, value := table_find(*table, "two");
// assert(found && value == 22 && table.count == 2);
//
// total := 0;
// for number, name: table total += number; // `it` is the value, `it_index` the key
// assert(total == 23);
//
// deinit(*table);
// ```
Table :: struct(Key_Type: Type, Value_Type: Type, given_hash_function: (Key_Type) -> u32 = null, given_compare_function: (Key_Type, Key_Type) -> bool = null, LOAD_FACTOR_PERCENT: u32 = 70, REFILL_REMOVED := true) {
    count: int;
    allocated: int;
    slots_filled: int;
    allocator: Allocator;
    Entry :: struct {
        hash: u32;
        key: Key_Type;
        value: Value_Type;
    }
    entries: [] Entry;
    #if COUNT_COLLISIONS {
        add_collisions: s64;
        find_collisions: s64;
    }
    #if given_hash_function {
        hash_function :: given_hash_function;
    } else {
        hash_function :: (key: Key_Type) -> u32 {
            return Hash.get_hash(key);
        }
    }
    #if given_compare_function {
        compare_function :: given_compare_function;
    } else {
        compare_function :: (a: Key_Type, b: Key_Type) -> bool {
            return a == b;
        }
    }
    SIZE_MIN :: 32;
    #assert LOAD_FACTOR_PERCENT > 0 && LOAD_FACTOR_PERCENT < 100;
}

// Deprecated and not needed: a zero-initialized `Table` is ready to use. With `slots_to_allocate` greater than zero it reserves that many slots (rounded up) through `expand`.
init :: (using table: *Table, slots_to_allocate: s64 = 0) #deprecated "Not needed: a zero-initialized Table is ready to use." {
    if slots_to_allocate > 0 expand(table, slots_to_allocate);
}

// The smallest power of two that is at least `x`, and `1` for anything below that. Fails an assert when the result would not fit an `int`.
//
// ```jai
// assert(next_power_of_two(33) == 64);
// assert(next_power_of_two(64) == 64);
// assert(next_power_of_two(0) == 1);
// ```
next_power_of_two :: inline (x: int) -> int {
    value: int = 1;
    while value < x {
        assert(value <= 0x3fffffffffffffff);
        value *= 2;
    }

    return value;
}

// Rehashes every entry into a new array of `new_allocated` slots, and frees the old one.
//
// The size is rounded up to a power of two and never below the minimum of 32 slots, nor below what the
// current entries need at the load factor. A negative `new_allocated` (the default) doubles the slots.
// Removed entries are dropped by the rehash, so it also reclaims the space they filled. Every pointer into
// the table is stale afterwards. Adds call this for you; reach for `table_resize` to plan ahead.
expand :: (table: *Table, new_allocated := -1) {
    size := new_allocated;
    if size < 0 size = max(table.SIZE_MIN, table.allocated * 2);

    size = next_power_of_two(max(size, max(table.SIZE_MIN, table.count * 100 / cast(s64) table.LOAD_FACTOR_PERCENT + 1)));
    if !table.allocator.proc table.allocator = context.allocator;

    memory := table.allocator.proc(.ALLOCATE, size * size_of(table.Entry), 0, null, table.allocator.data);

    assert(memory != null, "hash table allocation failed");
    memset(memory, 0, size * size_of(table.Entry));
    old := table.entries;

    table.entries = .{ size, cast(*table.Entry) memory };
    table.allocated = size;
    table.count = 0;
    table.slots_filled = 0;
    for entry: old if entry.hash >= FIRST_VALID_HASH insert_entry(table, entry.key, entry.value, entry.hash);
    if old.data table.allocator.proc(.FREE, 0, old.count * size_of(table.Entry), old.data, table.allocator.data);
}

// Makes sure `items` more entries can be added without the table growing again.
//
// Does nothing when there is room. When many removed entries fill the table, it rehashes at the size the live
// entries need instead of doubling. `table_add` calls it with `1`; call it with a larger number before a bulk
// add to avoid repeated growth. A negative `items` fails an assert.
table_ensure_space :: (using table: *Table, items: s64) {
    assert(items >= 0);

    if allocated == 0 || (slots_filled + items) * 100 >= allocated * cast(s64) table.LOAD_FACTOR_PERCENT {
        needed := (count + items) * 100 / cast(s64) table.LOAD_FACTOR_PERCENT + 1;

        // Removed entries still fill slots. When they outnumber the live ones, rehashing at
        // the size the live entries need reclaims them instead of doubling the table.
        if slots_filled - count > count expand(table, needed);
        else expand(table, max(allocated * 2, needed));
    }
}

// Rehashes the table so that `expected_count` entries (or the current count, if larger) fit within the load factor.
//
// Use it before a bulk add when the final size is known, or after many removals to rebuild a table that has
// filled up with removed markers. It never loses entries, and invalidates pointers into the table. A negative
// `expected_count` fails an assert.
//
// ```jai
// table: Table(int, int);
// table_resize(*table, 1000);
// slots := table.allocated;
// for 0..999 table_add(*table, it, it);
// assert(table.allocated == slots && table.count == 1000); // no growth in between
// deinit(*table);
// ```
table_resize :: (using table: *Table, expected_count: s64) {
    assert(expected_count >= 0);
    expand(table, max(expected_count, count) * 100 / cast(s64) table.LOAD_FACTOR_PERCENT + 1);
}

// Adds an entry, even when `key` is already present, and returns a pointer to the stored value.
//
// Because duplicates are allowed, a later `table_find` returns whichever entry the probe reaches first; use
// `table_set` when a key must stay unique. Grows the table when needed, which invalidates earlier pointers.
// The returned pointer is good until the table next grows.
//
// ```jai
// table: Table(int, string);
// slot := table_add(*table, 1, "one");
// slot.* = "uno";
// found, value := table_find(*table, 1);
// assert(found && value == "uno");
// deinit(*table);
// ```
table_add :: (table: *Table, key: table.Key_Type, value: table.Value_Type) -> *table.Value_Type {
    table_ensure_space(table, 1);
    return insert_entry(table, key, value, valid_hash(table, key));
}

// Sets the value of `key`, replacing the first entry with an equal key, or adding one if there is none. Returns a pointer to the stored value.
//
// With duplicate keys present only the first match found is replaced. Pointers are stable unless the call
// had to add an entry and the table grew.
table_set :: (table: *Table, key: table.Key_Type, value: table.Value_Type) -> *table.Value_Type {
    // One hash serves both the lookup and the insert.
    hash := valid_hash(table, key);
    entry := find_entry(table, key, hash);

    if entry {
        entry.value = value;
        return *entry.value;
    }

    table_ensure_space(table, 1);

    return insert_entry(table, key, value, hash);
}

// A pointer to the value stored for `key`, or `null` when there is none, so the value can be changed in place.
//
// The pointer is valid until the table grows or the entry is removed. Looking up in a table that never held
// anything is fine and finds nothing.
//
// ```jai
// counts: Table(string, int);
// table_add(*counts, "a", 1);
// pointer := table_find_pointer(*counts, "a");
// pointer.* += 1;
// assert(table_find_pointer(*counts, "a").* == 2);
// assert(table_find_pointer(*counts, "b") == null);
// deinit(*counts);
// ```
table_find_pointer :: (table: *Table, key: table.Key_Type) -> *table.Value_Type {
    entry := find_entry(table, key);
    if !entry return null;
    return *entry.value;
}

// ```jai
// table: Table(int, string);
// table_add(*table, 7, "seven");
// assert(table_contains(*table, 7) && !table_contains(*table, 8));
// ```
table_contains :: (table: *Table, key: table.Key_Type) -> bool {
    return table_find_pointer(table, key) != null;
}

// Looks up `key`; returns whether it was found and a copy of the value (the default value of the type when it was not).
//
// ```jai
// table: Table(int, float);
// table_add(*table, 3, 1.5);
// found, value := table_find(*table, 3);
// assert(found && value == 1.5);
// found, value = table_find(*table, 4);
// assert(!found && value == 0);
// deinit(*table);
// ```
table_find :: (table: *Table, key: table.Key_Type) -> (success: bool, table.Value_Type) {
    pointer := table_find_pointer(table, key);
    if pointer return true, pointer.*;
    value: table.Value_Type;
    return false, value;
}

// Every value stored under `key`, as a view into temporary storage (`talloc`), or an empty view when there are none.
//
// Meant for tables that hold duplicate keys through `table_add`. The values come in slot order, which is
// not necessarily the order they were added in. The view is only valid until the temporary storage is reset,
// so copy it if it must live longer. It does not need freeing.
//
// ```jai
// table: Table(int, string);
// table_add(*table, 1, "a");
// table_add(*table, 1, "b");
// table_add(*table, 2, "c");
// assert(table_find_multiple(*table, 1).count == 2);
// assert(table_find_multiple(*table, 9).count == 0);
// deinit(*table);
// ```
table_find_multiple :: (table: *Table, key: table.Key_Type) -> [] table.Value_Type {
    if table.count == 0 return .{};

    hash := valid_hash(table, key);
    mask := table.allocated - 1;

    // Matches sit in one probe run from the key's slot; an empty slot ends it, removed ones do not.
    count: s64;
    index := cast(s64) hash & mask;

    for 0..table.allocated - 1 {
        entry := table.entries.data + index;
        if entry.hash == NEVER_OCCUPIED_HASH break;
        if entry.hash == hash && table.compare_function(entry.key, key) count += 1;
        index = (index + 1) & mask;
    }

    if count == 0 return .{};

    data := cast(*table.Value_Type) talloc(count * size_of(table.Value_Type));

    assert(data != null);
    found: s64;

    index = cast(s64) hash & mask;

    while found < count {
        entry := table.entries.data + index;

        if entry.hash == hash && table.compare_function(entry.key, key) {
            data[found] = entry.value;
            found += 1;
        }

        index = (index + 1) & mask;
    }

    return .{ count, data };
}

// Removes one entry with this key; returns whether there was one and its value.
//
// With duplicate keys it removes the first match only; see `table_remove_multiple`. The slot keeps a marker
// until it is reused or the table is rebuilt, so removing does not shrink the table or move other entries.
//
// ```jai
// table: Table(int, string);
// table_add(*table, 5, "five");
// removed, value := table_remove(*table, 5);
// assert(removed && value == "five" && table.count == 0);
// removed, value = table_remove(*table, 5);
// assert(!removed);
// deinit(*table);
// ```
table_remove :: (table: *Table, key: table.Key_Type) -> (success: bool, value: table.Value_Type) {
    entry := find_entry(table, key);
    if !entry {
        value: table.Value_Type;
        return false, value;
    }

    value := entry.value;

    entry.hash = REMOVED_HASH;
    table.count -= 1;

    return true, value;
}

// Removes every entry with this key and returns how many there were.
//
// ```jai
// table: Table(int, int);
// table_add(*table, 1, 10);
// table_add(*table, 1, 11);
// table_add(*table, 2, 20);
// assert(table_remove_multiple(*table, 1) == 2 && table.count == 1);
// deinit(*table);
// ```
table_remove_multiple :: (table: *Table, key: table.Key_Type) -> (num_removed: s64) {
    count: s64;

    while true {
        entry := find_entry(table, key);
        if !entry break;

        entry.hash = REMOVED_HASH;
        table.count -= 1;
        count += 1;
    }

    return count;
}

// Finds the value for `key`, adding a zero-initialized one if there is none. Returns a pointer to it and whether it was just added.
//
// This is the way to update a value that may not exist yet with a single lookup.
//
// ```jai
// counts: Table(string, int);
// for word: string.["a", "b", "a"] {
//     count, added := find_or_add(*counts, word);
//     count.* += 1;
// }
// assert(table_find_pointer(*counts, "a").* == 2 && counts.count == 2);
// deinit(*counts);
// ```
find_or_add :: (table: *Table, key: table.Key_Type) -> (entry: *table.Value_Type, newly_added: bool) {
    hash := valid_hash(table, key);
    entry := find_entry(table, key, hash);

    if entry return *entry.value, false;

    value: table.Value_Type;

    table_ensure_space(table, 1);

    return insert_entry(table, key, value, hash), true;
}

// Removes every entry but keeps the allocated slots, so refilling the table up to its old size allocates nothing.
//
// Costs time linear in the slot count. Values are not freed; if they own memory, free it first.
table_reset_keeping_memory :: (using table: *Table) {
    memset(entries.data, 0, entries.count * size_of(table.Entry));
    count = 0;
    slots_filled = 0;
}

// Removes every entry and frees the slot array through the table's allocator. The table is usable again afterwards and allocates from scratch.
//
// Values and keys that own memory are not freed.
table_reset :: (using table: *Table) {
    if entries.data allocator.proc(.FREE, 0, entries.count * size_of(table.Entry), entries.data, allocator.data);

    entries = .{};
    count = 0;
    slots_filled = 0;
    allocated = 0;
}

// Frees the table's memory, as `table_reset` does. Call it when the table is no longer needed.
deinit :: (table: *Table) {
    table_reset(table);
} @FreesArgument

// Lets a table be used in a `for` loop: `it` is the value and `it_index` the key.
//
// `for *table` gives a pointer to the value to change it in place, and `for <` goes in reverse slot order.
// Inside the body `remove it;` deletes the current entry (it leaves a marker, as `table_remove` does). Entries
// come in slot order, not in the order they were added in. Do not add to the table in the body, since growth
// moves the entries mid-loop.
//
// ```jai
// table: Table(int, int);
// for 0..9 table_add(*table, it, it * it);
//
// for *square, key: table square.* += key; // in place
//
// for square, key: table {
//     if key % 2 == 0 remove square;
// }
// assert(table.count == 5);
// deinit(*table);
// ```
for_expansion :: (table: *$T/Table, body: Code, flags: For_Flags) #expand {
    #if (flags & .REVERSE) != 0 {
        i := table.entries.count;
        while i > 0 {
            i -= 1;
            entry := table.entries.data + i;
            if entry.hash < FIRST_VALID_HASH continue;

            `it_index := entry.key;

            #if (flags & .POINTER) != 0 {
                `it := *entry.value;
                #insert (remove = {
                    entry.hash = REMOVED_HASH;
                    table.count -= 1;
                }) body;
            } else {
                `it := entry.value;
                #insert (remove = {
                    entry.hash = REMOVED_HASH;
                    table.count -= 1;
                }) body;
            }
        }
    } else {
        for *entry: table.entries {
            if entry.hash < FIRST_VALID_HASH continue;

            `it_index := entry.key;

            #if (flags & .POINTER) != 0 {
                `it := *entry.value;
                #insert (remove = {
                    entry.hash = REMOVED_HASH;
                    table.count -= 1;
                }) body;
            } else {
                `it := entry.value;
                #insert (remove = {
                    entry.hash = REMOVED_HASH;
                    table.count -= 1;
                }) body;
            }
        }
    }
}

// Low-level macro that visits the slots a lookup of `key` would probe, in order, up to the first never-used slot.
//
// It reads the caller's variables `table` and `key`, and defines `hash` (the key's hash), `index` (the slot)
// and `table_while_loop` (the hash stored in that slot) for `code`. A removed entry shows as
// `REMOVED_HASH`. Use `continue table_while_loop` and `break table_while_loop` to control the walk. It exists to
// build lookups with custom matching, such as finding all entries that agree with `key` in some other way;
// prefer `table_find` and friends otherwise.
//
// ```jai
// table: Table(int, string);
// table_add(*table, 7, "seven");
//
// key := 7;
// matches := 0;
// Walk_Table(#code {
//     if table_while_loop == hash && table.entries[index].key == key matches += 1;
// });
// assert(matches == 1);
// deinit(*table);
// ```
Walk_Table :: (code: Code) #expand {
    `hash: u32 = `table.hash_function(`key);
    if `hash < FIRST_VALID_HASH `hash += FIRST_VALID_HASH;

    `index: u32 = `hash & cast,trunc(u32) (`table.allocated - 1);
    first_probe := true;
    while `table_while_loop := walk_probe_hash(`table, *`index, *first_probe) {
        #insert code;
    }
}

#scope_file
#import "Basic";
Hash :: #import "Hash";

// Advance at the next condition evaluation: continue advances, break does not.
walk_probe_hash :: inline (table: $T, index: *u32, first: *bool) -> u32 {
    if first.* first.* = false;
    else index.* = cast,trunc(u32) ((cast(u64) index.* + 1) & cast(u64) (table.allocated - 1));
    return table.entries[index.*].hash;
}

valid_hash :: (table: *Table, key: table.Key_Type) -> u32 {
    hash := table.hash_function(key);
    if hash < FIRST_VALID_HASH hash += FIRST_VALID_HASH;
    return hash;
}

insert_entry :: (table: *Table, key: table.Key_Type, value: table.Value_Type, hash: u32) -> *table.Value_Type {
    index := cast(s64) hash & (table.allocated - 1);

    while true {
        entry := table.entries.data + index;
        if entry.hash == NEVER_OCCUPIED_HASH || (table.REFILL_REMOVED && entry.hash == REMOVED_HASH) {
            if entry.hash == NEVER_OCCUPIED_HASH table.slots_filled += 1;

            entry.hash = hash;
            entry.key = key;
            entry.value = value;
            table.count += 1;

            return *entry.value;
        }

        #if COUNT_COLLISIONS table.add_collisions += 1;
        index = (index + 1) & (table.allocated - 1);
    }

    return null;
}

find_entry :: (table: *Table, key: table.Key_Type) -> *table.Entry {
    if table.allocated == 0 return null;
    return find_entry(table, key, valid_hash(table, key));
}

find_entry :: (table: *Table, key: table.Key_Type, hash: u32) -> *table.Entry {
    if table.allocated == 0 return null;

    index := cast(s64) hash & (table.allocated - 1);

    for 0..table.allocated - 1 {
        entry := table.entries.data + index;
        if entry.hash == NEVER_OCCUPIED_HASH return null;
        if entry.hash == hash && table.compare_function(entry.key, key) return entry;

        #if COUNT_COLLISIONS table.find_collisions += 1;
        index = (index + 1) & (table.allocated - 1);
    }

    return null;
}
