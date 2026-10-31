// `null.*` is a value of type `void`, which is how a `Table(K, void)` used as a set gets the
// value it ignores. It is not a pointer, so it never matches a pointer parameter.
// rules: ptr.23
#import "Basic";
#import "Hash_Table";

main :: () {
    set: Table(s64, void);

    table_add(*set, 5, null.*);
    table_add(*set, 9, null.*);

    assert(table_find(*set, 5));
    assert(table_find(*set, 9));
    assert(!table_find(*set, 6));
    assert(set.count == 2);
}
