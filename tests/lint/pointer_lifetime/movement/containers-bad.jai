#import "Basic";
H :: #import "Hash_Table";
B :: #import "Bucket_Array";
P :: #import "Pool";
F :: #import "Flat_Pool";

sink :: (p: *int) {
    print("%", p.*);
}

table_removed :: () {
    t: H.Table(int, int);
    p := H.table_find_pointer(*t, 7);
    H.table_remove(*t, 7);
    sink(p);
}

table_reset :: () {
    t: H.Table(int, int);
    p := H.table_find_pointer(*t, 7);
    H.table_reset_keeping_memory(*t);
    sink(p);
}

bucket_removed :: () {
    b: B.Bucket_Array(int, 4);
    locator, p := B.bucket_array_add(*b, 7);
    B.bucket_array_remove(*b, locator);
    sink(p);
}

bucket_found_removed :: (locator: B.Bucket_Locator) {
    b: B.Bucket_Array(int, 4);
    p := B.bucket_array_find_pointer(*b, locator);
    B.bucket_array_remove(*b, locator);
    sink(p);
}

pool_reset :: () {
    pool: P.Pool;
    p := cast(*int) P.get(*pool, 8);
    P.reset(*pool);
    sink(p);
}

flat_pool_reset :: () {
    pool: F.Flat_Pool;
    p := cast(*int) F.get(*pool, 8);
    F.reset(*pool);
    sink(p);
}

native_swap :: () {
    a: [4] int;
    p := *a[1];
    swap(*a[1], *a[3]);
    sink(p);
}

reverse :: (a: [] int) {} @MovesElements(reverse)
fixed_reverse :: () {
    a: [4] int;
    p := *a[0];
    reverse(a);
    sink(p);
}

bounded_offset :: () {
    a: [4] int;
    p := *a[0];
    q := p + 1;

    swap(*a[1], *a[3]);
    sink(q);
    sink(p);
}

swap_indices :: (a: [] int, first: int, second: int) {
    swap(*a[first], *a[second]);
}

wrapper_swap :: () {
    a: [4] int;
    p := *a[1];
    swap_indices(a, 1, 3);
    sink(p);
}
