// Generated private callback ABI; matching native profile owns the SDK instance.
#module_parameters (STATISTICS := true, RAW := false, PUBLIC_HEAPS := true, GLOBAL_CACHE := true, THREAD_CACHE := true, ASSERTS := true, VALIDATE_ARGS := true, ADAPTIVE := false)();

SDK :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = STATISTICS, RAW = RAW, PUBLIC_HEAPS = PUBLIC_HEAPS, GLOBAL_CACHE = GLOBAL_CACHE, THREAD_CACHE = THREAD_CACHE, ASSERTS = ASSERTS, VALIDATE_ARGS = VALIDATE_ARGS, ADAPTIVE = ADAPTIVE);

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_finalize";
} else #if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_validate_args_finalize";
} else #if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_asserts_finalize";
} else #if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_asserts_no_validate_args_finalize";
} else #if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_finalize";
} else #if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_finalize";
} else #if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_adaptive_finalize";
} else #if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_finalize";
} else #if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_finalize";
} else #if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_finalize";
} else #if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_finalize";
} else #if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_finalize";
} else #if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_finalize";
} else #if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_adaptive_finalize";
} else #if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_finalize";
} else #if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_finalize";
} else #if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_finalize";
} else #if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_finalize";
} else #if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_finalize";
} else #if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_finalize";
} else #if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_finalize";
} else #if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_finalize";
} else #if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_finalize";
} else {
    #assert false, "these rpmalloc options select a native profile that is not built; see tools/native_bindings/ci-rpmalloc-profiles.json";
}

#scope_file
#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks";
} else #if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_validate_args";
} else #if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_asserts";
} else #if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_asserts_no_validate_args";
} else #if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache";
} else #if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache_no_thread_cache";
} else #if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_adaptive";
} else #if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats";
} else #if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache";
} else #if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_thread_cache";
} else #if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base";
} else #if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache";
} else #if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache_no_thread_cache";
} else #if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_adaptive";
} else #if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats";
} else #if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache";
} else #if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_thread_cache";
} else #if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw";
} else #if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache";
} else #if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache_no_thread_cache";
} else #if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats";
} else #if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache";
} else #if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_thread_cache";
} else {
    #assert false, "these rpmalloc options select a native profile that is not built; see tools/native_bindings/ci-rpmalloc-profiles.json";
}
