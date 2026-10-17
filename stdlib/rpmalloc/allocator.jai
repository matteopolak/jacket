// Generic allocator protocol uses the canonical bootstrap records and modes.
#scope_file
Domain :: #import "internal/bindings/rpmalloc-domain"(Owner = rpmalloc_config_t, STATISTICS = ENABLE_STATISTICS, RAW = USE_RAW, PUBLIC_HEAPS = !USE_RAW || Allocator_Settings.FIRST_CLASS_HEAPS, GLOBAL_CACHE = ENABLE_GLOBAL_CACHE, THREAD_CACHE = ENABLE_THREAD_CACHE, ASSERTS = ENABLE_ASSERTS, VALIDATE_ARGS = ENABLE_VALIDATE_ARGS, ADAPTIVE = ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS);
Allocator_Basic :: #import "Basic";
Allocator_Settings :: #import "internal/bindings/rpmalloc-build";
Allocator_Frames :: #import "internal/bindings/callback-context";
Allocator_Bridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = ENABLE_STATISTICS, RAW = USE_RAW, PUBLIC_HEAPS = !USE_RAW || Allocator_Settings.FIRST_CLASS_HEAPS, GLOBAL_CACHE = ENABLE_GLOBAL_CACHE, THREAD_CACHE = ENABLE_THREAD_CACHE, ASSERTS = ENABLE_ASSERTS, VALIDATE_ARGS = ENABLE_VALIDATE_ARGS, ADAPTIVE = ENABLE_ADAPTIVE_THREAD_CACHE && !ENABLE_STATISTICS);

// Exact CAPS identity; static borrowed storage survives all calls.
allocator_name: string = "rpmalloc 1.4.4";

#scope_export
rpmalloc_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    previous := Allocator_Frames.enter(cast(*void) *context);

    defer Allocator_Frames.leave(previous);

    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = allocator_name;
        }

        caps: Allocator_Caps = .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR | .MULTIPLE_THREADS |
                              .FREE | .ACTUALLY_RESIZE | .CREATE_HEAP;

        return cast(*void) cast(u64) caps;
    }

    if mode == .STARTUP {
        // The generic callback explicitly ignores this result.
        initialize();
        return null;
    }

    if mode == .SHUTDOWN {
        finalize();
        return null;
    }

    if mode == .THREAD_START {
        if is_initialized() thread_initialize();
        return null;
    }

    if mode == .THREAD_STOP {
        if is_initialized() thread_finalize(true);
        return null;
    }

    if !is_initialized() && (mode == .CREATE_HEAP || mode == .IS_THIS_YOURS ||
                            mode == .ALLOCATE || mode == .RESIZE) return null;
    if mode == .FREE && old_memory == null return null;

    domain_scope: Domain.Scope;

    Domain.enter(*domain_scope);

    defer Domain.leave(*domain_scope);

    if mode == .CREATE_HEAP {
        if !is_initialized() return null;
        return Allocator_Bridge.acquire();
    }

    if mode == .DESTROY_HEAP {
        Allocator_Basic.assert(allocator_data != null, "the default rpmalloc heap cannot be destroyed");
        Allocator_Bridge.release(allocator_data);
        return null;
    }

    if mode == .IS_THIS_YOURS {
        if !is_initialized() return null;
        selected := allocator_data;
        if !selected selected = Allocator_Bridge.current();
        return cast(*void) cast(u64) Allocator_Bridge.contains(selected, old_memory, null);
    }

    if mode == .FREE {
        Allocator_Bridge.free(allocator_data, old_memory);
        return null;
    }

    if mode == .ALLOCATE || mode == .RESIZE {
        if requested_size < 0 || !is_initialized() return null;

        if allocator_data {
            if mode == .ALLOCATE return Allocator_Bridge.alloc(allocator_data, 0, cast(u64) requested_size);
            return Allocator_Bridge.realloc(allocator_data, old_memory, 0, cast(u64) requested_size, 0);
        }

        #if USE_RAW {
            if mode == .ALLOCATE return rpmalloc(cast(u64) requested_size);
            return rprealloc(old_memory, cast(u64) requested_size);
        } else {
            if mode == .ALLOCATE return rpmalloc(requested_size);
            return rprealloc(old_memory, requested_size);
        }
    }

    Allocator_Basic.assert(false, "invalid rpmalloc Allocator_Mode");

    return null;
}
