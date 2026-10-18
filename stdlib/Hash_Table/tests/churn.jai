// Repeated remove/add cycles keep the table correct and its storage bounded, with and without
// reusing removed slots, and table_find_multiple finds duplicates among tombstones.
#import "Basic";
H :: #import "Hash_Table";

Keep :: H.Table(s64, s64, REFILL_REMOVED = false);
Refill :: H.Table(s64, s64);

collide :: (key: s64) -> u32 {
    return cast(u32) (key % 4);
}

churn :: (table: *$T, rounds: s64) {
    before := table.count;

    for round: 0..rounds - 1 {
        for i: 0..9 H.table_add(table, round * 10 + i, i);

        for i: 0..9 {
            ok, value := H.table_remove(table, round * 10 + i);
            assert(ok && value == i);
        }
    }

    assert(table.count == before);
}

main :: () {
    kept: Keep;
    refilled: Refill;

    defer H.deinit(*kept);
    defer H.deinit(*refilled);

    for i: 0..19 {
        H.table_add(*kept, 1000000 + i, i);
        H.table_add(*refilled, 1000000 + i, i);
    }

    churn(*kept, 2000);
    churn(*refilled, 2000);

    assert(kept.count == 20 && refilled.count == 20);
    assert(kept.allocated <= 256);
    assert(refilled.allocated <= 256);

    for i: 0..19 {
        ok, value := H.table_find(*kept, 1000000 + i);
        assert(ok && value == i);
        ok, value = H.table_find(*refilled, 1000000 + i);
        assert(ok && value == i);
    }

    assert(!H.table_contains(*kept, 5));

    // Duplicates mixed with removed entries and colliding keys.
    dupes: H.Table(s64, s64, collide);

    defer H.deinit(*dupes);

    for i: 0..39 H.table_add(*dupes, i % 8, i);
    for i: 0..7 assert(H.table_find_multiple(*dupes, i).count == 5);

    H.table_remove(*dupes, 3);
    assert(H.table_find_multiple(*dupes, 3).count == 4);
    assert(H.table_remove_multiple(*dupes, 3) == 4);
    assert(H.table_find_multiple(*dupes, 3).count == 0);
    assert(H.table_find_multiple(*dupes, 7).count == 5);
    assert(H.table_find_multiple(*dupes, 99).count == 0);

    total: s64;
    for value: H.table_find_multiple(*dupes, 5) total += value;

    assert(total == 5 + 13 + 21 + 29 + 37);
    print("ok\n");
}
