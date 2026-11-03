// A bump allocator over one contiguous range of reserved address space.
//
// Use it when allocations never move and everything is dropped together: `get` hands out memory, `reset` takes it all back.
//
// ```jai,skip_on(wasm)
// pool: Flat_Pool;
// defer fini(*pool);
// list := cast(*u8) get(*pool, 100);
// list[99] = 1;
// assert(list[99] == 1);
// ```

// Canonical declarations share this module's parameterized instance.
#load "../internal/allocation/flat-pool.jai";
