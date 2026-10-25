#import "Basic";
#import "Extensions/Testing";

a_table_with_every_key_colliding_stays_correct :: () {
    table: H.Table(s64, s64, constant_hash);

    defer H.deinit(*table);

    for i: 0..99 H.table_add(*table, i, i * 2);

    for i: 0..99 {
        ok, value := H.table_find(*table, i);
        expect(ok && value == i * 2);
    }

    for i: 0..49 {
        ok, value := H.table_remove(*table, i);
        expect(ok && value == i * 2);
    }

    for i: 100..149 H.table_add(*table, i, i * 2);

    expect_equal(table.count, 100);
    H.table_add(*table, 90, 999);
    expect_equal(H.table_remove_multiple(*table, 90), 2);
    H.table_reset_keeping_memory(*table);
    expect(table.count == 0 && table.allocated >= 100);
} @test

string_keys_can_be_set_and_found :: () {
    strings: H.Table(string, s64);

    defer H.deinit(*strings);

    H.table_set(*strings, "oak", 19);
    ok, value := H.table_find(*strings, "oak");

    expect(ok && value == 19);
} @test

#scope_file

H :: #import "Hash_Table";

constant_hash :: (key: s64) -> u32 {
    return 5;
}
