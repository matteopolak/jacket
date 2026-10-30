// A collection with stable element addresses, grown a fixed-size bucket at a time.

// A growable collection whose elements never move: `Bucket_Array(Type, items_per_bucket)`.
//
// Storage is a list of fixed-size buckets of `items_per_bucket` slots, allocated one at a time from the
// allocator in the `allocator` field (`context.allocator` when it is unset at the first add). Adding never
// moves existing elements, so a pointer to an element stays valid until that element is removed or the
// array is reset; this is what it is for. Removing leaves a hole that a later add fills, lowest free slot
// first, so there is no gap-free order. `count` is the number of live elements.
//
// Elements are addressed by the `Bucket_Locator` that `bucket_array_add` returns. The `for` loop visits live
// elements bucket by bucket, slot by slot, which is insertion order only until something has been removed.
// With `always_iterate_by_pointer` set, `it` is a pointer to the element even without `for *`.
//
// **Looping.** `for value: array` visits the live elements. A second name, as in `for value, index: array`, is the *index*:
// how many elements were visited before this one (`0`, `1`, `2`, ...), free slots skipped. It is not a `Bucket_Locator`
// and not a slot number, so do not hand it to `array[n]` or `bucket_array_find` once anything has been removed; keep the
// locator that `bucket_array_add` returned for that. To delete the element you are on, write `remove value;` (the name of
// the loop variable, `it` when you gave none) inside the loop. The loop carries on with the next element, and it needs no
// locator.
//
// ```jai
// array: Bucket_Array(int, 4);
// for 10..14 bucket_array_add(*array, it);
//
// for value, index: array {
//     assert(value == 10 + index); // counts the elements visited, removed ones too
//     if value == 11 || value == 13 remove value;
// }
//
// assert(array.count == 3);
// for value, index: array assert(value == 10 + 2 * index); // 10, 12, 14: numbered afresh
// bucket_array_reset(*array);
// ```
//
// `array[n]` is the element in the slot numbered `n` (`*array[n]` is its address), counted as
// `bucket * items_per_bucket + slot`; it fails an assert on a free slot, so after removals it is not the n-th live element.
//
// A `Bucket_Array` that is never reset leaks its buckets: call `bucket_array_reset` to free them.
//
// ```jai
// array: Bucket_Array(string, 4);
// locator, pointer := bucket_array_add(*array, "first");
// bucket_array_add(*array, "second");
//
// assert(bucket_array_find(*array, locator) == "first");
// assert(pointer.* == "first" && array.count == 2);
//
// bucket_array_remove(*array, locator);
// for name: array assert(name == "second");
// bucket_array_reset(*array);
// ```
Bucket_Array :: Impl.Bucket_Array;
// One bucket of a `Bucket_Array`: an occupancy flag and storage for each of `items_per_bucket` slots. Reachable through `all_buckets`; it is not meant to be created directly.
Bucket :: Impl.Bucket;
// Where an element lives: the number of its bucket and its slot in that bucket.
//
// A locator stays valid, and keeps naming the same element, until that element is removed or the array is
// reset. Using a locator whose element was removed fails an assert for as long as the slot stays free
// (`bucket_array_find`, `bucket_array_find_pointer` and `bucket_array_remove` all check it). Once a later
// add reuses the slot the locator is not detected as stale: it then names the new occupant, so drop
// locators when you remove their elements.
Bucket_Locator :: Impl.Bucket_Locator;
// Frees every bucket through the array's allocator and empties the array, ready for reuse.
//
// Every pointer and locator into it becomes invalid. Elements are not destroyed first, so free what they own
// beforehand. Safe on an array that never held anything.
bucket_array_reset :: Impl.bucket_array_reset;
// Stores `item` in the first free slot and returns its `Bucket_Locator` and a pointer to the stored copy.
//
// A new bucket is allocated when all are full; its memory is zeroed and nothing else is moved. The pointer
// stays valid until the element is removed or the array reset.
//
// ```jai
// array: Bucket_Array(int, 2);
// a, pa := bucket_array_add(*array, 10);
// b, pb := bucket_array_add(*array, 20);
// c, pc := bucket_array_add(*array, 30); // starts a second bucket
// assert(array.count == 3 && c.bucket_index == 1);
//
// pa.* += 1; // pointers stay valid as the array grows
// assert(bucket_array_find(*array, a) == 11);
// bucket_array_reset(*array);
// ```
bucket_array_add :: Impl.bucket_array_add;
// The element at `locator`, by value. Fails an assert when the locator is out of range or names a free slot.
bucket_array_find :: Impl.bucket_array_find;
// A pointer to the element at `locator`, to change it in place. Fails an assert when the locator is out of range or names a free slot.
//
// ```jai
// array: Bucket_Array(int, 4);
// locator := bucket_array_add(*array, 1);
// bucket_array_find_pointer(*array, locator).* = 5;
// assert(bucket_array_find(*array, locator) == 5);
// bucket_array_reset(*array);
// ```
bucket_array_find_pointer :: Impl.bucket_array_find_pointer;
// Removes the element at `locator`, freeing its slot for a later add. Fails an assert when the locator names a free or invalid slot.
//
// The memory of the bucket is kept, and the element is not destroyed or cleared. No other element moves.
// To remove while looping, write `remove value;` in the `for` loop, with `value` the loop's variable (`it` by default),
// instead of calling this with a locator.
//
// ```jai
// array: Bucket_Array(int, 2);
// for 1..4 bucket_array_add(*array, it);
// for value: array {
//     if value % 2 == 0 remove value;
// }
// assert(array.count == 2);
// bucket_array_reset(*array);
// ```
bucket_array_remove :: Impl.bucket_array_remove;
// Claims a free slot and returns a pointer to it and its locator; the lower-level half of `bucket_array_add`.
//
// The element is set to the default value of the type, or left uninitialized with `$initialize` set to
// `false`, in which case the caller must write all of it.
find_and_occupy_empty_slot :: Impl.find_and_occupy_empty_slot;
// Lets a `Bucket_Array` be used in a `for` loop over its live elements, in bucket and slot order; `it_index` numbers the elements visited, `0`, `1`, `2`, ... with free slots skipped.
//
// `it_index`, the second name in `for value, index: array`, is a visit count, not a slot number and not a
// `Bucket_Locator`: after a removal it no longer matches what `array[n]` accepts. Keep the `Bucket_Locator` from
// `bucket_array_add` when you need to address an element.
//
// `for *` gives pointers, `for <` goes backwards, and `remove value;` (with `value` the loop's variable, `it` by default)
// deletes the current element, as `bucket_array_remove` would. Adding while iterating may or may not visit the new element.
//
// ```jai
// array: Bucket_Array(string, 2);
// for string.["a", "b", "c"] bucket_array_add(*array, it);
//
// for name, index: array {
//     if name == "b" remove name;
// }
// assert(array.count == 2);
// for name, index: array assert(name == ifx index == 0 then "a" else "c");
// bucket_array_reset(*array);
// ```
for_expansion :: Impl.for_expansion;

using,only(.["*[]"]) #import "internal/collections/bucket-array";

#scope_file
Impl :: #import "internal/collections/bucket-array";
