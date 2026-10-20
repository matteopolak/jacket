// `remove` in a `for` over a hash table drops the entry, keeps `count` in step and leaves
// every other entry findable, for forward, reverse and by-pointer walks.
#import "Basic";
#import "Hash_Table";

main :: () {
    table: Table(string, int);

    table_add(*table, "a", 129);
    table_add(*table, "b", 8);
    table_add(*table, "c", 21);
    table_add(*table, "d", 3);

    visited := 0;

    for table {
        visited += 1;
        if it < 50 remove it;
    }

    assert(visited == 4);
    assert(table.count == 1);
    found, value := table_find(*table, "a");

    assert(found && value == 129);
    assert(!table_contains(*table, "b") && !table_contains(*table, "c") && !table_contains(*table, "d"));

    // The key can be added again after it was removed.
    table_add(*table, "b", 50);
    assert(table.count == 2);

    numbers: Table(int, int);
    for 1..50 table_add(*numbers, it, it * it);

    for < numbers {
        if it % 2 == 0 remove it;
    }

    assert(numbers.count == 25);

    for *numbers {
        if it.* > 1000 remove it;
    }

    assert(numbers.count == 16);

    for 1..50 {
        ok, square := table_find(*numbers, it);
        assert(ok == (it % 2 == 1 && it <= 31));
        if ok assert(square == it * it);
    }

    total := 0;
    for numbers total += 1;

    assert(total == numbers.count);

    deinit(*table);
    deinit(*numbers);

    print("ok\n");
}
