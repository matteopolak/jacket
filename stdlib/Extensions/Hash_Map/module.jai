// An opt-in hash map that probes sixteen slots at a time. A jaic extension, not
// official Jai. See docs/extensions/hash-map.md.
//
//     Map :: #import "Extensions/Hash_Map";
//
//     ages: Map.Map(string, int);
//     defer Map.deinit(*ages);
//
//     Map.insert(*ages, "ada", 36);
//     age := Map.get(*ages, "ada");
//     for value, name: ages print("% is %\n", name, value);
//
// A key is stored once. Iteration order is whatever the layout gives, and changes whenever the
// map grows.

// Compare the sixteen control bytes of a group with the vector unit. Set it to true to use plain
// loops instead (wasm targets always do), which gives the same answers; the tests rely on that.
#module_parameters(SCALAR_GROUPS := false);

GROUP_SIZE :: 16;

Map :: struct(Key: Type, Value: Type, hash: (Key) -> u32 = null, equal: (Key, Key) -> bool = null) {
    // Entries in the map. Read only.
    count: int;
    // Slots in the map: zero before the first insert, then a power of two, at least GROUP_SIZE.
    // Read only.
    capacity: int;

    // The fields below are internal: do not read or write them.

    // Empty slots that may still be filled before the map has to be rebuilt.
    growth_left: int;
    allocator: Allocator;
    // One byte per slot, followed in the same allocation by the entries.
    control: *u8;
    entries: *Entry;

    Entry :: struct {
        key: Key;
        value: Value;
    }

    #if hash {
        hash_function :: hash;
    } else #if Key == string {
        hash_function :: (key: string) -> u32 {
            return hash_text(key);
        }
    } else {
        hash_function :: (key: Key) -> u32 {
            return Hash.get_hash(key);
        }
    }
    #if equal {
        equal_function :: equal;
    } else {
        equal_function :: (a: Key, b: Key) -> bool {
            return a == b;
        }
    }
}

// Stores the value under the key. Returns the value it replaced (zero if there was none) and
// whether there was one.
insert :: (map: *Map, key: map.Key, value: map.Value) -> (previous: map.Value, replaced: bool) {
    if map.capacity == 0 rebuild(map, GROUP_SIZE);

    start, tag := split_hash(map, key);
    free_slot: s64;
    found := locate(map, key, start, tag, *free_slot);

    previous: map.Value;

    if found >= 0 {
        item := map.entries + found;
        previous = item.value;
        item.value = value;
        return previous, true;
    }

    claim(map, key, value, start, tag, free_slot);

    return previous, false;
}

// A pointer to the key's value, or null if it is missing. The pointer stays valid until the map
// is next rebuilt by an insert or entry.
find :: (map: *Map, key: map.Key) -> *map.Value {
    slot := find_slot(map, key);
    if slot < 0 return null;
    return *(map.entries + slot).value;
}

// A copy of the key's value, or `default` if it is missing.
get :: (map: *Map, key: map.Key, default: map.Value = .{}) -> map.Value {
    slot := find_slot(map, key);
    if slot < 0 return default;
    return (map.entries + slot).value;
}

// A pointer to the key's value, adding a zero-initialized one if the key is missing. `added`
// says whether it was added.
entry :: (map: *Map, key: map.Key) -> (value: *map.Value, added: bool) {
    if map.capacity == 0 rebuild(map, GROUP_SIZE);

    start, tag := split_hash(map, key);
    free_slot: s64;
    found := locate(map, key, start, tag, *free_slot);

    if found >= 0 return *(map.entries + found).value, false;

    zero: map.Value;

    return claim(map, key, zero, start, tag, free_slot), true;
}

// Removes the key. Returns its value (zero if it was missing) and whether it was there.
delete :: (map: *Map, key: map.Key) -> (value: map.Value, removed: bool) {
    value: map.Value;

    slot := find_slot(map, key);

    if slot < 0 return value, false;

    value = (map.entries + slot).value;
    remove_slot(map, slot);

    return value, true;
}

contains :: (map: *Map, key: map.Key) -> bool {
    return find_slot(map, key) >= 0;
}

// Room for `expected_count` entries in total, so inserting up to that many never rebuilds the map.
reserve :: (map: *Map, expected_count: s64) {
    assert(expected_count >= 0);

    wanted := max(expected_count, map.count);
    capacity: s64 = GROUP_SIZE;

    while max_load(capacity) < wanted capacity *= 2;

    if capacity > map.capacity rebuild(map, capacity);
}

// Removes every entry and keeps the memory.
clear :: (map: *Map) {
    if map.capacity == 0 return;

    memset(map.control, EMPTY, map.capacity);
    map.count = 0;
    map.growth_left = max_load(map.capacity);
}

// Removes every entry and frees the memory.
deinit :: (map: *Map) {
    if map.control map.allocator.proc(.FREE, 0, block_size(map), map.control, map.allocator.data);

    map.control = null;
    map.entries = null;
    map.count = 0;
    map.capacity = 0;
    map.growth_left = 0;
} @FreesArgument

