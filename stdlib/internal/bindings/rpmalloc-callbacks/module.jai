// Generated private callback ABI; matching native profile owns the SDK instance.
#module_parameters (STATISTICS := true, RAW := false, PUBLIC_HEAPS := true, GLOBAL_CACHE := true, THREAD_CACHE := true, ASSERTS := true, VALIDATE_ARGS := true, ADAPTIVE := false)();

SDK :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = STATISTICS, RAW = RAW, PUBLIC_HEAPS = PUBLIC_HEAPS, GLOBAL_CACHE = GLOBAL_CACHE, THREAD_CACHE = THREAD_CACHE, ASSERTS = ASSERTS, VALIDATE_ARGS = VALIDATE_ARGS, ADAPTIVE = ADAPTIVE);

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_finalize";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_validate_args_finalize";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_asserts_finalize";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_asserts_no_validate_args_finalize";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_thread_cache_finalize";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_thread_cache_no_validate_args_finalize";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_thread_cache_no_asserts_finalize";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_finalize";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_no_validate_args_finalize";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_no_asserts_finalize";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_finalize";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_finalize";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_finalize";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_finalize";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_adaptive_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_validate_args_adaptive_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_validate_args_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_asserts_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_asserts_adaptive_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_asserts_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_asserts_no_validate_args_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_thread_cache_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_thread_cache_no_asserts_finalize";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_adaptive_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_validate_args_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_asserts_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_finalize";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_asserts_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_asserts_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_thread_cache_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_asserts_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_finalize";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_adaptive_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_asserts_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_thread_cache_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_finalize";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_asserts_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_asserts_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_thread_cache_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_thread_cache_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_thread_cache_no_asserts_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_asserts_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_finalize";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_adaptive_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_asserts_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_asserts_adaptive_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_asserts_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_thread_cache_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_finalize";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_initialize";
    finalize :: () #foreign bridge "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_finalize";
}

#scope_file
#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_validate_args";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_asserts";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_asserts_no_validate_args";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_thread_cache";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_thread_cache_no_validate_args";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_thread_cache_no_asserts";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_thread_cache_no_asserts_no_validate_args";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache_no_validate_args";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache_no_asserts";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache_no_asserts_no_validate_args";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache_no_thread_cache";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache_no_thread_cache_no_validate_args";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache_no_thread_cache_no_asserts";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_adaptive";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_validate_args_adaptive";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_validate_args";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_asserts_adaptive";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_asserts";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_asserts_no_validate_args_adaptive";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_asserts_no_validate_args";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_thread_cache";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_thread_cache_no_validate_args";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_thread_cache_no_asserts";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_thread_cache_no_asserts_no_validate_args";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_adaptive";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_validate_args_adaptive";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_validate_args";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_asserts_adaptive";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_asserts";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_asserts_no_validate_args_adaptive";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_asserts_no_validate_args";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_thread_cache";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_thread_cache_no_validate_args";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_thread_cache_no_asserts";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_thread_cache";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_thread_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_thread_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache_no_thread_cache";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache_no_thread_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache_no_thread_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_validate_args_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_asserts_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_asserts_no_validate_args_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_thread_cache";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_thread_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_thread_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_validate_args_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_asserts_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_asserts_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_thread_cache";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_asserts";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_thread_cache";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_thread_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_thread_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache_no_thread_cache";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache_no_thread_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache_no_thread_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_validate_args_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_asserts_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_asserts";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_asserts_no_validate_args_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_thread_cache";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_thread_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_thread_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_thread_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_validate_args_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_asserts_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_asserts_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_thread_cache";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_thread_cache_no_validate_args";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_thread_cache_no_asserts";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    bridge :: #library,system "binding_rpmalloc_callbacks_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args";
}
