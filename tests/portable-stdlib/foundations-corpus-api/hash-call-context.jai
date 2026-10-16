#import "Basic";
#import "Hash_Table";
H :: #import "Hash";

Pair :: struct {
    fd: s32;
    event: s32;
}

hash_pair :: (x: Pair) -> u32 {
    return H.get_hash(x.fd) ^ H.get_hash(x.event);
}

equal_pair :: (a: Pair, b: Pair) -> bool {
    return a.fd == b.fd && a.event == b.event;
}

main :: () {
    fd: s32 = 42;
    seed: u32 = 5381;
    expected := cast(u32) (H.knuth_hash((cast(u64) fd) ^ cast(u64) seed) >> 32);

    assert(H.get_hash(fd) == expected && H.get_hash(fd, seed) == expected);
    assert(H.get_hash(cast(s64) fd) == expected);
    table: Table(Pair, s64, given_compare_function = equal_pair, given_hash_function = hash_pair);

    table_add(*table, .{ 7, 1 }, 42);
    ok, value := table_find(*table, .{ 7, 1 });

    assert(ok && value == 42);
    deinit(*table);
    print("ok\n");
}
