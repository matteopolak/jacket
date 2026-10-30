// RadixSort: ranks `u32`, `u64` and `float` data by radix sorting, without moving it.

// Scratch state for radix sorting, which orders the numbers in an array by their bytes rather than by comparing them.
//
// `sort` does not move the input. It fills a table of ranks: after sorting, `rank(*sorter, i)` (also
// `sorter[i]`) is the index in the input of the element that belongs at position `i`. Use the ranks to read
// the input in order, or `reorder` to rearrange a dynamic array to match. The sort is stable, linear in the
// input size and needs one pass per byte of the element type, so it beats comparison sorts on large arrays
// of `u32`, `u64` and `float`. Signed integers are not supported: convert them first.
//
// The rank tables are allocated by `sort` (from `allocator`, or `context.allocator` when it is unset) and
// reused by later sorts of the same or a smaller size. Release them with `free_resources`.
//
// ```jai
// sorter: RadixSort;
// values := u32.[40, 7, 19];
// sort(*sorter, values);
// assert(sorter[0] == 1 && sorter[1] == 2 && sorter[2] == 0); // 7, 19, 40
// free_resources(*sorter);
// ```
RadixSort :: Impl.RadixSort;
// Marks the ranks as stale, so `rank` and `reorder` refuse to use them until the next `sort`. The tables are kept.
reset :: Impl.reset;
// Makes the rank tables hold `new_size` entries, allocating if they must grow, and marks the ranks as stale. `sort` calls this for you; call it first to allocate ahead of time.
resize :: Impl.resize;
// Frees the rank tables and clears the ranks. The sorter can be used again afterwards, allocating anew.
free_resources :: Impl.free_resources;
// The index in the sorted input of the `i`-th smallest element. Fails an assert when no sort has produced valid ranks or `i` is out of range.
rank :: Impl.rank;
// Ranks `count` numbers at `input` from smallest to largest, or all of an array view; supports `u32`, `u64` and `float`.
//
// The input is left as it is and the result is read back through `rank`. Equal values keep their input order. For
// `float`, `-0.0` sorts before `0.0` and NaNs sort to the ends by their bit pattern (positive ones last). An empty
// input (or a `null` pointer) gives valid, empty ranks: any earlier ranks are dropped, `reorder` of an empty
// array works and `rank` of any index fails its range assert. The count is a `u32`.
//
// ```jai
// sorter: RadixSort;
// values := float.[2.5, -1, 9, 0];
// sort(*sorter, values);
//
// smallest := values[rank(*sorter, 0)];
// largest := values[rank(*sorter, 3)];
// assert(smallest == -1 && largest == 9);
// free_resources(*sorter);
// ```
sort :: Impl.sort;
// Rearranges a dynamic array into sorted order, using the ranks of the last `sort` of that array.
//
// A new array is built with the same allocator, the old storage is freed and `input` points to the new one,
// so earlier pointers into it are stale. The array must be the one that was sorted: a different count, or
// ranks that are not valid, fail an assert.
//
// ```jai
// values: [..] u64;
// array_add(*values, 30, 10, 20);
//
// sorter: RadixSort;
// sort(*sorter, values);
// reorder(*values, sorter);
// assert(values[0] == 10 && values[1] == 20 && values[2] == 30);
//
// free_resources(*sorter);
// array_free(values);
// ```
reorder :: Impl.reorder;
// A built-in smoke test that sorts three numbers and asserts the ranks. It is not part of the sorting API.
radix_tests :: Impl.radix_tests;

using,only(.["[]"]) #import "internal/collections/radixsort";

#scope_file
Impl :: #import "internal/collections/radixsort";
