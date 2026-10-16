// Reviewed Hash_Table.Walk_Table caller bindings, tombstones and loop control.
#import "Basic";
H :: #import "Hash_Table";

collide :: (key: s64) -> u32 {
    return 0;
}

main :: () {
    table: H.Table(s64, s64, collide);

    H.table_add(*table, 4, 40);
    H.table_add(*table, 5, 50);
    H.table_add(*table, 6, 60);
    removed, value := H.table_remove(*table, 4);
    if !removed || value != 40 exit(2);

    key: s64 = 6;

    {
        visits, tombstones, live: s64;

        H.Walk_Table(#code {
            visits += 1;

            if table_while_loop == H.REMOVED_HASH {
                tombstones += 1;
                continue table_while_loop;
            }

            live += 1;
        });
        if visits != 3 || tombstones != 1 || live != 2 exit(3);
        if hash < H.FIRST_VALID_HASH || index >= table.entries.count exit(4);
        if table.entries[index].hash != H.NEVER_OCCUPIED_HASH exit(5);
    }

    repeats: s64;

    for 0..1 {
        found := false;

        H.Walk_Table(#code {
            if table_while_loop == hash && table.entries[index].key == key {
                found = true;
                break table_while_loop;
            }
        });
        if !found || table.entries[index].value != 60 exit(6);

        repeats += 1;
    }

    if repeats != 2 exit(7);

    H.deinit(*table);
    write_string("ok\n");
}
