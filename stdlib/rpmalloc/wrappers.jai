// Public allocation boundary. Native SDK records remain private.
#scope_file
Domain :: #import "internal/bindings/rpmalloc-domain"(Owner = rpmalloc_config_t, STATISTICS = ENABLE_STATISTICS, RAW = USE_RAW, PUBLIC_HEAPS = !USE_RAW || Build_Settings.FIRST_CLASS_HEAPS, GLOBAL_CACHE = ENABLE_GLOBAL_CACHE, THREAD_CACHE = ENABLE_THREAD_CACHE, ASSERTS = ENABLE_ASSERTS, VALIDATE_ARGS = ENABLE_VALIDATE_ARGS, ADAPTIVE = ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS);
Build_Settings :: #import "internal/bindings/rpmalloc-build";

Public_Heap_Profile :: !USE_RAW || Build_Settings.FIRST_CLASS_HEAPS;

Native :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = ENABLE_STATISTICS, RAW = USE_RAW, PUBLIC_HEAPS = Public_Heap_Profile, GLOBAL_CACHE = ENABLE_GLOBAL_CACHE, THREAD_CACHE = ENABLE_THREAD_CACHE, ASSERTS = ENABLE_ASSERTS, VALIDATE_ARGS = ENABLE_VALIDATE_ARGS, ADAPTIVE = ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS);
Callbacks :: #import "internal/bindings/rpmalloc-callbacks"(STATISTICS = ENABLE_STATISTICS, RAW = USE_RAW, PUBLIC_HEAPS = Public_Heap_Profile, GLOBAL_CACHE = ENABLE_GLOBAL_CACHE, THREAD_CACHE = ENABLE_THREAD_CACHE, ASSERTS = ENABLE_ASSERTS, VALIDATE_ARGS = ENABLE_VALIDATE_ARGS, ADAPTIVE = ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS);
Checked :: #import "internal/bindings/rpmalloc-checked"(STATISTICS = ENABLE_STATISTICS, RAW = USE_RAW, PUBLIC_HEAPS = Public_Heap_Profile, GLOBAL_CACHE = ENABLE_GLOBAL_CACHE, THREAD_CACHE = ENABLE_THREAD_CACHE, ASSERTS = ENABLE_ASSERTS, VALIDATE_ARGS = ENABLE_VALIDATE_ARGS, ADAPTIVE = ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS);
NativeContext :: #import "internal/bindings/callback-context";
HeapBridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = ENABLE_STATISTICS, RAW = USE_RAW, PUBLIC_HEAPS = Public_Heap_Profile, GLOBAL_CACHE = ENABLE_GLOBAL_CACHE, THREAD_CACHE = ENABLE_THREAD_CACHE, ASSERTS = ENABLE_ASSERTS, VALIDATE_ARGS = ENABLE_VALIDATE_ARGS, ADAPTIVE = ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS);

#if USE_RAW {
    Boundary_Size :: u64;
} else {
    Boundary_Size :: s64;
}

initialized: bool;
active_config: rpmalloc_config_t;

// All reviewed option combinations have a private storage/policy route.
// Native build identity and live domain ownership are checked before mutation.
#scope_file
boundary_initialize :: () -> s32 {
    return boundary_initialize_config(null);
}

