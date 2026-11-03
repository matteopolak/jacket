// A bump allocator over blocks of memory that are reused or freed all together.
//
// Allocation is cheap and there is no per-item free: you `reset` the pool to take everything back at once. Use it for data
// that lives and dies together.
//
// ```jai
// pool: Pool;
// defer release(*pool);
// values := cast(*int) get(*pool, 4 * size_of(int));
// for 0..3 values[it] = it * it;
// assert(values[3] == 9);
// reset(*pool);
// ```

#module_parameters(
    // Gets blocks from the `Unmapping_Allocator`, which returns them to the OS when the pool is reset.
    USE_UNMAPPING_ALLOCATOR := false
);

#load "../internal/allocation/pool.jai";
