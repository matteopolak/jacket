// Swiss_Table: an opt-in hash map that probes sixteen slots at a time. A jaic extension, not
// official Jai. See docs/extensions/swiss-table.md.
//
//     Swiss :: #import "Extensions/Swiss_Table";
//
//     ages: Swiss.Table(string, int);
//     defer Swiss.deinit(*ages);
//
//     Swiss.table_set(*ages, "ada", 36);
//     found, age := Swiss.table_find(*ages, "ada");
//     for ages print("% is %\n", it_index, it);
//
// The interface follows Hash_Table (table_set, table_find, table_remove, for loops), with two
// differences that matter. A key is stored once: table_add refuses to add a key that is already
// there, and there are no table_find_multiple / table_remove_multiple. And iteration order is
// whatever the layout gives; it differs from Hash_Table's and changes whenever the table grows.

// Compare the sixteen control bytes of a group with the vector unit. Set it to true to use plain
// loops instead (wasm targets always do), which gives the same answers; the tests rely on that.
#module_parameters(SCALAR_GROUPS := false);

GROUP_SIZE :: 16;

Table :: struct(Key_Type: Type, Value_Type: Type, given_hash_function: (Key_Type) -> u32 = null, given_compare_function: (Key_Type, Key_Type) -> bool = null) {
    count: int;
    // Slots in the table: zero before the first insert, then a power of two, at least GROUP_SIZE.
    capacity: int;
    // Empty slots that may still be filled before the table has to be rebuilt.
    growth_left: int;
    allocator: Allocator;
    // One byte per slot, followed in the same allocation by the entries.
    control: *u8;
    entries: *Entry;

    Entry :: struct {
        key: Key_Type;
        value: Value_Type;
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
}

// Space for `expected_count` entries up front, so inserting that many never rebuilds the table.
init :: (table: *Table, expected_count: s64 = 0) {
    if expected_count > 0 table_reserve(table, expected_count);
}

table_reserve :: (table: *Table, expected_count: s64) {
    assert(expected_count >= 0);

    wanted := max(expected_count, table.count);
    capacity: s64 = GROUP_SIZE;

    while max_load(capacity) < wanted capacity *= 2;

    if capacity > table.capacity rebuild(table, capacity);
}

// Adds the key if it is missing. Returns false and the existing value's pointer, untouched, if it
// is already there.
table_add :: (table: *Table, key: table.Key_Type, value: table.Value_Type) -> (success: bool, value: *table.Value_Type) {
    if table.capacity == 0 rebuild(table, GROUP_SIZE);

    start, tag := split_hash(table, key);
    free_slot: s64;
    found := locate(table, key, start, tag, *free_slot);

    if found >= 0 return false, *(table.entries + found).value;

    return true, claim(table, key, value, start, tag, free_slot);
}

// Stores the value under the key, replacing the old one if there is one. Returns a pointer to it.
table_set :: (table: *Table, key: table.Key_Type, value: table.Value_Type) -> *table.Value_Type {
    if table.capacity == 0 rebuild(table, GROUP_SIZE);

    start, tag := split_hash(table, key);
    free_slot: s64;
    found := locate(table, key, start, tag, *free_slot);

    if found >= 0 {
        entry := table.entries + found;
        entry.value = value;
        return *entry.value;
    }

    return claim(table, key, value, start, tag, free_slot);
}

table_find_pointer :: (table: *Table, key: table.Key_Type) -> *table.Value_Type {
    slot := find_slot(table, key);
    if slot < 0 return null;
    return *(table.entries + slot).value;
}

table_contains :: (table: *Table, key: table.Key_Type) -> bool {
    return find_slot(table, key) >= 0;
}

table_find :: (table: *Table, key: table.Key_Type) -> (success: bool, table.Value_Type) {
    slot := find_slot(table, key);
    if slot >= 0 return true, (table.entries + slot).value;

    value: table.Value_Type;
    return false, value;
}

table_remove :: (table: *Table, key: table.Key_Type) -> (success: bool, value: table.Value_Type) {
    slot := find_slot(table, key);
    if slot < 0 {
        value: table.Value_Type;
        return false, value;
    }

    value := (table.entries + slot).value;

    remove_slot(table, slot);

    return true, value;
}

// Like table_find_pointer, but adds a zeroed value when the key is missing.
find_or_add :: (table: *Table, key: table.Key_Type) -> (entry: *table.Value_Type, newly_added: bool) {
    value: table.Value_Type;
    added, pointer := table_add(table, key, value);
    return pointer, added;
}

// Removes every entry and keeps the memory.
table_reset_keeping_memory :: (table: *Table) {
    if table.capacity == 0 return;

    memset(table.control, EMPTY, table.capacity);
    table.count = 0;
    table.growth_left = max_load(table.capacity);
}

// Removes every entry and frees the memory.
table_reset :: (table: *Table) {
    if table.control table.allocator.proc(.FREE, 0, block_size(table), table.control, table.allocator.data);

    table.control = null;
    table.entries = null;
    table.count = 0;
    table.capacity = 0;
    table.growth_left = 0;
}

deinit :: (table: *Table) {
    table_reset(table);
} @Releases

// for it, key: table {...}: `it` is the value (a pointer to it under `for *`), `it_index` the
// key, as with Hash_Table. `remove it` takes the current entry out. Anything else that changes
// the table inside the loop, adding included, invalidates the loop.
for_expansion :: (table: *$T/Table, body: Code, flags: For_Flags) #expand {
    #if (flags & .REVERSE) != 0 {
        slot := table.capacity;
        full: u32;

        while slot > 0 {
            slot -= 1;
            index := slot;

            if (index & (GROUP_SIZE - 1)) == GROUP_SIZE - 1 {
                full = match_full(table.control + (index - (GROUP_SIZE - 1)));
                if full == 0 {
                    slot -= GROUP_SIZE - 1;
                    continue;
                }
            }

            if ((full >> (index & (GROUP_SIZE - 1))) & 1) == 0 continue;

            entry := table.entries + index;

            `it_index := entry.key;

            #if (flags & .POINTER) != 0 {
                `it := *entry.value;
                #insert (remove = {
                    remove_slot(table, index);
                }) body;
            } else {
                `it := entry.value;
                #insert (remove = {
                    remove_slot(table, index);
                }) body;
            }
        }
    } else {
        slot: s64;
        full: u32;

        while slot < table.capacity {
            index := slot;

            slot += 1;

            if (index & (GROUP_SIZE - 1)) == 0 {
                full = match_full(table.control + index);
                if full == 0 {
                    slot += GROUP_SIZE - 1;
                    continue;
                }
            }

            if ((full >> (index & (GROUP_SIZE - 1))) & 1) == 0 continue;

            entry := table.entries + index;

            `it_index := entry.key;

            #if (flags & .POINTER) != 0 {
                `it := *entry.value;
                #insert (remove = {
                    remove_slot(table, index);
                }) body;
            } else {
                `it := entry.value;
                #insert (remove = {
                    remove_slot(table, index);
                }) body;
            }
        }
    }
}