boundary_initialize_config :: (boundary_config: *rpmalloc_config_t) -> s32 {
    expected_flags: u32 = 0x80000 | (cast(u32) ENABLE_STATISTICS) | ((cast(u32) USE_RAW) << 1) |
                      ((cast(u32) Public_Heap_Profile) << 2) | ((cast(u32) ENABLE_GLOBAL_CACHE) << 3) |
                      ((cast(u32) ENABLE_THREAD_CACHE) << 4) | ((cast(u32) ENABLE_ASSERTS) << 5) |
                      ((cast(u32) ENABLE_VALIDATE_ARGS) << 6) |
                      ((cast(u32) (ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS)) << 7);
    if HeapBridge.build_flags() != expected_flags return 22;

    if !Domain.ensure() return 12;

    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);

    if initialized {
        Native.rpmalloc_thread_initialize();
        return 0;
    }

    dimensions := HeapBridge.configure_dimensions(cast(u64) HEAP_ARRAY_SIZE,
                                                  cast(u64) DEFAULT_SPAN_MAP_COUNT,
                                                  cast(u64) GLOBAL_CACHE_MULTIPLIER);
    if dimensions != 0 return dimensions;

    policy: u32 = (cast(u32) LINUX_HUGE_PAGES) | ((cast(u32) ENABLE_UNLIMITED_CACHE) << 1) |
                  ((cast(u32) DISABLE_UNMAP) << 2);
    policy_result := HeapBridge.configure_policy(policy);

    if policy_result != 0 return policy_result;

    live_result := HeapBridge.configure_live(callback_name);
    if live_result != 0 return live_result;

    active_config = .{};
    native: Native.rpmalloc_config_t;

    if boundary_config {
        if CONFIGURABLE {
            page := boundary_config.page_size;
            span := boundary_config.span_size;

            #if !USE_RAW {
                if page < 0 || boundary_config.span_map_count < 0 return 22;
            }

            if page != 0 && (page & (page - 1)) != 0 return 22;
            if span != 0 && (span < 4096 || span > 262144 || (span & (span - 1)) != 0) return 22;

            native.page_size = cast(u64) page;
            native.span_size = cast(u64) span;
            native.span_map_count = cast(u64) boundary_config.span_map_count;
        }

        active_config = boundary_config.*;

        #if USE_RAW {
            native.enable_huge_pages = boundary_config.enable_huge_pages;
        } else {
            native.enable_huge_pages = ifx boundary_config.enable_huge_pages then 1 else 0;
        }
    }

    if !active_config.memory_map || !active_config.memory_unmap {
        active_config.memory_map = default_map;
        active_config.memory_unmap = default_unmap;
    }

    native.memory_map = callback_map;
    native.memory_unmap = callback_unmap;
    native.map_fail_callback = callback_failure;
    native.error_callback = callback_error;

    result := Callbacks.initialize(*native, active_config.page_name.data,
                                 cast(u64) active_config.page_name.count,
                                 active_config.huge_page_name.data,
                                 cast(u64) active_config.huge_page_name.count);
    if result == 0 {
        // The reviewed fallback/page-aligned batch is observable through
        // the module-owned configuration, not merely the private SDK copy.
        active_config.span_map_count = cast(Boundary_Size) Native.rpmalloc_config().span_map_count;
        active_config.page_size = cast(Boundary_Size) Native.rpmalloc_config().page_size;
        active_config.span_size = cast(Boundary_Size) Native.rpmalloc_config().span_size;

        #if USE_RAW {
            active_config.enable_huge_pages = Native.rpmalloc_config().enable_huge_pages;
        } else {
            active_config.enable_huge_pages = Native.rpmalloc_config().enable_huge_pages != 0;
        }

        HeapBridge.install(cast(u32) LINUX_FREE_ADVICE);
        initialized = true;
    } else active_config = .{};

    return result;
}

boundary_finalize :: () {
    if !initialized return;

    defer Domain.destroy();

    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);
    HeapBridge.require_quiescent();
    Callbacks.finalize();
    HeapBridge.reset();
    initialized = false;
    active_config = .{};
}

boundary_is_initialized :: inline () -> bool {
    return initialized;
}

boundary_config :: () -> *rpmalloc_config_t {
    return *active_config;
}

boundary_thread_initialize :: inline () {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);
    Native.rpmalloc_thread_initialize();
}

boundary_set_main_thread :: () {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);
    Native.rpmalloc_set_main_thread();
}

boundary_thread_finalize :: (release_caches: bool) {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);
    mutation := HeapBridge.thread_mutation();

    defer HeapBridge.thread_finalized(mutation);
    Native.rpmalloc_thread_finalize(ifx release_caches then 1 else 0);
}

