#module_parameters (
    HEAP_ARRAY_SIZE := 47,
    DEFAULT_SPAN_MAP_COUNT := 64,
    GLOBAL_CACHE_MULTIPLIER := 8,
    ENABLE_THREAD_CACHE := true,
    ENABLE_GLOBAL_CACHE := true,
    ENABLE_STATISTICS := true,
    ENABLE_ASSERTS := true,
    ENABLE_VALIDATE_ARGS := true,
    LINUX_FREE_ADVICE: enum {
        DONTNEED;
        FREE;
    } = .DONTNEED,
    CONFIGURABLE := false,
    ENABLE_ADAPTIVE_THREAD_CACHE := false,
    LINUX_HUGE_PAGES := false,
    USE_RAW := false,
    DISABLE_UNMAP := false,
    ENABLE_UNLIMITED_CACHE := false
)();

#assert HEAP_ARRAY_SIZE > 0, "HEAP_ARRAY_SIZE must be positive";
#assert DEFAULT_SPAN_MAP_COUNT > 0, "DEFAULT_SPAN_MAP_COUNT must be positive";
#assert GLOBAL_CACHE_MULTIPLIER > 0, "GLOBAL_CACHE_MULTIPLIER must be positive";
#assert !ENABLE_GLOBAL_CACHE || ENABLE_THREAD_CACHE, "ENABLE_GLOBAL_CACHE requires ENABLE_THREAD_CACHE";
#assert !ENABLE_UNLIMITED_CACHE || ENABLE_GLOBAL_CACHE, "ENABLE_UNLIMITED_CACHE requires ENABLE_GLOBAL_CACHE";
#assert !DISABLE_UNMAP || (ENABLE_GLOBAL_CACHE && ENABLE_UNLIMITED_CACHE), "DISABLE_UNMAP requires global and unlimited caches";
#assert !ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_THREAD_CACHE, "ENABLE_ADAPTIVE_THREAD_CACHE requires ENABLE_THREAD_CACHE";

#load "context.jai";
#load "declarations.jai";
#load "wrappers.jai";
#load "allocator.jai";
#load "statistics.jai";
