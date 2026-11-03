// A debugging allocator that fills memory with a recognizable byte on allocation and on free.
//
// [get_overwriting_allocator] wraps another allocator. Use it with `push_allocator` while hunting for code that reads memory before writing it or after freeing it.
//
// ```jai
// allocator := get_overwriting_allocator();
// {
//     push_allocator(allocator);
//     bytes := cast(*u8) alloc(4);
//     assert(bytes[0] == 0xcd);
//     free(bytes);
// }
// deinit_overwriting_allocator(allocator);
// ```

// The state of an overwriting allocator: the allocator it wraps, a record of live allocations and the fill byte.
Overwriting_Allocator :: Impl.Overwriting_Allocator;
// Returns an allocator that wraps `base_allocator` (the current allocator when unset) and fills memory with `overwrite_byte` when it is allocated and again when it is freed, so reads of uninitialized or freed memory show an obvious pattern. Freeing a pointer it did not hand out, or twice, is reported. Free it with [deinit_overwriting_allocator].
//
// ```jai
// allocator := get_overwriting_allocator();
// memory := cast(*u8) allocator.proc(.ALLOCATE, 4, 0, null, allocator.data);
// assert(memory[0] == 0xcd);
// memory[0] = 1;
// allocator.proc(.FREE, 0, 4, memory, allocator.data);
// deinit_overwriting_allocator(allocator);
// ```
get_overwriting_allocator :: Impl.get_overwriting_allocator;
// The allocator procedure; use it through the `Allocator` from [get_overwriting_allocator].
overwriting_allocator_proc :: Impl.overwriting_allocator_proc;
// Releases the bookkeeping of an [Overwriting_Allocator] that was set up by hand. [deinit_overwriting_allocator] does this and frees the allocator.
deinit :: Impl.deinit;
// Frees an allocator made by [get_overwriting_allocator], along with its records. Memory still allocated from it is not freed.
deinit_overwriting_allocator :: Impl.deinit_overwriting_allocator;

#scope_file
Impl :: #import "internal/allocation/overwriting";