boundary_rpmalloc :: inline (size: Boundary_Size) -> *void {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);

    #if !USE_RAW {
        if size < 0 return null;
    }

    if HeapBridge.valid_request(cast(u64) size, 0) == 0 return null;

    mutation := HeapBridge.thread_mutation();

    defer HeapBridge.end_mutation(mutation);

    return Native.rpmalloc(cast(u64) size);
}

boundary_rprealloc :: inline (ptr: *void, size: Boundary_Size) -> *void {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);

    #if !USE_RAW {
        if size < 0 return null;
    }

    if HeapBridge.valid_request(cast(u64) size, 0) == 0 return null;

    mutation := HeapBridge.thread_mutation();

    defer HeapBridge.end_mutation(mutation);

    return Native.rprealloc(ptr, cast(u64) size);
}

boundary_rpaligned_alloc :: (alignment: Boundary_Size, size: Boundary_Size) -> *void {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);

    #if !USE_RAW {
        if size < 0 || alignment < 0 return null;
    }

    if alignment == 0 return null;
    if alignment & (alignment - 1) != 0 return null;
    if HeapBridge.valid_request(cast(u64) size, cast(u64) alignment) == 0 return null;

    mutation := HeapBridge.thread_mutation();

    defer HeapBridge.end_mutation(mutation);

    return Native.rpaligned_alloc(cast(u64) alignment, cast(u64) size);
}

boundary_rpaligned_realloc :: (ptr: *void, alignment: Boundary_Size, size: Boundary_Size, oldsize: Boundary_Size, flags: Realloc_Flags) -> *void {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);

    #if !USE_RAW {
        if size < 0 || oldsize < 0 || alignment < 0 return null;
    }

    if alignment == 0 return null;
    if alignment & (alignment - 1) != 0 return null;
    if HeapBridge.valid_request(cast(u64) size, cast(u64) alignment) == 0 return null;

    mutation := HeapBridge.thread_mutation();

    defer HeapBridge.end_mutation(mutation);

    return Native.rpaligned_realloc(ptr, cast(u64) alignment, cast(u64) size, cast(u64) oldsize, cast(u32) flags);
}

boundary_usable_size :: inline (ptr: *void) -> Boundary_Size {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);

    return cast(Boundary_Size) Native.rpmalloc_usable_size(ptr);
}

boundary_is_thread_initialized :: () -> bool {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);

    return Native.rpmalloc_is_thread_initialized() != 0;
}

boundary_global_statistics :: (stats: *rpmalloc_global_statistics_t) {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);
    native: Native.rpmalloc_global_statistics_t;

    Native.rpmalloc_global_statistics(*native);
    stats.mapped = cast(Boundary_Size) native.mapped;
    stats.mapped_peak = cast(Boundary_Size) native.mapped_peak;
    stats.cached = cast(Boundary_Size) native.cached;
    stats.huge_alloc = cast(Boundary_Size) native.huge_alloc;
    stats.huge_alloc_peak = cast(Boundary_Size) native.huge_alloc_peak;
    stats.mapped_total = cast(Boundary_Size) native.mapped_total;
    stats.unmapped_total = cast(Boundary_Size) native.unmapped_total;
}

