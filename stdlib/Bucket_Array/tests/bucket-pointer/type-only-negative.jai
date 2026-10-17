#import "Type_Only";

verify :: () -> bool {
    bucket: Bucket(s64, 1);
    bucket.occupied[0] = true;
    bucket.data[0] = 17;
    storage := (*Bucket(s64, 1)).[*bucket];
    array: Bucket_Array(s64, 1);
    array.all_buckets = .{ count = 1, allocated = 1, data = storage.data };
    array.count = 1;
    pointer := *array[0];

    array[0] = 31;

    return pointer == bucket.data.data && pointer.* == 31;
}

#assert verify();
main :: () {}