#scope_file
#import "Basic";
Hash :: #import "Hash";
SIMD :: #import "Extensions/SIMD";

// A control byte is EMPTY (never used since the last rebuild), DELETED (held an entry once), or
// the top seven bits of the spread hash of the entry in its slot. The top bit is what tells them apart.
EMPTY: u8 : 0xFF;
DELETED: u8 : 0x80;

#if SCALAR_GROUPS || OS == .WASM {
    // Each of these gives one bit per slot of the group at `group`, lowest slot first.
    match_tag :: inline (group: *u8, tag: u8) -> u32 {
        bits: u32;
        for 0..GROUP_SIZE - 1 if group[it] == tag bits |= cast(u32) 1 << it;
        return bits;
    }

    match_empty :: inline (group: *u8) -> u32 {
        bits: u32;
        for 0..GROUP_SIZE - 1 if group[it] == EMPTY bits |= cast(u32) 1 << it;
        return bits;
    }

    match_free :: inline (group: *u8) -> u32 {
        bits: u32;
        for 0..GROUP_SIZE - 1 if (group[it] & 0x80) != 0 bits |= cast(u32) 1 << it;
        return bits;
    }
} else {
    match_tag :: inline (group: *u8, tag: u8) -> u32 {
        return cast(u32) SIMD.to_bits(SIMD.eq(SIMD.load(SIMD.u8x16, group), tag));
    }

    match_empty :: inline (group: *u8) -> u32 {
        return cast(u32) SIMD.to_bits(SIMD.eq(SIMD.load(SIMD.u8x16, group), EMPTY));
    }

    match_free :: inline (group: *u8) -> u32 {
        return cast(u32) SIMD.to_bits(SIMD.ge(SIMD.load(SIMD.u8x16, group), DELETED));
    }
}

match_full :: inline (group: *u8) -> u32 {
    return ~match_free(group) & 0xFFFF;
}

lowest :: inline (bits: u32) -> s64 {
    return SIMD.trailing_zeros(cast(u64) bits);
}

// Entries a table of this many slots holds before it is rebuilt: seven eighths.
max_load :: inline (capacity: s64) -> s64 {
    return capacity - capacity / 8;
}

block_size :: inline (table: *Table) -> s64 {
    return table.capacity + table.capacity * size_of(table.Entry);
}

// The hash is spread once more here, so that a weak user hash still gives a good first group
// (the low bits of `start`) and a good tag (the top seven bits).
split_hash :: (table: *Table, key: table.Key_Type) -> (start: s64, tag: u8) #no_aoc {
    spread: u64 = cast(u64) table.hash_function(key) * 0x9E3779B97F4A7C15;
    spread ^= spread >> 32;
    return cast(s64) (spread & 0xFFFFFFFF), cast(u8) (spread >> 57);
}

