// Generated private typed SDK ownership boundary; public records never reach C SDK calls.
#module_parameters (STATISTICS := true, RAW := false, PUBLIC_HEAPS := true, GLOBAL_CACHE := true, THREAD_CACHE := true, ASSERTS := true, VALIDATE_ARGS := true, ADAPTIVE := false)();

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_facade_snapshot_global";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_validate_args_facade_snapshot_global";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_asserts_facade_snapshot_global";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_asserts_no_validate_args_facade_snapshot_global";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_thread_cache_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_thread_cache_facade_snapshot_global";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_validate_args_facade_snapshot_global";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_facade_snapshot_global";
}

#if !RAW && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_global";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_global_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_global_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_global_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_global_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_global_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_global_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_global_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_global_cache_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_global_cache_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_global_cache_facade_snapshot_global";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_global_cache_no_validate_args_facade_snapshot_global";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_facade_snapshot_global";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_global_cache_no_asserts_no_validate_args_facade_snapshot_global";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_facade_snapshot_global";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_global";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_global";
}

#if !RAW && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_global";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_adaptive_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_adaptive_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_validate_args_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_adaptive_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_adaptive_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_asserts_no_validate_args_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_validate_args_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_facade_contains";
}

#if !RAW && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_thread_cache_no_asserts_no_validate_args_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_adaptive_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_adaptive_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_validate_args_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_adaptive_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_asserts_no_validate_args_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_facade_contains";
}

#if !RAW && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_validate_args_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_asserts_no_validate_args_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_validate_args_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_validate_args_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_asserts_no_validate_args_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_base_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_global";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_adaptive_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_adaptive_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_validate_args_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_adaptive_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_adaptive_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_asserts_no_validate_args_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_validate_args_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_thread_cache_no_asserts_no_validate_args_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_adaptive_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_adaptive_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_validate_args_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_adaptive_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_asserts_no_validate_args_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_facade_contains";
}

#if RAW && !PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_base_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_contains";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_validate_args_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_asserts_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_asserts_no_validate_args_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_validate_args_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_validate_args_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_asserts_no_validate_args_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_validate_args_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_contains";
    snapshot_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_create";
    snapshot_release :: (snapshot: *void) #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_release";
    snapshot_count :: (snapshot: *void) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_count";
    snapshot_header :: (snapshot: *void, heap: u64, field: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_header";
    snapshot_counters :: (snapshot: *void, heap: u64, family: u32) -> *s32 #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_counters";
    snapshot_global :: (snapshot: *void) -> *SDK.rpmalloc_global_statistics_t #foreign sdk "binding_rpmalloc_raw_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_snapshot_global";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_adaptive_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_adaptive_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_validate_args_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_adaptive_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_adaptive_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_asserts_no_validate_args_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_validate_args_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_thread_cache_no_asserts_no_validate_args_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_adaptive_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_adaptive_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_validate_args_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_adaptive_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_adaptive_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_asserts_no_validate_args_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_validate_args_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_facade_contains";
}

#if RAW && PUBLIC_HEAPS && !STATISTICS && !GLOBAL_CACHE && !THREAD_CACHE && !ASSERTS && !VALIDATE_ARGS && !ADAPTIVE {
    scope_enter :: (domain: *void, scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_enter";
    scope_leave :: (scope: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_scope_leave";
    domain_create :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_create";
    domain_destroy :: (domain: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_destroy";
    domain_prepare_thread :: (domain: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_domain_prepare_thread";
    select_domain :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_select_domain";
    restore_domain :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_restore_domain";
    configure_live :: (provider: #type (huge: s32, bytes: **u8, length: *u64) #c_call) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_live";
    name_copy :: (huge: u32) -> *u8 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_name_copy";
    default_map :: (size: u64, offset: *u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_map";
    default_unmap :: (address: *void, size: u64, offset: u64, full: u64) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_unmap";
    default_error :: (message: *u8) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_default_error";
    configure_policy :: (policy: u32) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_policy";
    configure_dimensions :: (heaps: u64, map_count: u64, cache_multiplier: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_configure_dimensions";
    build_flags :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_build_flags";
    enter :: (domain: *void) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_enter";
    leave :: (previous: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_leave";
    pause :: () -> u32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_pause";
    resume :: (depth: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_resume";
    reset :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_reset";
    install :: (advice: u32) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_install";
    thread_mutation :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_mutation";
    end_mutation :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_end_mutation";
    thread_finalized :: (token: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_thread_finalized";
    require_quiescent :: () #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_require_quiescent";
    valid_request :: (size: u64, alignment: u64) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_valid_request";
    free_any :: (memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_any";
    native_heap :: (heap: *void) -> *SDK.rpmalloc_heap_t #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_native_heap";
    layout_size :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_size";
    layout_offset :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_offset";
    layout_width :: (index: u32) -> u64 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_layout_width";
    acquire :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_acquire";
    current :: () -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_current";
    release :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_release";
    alloc :: (heap: *void, alignment: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_alloc";
    calloc :: (heap: *void, alignment: u64, num: u64, size: u64) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_calloc";
    realloc :: (heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_realloc";
    free :: (heap: *void, memory: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free";
    free_all :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_free_all";
    set_current :: (heap: *void) #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_set_current";
    contains :: (heap: *void, memory: *void, span: *void) -> s32 #foreign sdk "binding_rpmalloc_raw_nostats_no_global_cache_no_thread_cache_no_asserts_no_validate_args_facade_contains";
}

#scope_file
SDK :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = STATISTICS, RAW = RAW, PUBLIC_HEAPS = PUBLIC_HEAPS, GLOBAL_CACHE = GLOBAL_CACHE, THREAD_CACHE = THREAD_CACHE, ASSERTS = ASSERTS, VALIDATE_ARGS = VALIDATE_ARGS, ADAPTIVE = ADAPTIVE);

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
