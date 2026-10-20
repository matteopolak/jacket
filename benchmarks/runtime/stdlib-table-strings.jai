// Hash_Table with string keys: inserts, lookups that hit and miss, removals.
#import "Basic";
#import "Hash_Table";

COUNT :: 400_000;

main :: () {
    // Keys are written straight into one buffer so building them costs little next to the table work.
    buffer := cast(*u8) alloc(COUNT * 16);
    keys := NewArray(COUNT, string);
    state: u32 = 7;

    for i: 0..COUNT - 1 {
        key := string.{ 0, buffer + i * 16 };
        key.count = 6 + i % 10;

        for j: 0..key.count - 1 {
            state = state * 1664525 + 1013904223;
            key.data[j] = cast(u8) (#char "a" + (state >> 24) % 26);
        }

        keys[i] = key;
    }

    table: Table(string, int);
    for keys table_set(*table, it, it_index);

    hits, sum := 0, 0;

    for round: 0..3 {
        for keys {
            value := table_find_pointer(*table, it);
            if value {
                hits += 1;
                sum += value.*;
            }
        }

        // Misses: the same keys with the first byte changed to one no key uses.
        for keys {
            saved := it.data[0];
            it.data[0] = #char "#";
            if table_find_pointer(*table, it) hits += 1;
            it.data[0] = saved;
        }
    }

    removed := 0;
    for keys if it_index % 2 == 0 && table_remove(*table, it) removed += 1;
    for keys if table_find_pointer(*table, it) hits += 1;

    print("% % % %\n", table.count, hits, sum, removed);
}
