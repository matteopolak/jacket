#module_parameters (STATISTICS := true, RAW := false, PUBLIC_HEAPS := true, GLOBAL_CACHE := true, THREAD_CACHE := true, ASSERTS := true, VALIDATE_ARGS := true, ADAPTIVE := false)();
// Generated from complete pinned official rpmalloc C SDK; not a public Jai facade.
// Opaque native heaps never alias the reviewed public Jai heap records.
#assert size_of(*void) == 8, "The private rpmalloc SDK backend requires the verified 64-bit C ABI";

rpmalloc_config_t :: struct {
    memory_map: #type (arg0: u64, arg1: *u64) -> *void #c_call;
    memory_unmap: #type (arg0: *void, arg1: u64, arg2: u64, arg3: u64) #c_call;
    error_callback: #type (arg0: *u8) #c_call;
    map_fail_callback: #type (arg0: u64) -> s32 #c_call;
    page_size: u64;
    span_size: u64;
    span_map_count: u64;
    enable_huge_pages: s32;
    page_name: *u8;
    huge_page_name: *u8;
}

rpmalloc_global_statistics_t :: struct {
    mapped: u64;
    mapped_peak: u64;
    cached: u64;
    huge_alloc: u64;
    huge_alloc_peak: u64;
    mapped_total: u64;
    unmapped_total: u64;
}

rpmalloc_thread_statistics_t :: struct {
    sizecache: u64;
    spancache: u64;
    thread_to_global: u64;
    global_to_thread: u64;
    span_use: [64] struct {
        current: u64;
        peak: u64;
        to_global: u64;
        from_global: u64;
        to_cache: u64;
        from_cache: u64;
        to_reserved: u64;
        from_reserved: u64;
        map_calls: u64;
    };
    size_use: [128] struct {
        alloc_current: u64;
        alloc_peak: u64;
        alloc_total: u64;
        free_total: u64;
        spans_to_cache: u64;
        spans_from_cache: u64;
        spans_from_reserved: u64;
        map_calls: u64;
    };
}

rpmalloc_heap_t :: struct {}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk;
    rpmalloc_finalize :: () #foreign sdk;
    rpmalloc_initialize :: () -> s32 #foreign sdk;
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk;
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk;
    rpmalloc_set_main_thread :: () #foreign sdk;
    rpmalloc_thread_collect :: () #foreign sdk;
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk;
    rpmalloc_thread_initialize :: () #foreign sdk;
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk;
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk;
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk;
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk;
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk;
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk;
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk;
    rpfree :: (ptr: *void) #foreign sdk;
    rpmalloc :: (size: u64) -> *void #foreign sdk;
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk;
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk;
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk;
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk;
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk;
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk;
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk;
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk;
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk;
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk;
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk;
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk;
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk;
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk;
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk;
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk;
    rpmalloc_linker_reference :: () #foreign sdk;
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_asserts_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_thread_cache_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_global_cache_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_adaptive_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_rpmalloc_linker_reference";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    rpmalloc_config :: () -> *rpmalloc_config_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_config";
    rpmalloc_finalize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_finalize";
    rpmalloc_initialize :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize";
    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_initialize_config";
    rpmalloc_is_thread_initialized :: () -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_is_thread_initialized";
    rpmalloc_set_main_thread :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_set_main_thread";
    rpmalloc_thread_collect :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_collect";
    rpmalloc_thread_finalize :: (release_caches: s32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_finalize";
    rpmalloc_thread_initialize :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_initialize";
    rpmalloc_dump_statistics :: (file: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_dump_statistics";
    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_global_statistics";
    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_thread_statistics";
    rpaligned_alloc :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_alloc";
    rpaligned_calloc :: (alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_calloc";
    rpaligned_realloc :: (ptr: *void, alignment: u64, size: u64, oldsize: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpaligned_realloc";
    rpcalloc :: (num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpcalloc";
    rpfree :: (ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpfree";
    rpmalloc :: (size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc";
    rpmalloc_usable_size :: (ptr: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_usable_size";
    rpmemalign :: (alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmemalign";
    rpposix_memalign :: (memptr: **void, alignment: u64, size: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpposix_memalign";
    rprealloc :: (ptr: *void, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rprealloc";
    rpmalloc_get_heap_for_ptr :: (ptr: *void) -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_get_heap_for_ptr";
    rpmalloc_heap_acquire :: () -> *rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_acquire";
    rpmalloc_heap_aligned_alloc :: (heap: *rpmalloc_heap_t, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_alloc";
    rpmalloc_heap_aligned_calloc :: (heap: *rpmalloc_heap_t, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_calloc";
    rpmalloc_heap_aligned_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_aligned_realloc";
    rpmalloc_heap_alloc :: (heap: *rpmalloc_heap_t, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_alloc";
    rpmalloc_heap_calloc :: (heap: *rpmalloc_heap_t, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_calloc";
    rpmalloc_heap_free :: (heap: *rpmalloc_heap_t, ptr: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free";
    rpmalloc_heap_free_all :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_free_all";
    rpmalloc_heap_realloc :: (heap: *rpmalloc_heap_t, ptr: *void, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_realloc";
    rpmalloc_heap_release :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_release";
    rpmalloc_heap_thread_set_current :: (heap: *rpmalloc_heap_t) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_heap_thread_set_current";
    rpmalloc_linker_reference :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_rpmalloc_linker_reference";
}

#scope_file
#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_validate_args";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_asserts";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_asserts_no_validate_args";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_thread_cache";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_thread_cache_no_validate_args";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_thread_cache_no_asserts";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_thread_cache_no_asserts_no_validate_args";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_global_cache";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_global_cache_no_validate_args";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_global_cache_no_asserts";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_global_cache_no_asserts_no_validate_args";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_global_cache_no_thread_cache";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_global_cache_no_thread_cache_no_validate_args";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_global_cache_no_thread_cache_no_asserts";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_adaptive";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_validate_args_adaptive";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_validate_args";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_asserts_adaptive";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_asserts";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_asserts_no_validate_args_adaptive";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_asserts_no_validate_args";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_thread_cache";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_thread_cache_no_validate_args";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_thread_cache_no_asserts";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_thread_cache_no_asserts_no_validate_args";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_adaptive";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_validate_args_adaptive";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_validate_args";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_asserts_adaptive";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_asserts";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_asserts_no_validate_args_adaptive";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_asserts_no_validate_args";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_thread_cache";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_thread_cache_no_validate_args";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_thread_cache_no_asserts";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_thread_cache";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_thread_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_thread_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_global_cache";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_global_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_global_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_global_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_global_cache_no_thread_cache";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_global_cache_no_thread_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_global_cache_no_thread_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_validate_args_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_asserts_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_asserts_no_validate_args_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_thread_cache";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_thread_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_thread_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_validate_args_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_asserts_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_thread_cache";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_asserts";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_thread_cache";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_thread_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_thread_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_global_cache";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_global_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_global_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_global_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_global_cache_no_thread_cache";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_global_cache_no_thread_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_global_cache_no_thread_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_validate_args_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_asserts_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_asserts";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_asserts_no_validate_args_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_thread_cache";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_thread_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_thread_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_validate_args_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_asserts_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_thread_cache";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_thread_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_thread_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    sdk :: #library,system "binding_rpmalloc_sdk_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}
