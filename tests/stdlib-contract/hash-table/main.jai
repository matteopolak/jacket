// Duplicate insertion, first-match setting, lookup and reset retention.
#import "Basic";
Hash_Table :: #import "Hash_Table";

main :: () {
    Table :: Hash_Table.Table(s64, s64);
    table: Table;
    first := Hash_Table.table_add(*table, 7, 10);
    second := Hash_Table.table_add(*table, 7, 20);
    assert(table.count == 2 && table.allocated >= table.count);
    assert(table.entries.count == table.allocated);
    assert(first != null && second != null);
    assert(Hash_Table.table_set(*table, 7, 30) == first);
    found, value := Hash_Table.table_find(*table, 7);
    assert(found && value == 30);
    removed, old_value := Hash_Table.table_remove(*table, 7);
    assert(removed && old_value == 30);
    found, value = Hash_Table.table_find(*table, 7);
    assert(found && value == 20);
    Hash_Table.table_reset_keeping_memory(*table);
    assert(!Hash_Table.table_contains(*table, 7));
    Hash_Table.deinit(*table);
    print("ok\n");
}
