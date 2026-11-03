#import "Basic";
H :: #import "Hash_Table";

main :: () {
    table: H.Table(int, int);

    defer H.deinit(*table);

    slot, added := H.find_or_add(*table, 7);

    assert(added);
    slot.* = 42;
    replacement := H.table_set(*table, 7, 99);

    assert(replacement.* == 99);
}

release_and_return :: (p: *int) -> *int {
    free(p);
    return p;
}

bad_release_result :: () {
    p := New(int);
    released := release_and_return(p);
    assert(released.* == 0);
}
