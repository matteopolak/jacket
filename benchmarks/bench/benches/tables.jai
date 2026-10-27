// Hash_Table with integer and string keys. The prebuilt tables and key buffers are made once, on
// the first call, so building them is not part of what the lookups measure.
#import "Basic";
#import "Hash_Table";
#import "Extensions/Testing";

INT_INSERTS :: 2_000_000;
INT_LOOKUPS :: 4_000_000;
STRING_KEYS :: 400_000;

int_table: Table(int, int);
string_table: Table(string, int);
string_keys: [] string;

fill_int_table :: (table: *Table(int, int)) {
    for 0..INT_INSERTS - 1 table_set(table, it * 7919 % 4_000_037, it);
}

// Keys are written straight into one buffer so building them costs little next to the table work.
make_string_keys :: () {
    buffer := cast(*u8) alloc(STRING_KEYS * 16);
    keys := NewArray(STRING_KEYS, string);
    state: u32 = 7;

    for i: 0..STRING_KEYS - 1 {
        key := string.{ 0, buffer + i * 16 };
        key.count = 6 + i % 10;

        for j: 0..key.count - 1 {
            state = state * 1664525 + 1013904223;
            key.data[j] = cast(u8) (#char "a" + (state >> 24) % 26);
        }

        keys[i] = key;
    }

    string_keys = keys;
}

table_int_insert :: (b: *Bench) {
    set_items(b, INT_INSERTS);

    for 0..b.iterations - 1 {
        table: Table(int, int);

        fill_int_table(*table);
        black_box(table.count);
        deinit(*table);
    }
} @bench @samples(20)

table_int_lookup :: (b: *Bench) {
    if !int_table.count fill_int_table(*int_table);

    reset_timer(b);
    set_items(b, INT_LOOKUPS);

    for 0..b.iterations - 1 {
        hits, sum := 0, 0;

        for 0..INT_LOOKUPS - 1 {
            value := table_find_pointer(*int_table, it);
            if value {
                hits += 1;
                sum += value.*;
            }
        }

        black_box(hits);
        black_box(sum);
    }
} @bench @samples(20)

table_string_insert :: (b: *Bench) {
    if !string_keys.count make_string_keys();

    reset_timer(b);
    set_items(b, STRING_KEYS);

    for 0..b.iterations - 1 {
        table: Table(string, int);

        for string_keys table_set(*table, it, it_index);

        black_box(table.count);
        deinit(*table);
    }
} @bench @samples(20)

// Four rounds of lookups that hit, each followed by the same keys with the first byte changed
// to one no key uses (misses).
table_string_lookup :: (b: *Bench) {
    if !string_keys.count make_string_keys();

    if !string_table.count {
        for string_keys table_set(*string_table, it, it_index);
    }

    reset_timer(b);
    set_items(b, STRING_KEYS * 8);

    for 0..b.iterations - 1 {
        hits, sum := 0, 0;

        for round: 0..3 {
            for string_keys {
                value := table_find_pointer(*string_table, it);
                if value {
                    hits += 1;
                    sum += value.*;
                }
            }

            for string_keys {
                saved := it.data[0];
                it.data[0] = #char "#";
                if table_find_pointer(*string_table, it) hits += 1;
                it.data[0] = saved;
            }
        }

        black_box(hits);
        black_box(sum);
    }
} @bench @samples(20)

// Inserts every key, removes every second one, then looks the keys up again.
table_string_remove :: (b: *Bench) {
    if !string_keys.count make_string_keys();

    reset_timer(b);
    set_items(b, STRING_KEYS);

    for 0..b.iterations - 1 {
        table: Table(string, int);
        hits, removed := 0, 0;

        for string_keys table_set(*table, it, it_index);
        for string_keys if it_index % 2 == 0 && table_remove(*table, it) removed += 1;
        for string_keys if table_find_pointer(*table, it) hits += 1;

        black_box(hits);
        black_box(removed);
        deinit(*table);
    }
} @bench @samples(20)
