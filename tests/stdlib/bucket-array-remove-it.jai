// `remove it` inside a for loop over a Bucket_Array frees that slot.
#import "Basic";
#import "Bucket_Array";
main :: () {
    array: Bucket_Array(int, 4);
    for 1..10  bucket_array_add(*array, it);
    for * array  if it.* % 2 == 0  remove it;
    sum := 0;
    for array  sum += it;
    assert(array.count == 5 && sum == 25);
}
