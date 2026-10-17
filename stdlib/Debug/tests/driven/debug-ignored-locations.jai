// Public assertion storage uses canonical Hash_Table identity and content keys.
#import "Basic";
D :: #import "Debug";
T :: #import "Hash_Table";

main :: () {
    defer T.table_reset(*D.ignored_asserts);
    first := Source_Code_Location.{ "original-assert.jai", 12, 7 };
    same := first;
    distinct := first;

    distinct.character_number += 1;
    assert(!T.table_contains(*D.ignored_asserts, first));
    T.table_set(*D.ignored_asserts, first, true);
    found, ignored := T.table_find(*D.ignored_asserts, same);

    assert(found && ignored);
    assert(!T.table_contains(*D.ignored_asserts, distinct));
    T.table_set(*D.ignored_asserts, same, false);
    found, ignored = T.table_find(*D.ignored_asserts, first);
    assert(found && !ignored && D.ignored_asserts.count == 1);
    print("ok\n");
}
