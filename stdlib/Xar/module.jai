// A growable array whose elements never move, so pointers to them stay valid while you add more.
//
// - [Xar] is the type; [xar_add] appends and [xar_reset] frees.
// - `array[i]` indexes, `array.count` counts, and `for` walks it in order.
// - [xar_iterator], [xar_next] and [xar_prev] walk it cheaply by hand.
// - [quick_sort] sorts it.
//
// ```jai
// array: Xar(int, 2, 8);
// first := xar_add(*array, 10);
//
// for 1..20 xar_add(*array, it * 10);
//
// assert(array.count == 21 && first == *array[0]);
// assert(array[5] == 50);
// xar_reset(*array);
// ```

// `Xar(Element_Type, first_chunk_size_shift, max_chunks)` stores its elements in separately allocated chunks.
//
// The first two chunks hold `1 << first_chunk_size_shift` elements and each later chunk doubles, so the three parameters fix the capacity; adding past it fails an assert. Chunks come from `allocator` (`context.allocator` when unset) as they are needed, and `xar_reset` frees them.
//
// ```jai
// array: Xar(string, 3, 4); // chunks of 8, 8, 16, 32
//
// xar_add(*array, "a");
// xar_add(*array, "b");
// assert(array.count == 2 && array[1] == "b");
// xar_reset(*array);
// ```
Xar :: Impl.Xar;
// Appends `el` and returns a pointer to the stored copy, or `null` if a chunk could not be allocated.
//
// ```jai
// array: Xar(int, 2, 4);
// slot := xar_add(*array, 7);
// slot.* += 1;
// assert(array[0] == 8);
// xar_reset(*array);
// ```
xar_add :: Impl.xar_add;
// Removes element `index` by copying the last element over it. The order changes and no memory is freed.
//
// ```jai
// array: Xar(int, 2, 4);
// for 1..4 xar_add(*array, it);
//
// xar_unordered_remove(*array, 0);
// assert(array.count == 3 && array[0] == 4);
// xar_reset(*array);
// ```
xar_unordered_remove :: Impl.xar_unordered_remove;
// Frees every chunk and empties the array, so it can be used again. Pointers into it are invalid afterwards. The elements are not destroyed first.
//
// ```jai
// array: Xar(int, 2, 4);
// xar_add(*array, 1);
// xar_reset(*array);
// assert(array.count == 0);
// ```
xar_reset :: Impl.xar_reset;
// Returns an iterator at element `index`, for walking with [xar_get], [xar_next] and [xar_prev]. Cheaper than indexing at every step.
//
// ```jai
// array: Xar(int, 2, 4);
// for 1..3 xar_add(*array, it);
//
// iterator := xar_iterator(*array, 1);
//
// assert(xar_get(*array, iterator).* == 2);
// xar_reset(*array);
// ```
xar_iterator :: Impl.xar_iterator;
// Returns a pointer to the element the iterator is at. Fails an assert when the iterator is past the end.
xar_get :: Impl.xar_get;
// Moves the iterator to the next element and returns a pointer to it, or `null` at the end (the iterator then stays put).
//
// ```jai
// array: Xar(int, 1, 4);
// for 1..5 xar_add(*array, it);
//
// iterator := xar_iterator(*array, 0);
// sum := xar_get(*array, iterator).*;
//
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
// Moves the iterator to the previous element and returns a pointer to it, or `null` at the start (the iterator then stays put).
//
// ```jai
// array: Xar(int, 2, 4);
// for 1..3 xar_add(*array, it);
//
// iterator := xar_iterator(*array, 2);
//
// assert(xar_prev(*array, *iterator).* == 2);
// xar_reset(*array);
// ```
xar_prev :: Impl.xar_prev;
// Makes a `Xar` work in a `for` loop: `it` is the element (a pointer with `for *`) and `it_index` its index. Do not add during the loop. Reverse loops are a compile error.
//
// ```jai
// array: Xar(int, 2, 4);
// for 1..3 xar_add(*array, it);
//
// total := 0;
// for array total += it * it_index;
//
// assert(total == 8);
// xar_reset(*array);
// ```
for_expansion :: Impl.for_expansion;
// Sorts in place with a three-way comparator (negative, zero or positive). Despite the name it is an unstable heap sort, `O(n log n)`; it returns nothing and has no key form.
//
// ```jai
// array: Xar(int, 2, 4);
// for value: int.[3, 1, 2] xar_add(*array, value);
//
// quick_sort(array, (a: int, b: int) -> int {
//     return a - b;
// });
// assert(array[0] == 1 && array[2] == 3);
// xar_reset(*array);
// ```
quick_sort :: Impl.quick_sort;
// The same sort as [quick_sort] under another name, so code written for either works.
//
// ```jai
// array: Xar(int, 2, 4);
// for value: int.[2, 1] xar_add(*array, value);
//
// quick_sort_simple(array, (a: int, b: int) -> int {
//     return a - b;
// });
// assert(array[0] == 1);
// xar_reset(*array);
// ```
quick_sort_simple :: Impl.quick_sort_simple;
// The same sort as [quick_sort] under another name, so code written for either works.
quick_sort_iter :: Impl.quick_sort_iter;

using,only(.["*[]"]) #import "internal/collections/xar";

#scope_file
Impl :: #import "internal/collections/xar";
