// Bucket_Array public shape: allocator field, locators, find_and_occupy_empty_slot, iteration.
#import "Basic";
#import "Bucket_Array";

Cmd :: struct {
    v: int;
}
Holder :: struct {
    pools: Bucket_Array(Cmd, 4);
}

main :: () -> s64 {
    h: Holder;
    h.pools.allocator = context.allocator;
    locs: [10] Bucket_Locator;
    for 0..9 {
        l, p := bucket_array_add(*h.pools, .{ it });
        locs[it] = l;
    }
    if h.pools.count != 10 || h.pools.all_buckets.count != 3 return 1;
    bucket_array_remove(*h.pools, locs[5]);
    sum := 0;
    for h.pools sum += it.v;
    if sum != 40 return 2;
    p, l := find_and_occupy_empty_slot(*h.pools);
    if l.bucket_index != locs[5].bucket_index || l.slot_index != locs[5].slot_index || p.v != 0 return 3;
    if bucket_array_find(*h.pools, locs[9]).v != 9 return 4;
    bucket_array_reset(*h.pools);
    if h.pools.count != 0 return 5;
    print("ok\n");
    return 0;
}