boundary_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);
    native: Native.rpmalloc_thread_statistics_t;

    Native.rpmalloc_thread_statistics(*native);
    stats.sizecache = cast(Boundary_Size) native.sizecache;
    stats.spancache = cast(Boundary_Size) native.spancache;
    stats.thread_to_global = cast(Boundary_Size) native.thread_to_global;
    stats.global_to_thread = cast(Boundary_Size) native.global_to_thread;

    for 0..63 {
        source := *native.span_use[it];
        target := *stats.span_use[it];
        target.current = cast(Boundary_Size) source.current;
        target.peak = cast(Boundary_Size) source.peak;
        target.to_global = cast(Boundary_Size) source.to_global;
        target.from_global = cast(Boundary_Size) source.from_global;
        target.to_cache = cast(Boundary_Size) source.to_cache;
        target.from_cache = cast(Boundary_Size) source.from_cache;
        target.to_reserved = cast(Boundary_Size) source.to_reserved;
        target.from_reserved = cast(Boundary_Size) source.from_reserved;
        target.map_calls = cast(Boundary_Size) source.map_calls;
    }

    for 0..127 {
        source := *native.size_use[it];
        target := *stats.size_use[it];
        target.alloc_current = cast(Boundary_Size) source.alloc_current;
        target.alloc_peak = cast(Boundary_Size) source.alloc_peak;
        target.alloc_total = cast(Boundary_Size) source.alloc_total;
        target.free_total = cast(Boundary_Size) source.free_total;
        target.spans_to_cache = cast(Boundary_Size) source.spans_to_cache;
        target.spans_from_cache = cast(Boundary_Size) source.spans_from_cache;
        target.spans_from_reserved = cast(Boundary_Size) source.spans_from_reserved;
        target.map_calls = cast(Boundary_Size) source.map_calls;
    }
}

boundary_rpfree :: inline (ptr: *void) {
    previous_context := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous_context);
    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);
    HeapBridge.free_any(ptr);
}

#scope_file
callback_map :: (size: u64, offset: *u64) -> *void #c_call {
    paused := HeapBridge.pause();

    defer HeapBridge.resume(paused);
    caller := cast(*#Context) NativeContext.current();
    if caller {
        push_context caller.* {
            return invoke_map(size, offset);
        }
    }

    push_context, defer_pop;

    return invoke_map(size, offset);
}

invoke_map :: (size: u64, offset: *u64) -> *void {
    #if !USE_RAW {
        if size > 0x7fff_ffff_ffff_ffff return null;
    }

    public_offset: Boundary_Size;
    result := active_config.memory_map(cast(Boundary_Size) size, *public_offset);

    offset.* = cast(u64) public_offset;

    return result;
}

callback_unmap :: (address: *void, size: u64, offset: u64, full: u64) #c_call {
    paused := HeapBridge.pause();

    defer HeapBridge.resume(paused);
    caller := cast(*#Context) NativeContext.current();
    if caller {
        push_context caller.* {
            invoke_unmap(address, size, offset, full);
        }

        return;
    }

    push_context, defer_pop;
    invoke_unmap(address, size, offset, full);
}

invoke_unmap :: (address: *void, size: u64, offset: u64, full: u64) {
    active_config.memory_unmap(address, cast(Boundary_Size) size, cast(Boundary_Size) offset, cast(Boundary_Size) full);
}

callback_failure :: (size: u64) -> s32 #c_call {
    paused := HeapBridge.pause();

    defer HeapBridge.resume(paused);
    caller := cast(*#Context) NativeContext.current();
    if caller {
        push_context caller.* {
            return invoke_failure(size);
        }
    }

    push_context, defer_pop;

    return invoke_failure(size);
}

invoke_failure :: (size: u64) -> s32 {
    #if !USE_RAW {
        if size > 0x7fff_ffff_ffff_ffff return 0;
    }

    if !active_config.map_fail_callback return 0;

    return active_config.map_fail_callback(cast(Boundary_Size) size);
}

callback_error :: (text: *u8) #c_call {
    paused := HeapBridge.pause();

    defer HeapBridge.resume(paused);
    caller := cast(*#Context) NativeContext.current();
    if caller {
        push_context caller.* {
            invoke_error(text);
        }

        return;
    }

    push_context, defer_pop;
    invoke_error(text);
}

invoke_error :: (text: *u8) {
    if !active_config.error_callback {
        HeapBridge.default_error(text);
        return;
    }

    message: string;
    message.data = text;

    if text {
        while text[message.count] != 0 message.count += 1;
    }

    #if USE_RAW {
        active_config.error_callback(message);
    } else {
        active_config.error_callback(message, #location);
    }
}

default_map :: (size: Boundary_Size, offset: *Boundary_Size) -> *void {
    previous_domain := Domain.select();

    defer Domain.restore(previous_domain);
    previous := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous);
    native_offset: u64;
    result := HeapBridge.default_map(cast(u64) size, *native_offset);

    offset.* = cast(Boundary_Size) native_offset;

    return result;
}

