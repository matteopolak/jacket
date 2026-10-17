#import "Basic";
#import "Bucket_Array";

verify :: () -> bool {
    first: Bucket(s64, 3);
    second: Bucket(s64, 3);

    first.bucket_index = 0;
    second.bucket_index = 1;
    first.count = 3;
    second.count = 3;
    first.lowest_maybe_not_occupied = 3;
    second.lowest_maybe_not_occupied = 3;

    for i: 0..2 {
        first.occupied[i] = true;
        second.occupied[i] = true;
        first.data[i] = i + 1;
        second.data[i] = i + 4;
    }

    storage := (*Bucket(s64, 3)).[*first, *second];
    unfull: [2] *Bucket(s64, 3);
    array: Bucket_Array(s64, 3);
    array.all_buckets = .{ count = 2, allocated = 2, data = storage.data };
    array.unfull_buckets = .{ count = 0, allocated = 2, data = unfull.data };
    array.count = 6;
    stable := *array[2];

    bucket_array_remove(*array, .{ 0, 1 });
    if array.count != 5 || first.count != 2 || first.occupied[1] return false;
    if array[2] != 3 || array[3] != 4 || array[5] != 6 return false;
    if *array[2] != stable || *array[5] != second.data.data + 2 return false;

    visited: int;

    for item, index: array {
        if index != visited return false;
        if index == 1 && item != 3 return false;
        visited += 1;
    }

    if visited != 5 return false;

    locator, replacement := bucket_array_add(*array, cast(s64) 71);
    if locator.bucket_index != 0 || locator.slot_index != 1 return false;
    if replacement != *array[1] || array[1] != 71 return false;
    if array.count != 6 || array.all_buckets.count != 2 || *array[2] != stable return false;

    return true;
}

#assert verify();

main :: () {
    assert(verify());
    print("ok\n");
}
