// Prepared actual Hash_Table fixture. It must execute through the real module.
// init() rounds an explicit size up to a power of two without raising it to the default
// minimum; the default size applies only when no size is given.
#import "Hash_Table";

allocated_after_init :: (slots: int) -> int {
    table: Table(int, int);
    init(*table, slots);
    defer deinit(*table);
    return table.allocated;
}

main :: () -> s64 {
    if allocated_after_init(1) != 1 return 1;
    if allocated_after_init(5) != 8 return 2;
    if allocated_after_init(31) != 32 return 3;
    if allocated_after_init(33) != 64 return 4;
    if allocated_after_init(100) != 128 return 5;
    if allocated_after_init(0) != 32 return 6;

    table: Table(int, int);
    defer deinit(*table);
    table_add(*table, 1, 10);
    if table.allocated != 32 return 7;

    // A small table still grows as entries arrive.
    small: Table(int, int);
    defer deinit(*small);
    init(*small, 1);
    for 1..20 table_add(*small, it, it * 2);
    if small.count != 20 return 8;
    for 1..20 {
        found, value := table_find(*small, it);
        if !found || value != it * 2 return 9;
    }
    return 0;
}
