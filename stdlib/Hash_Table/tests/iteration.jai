// Reviewed Hash_Table.for_expansion: values, keys, POINTER and REVERSE flags.
// Entry order is deliberately observed, never prescribed by the test.
#import "Basic";
#import "Extensions/Testing";

for_expansion_visits_values_keys_pointers_and_reverse :: () {
    table: H.Table(s64, s64, hash_key);

    defer H.deinit(*table);

    for i: 0..39 H.table_add(*table, i, i * 3);
    for i: 0..9 H.table_remove(*table, i * 2);

    keys: [30] s64;
    visited: [40] bool;
    count: s64;

    for value, key: table {
        expect(key >= 0 && key < 40 && !visited[key]);
        expect_equal(value, key * 3);
        visited[key] = true;
        keys[count] = key;
        count += 1;
    }

    expect(count == table.count && count == 30);

    for < value, key: table {
        count -= 1;
        expect(key == keys[count] && value == key * 3);
    }

    expect_equal(count, 0);

    for *value, key: table {
        expect(value == H.table_find_pointer(*table, key));
        value.* += 7;
    }

    for key: keys expect_equal(H.table_find_pointer(*table, key).*, key * 3 + 7);

    H.table_reset_keeping_memory(*table);
    for value: table expect(false, "reset table must not yield a value");
} @test

#scope_file

H :: #import "Hash_Table";

hash_key :: (key: s64) -> u32 {
    return 2;
}
