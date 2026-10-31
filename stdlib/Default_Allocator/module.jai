// The general-purpose heap allocator behind `context.allocator`. It is thread safe, can create separate heaps, and tells you whether it owns a pointer.
//
// - [allocator_proc] is the allocator procedure; call it directly or wrap it in an `Allocator`.
// - [usable_size], [is_this_yours] and [heap_free_all] ask about or clear a heap.
//
// Module parameters `ENABLE_ASSERTS` and `ENABLE_VALIDATE_ARGS` (both on by default) turn its internal checks and its argument checks on or off.
//
// ```jai
// memory := cast(*u8) allocator_proc(.ALLOCATE, 16, 0, null, null);
// memory[0] = 42;
// assert(memory[0] == 42);
// allocator_proc(.FREE, 0, 16, memory, null);
// ```

#module_parameters(ENABLE_ASSERTS := true, ENABLE_VALIDATE_ARGS := true);

#load "../internal/allocation/aligned-blocks.jai";

#if (CPU == .X64 && OS != .IOS) || OS == .NN_SWITCH2 {
    #load "../internal/allocation/default-records.jai";
    #load "../internal/allocation/default-heaps.jai";
} else {
    #load "../internal/allocation/default-c.jai";
}