// Probing visits groups at offsets 0, 1, 3, 6, ... from the first one, which reaches every
// group of a power-of-two table.
find_slot :: (table: *Table, key: table.Key_Type) -> s64 {
    if table.capacity == 0 return -1;

    start, tag := split_hash(table, key);
    group_mask := table.capacity / GROUP_SIZE - 1;
    group := start & group_mask;
    stride: s64;

    while true {
        first := group * GROUP_SIZE;
        base := table.control + first;
        hits := match_tag(base, tag);

        while hits != 0 {
            slot := first + lowest(hits);
            hits &= hits - 1;
            if table.compare_function((table.entries + slot).key, key) return slot;
        }

        // A group with an empty slot never had to send a key onward, so the key is not here.
        if match_empty(base) != 0 return -1;

        stride += 1;
        group = (group + stride) & group_mask;
    }

    return -1;
}

// Like find_slot, and also leaves in `free_slot` the first empty or deleted slot on the way, the
// one a new key would take. The table is not empty, and always keeps an empty slot, so the walk
// ends.
locate :: (table: *Table, key: table.Key_Type, start: s64, tag: u8, free_slot: *s64) -> s64 {
    group_mask := table.capacity / GROUP_SIZE - 1;
    group := start & group_mask;
    stride: s64;

    free_slot.* = -1;

    while true {
        first := group * GROUP_SIZE;
        base := table.control + first;
        hits := match_tag(base, tag);

        while hits != 0 {
            slot := first + lowest(hits);
            hits &= hits - 1;
            if table.compare_function((table.entries + slot).key, key) return slot;
        }

        if free_slot.* < 0 {
            free := match_free(base);
            if free != 0 free_slot.* = first + lowest(free);
        }

        if match_empty(base) != 0 return -1;

        stride += 1;
        group = (group + stride) & group_mask;
    }

    return -1;
}

// The first empty or deleted slot on the key's probe path.
first_free_slot :: (table: *Table, start: s64) -> s64 {
    group_mask := table.capacity / GROUP_SIZE - 1;
    group := start & group_mask;
    stride: s64;

    while true {
        free := match_free(table.control + group * GROUP_SIZE);
        if free != 0 return group * GROUP_SIZE + lowest(free);

        stride += 1;
        group = (group + stride) & group_mask;
    }

    return -1;
}

claim :: (table: *Table, key: table.Key_Type, value: table.Value_Type, start: s64, tag: u8, free_slot: s64) -> *table.Value_Type {
    slot := free_slot;

    // Only an empty slot counts against the load limit: a deleted one was already spent. When
    // the limit is reached the table is rebuilt first, which moves where free slots are.
    if table.control[slot] == EMPTY && table.growth_left == 0 {
        if table.count + 1 <= max_load(table.capacity) / 2 rebuild(table, table.capacity);
        else rebuild(table, table.capacity * 2);

        slot = first_free_slot(table, start);
    }

    if table.control[slot] == EMPTY table.growth_left -= 1;

    table.control[slot] = tag;

    entry := table.entries + slot;
    entry.key = key;
    entry.value = value;
    table.count += 1;

    return *entry.value;
}

// A slot whose group still has an empty one can be emptied: no probe ever went past that group.
// Otherwise it becomes deleted so probes carry on through it.
remove_slot :: (table: *Table, slot: s64) {
    base := table.control + slot - (slot & (GROUP_SIZE - 1));

    if match_empty(base) != 0 {
        table.control[slot] = EMPTY;
        table.growth_left += 1;
    } else {
        table.control[slot] = DELETED;
    }

    table.count -= 1;
}

// Moves every entry into fresh storage of `new_capacity` slots, which drops the deleted slots.
rebuild :: (table: *Table, new_capacity: s64) {
    assert(new_capacity >= GROUP_SIZE && new_capacity >= table.count);
    if !table.allocator.proc table.allocator = context.allocator;

    old_control := table.control;
    old_entries := table.entries;
    old_capacity := table.capacity;
    old_size := block_size(table);

    size := new_capacity + new_capacity * size_of(table.Entry);
    memory := table.allocator.proc(.ALLOCATE, size, 0, null, table.allocator.data);

    assert(memory != null, "hash table allocation failed");

    table.control = cast(*u8) memory;
    table.entries = cast(*table.Entry) (table.control + new_capacity);
    table.capacity = new_capacity;
    memset(table.control, EMPTY, new_capacity);

    for slot: 0..old_capacity - 1 {
        if (old_control[slot] & 0x80) != 0 continue;

        old := old_entries + slot;
        start, tag := split_hash(table, old.key);
        target := first_free_slot(table, start);

        table.control[target] = tag;
        (table.entries + target).* = old.*;
    }

    table.growth_left = max_load(new_capacity) - table.count;

    if old_control table.allocator.proc(.FREE, 0, old_size, old_control, table.allocator.data);
}
