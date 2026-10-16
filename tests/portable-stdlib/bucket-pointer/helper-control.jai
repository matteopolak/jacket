#import "Basic";
B :: #import "Bucket_Array";

verify :: () -> bool {
    bucket: B.Bucket(s64, 2);
    bucket.occupied[1] = true;
    bucket.data[1] = 17;
    bucket.count = 1;
    storage := (*B.Bucket(s64, 2)).[*bucket];
    array: B.Bucket_Array(s64, 2);
    array.all_buckets = .{ count = 1, allocated = 1, data = storage.data };
    array.count = 1;
    pointer := B.bucket_array_find_pointer(*array, .{ 0, 1 });
    pointer.* = 31;

    return pointer == bucket.data.data + 1 && bucket.data[1] == 31;
}

#assert verify();

main :: () {
    assert(verify());
    print("ok\n");
}
