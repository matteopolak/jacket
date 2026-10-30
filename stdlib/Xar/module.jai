// Xar: a growable array that allocates in chunks, so its elements never move.

// `Xar(Element_Type, first_chunk_size_shift, max_chunks)`: a growable array whose elements never move.
//
// Elements live in up to `max_chunks` separately allocated chunks. The first two chunks hold
// `1 << first_chunk_size_shift` elements each and every later chunk doubles, so the capacity is fixed by
// the parameters; adding past it fails an assert. Chunks come from `allocator` (`context.allocator` when
// unset) as they are first needed and are freed by `xar_reset`. A pointer to an element stays valid
// until the array is reset; unlike a dynamic array, adding never invalidates it.
//
// `array[i]` is element `i` (`*array[i]` is its address), `count` the number of elements, and `for` visits them in
// order. There is no way to remove the last element but `xar_unordered_remove`.
//
// ```jai
// array: Xar(int, 2, 8); // chunks of 4, 4, 8, 16, ...
// first := xar_add(*array, 10);
// for 1..20 xar_add(*array, it * 10);
//
// assert(array.count == 21 && first.* == 10 && first == *array[0]);
// assert(array[5] == 50);
// xar_reset(*array);
// ```
Xar :: Impl.Xar;
// Appends `el` and returns a pointer to the stored copy, which stays valid until `xar_reset`. Returns `null` if a chunk could not be allocated.
//
// Exceeding the capacity set by the type's parameters fails an assert.
xar_add :: Impl.xar_add;
// Removes element `index` by copying the last element over it and shrinking the count. The order changes and no memory is freed.
//
// An out-of-range index fails an assert. Pointers to the last element and to `index` now name different values.
//
// ```jai
// array: Xar(int, 2, 4);
// for 1..4 xar_add(*array, it);
// xar_unordered_remove(*array, 0);
// assert(array.count == 3 && array[0] == 4);
// xar_reset(*array);
// ```
xar_unordered_remove :: Impl.xar_unordered_remove;
// Frees every chunk and empties the array, so it can be used again. All pointers into it are invalid afterwards. The elements are not destroyed first.
xar_reset :: Impl.xar_reset;
// An iterator positioned at element `index`, for walking with `xar_get`, `xar_next` and `xar_prev`; cheaper than indexing each step.
xar_iterator :: Impl.xar_iterator;
// A pointer to the element an iterator is at. Fails an assert when the iterator is past the end or was never positioned.
xar_get :: Impl.xar_get;
// Moves the iterator to the following element and returns a pointer to it, or `null` at the end (the iterator then stays where it was).
//
// ```jai
// array: Xar(int, 1, 4);
// for 1..5 xar_add(*array, it);
//
// iterator := xar_iterator(*array, 0);
// sum := xar_get(*array, iterator).*;
// while true {
//     element := xar_next(*array, *iterator);
//     if !element break;
//     sum += element.*;
// }
//
// assert(sum == 15);
// xar_reset(*array);
// ```
xar_next :: Impl.xar_next;
// Moves the iterator to the preceding element and returns a pointer to it, or `null` at the start (the iterator then stays where it was).
xar_prev :: Impl.xar_prev;
// Lets a `Xar` be used in a `for` loop over its elements in order: `it` is the element (a pointer with `for *`) and `it_index` its index.
//
// Reverse iteration is not supported and is rejected at compile time. Do not add during the loop.
for_expansion :: Impl.for_expansion;
// Sorts a `Xar` in place with a three-way comparator (negative, zero or positive). Despite the name it is a heap sort, not a quicksort.
//
// It is `O(n log n)` in the worst case and unstable: equal elements may change their relative order. The
// comparator should be a strict weak order; an inconsistent one only gives an unspecified order. Unlike
// `Sort.quick_sort` it has no key form and no insertion-sort fast path, and it returns nothing.
//
// ```jai
// array: Xar(int, 2, 4);
// for value: int.[3, 1, 2] xar_add(*array, value);
// quick_sort(array, (a: int, b: int) -> int { return a - b; });
// assert(array[0] == 1 && array[2] == 3);
// xar_reset(*array);
// ```
quick_sort :: Impl.quick_sort;
// The same heap sort as `quick_sort` (unstable, `O(n log n)`); the name is kept so that code written for either works.
quick_sort_simple :: Impl.quick_sort_simple;
// The same heap sort as `quick_sort` (unstable, `O(n log n)`); the name is kept so that code written for either works.
quick_sort_iter :: Impl.quick_sort_iter;

using,only(.["*[]"]) #import "internal/collections/xar";

#scope_file
Impl :: #import "internal/collections/xar";
