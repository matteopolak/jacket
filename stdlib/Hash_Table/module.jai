// `Table(Key, Value)` maps keys to values.
//
// ```jai
// table: Table(string, int);
// table_add(*table, "one", 1);
// found, value := table_find(*table, "one");
// assert(found && value == 1);
// ```

// Canonical declarations are instantiated in this public module scope.
#load "../internal/collections/hash-table.jai";
