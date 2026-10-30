#import "Basic";
#import "Bucket_Array";
B :: #import "Bucket_Array";

main :: () {
    array: B.Bucket_Array(s64, 2);

    defer B.bucket_array_reset(*array);

    first, _ := B.bucket_array_add(*array, 10);
    second, _ := B.bucket_array_add(*array, 20);
    third, _ := B.bucket_array_add(*array, 30);

    // A locator is two fields, so programs can build one by hand.
    by_hand := B.Bucket_Locator.{ second.bucket_index, second.slot_index };

    assert(B.bucket_array_find(*array, by_hand) == 20);

    B.bucket_array_remove(*array, first);

    // The freed slot is reused by the next add, in the same place.
    again, _ := B.bucket_array_add(*array, 40);

    assert(again.bucket_index == first.bucket_index && again.slot_index == first.slot_index);
    assert(B.bucket_array_find(*array, again) == 40);

    // `it_index` counts the elements visited, with free slots skipped, in both directions.
    B.bucket_array_remove(*array, second);

    visited: s64;

    for value, index: array {
        assert(index == visited);
        visited += 1;
    }

    assert(visited == 2 && array.count == 2);

    visited = 1;

    for < value, index: array {
        assert(index == visited);
        visited -= 1;
    }

    assert(visited == -1);
    assert(B.bucket_array_find(*array, third) == 30);

    print("ok\n");
}
