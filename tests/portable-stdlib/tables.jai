#import "Basic";
H :: #import "Hash_Table";

collision_hash :: (key: s64) -> u32 {
    return 5;
}

main :: () {
    table: H.Table(s64, s64, collision_hash);
    for i: 0..99 H.table_add(*table, i, i * 2);

    for i: 0..99 {
        ok, value := H.table_find(*table, i);
        assert(ok && value == i * 2);
    }

    for i: 0..49 {
        ok, value := H.table_remove(*table, i);
        assert(ok && value == i * 2);
    }

    for i: 100..149 H.table_add(*table, i, i * 2);

    assert(table.count == 100);
    H.table_add(*table, 90, 999);
    assert(H.table_remove_multiple(*table, 90) == 2);
    H.table_reset_keeping_memory(*table);
    assert(table.count == 0 && table.allocated >= 100);
    H.deinit(*table);
    strings: H.Table(string, s64);

    H.table_set(*strings, "oak", 19);
    ok, value := H.table_find(*strings, "oak");

    assert(ok && value == 19);
    H.deinit(*strings);
    print("ok\n");
}