default_unmap :: (address: *void, size: Boundary_Size, offset: Boundary_Size, full: Boundary_Size) {
    previous_domain := Domain.select();

    defer Domain.restore(previous_domain);
    previous := NativeContext.enter(cast(*void) *context);

    defer NativeContext.leave(previous);
    HeapBridge.default_unmap(address, cast(u64) size, cast(u64) offset, cast(u64) full);
}

callback_name :: (huge: s32, bytes: **u8, length: *u64) #c_call {
    name := ifx huge != 0 then active_config.huge_page_name else active_config.page_name;
    bytes.* = name.data;
    length.* = cast(u64) name.count;
}

#scope_export
#if !USE_RAW {
    heap_acquire :: inline () -> *Heap {
        if !initialized return null;

        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);

        return cast(*Heap) HeapBridge.acquire();
    }

    heap_release :: inline (heap: *Heap) {
        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);
        HeapBridge.release(cast(*void) heap);
    }

    heap_alloc :: inline (heap: *Heap, size: s64) -> *void {
        if size < 0 return null;

        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);

        return HeapBridge.alloc(cast(*void) heap, 0, cast(u64) size);
    }

    heap_aligned_alloc :: inline (heap: *Heap, alignment: s64, size: s64) -> *void {
        if size < 0 || alignment <= 0 || (alignment & (alignment - 1)) != 0 return null;

        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);

        return HeapBridge.alloc(cast(*void) heap, cast(u64) alignment, cast(u64) size);
    }

    heap_realloc :: inline (heap: *Heap, ptr: *void, size: s64, flags: Realloc_Flags) -> *void {
        if size < 0 return null;

        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);

        return HeapBridge.realloc(cast(*void) heap, ptr, 0, cast(u64) size, cast(u32) flags);
    }

    heap_aligned_realloc :: inline (heap: *Heap, ptr: *void, alignment: s64, size: s64, flags: Realloc_Flags) -> *void {
        if size < 0 || alignment <= 0 || (alignment & (alignment - 1)) != 0 return null;

        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);

        return HeapBridge.realloc(cast(*void) heap, ptr, cast(u64) alignment, cast(u64) size, cast(u32) flags);
    }

    heap_free :: inline (heap: *Heap, ptr: *void) {
        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);
        HeapBridge.free(cast(*void) heap, ptr);
    }

    heap_free_all :: inline (heap: *Heap) {
        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);
        HeapBridge.free_all(cast(*void) heap);
    }

    heap_thread_set_current :: inline (heap: *Heap) {
        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);
        HeapBridge.set_current(cast(*void) heap);
    }

    heap_is_this_yours :: (heap: *Heap, memory: *void) -> bool {
        if !heap return false;

        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);

        return HeapBridge.contains(cast(*void) heap, memory, null) != 0;
    }

    is_in_span :: inline (memory: *void, span: *Span) -> bool {
        if !span return false;

        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);

        return HeapBridge.contains(null, memory, cast(*void) span) != 0;
    }
}