// for value, key: map {...}: `it` is the value (a pointer to it under `for *`), `it_index` the
// key. `remove it` takes the current entry out. Anything else that changes the map inside the
// loop, adding included, invalidates the loop.
for_expansion :: (map: *$T/Map, body: Code, flags: For_Flags) #expand {
    #if (flags & .REVERSE) != 0 {
        next_group := map.capacity / GROUP_SIZE;
        base: s64;
        bits: u32;

        while true {
            if bits == 0 {
                while next_group > 0 {
                    next_group -= 1;
                    base = next_group * GROUP_SIZE;
                    bits = match_full(map.control + base);
                    if bits != 0 break;
                }

                if bits == 0 break;
            }

            index := base + highest(bits);

            bits &= ~(cast(u32) 1 << (index - base));

            item := map.entries + index;

            `it_index := item.key;

            #if (flags & .POINTER) != 0 {
                `it := *item.value;
                #insert (remove = {
                    remove_slot(map, index);
                }) body;
            } else {
                `it := item.value;
                #insert (remove = {
                    remove_slot(map, index);
                }) body;
            }
        }
    } else {
        next_group: s64;
        groups := map.capacity / GROUP_SIZE;
        base: s64;
        bits: u32;

        while true {
            if bits == 0 {
                while next_group < groups {
                    base = next_group * GROUP_SIZE;
                    next_group += 1;
                    bits = match_full(map.control + base);
                    if bits != 0 break;
                }

                if bits == 0 break;
            }

            index := base + lowest(bits);

            bits &= bits - 1;

            item := map.entries + index;

            `it_index := item.key;

            #if (flags & .POINTER) != 0 {
                `it := *item.value;
                #insert (remove = {
                    remove_slot(map, index);
                }) body;
            } else {
                `it := item.value;
                #insert (remove = {
                    remove_slot(map, index);
                }) body;
            }
        }
    }
}

#scope_file
#import "Basic";
Hash :: #import "Hash";
SIMD :: #import "Extensions/SIMD";

// Groups up to which a rebuild keeps its fill counts on the stack.
REBUILD_STACK_GROUPS :: 512;

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

highest :: inline (bits: u32) -> s64 {
    return 63 - SIMD.leading_zeros(cast(u64) bits);
}

// Entries a map of this many slots holds before it is rebuilt: seven eighths.
max_load :: inline (capacity: s64) -> s64 {
    return capacity - capacity / 8;
}

block_size :: inline (map: *Map) -> s64 {
    return map.capacity + map.capacity * size_of(map.Entry);
}

// The default hash of a string key: eight bytes per step, where `Hash.get_hash` takes one. The
// length is part of the hash, and the last bytes are read as one overlapping word, so no byte
// is visited twice by a loop.
hash_text :: (text: string) -> u32 #no_aoc {
    data := text.data;
    count := text.count;
    state: u64 = cast(u64) count * 0x9E3779B97F4A7C15 + 0x2545F4914F6CDD1D;

    if count >= 8 {
        at := 0;

        while at + 8 < count {
            state = (state ^ (cast(*u64) (data + at)).*) * 0xFF51AFD7ED558CCD;
            state ^= state >> 32;
            at += 8;
        }

        state = (state ^ (cast(*u64) (data + count - 8)).*) * 0xFF51AFD7ED558CCD;
    } else if count >= 4 {
        low: u64 = (cast(*u32) data).*;
        high: u64 = (cast(*u32) (data + count - 4)).*;
        word := low | (high << 32);

        state = (state ^ word) * 0xFF51AFD7ED558CCD;
    } else if count > 0 {
        first: u64 = data[0];
        middle: u64 = data[count / 2];
        last: u64 = data[count - 1];
        word := first | (middle << 8) | (last << 16);

        state = (state ^ word) * 0xFF51AFD7ED558CCD;
    }

    state ^= state >> 32;
    state *= 0xC4CEB9FE1A85EC53;

    return cast(u32) (state >> 32);
}

// The hash is spread once more here, so that a weak user hash still gives a good first group
// (the low bits of `start`) and a good tag (the top seven bits).
split_hash :: (map: *Map, key: map.Key) -> (start: s64, tag: u8) #no_aoc {
    spread: u64 = cast(u64) map.hash_function(key) * 0x9E3779B97F4A7C15;
    spread ^= spread >> 32;
    return cast(s64) (spread & 0xFFFFFFFF), cast(u8) (spread >> 57);
}

// Probing visits groups at offsets 0, 1, 3, 6, ... from the first one, which reaches every
// group of a power-of-two map.
find_slot :: (map: *Map, key: map.Key) -> s64 {
    if map.capacity == 0 return -1;

    start, tag := split_hash(map, key);
    group_mask := map.capacity / GROUP_SIZE - 1;
    group := start & group_mask;
    stride: s64;

    while true {
        first := group * GROUP_SIZE;
        base := map.control + first;
        hits := match_tag(base, tag);

        while hits != 0 {
            slot := first + lowest(hits);
            hits &= hits - 1;
            if map.equal_function((map.entries + slot).key, key) return slot;
        }

        // A group with an empty slot never had to send a key onward, so the key is not here.
        if match_empty(base) != 0 return -1;

        stride += 1;
        group = (group + stride) & group_mask;
    }

    return -1;
}

