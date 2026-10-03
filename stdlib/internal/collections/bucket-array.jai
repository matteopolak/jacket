Bucket_Array :: struct(type: Type, items_per_bucket: int, always_iterate_by_pointer := false) {
    count: s64;
    allocator: Allocator;
    My_Bucket :: Bucket(type, items_per_bucket);
    all_buckets: [..] *My_Bucket;
    unfull_buckets: [..] *My_Bucket;
    #assert items_per_bucket > 0;
}

Bucket :: struct(type: Type, items_per_bucket: int) {
    occupied: [items_per_bucket] bool;
    data: [items_per_bucket] type = ---;
    lowest_maybe_not_occupied: s32;
    bucket_index: u32;
    count := 0;
}

Bucket_Locator :: struct {
    bucket_index: u32;
    slot_index: s32;
}

find_and_occupy_empty_slot :: (using array: *Bucket_Array, $initialize := true) -> *array.type, Bucket_Locator {
    if !allocator.proc allocator = context.allocator;

    bucket: *array.My_Bucket;
    if unfull_buckets.count > 0 bucket = peek(unfull_buckets);
    else {
        bucket = cast(*array.My_Bucket) allocator.proc(.ALLOCATE, size_of(array.My_Bucket), 0, null, allocator.data);
        assert(bucket != null, "bucket allocation failed");
        memset(bucket, 0, size_of(array.My_Bucket));
        bucket.bucket_index = cast(u32) all_buckets.count;
        if !all_buckets.allocator.proc all_buckets.allocator = allocator;
        if !unfull_buckets.allocator.proc unfull_buckets.allocator = allocator;

        array_add(*all_buckets, bucket);
        array_add(*unfull_buckets, bucket);
    }

    index := bucket.lowest_maybe_not_occupied;
    while index < array.items_per_bucket && bucket.occupied[index] index += 1;

    assert(index < array.items_per_bucket);
    bucket.occupied[index] = true;
    bucket.count += 1;
    array.count += 1;
    bucket.lowest_maybe_not_occupied = index + 1;
    if bucket.count == array.items_per_bucket unfull_buckets.count -= 1;

    pointer := bucket.data.data + index;

    #if initialize Initialize(pointer);

    return pointer, Bucket_Locator.{ bucket.bucket_index, index };
}

bucket_array_add :: (using array: *Bucket_Array, item: array.type) -> Bucket_Locator, pointer: *array.type {
    pointer, locator := find_and_occupy_empty_slot(array, false);
    pointer.* = item;
    return locator, pointer;
}

bucket_array_find_pointer :: (using array: *Bucket_Array, locator: Bucket_Locator) -> *array.type {
    assert(locator.bucket_index < all_buckets.count);
    bucket := all_buckets[locator.bucket_index];
    assert(locator.slot_index >= 0 && locator.slot_index < array.items_per_bucket && bucket.occupied[locator.slot_index]);
    return bucket.data.data + locator.slot_index;
}

operator *[] :: (using array: Bucket_Array, index: int) -> *array.type {
    assert(index >= 0);
    bucket_index := index / array.items_per_bucket;
    slot_index := index % array.items_per_bucket;

    assert(bucket_index < all_buckets.count);
    bucket := all_buckets[bucket_index];

    assert(bucket.occupied[slot_index]);

    return bucket.data.data + slot_index;
}

bucket_array_find :: (using array: *Bucket_Array, locator: Bucket_Locator) -> array.type {
    return bucket_array_find_pointer(array, locator).*;
}

bucket_array_remove :: (using array: *Bucket_Array, locator: Bucket_Locator) {
    bucket_array_find_pointer(array, locator);
    bucket := all_buckets[locator.bucket_index];

    if bucket.count == array.items_per_bucket array_add(*unfull_buckets, bucket);

    bucket.occupied[locator.slot_index] = false;
    bucket.lowest_maybe_not_occupied = min(bucket.lowest_maybe_not_occupied, locator.slot_index);
    bucket.count -= 1;
    array.count -= 1;
}

bucket_array_reset :: (using array: *Bucket_Array) {
    for bucket: all_buckets allocator.proc(.FREE, 0, size_of(array.My_Bucket), bucket, allocator.data);
    array_reset(*all_buckets);
    array_reset(*unfull_buckets);
    count = 0;
}

for_expansion :: (array: *$T/Bucket_Array, body: Code, flags: For_Flags) #expand {
    #if (flags & .REVERSE) != 0 {
        index := array.count;
        b := array.all_buckets.count;

        while b > 0 {
            b -= 1;
            bucket := array.all_buckets[b];
            slot := array.items_per_bucket;

            while slot > 0 {
                slot -= 1;
                if !bucket.occupied[slot] continue;

                index -= 1;
                `it_index := index;

                #if (flags & .POINTER) != 0 || array.always_iterate_by_pointer {
                    `it := bucket.data.data + slot;
                    #insert (remove = {
                        bucket_array_remove(array, Bucket_Locator.{ bucket.bucket_index, cast(s32) slot });
                    }) body;
                } else {
                    `it := bucket.data[slot];
                    #insert (remove = {
                        bucket_array_remove(array, Bucket_Locator.{ bucket.bucket_index, cast(s32) slot });
                    }) body;
                }
            }
        }
    } else {
        index: int;

        for bucket: array.all_buckets for slot : 0..array.items_per_bucket - 1 {
            if !bucket.occupied[slot] continue;

            `it_index := index;
            defer index += 1;

            #if (flags & .POINTER) != 0 || array.always_iterate_by_pointer {
                `it := bucket.data.data + slot;
                #insert (remove = {
                    bucket_array_remove(array, Bucket_Locator.{ bucket.bucket_index, cast(s32) slot });
                }) body;
            } else {
                `it := bucket.data[slot];
                #insert (remove = {
                    bucket_array_remove(array, Bucket_Locator.{ bucket.bucket_index, cast(s32) slot });
                }) body;
            }
        }
    }
}

#scope_file
#import "Basic";
