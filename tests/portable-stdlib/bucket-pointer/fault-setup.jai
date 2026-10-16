#import "Basic";
#import "Bucket_Array";

invalid_index :: (index: int, occupied: bool, has_bucket: bool) -> bool {
    bucket: Bucket(s64, 2);
    bucket.occupied[0] = true;
    bucket.occupied[1] = true;
    bucket.count = 2;
    bucket.lowest_maybe_not_occupied = 2;
    storage := (*Bucket(s64, 2)).[*bucket];
    array: Bucket_Array(s64, 2);
    available: [1] *Bucket(s64, 2);

    array.all_buckets = .{ count = ifx has_bucket then 1 else 0, allocated = 1, data = storage.data };
    array.unfull_buckets = .{ count = 0, allocated = 1, data = available.data };
    array.count = ifx has_bucket then 2 else 0;
    if has_bucket && !occupied bucket_array_remove(*array, .{ 0, 1 });

    pointer := *array[index];

    return pointer != null;
}
