#import "Basic";
#import "Bucket_Array";
B :: #import "Bucket_Array";
Again :: #import "Bucket_Array";

Record :: struct {
    id: s64;
    weight: float;
}

verify :: ($T: Type, initial: T, replacement: T, $pointer_mode: bool) -> bool {
    first: Bucket(T, 2);
    second: Bucket(T, 2);
    third: Bucket(T, 2);

    first.occupied[0] = true;
    first.occupied[1] = true;
    first.count = 2;
    second.occupied[0] = true;
    second.occupied[1] = true;
    second.count = 2;
    second.bucket_index = 1;
    first.data[0] = initial;
    first.data[1] = initial;
    second.data[0] = initial;
    second.data[1] = initial;
    storage := (*Bucket(T, 2)).[*first, *second, null];
    array: Bucket_Array(T, 2, pointer_mode);
    array.all_buckets = .{ count = 2, allocated = 3, data = storage.data };
    array.count = 4;
    p0 := *array[0];
    p1 := *array[1];
    p2 := *array[2];
    p3 := *array[3];

    if p0 != first.data.data || p1 != first.data.data + 1 return false;
    if p2 != second.data.data || p3 != second.data.data + 1 return false;
    if type_of(p2) != *T || type_of(array[2]) != T return false;

    array[2] = replacement;
    if *array[2] != p2 || *array[0] != p0 return false;

    #if T == s64 if p2.* != replacement return false;
    #if T == Record if p2.id != replacement.id || p2.weight != replacement.weight return false;
    locator := Bucket_Locator.{ 1, 0 };
    if bucket_array_find_pointer(*array, locator) != p2 return false;
    if type_info(B.Bucket_Array(T, 2, pointer_mode)) != type_info(Again.Bucket_Array(T, 2, pointer_mode)) return false;

    third.occupied[0] = true;
    third.data[0] = initial;
    third.count = 1;
    third.bucket_index = 2;
    array_add(*array.all_buckets, *third);
    array.count += 1;
    if *array[4] != third.data.data || *array[0] != p0 || *array[2] != p2 return false;

    return true;
}

#assert verify(s64, 17, 31, false);
#assert verify(s64, 17, 31, true);
#assert verify(Record, .{ 17, 3 }, .{ 31, 5 }, false);
#assert verify(Record, .{ 17, 3 }, .{ 31, 5 }, true);

main :: () {
    assert(verify(s64, 17, 31, false));
    assert(verify(s64, 17, 31, true));
    assert(verify(Record, .{ 17, 3 }, .{ 31, 5 }, false));
    assert(verify(Record, .{ 17, 3 }, .{ 31, 5 }, true));
    print("ok\n");
}
