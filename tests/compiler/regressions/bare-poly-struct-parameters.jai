// Each parameter written as a bare polymorphic struct (`a: Table`, `b: Table`) takes its own
// instance, so one call may pass different instantiations.
#import "Basic";
#import "Hash_Table";

grow :: (a: Table, b: Table) -> s64 {
    return a.count + b.count;
}

second :: (_a: *Table, b: Table) -> s64 {
    return b.count;
}

main :: () {
    x: Table(string, s64);
    y: Table(string, [2] string);

    table_set(*x, "one", 1);
    table_set(*y, "one", .["a", "b"]);
    table_set(*y, "two", .["c", "d"]);

    assert(grow(x, y) == 3, "%", grow(x, y));
    assert(second(*x, y) == 2);
    assert(second(*y, x) == 1);
    assert(grow(y, x) == 3);
    print("ok\n");
}
