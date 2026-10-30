// A bump allocator over blocks of memory that are reused or freed all together.

#module_parameters(USE_UNMAPPING_ALLOCATOR := false);

#load "../internal/allocation/pool.jai";
