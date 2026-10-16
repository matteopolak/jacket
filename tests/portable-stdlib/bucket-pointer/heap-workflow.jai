#import "Basic";
#import "Bucket_Array";

Entry :: struct {
    id: s64;
    weight: float;
}

main :: () {
    array: Bucket_Array(Entry, 3);
    pointers: [9] *Entry;
    locators: [9] Bucket_Locator;

    for i: 0..8 {
        locator, pointer := bucket_array_add(*array, Entry.{ i, cast(float) i });

        locators[i] = locator;
        pointers[i] = pointer;
        assert(locator.bucket_index == i / 3 && locator.slot_index == i % 3);
        assert(*array[i] == pointer && array[i].id == i);
    }

    assert(array.all_buckets.count == 3);
    bucket_array_remove(*array, locators[1]);
    bucket_array_remove(*array, locators[4]);
    assert(array.count == 7 && array[8].id == 8 && *array[8] == pointers[8]);
    array[8].id = 81;
    assert(pointers[8].id == 81);
    pointers[0].id = 90;
    assert(array[0].id == 90);
    first_locator, first := bucket_array_add(*array, Entry.{ 101, 1 });
    second_locator, second := bucket_array_add(*array, Entry.{ 102, 2 });
    first_index := cast(int) first_locator.bucket_index * 3 + first_locator.slot_index;
    second_index := cast(int) second_locator.bucket_index * 3 + second_locator.slot_index;

    assert(*array[first_index] == first && *array[second_index] == second);
    assert(first_index != second_index && (first_index == 1 || first_index == 4));
    assert(second_index == 1 || second_index == 4);
    assert(array.count == 9 && array.all_buckets.count == 3);
    assert(*array[0] == pointers[0] && *array[2] == pointers[2] && *array[8] == pointers[8]);
    bucket_array_reset(*array);
    assert(array.count == 0 && array.all_buckets.count == 0 && array.unfull_buckets.count == 0);
    print("ok\n");
}
