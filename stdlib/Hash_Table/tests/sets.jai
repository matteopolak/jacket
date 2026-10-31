// Tables whose value type is `void` work as sets.
#import "Basic";
#import "Extensions/Testing";

finding_keys :: () {
    set: H.Table(s64, void);

    defer H.deinit(*set);

    for 0..99 H.find_or_add(*set, it * 3);

    expect_equal(set.count, 100);
    expect(H.table_find(*set, 33));
    expect(!H.table_find(*set, 34));
    expect(H.table_contains(*set, 297));
    expect(!H.table_contains(*set, 298));
    expect(H.table_find_pointer(*set, 3) != null);
    expect(H.table_find_pointer(*set, 4) == null);
} @test

find_returns_a_void_second_value :: () {
    set: H.Table(s64, void);

    defer H.deinit(*set);

    H.find_or_add(*set, 5);

    found, value := H.table_find(*set, 5);

    expect(found);
    expect(type_of(value) == void);

    found, _ = H.table_find(*set, 6);
    expect(!found);

    expect(H.table_find(*set, 5));
} @test

removing_keys :: () {
    set: H.Table(s64, void);

    defer H.deinit(*set);

    for 0..9 H.find_or_add(*set, it);

    expect(H.table_remove(*set, 4));
    expect(!H.table_remove(*set, 4));
    expect(!H.table_contains(*set, 4));
    expect_equal(set.count, 9);

    removed, value := H.table_remove(*set, 5);

    expect(removed);
    expect(type_of(value) == void);
} @test

find_or_add_reports_new_keys :: () {
    set: H.Table(string, void);

    defer H.deinit(*set);

    entry, added := H.find_or_add(*set, "a");

    expect(added && entry != null);

    entry, added = H.find_or_add(*set, "a");
    expect(!added && entry != null);

    expect_equal(set.count, 1);
    expect(!H.table_find(*set, "b"));
} @test

iterating_a_set_visits_each_key_once :: () {
    set: H.Table(s64, void);

    defer H.deinit(*set);

    for 0..19 H.find_or_add(*set, it);

    sum := 0;
    visited := 0;

    for _, key: set {
        sum += key;
        visited += 1;
    }

    expect_equal(visited, 20);
    expect_equal(sum, 190);

    for _, key: set if key % 2 == 0 remove _;

    expect_equal(set.count, 10);
    expect(!H.table_contains(*set, 4));
    expect(H.table_contains(*set, 5));
} @test

resetting_a_set_empties_it :: () {
    set: H.Table(s64, void);

    defer H.deinit(*set);

    for 0..9 H.find_or_add(*set, it);

    H.table_reset(*set);
    expect_equal(set.count, 0);
    expect(!H.table_contains(*set, 3));

    H.find_or_add(*set, 3);
    expect(H.table_contains(*set, 3));
} @test

#scope_file

H :: #import "Hash_Table";