#if !USE_RAW {
    initialize :: () -> s32 {
        return boundary_initialize();
    }

    initialize_config :: (config: *rpmalloc_config_t) -> s32 {
        return boundary_initialize_config(config);
    }

    finalize :: () {
        boundary_finalize();
    }

    is_initialized :: inline () -> bool {
        return boundary_is_initialized();
    }

    config :: () -> *rpmalloc_config_t {
        return boundary_config();
    }

    thread_initialize :: inline () {
        boundary_thread_initialize();
    }

    set_main_thread :: () {
        boundary_set_main_thread();
    }

    thread_finalize :: (release_caches: bool) {
        boundary_thread_finalize(release_caches);
    }

    is_thread_initialized :: () -> bool {
        return boundary_is_thread_initialized();
    }

    rpmalloc :: inline (size: s64) -> *void {
        return boundary_rpmalloc(size);
    }

    rpfree :: inline (ptr: *void) {
        boundary_rpfree(ptr);
    }

    rprealloc :: inline (ptr: *void, size: s64) -> *void {
        return boundary_rprealloc(ptr, size);
    }

    rpaligned_alloc :: (alignment: s64, size: s64) -> *void {
        return boundary_rpaligned_alloc(alignment, size);
    }

    rpaligned_realloc :: (ptr: *void, alignment: s64, size: s64, oldsize: s64, flags: Realloc_Flags) -> *void {
        return boundary_rpaligned_realloc(ptr, alignment, size, oldsize, flags);
    }

    usable_size :: inline (ptr: *void) -> s64 {
        return boundary_usable_size(ptr);
    }

    global_statistics :: (stats: *rpmalloc_global_statistics_t) {
        boundary_global_statistics(stats);
    }

    thread_statistics :: (stats: *rpmalloc_thread_statistics_t) {
        boundary_thread_statistics(stats);
    }
}

