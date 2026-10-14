// Test-only small map with the reviewed Table storage shape. This is not a
// production Hash_Table replacement or evidence of that module's completeness.
Table :: struct(Key_Type: Type, Value_Type: Type, given_hash_function: (Key_Type) -> u32 = null, given_compare_function: (Key_Type, Key_Type) -> bool = null, LOAD_FACTOR_PERCENT: u32 = 70, REFILL_REMOVED := true) {
    count, allocated, slots_filled: int;
    allocator: Allocator;
    Entry :: struct {
        hash: u32;
        key: Key_Type;
        value: Value_Type;
    }
    entries: [] Entry;
}

table_find_pointer :: (table: *Table, key: table.Key_Type) -> *table.Value_Type {
    for *entry: table.entries if entry.key == key return *entry.value;
    return null;
}

table_set :: (table: *Table, key: table.Key_Type, value: table.Value_Type) -> *table.Value_Type {
    found := table_find_pointer(table, key);
    if found != null {
        found.* = value;
        return found;
    }

    E :: table.Entry;
    old := table.count;
    mem := cast(*E) realloc(table.entries.data, (old + 1) * size_of(E), old * size_of(E));

    table.entries = .{ old + 1, mem };
    table.count += 1;
    table.allocated = table.count;
    table.slots_filled = table.count;
    mem[old] = .{ 2, key, value };

    return *mem[old].value;
}

#scope_file
#import "Basic";
