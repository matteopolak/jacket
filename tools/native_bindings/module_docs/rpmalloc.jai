// A general-purpose allocator for programs that allocate from many threads, with per-thread caches and separate heaps.
//
// Call [initialize] (or [initialize_config]) before allocating and [finalize] at the end. Allocate with [rpmalloc],
// [rpaligned_alloc] and [rpcalloc]'s relatives, resize with [rprealloc], and free with [rpfree]. [rpmalloc_allocator_proc]
// is the `Allocator` procedure that routes the default allocation calls here. [heap_acquire] gives a separate heap that
// [heap_free_all] empties in one call.
//
// Every import with the same module parameters shares one allocator, and imports with different parameters get separate
// ones. The parameters turn on statistics, the caches, argument checks and the settings [initialize_config] accepts
// (`CONFIGURABLE`); with `USE_RAW := true` the procedures take the C types and keep their C names.
