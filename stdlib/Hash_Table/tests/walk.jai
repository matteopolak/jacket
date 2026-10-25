// Reviewed Hash_Table.Walk_Table caller bindings, tombstones and loop control.
#import "Basic";
#import "Extensions/Testing";

walk_table_binds_the_caller_names_and_skips_tombstones :: () {
    table: H.Table(s64, s64, collide);

    defer H.deinit(*table);

    H.table_add(*table, 4, 40);
    H.table_add(*table, 5, 50);
    H.table_add(*table, 6, 60);
    removed, value := H.table_remove(*table, 4);

    require(removed && value == 40);

    // Walk_Table reads the key being looked for from the caller's `key`.
    key: s64 = 6;
    visits, tombstones, live: s64;

    H.Walk_Table(#code {
        visits += 1;

        if table_while_loop == H.REMOVED_HASH {
            tombstones += 1;
            continue table_while_loop;
        }

        live += 1;
    });
    expect(visits == 3 && tombstones == 1 && live == 2);
    expect(hash >= H.FIRST_VALID_HASH && index < table.entries.count);
    expect_equal(table.entries[index].hash, H.NEVER_OCCUPIED_HASH);
} @test

walk_table_can_break_out_and_run_again :: () {
    table: H.Table(s64, s64, collide);

    defer H.deinit(*table);

    H.table_add(*table, 4, 40);
    H.table_add(*table, 5, 50);
    H.table_add(*table, 6, 60);
    require(H.table_remove(*table, 4));

    key: s64 = 6;
    repeats: s64;

    for 0..1 {
        found := false;

        H.Walk_Table(#code {
            if table_while_loop == hash && table.entries[index].key == key {
                found = true;
                break table_while_loop;
            }
        });
        expect(found && table.entries[index].value == 60);

        repeats += 1;
    }

    expect_equal(repeats, 2);
} @test

#scope_file

H :: #import "Hash_Table";

collide :: (key: s64) -> u32 {
    return 0;
}
