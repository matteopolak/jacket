// A debugging allocator that turns use-after-free and buffer overruns into immediate crashes by giving each allocation its own pages.
//
// Make one with [get_unmapping_allocator] and use it with `push_allocator` around the code you suspect. Needs a system with virtual memory: macOS, Linux, Android or Windows.
//
// ```jai,skip_on(wasm)
// allocator := get_unmapping_allocator();
// {
//     push_allocator(allocator);
//     bytes := cast(*u8) alloc(8);
//     bytes[7] = 1;
//     free(bytes);
// }
// deinit_unmapping_allocator(allocator);
// ```

#assert OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS;
// The state of an unmapping allocator: a record of live allocations and a lock.
Unmapping_Allocator :: Impl.Unmapping_Allocator;
// Returns an allocator, backed by the current one for its records, that gives every allocation its own pages. The pages end at an inaccessible guard page, so running off the end crashes at once, and a freed block is made read-only, so writing through a stale pointer crashes too. It is slow and uses a page or more per allocation; free it with [deinit_unmapping_allocator].
//
// ```jai
// allocator := get_unmapping_allocator();
// memory := cast(*u8) allocator.proc(.ALLOCATE, 16, 0, null, allocator.data);
// memory[0] = 1;
// allocator.proc(.FREE, 0, 16, memory, allocator.data);
// deinit_unmapping_allocator(allocator);
// ```
get_unmapping_allocator :: Impl.get_unmapping_allocator;
// The allocator procedure; use it through the `Allocator` from [get_unmapping_allocator].
unmapping_allocator_proc :: Impl.unmapping_allocator_proc;
// Releases the bookkeeping of an [Unmapping_Allocator] that was set up by hand. [deinit_unmapping_allocator] does this and frees the allocator.
deinit :: Impl.deinit;
// Frees an allocator made by [get_unmapping_allocator], along with its records.
deinit_unmapping_allocator :: Impl.deinit_unmapping_allocator;

#scope_file
Impl :: #import "internal/allocation/unmapping";
