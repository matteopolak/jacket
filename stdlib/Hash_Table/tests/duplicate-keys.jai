// Duplicate insertion, first-match setting, lookup and reset retention.
#import "Basic";
#import "Extensions/Testing";

duplicate_keys_are_stored_and_the_first_is_set_and_removed_first :: () {
    table: Table;

    defer Hash_Table.deinit(*table);

    expect(table.count == 0 && table.allocated == 0 && table.slots_filled == 0);
    expect_equal(table.entries.count, 0);
    first := Hash_Table.table_add(*table, 7, 10);
    second := Hash_Table.table_add(*table, 7, 20);

    expect(table.count == 2 && table.allocated >= table.count);
    expect_equal(table.entries.count, table.allocated);
    expect(first != null && second != null);
    expect(Hash_Table.table_set(*table, 7, 30) == first);
    found, value := Hash_Table.table_find(*table, 7);

    expect(found && value == 30);
    removed, old_value := Hash_Table.table_remove(*table, 7);

    expect(removed && old_value == 30);
    found, value = Hash_Table.table_find(*table, 7);
    expect(found && value == 20);
    Hash_Table.table_reset_keeping_memory(*table);
    expect(!Hash_Table.table_contains(*table, 7));
} @test

#scope_file

Hash_Table :: #import "Hash_Table";

Table :: Hash_Table.Table(s64, s64);
