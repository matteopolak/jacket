// Stable addresses, physical locators, removals, and live-value iteration.
#import "Basic";
Bucket_Array :: #import "Bucket_Array";

main :: () {
    values: Bucket_Array.Bucket_Array(s64, 2);
    first, first_pointer := Bucket_Array.bucket_array_add(*values, 11);
    second, second_pointer := Bucket_Array.bucket_array_add(*values, 22);
    third, third_pointer := Bucket_Array.bucket_array_add(*values, 33);
    assert(values.count == 3 && first_pointer.* == 11 && second_pointer.* == 22);
    assert(Bucket_Array.bucket_array_find(*values, third) == 33);

    Bucket_Array.bucket_array_remove(*values, second);
    assert(values.count == 2 && first_pointer.* == 11 && third_pointer.* == 33);
    replacement, replacement_pointer := Bucket_Array.bucket_array_add(*values, 44);
    assert(replacement_pointer.* == 44);
    assert(Bucket_Array.bucket_array_find(*values, replacement) == 44);

    sum: s64;
    visits: s64;
    for values {
        sum += it;
        visits += 1;
    }
    assert(visits == 3 && sum == 88);

    pointers: Bucket_Array.Bucket_Array(s64, 2, true);
    ignored_locator, ignored_pointer := Bucket_Array.bucket_array_add(*pointers, 7);
    pointer_sum: s64;
    for pointers {
        pointer_sum += it.*;
    }
    assert(ignored_locator.slot_index >= 0 && ignored_pointer.* == 7);
    assert(pointer_sum == 7);

    Bucket_Array.bucket_array_reset(*values);
    assert(values.count == 0);
    print("ok\n");
}
