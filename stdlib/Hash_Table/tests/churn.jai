// Repeated remove/add cycles keep the table correct and its storage bounded, with and without
// reusing removed slots, and table_find_multiple finds duplicates among tombstones.
#import "Basic";
#import "Extensions/Testing";

removing_and_adding_keeps_the_table_correct_and_bounded :: () {
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

    expect(kept.count == 20 && refilled.count == 20);
    expect(kept.allocated <= 256);
    expect(refilled.allocated <= 256);

    for i: 0..19 {
        ok, value := H.table_find(*kept, 1000000 + i);
        expect(ok && value == i);
        ok, value = H.table_find(*refilled, 1000000 + i);
        expect(ok && value == i);
    }

    expect(!H.table_contains(*kept, 5));
} @test

find_multiple_sees_duplicates_among_removed_entries :: () {
    // Duplicates mixed with removed entries and colliding keys.
    dupes: H.Table(s64, s64, collide);

    defer H.deinit(*dupes);

    for i: 0..39 H.table_add(*dupes, i % 8, i);
    for i: 0..7 expect_equal(H.table_find_multiple(*dupes, i).count, 5);

    H.table_remove(*dupes, 3);
    expect_equal(H.table_find_multiple(*dupes, 3).count, 4);
    expect_equal(H.table_remove_multiple(*dupes, 3), 4);
    expect_equal(H.table_find_multiple(*dupes, 3).count, 0);
    expect_equal(H.table_find_multiple(*dupes, 7).count, 5);
    expect_equal(H.table_find_multiple(*dupes, 99).count, 0);

    total: s64;
    for value: H.table_find_multiple(*dupes, 5) total += value;

    expect_equal(total, 5 + 13 + 21 + 29 + 37);
} @test

#scope_file

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
            expect(ok && value == i);
        }
    }

    expect_equal(table.count, before);
}