#if USE_RAW {
    #if RPMALLOC_FIRST_CLASS_HEAPS {
        rpmalloc_heap_acquire :: inline () -> *rpmalloc_heap_t {
            if !initialized return null;

            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);

            return cast(*rpmalloc_heap_t) HeapBridge.acquire();
        }

        rpmalloc_heap_release :: inline (heap: *rpmalloc_heap_t) {
            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);
            HeapBridge.release(cast(*void) heap);
        }

        rpmalloc_heap_alloc :: inline (heap: *rpmalloc_heap_t, size: size_t) -> *void {
            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);

            return HeapBridge.alloc(cast(*void) heap, 0, size);
        }

        rpmalloc_heap_aligned_alloc :: inline (heap: *rpmalloc_heap_t, alignment: size_t, size: size_t) -> *void {
            if alignment == 0 || alignment & (alignment - 1) != 0 return null;

            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);

            return HeapBridge.alloc(cast(*void) heap, alignment, size);
        }

        rpmalloc_heap_calloc :: inline (heap: *rpmalloc_heap_t, num: size_t, size: size_t) -> *void {
            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);

            return HeapBridge.calloc(cast(*void) heap, 0, num, size);
        }

        rpmalloc_heap_aligned_calloc :: inline (heap: *rpmalloc_heap_t, alignment: size_t, num: size_t, size: size_t) -> *void {
            if alignment == 0 || alignment & (alignment - 1) != 0 return null;

            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);

            return HeapBridge.calloc(cast(*void) heap, alignment, num, size);
        }

        rpmalloc_heap_realloc :: inline (heap: *rpmalloc_heap_t, ptr: *void, size: size_t, flags: Realloc_Flags) -> *void {
            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);

            return HeapBridge.realloc(cast(*void) heap, ptr, 0, size, cast(u32) flags);
        }

        rpmalloc_heap_aligned_realloc :: inline (heap: *rpmalloc_heap_t, ptr: *void, alignment: size_t, size: size_t, flags: Realloc_Flags) -> *void {
            if alignment == 0 || alignment & (alignment - 1) != 0 return null;

            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);

            return HeapBridge.realloc(cast(*void) heap, ptr, alignment, size, cast(u32) flags);
        }

        rpmalloc_heap_free :: inline (heap: *rpmalloc_heap_t, ptr: *void) {
            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);
            HeapBridge.free(cast(*void) heap, ptr);
        }

        rpmalloc_heap_free_all :: inline (heap: *rpmalloc_heap_t) {
            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);
            HeapBridge.free_all(cast(*void) heap);
        }

        rpmalloc_heap_thread_set_current :: inline (heap: *rpmalloc_heap_t) {
            previous := NativeContext.enter(cast(*void) *context);

            defer NativeContext.leave(previous);
            domain_scope: Domain.Scope;

            Domain.enter(*domain_scope);

            defer Domain.leave(*domain_scope);
            HeapBridge.set_current(cast(*void) heap);
        }
    }

    rpmalloc_initialize :: () -> _int {
        return boundary_initialize();
    }

    rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> _int {
        return boundary_initialize_config(config);
    }

    rpmalloc_finalize :: () {
        boundary_finalize();
    }

    rpmalloc_is_initialized :: inline () -> bool {
        return boundary_is_initialized();
    }

    rpmalloc_config :: () -> *rpmalloc_config_t {
        return boundary_config();
    }

    rpmalloc_thread_initialize :: inline () {
        boundary_thread_initialize();
    }

    rpmalloc_set_main_thread :: () {
        boundary_set_main_thread();
    }

    rpmalloc_thread_finalize :: inline (release_caches: bool) {
        boundary_thread_finalize(release_caches);
    }

    rpmalloc_is_thread_initialized :: () -> bool {
        return boundary_is_thread_initialized();
    }

    rpmalloc :: inline (size: size_t) -> *void {
        return boundary_rpmalloc(size);
    }

    rpfree :: inline (ptr: *void) {
        boundary_rpfree(ptr);
    }

    rprealloc :: inline (ptr: *void, size: size_t) -> *void {
        return boundary_rprealloc(ptr, size);
    }

    rpaligned_alloc :: (alignment: size_t, size: size_t) -> *void {
        return boundary_rpaligned_alloc(alignment, size);
    }

    rpaligned_realloc :: (ptr: *void, alignment: size_t, size: size_t, oldsize: size_t, flags: Realloc_Flags) -> *void {
        return boundary_rpaligned_realloc(ptr, alignment, size, oldsize, flags);
    }

    rpmalloc_usable_size :: inline (ptr: *void) -> size_t {
        return boundary_usable_size(ptr);
    }

    rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t) {
        boundary_global_statistics(stats);
    }

    rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t) {
        boundary_thread_statistics(stats);
    }

    rpmalloc_thread_finalize :: (release_caches: _int) {
        boundary_thread_finalize(release_caches != 0);
    }

    rpcalloc :: inline (num: size_t, size: size_t) -> *void {
        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);

        if num != 0 && size > 0xffff_ffff_ffff_ffff / num return null;
        if HeapBridge.valid_request(num * size, 0) == 0 return null;

        mutation := HeapBridge.thread_mutation();

        defer HeapBridge.end_mutation(mutation);

        return Native.rpcalloc(num, size);
    }

    rpaligned_calloc :: inline (alignment: size_t, num: size_t, size: size_t) -> *void {
        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);
        if alignment == 0 || alignment & (alignment - 1) != 0 return null;

        if num != 0 && size > 0xffff_ffff_ffff_ffff / num return null;
        if HeapBridge.valid_request(num * size, alignment) == 0 return null;

        mutation := HeapBridge.thread_mutation();

        defer HeapBridge.end_mutation(mutation);

        return Native.rpaligned_calloc(alignment, num, size);
    }

    rpmemalign :: inline (alignment: size_t, size: size_t) -> *void {
        return boundary_rpaligned_alloc(alignment, size);
    }

    rpposix_memalign :: inline (memptr: **void, alignment: size_t, size: size_t) -> _int {
        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);
        if HeapBridge.valid_request(size, alignment) == 0 return 22;

        mutation := HeapBridge.thread_mutation();

        defer HeapBridge.end_mutation(mutation);

        return Checked.posix_memalign(memptr, alignment, size);
    }

    rpmalloc_thread_collect :: inline () {
        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);
        Native.rpmalloc_thread_collect();
    }

    rpmalloc_dump_statistics :: (file: *void) {
        previous := NativeContext.enter(cast(*void) *context);

        defer NativeContext.leave(previous);
        domain_scope: Domain.Scope;

        Domain.enter(*domain_scope);

        defer Domain.leave(*domain_scope);

        #if ENABLE_STATISTICS && Build_Settings.PRETTY_PRINT {
            Native.rpmalloc_dump_statistics(file);
        }
    }

    rpmalloc_linker_reference :: () {
        Native.rpmalloc_linker_reference();
    }
}
