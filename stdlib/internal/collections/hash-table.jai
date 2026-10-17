#module_parameters(COUNT_COLLISIONS := false);
NEVER_OCCUPIED_HASH :: 0;
REMOVED_HASH :: 1;
FIRST_VALID_HASH :: 2;

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

init :: (using table: *Table, slots_to_allocate: s64 = 0) #deprecated "Not needed: a zero-initialized Table is ready to use." {
    if slots_to_allocate > 0 expand(table, slots_to_allocate);
}

next_power_of_two :: inline (x: int) -> int {
    value: int = 1;
    while value < x {
        assert(value <= 0x3fffffffffffffff);
        value *= 2;
    }

    return value;
}

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

table_ensure_space :: (using table: *Table, items: s64) {
    assert(items >= 0);
    if allocated == 0 || (slots_filled + items) * 100 >= allocated * cast(s64) table.LOAD_FACTOR_PERCENT
        expand(table, max(allocated * 2, (count + items) * 100 / cast(s64) table.LOAD_FACTOR_PERCENT + 1));
}

table_resize :: (using table: *Table, expected_count: s64) {
    assert(expected_count >= 0);
    expand(table, max(expected_count, count) * 100 / cast(s64) table.LOAD_FACTOR_PERCENT + 1);
}

table_add :: (table: *Table, key: table.Key_Type, value: table.Value_Type) -> *table.Value_Type {
    table_ensure_space(table, 1);
    return insert_entry(table, key, value, valid_hash(table, key));
}

table_set :: (table: *Table, key: table.Key_Type, value: table.Value_Type) -> *table.Value_Type {
    pointer := table_find_pointer(table, key);
    if pointer {
        pointer.* = value;
        return pointer;
    }

    return table_add(table, key, value);
}

table_find_pointer :: (table: *Table, key: table.Key_Type) -> *table.Value_Type {
    entry := find_entry(table, key);
    if !entry return null;
    return *entry.value;
}

table_contains :: (table: *Table, key: table.Key_Type) -> bool {
    return table_find_pointer(table, key) != null;
}

table_find :: (table: *Table, key: table.Key_Type) -> (success: bool, table.Value_Type) {
    pointer := table_find_pointer(table, key);
    if pointer return true, pointer.*;
    value: table.Value_Type;
    return false, value;
}

table_find_multiple :: (table: *Table, key: table.Key_Type) -> [] table.Value_Type {
    if table.count == 0 return .{};

    count: s64;
    hash := valid_hash(table, key);

    for entry: table.entries if entry.hash == hash && table.compare_function(entry.key, key) count += 1;
    if count == 0 return .{};

    data := cast(*table.Value_Type) talloc(count * size_of(table.Value_Type));

    assert(data != null);
    index: s64;

    for entry: table.entries if entry.hash == hash && table.compare_function(entry.key, key) {
        data[index] = entry.value;
        index += 1;
    }

    return .{ count, data };
}

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

find_or_add :: (table: *Table, key: table.Key_Type) -> (entry: *table.Value_Type, newly_added: bool) {
    pointer := table_find_pointer(table, key);
    if pointer return pointer, false;
    value: table.Value_Type;
    return table_add(table, key, value), true;
}

table_reset_keeping_memory :: (using table: *Table) {
    memset(entries.data, 0, entries.count * size_of(table.Entry));
    count = 0;
    slots_filled = 0;
}

table_reset :: (using table: *Table) {
    if entries.data allocator.proc(.FREE, 0, entries.count * size_of(table.Entry), entries.data, allocator.data);

    entries = .{};
    count = 0;
    slots_filled = 0;
    allocated = 0;
}

deinit :: (table: *Table) {
    table_reset(table);
} @Releases

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
                #insert body;
            } else {
                `it := entry.value;
                #insert body;
            }
        }
    } else {
        for *entry: table.entries {
            if entry.hash < FIRST_VALID_HASH continue;

            `it_index := entry.key;

            #if (flags & .POINTER) != 0 {
                `it := *entry.value;
                #insert body;
            } else {
                `it := entry.value;
                #insert body;
            }
        }
    }
}

Walk_Table :: (code: Code) #expand {
    `hash: u32 = `table.hash_function(`key);
    if `hash < FIRST_VALID_HASH `hash += FIRST_VALID_HASH;

    `index: u32 = `hash & cast,truncate(u32) (`table.allocated - 1);
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
    else index.* = cast,truncate(u32) ((cast(u64) index.* + 1) & cast(u64) (table.allocated - 1));
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

    hash := valid_hash(table, key);
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