// Like find_slot, and also leaves in `free_slot` the first empty or deleted slot on the way, the
// one a new key would take. The map is not empty, and always keeps an empty slot, so the walk
// ends.
locate :: (map: *Map, key: map.Key, start: s64, tag: u8, free_slot: *s64) -> s64 {
    group_mask := map.capacity / GROUP_SIZE - 1;
    group := start & group_mask;
    stride: s64;

    free_slot.* = -1;

    while true {
        first := group * GROUP_SIZE;
        base := map.control + first;
        hits := match_tag(base, tag);

        while hits != 0 {
            slot := first + lowest(hits);
            hits &= hits - 1;
            if map.equal_function((map.entries + slot).key, key) return slot;
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
first_free_slot :: (map: *Map, start: s64) -> s64 {
    group_mask := map.capacity / GROUP_SIZE - 1;
    group := start & group_mask;
    stride: s64;

    while true {
        free := match_free(map.control + group * GROUP_SIZE);
        if free != 0 return group * GROUP_SIZE + lowest(free);

        stride += 1;
        group = (group + stride) & group_mask;
    }

    return -1;
}

claim :: (map: *Map, key: map.Key, value: map.Value, start: s64, tag: u8, free_slot: s64) -> *map.Value {
    slot := free_slot;

    // Only an empty slot counts against the load limit: a deleted one was already spent. When
    // the limit is reached the map is rebuilt first, which moves where free slots are.
    if map.control[slot] == EMPTY && map.growth_left == 0 {
        if map.count + 1 <= max_load(map.capacity) / 2 rebuild(map, map.capacity);
        else rebuild(map, map.capacity * 2);

        slot = first_free_slot(map, start);
    }

    if map.control[slot] == EMPTY map.growth_left -= 1;

    map.control[slot] = tag;

    item := map.entries + slot;
    item.key = key;
    item.value = value;
    map.count += 1;

    return *item.value;
}

// A slot whose group still has an empty one can be emptied: no probe ever went past that group.
// Otherwise it becomes deleted so probes carry on through it.
remove_slot :: (map: *Map, slot: s64) {
    base := map.control + slot - (slot & (GROUP_SIZE - 1));

    if match_empty(base) != 0 {
        map.control[slot] = EMPTY;
        map.growth_left += 1;
    } else {
        map.control[slot] = DELETED;
    }

    map.count -= 1;
}

// Moves every entry into fresh storage of `new_capacity` slots, which drops the deleted slots.
rebuild :: (map: *Map, new_capacity: s64) {
    assert(new_capacity >= GROUP_SIZE && new_capacity >= map.count);
    if !map.allocator.proc map.allocator = context.allocator;

    old_control := map.control;
    old_entries := map.entries;
    old_capacity := map.capacity;
    old_size := block_size(map);

    size := new_capacity + new_capacity * size_of(map.Entry);
    memory := map.allocator.proc(.ALLOCATE, size, 0, null, map.allocator.data);

    assert(memory != null, "hash map allocation failed");

    map.control = cast(*u8) memory;
    map.entries = cast(*map.Entry) (map.control + new_capacity);
    map.capacity = new_capacity;
    memset(map.control, EMPTY, new_capacity);

    // The new block starts empty and only fills, so a group's next free slot is its fill count.
    // Reading the control bytes back to find it would load a group right after a byte store to
    // it, which the next entry of the same old group does (it lands in the same new group), and
    // such a load has to wait for the store.
    group_count := new_capacity / GROUP_SIZE;
    group_mask := group_count - 1;
    fill_stack: [REBUILD_STACK_GROUPS] u8 = ---;
    fills := fill_stack.data;

    if group_count > REBUILD_STACK_GROUPS fills = map.allocator.proc(.ALLOCATE, group_count, 0, null, map.allocator.data);

    memset(fills, 0, group_count);

    for slot: 0..old_capacity - 1 {
        if (old_control[slot] & 0x80) != 0 continue;

        old := old_entries + slot;
        start, tag := split_hash(map, old.key);
        group := start & group_mask;
        stride: s64;

        while fills[group] == GROUP_SIZE {
            stride += 1;
            group = (group + stride) & group_mask;
        }

        target := group * GROUP_SIZE + fills[group];

        fills[group] += 1;

        map.control[target] = tag;
        (map.entries + target).* = old.*;
    }

    if group_count > REBUILD_STACK_GROUPS map.allocator.proc(.FREE, 0, group_count, fills, map.allocator.data);

    map.growth_left = max_load(new_capacity) - map.count;

    if old_control map.allocator.proc(.FREE, 0, old_size, old_control, map.allocator.data);
}
