// test: compile_fail
#import "Basic";
#import "Bucket_Array";

removed_find :: () -> bool {
    array: Bucket_Array(s64, 2);
    first := bucket_array_add(*array, 1);

    bucket_array_remove(*array, first);

    // The slot is free again, so using the old locator is caught.
    return bucket_array_find(*array, first) == 1;
}

#assert removed_find();
main :: () {}
