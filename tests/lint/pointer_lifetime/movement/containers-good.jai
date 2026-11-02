#import "Basic";
H :: #import "Hash_Table";
B :: #import "Bucket_Array";
P :: #import "Pool";

sink :: (p: *int) {
    print("%", p.*);
}

table_other_key :: () {
    t: H.Table(int, int);
    p := H.table_find_pointer(*t, 7);
    H.table_remove(*t, 8);
    sink(p);
}

bucket_other_slot :: (one: B.Bucket_Locator, two: B.Bucket_Locator) {
    b: B.Bucket_Array(int, 4);
    p := B.bucket_array_find_pointer(*b, one);
    B.bucket_array_remove(*b, two);
    sink(p);
}

other_pool :: () {
    a: P.Pool;
    b: P.Pool;
    p := cast(*int) P.get(*a, 8);

    P.reset(*b);
    sink(p);
}

same_native_slot :: () {
    a: [4] int;
    p := *a[1];
    swap(*a[1], *a[1]);
    sink(p);
}

reverse :: (a: [] int) {} @MovesElements(reverse)
reverse_center :: () {
    a: [5] int;
    p := *a[2];
    reverse(a);
    sink(p);
}

swap_indices :: (a: [] int, first: int, second: int) {
    swap(*a[first], *a[second]);
}

wrapper_same_slot :: () {
    a: [4] int;
    p := *a[1];
    swap_indices(a, 1, 1);
    sink(p);
}
