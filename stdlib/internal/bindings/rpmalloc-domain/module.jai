// Canonical nominal owner identity separates allocator instances without
// numeric library-name matrices or public record/layout changes.
#module_parameters (Owner: Type, STATISTICS := true, RAW := false, PUBLIC_HEAPS := true, GLOBAL_CACHE := true, THREAD_CACHE := true, ASSERTS := true, VALIDATE_ARGS := true, ADAPTIVE := false)();
#assert size_of(Owner) > 0;
Domain_Handle :: struct {}
handle: *Domain_Handle;

// Private C operation protocol; never a public or native SDK record.
Scope :: struct {
    slot_domain: *void;
    slot_heap: *void;
    slot_depth: u32;
    slot_generation: u64;
    slot_next: *void;
    slot_redirect: *void;
    previous_slot: *void;
    previous_domain: *void;
    previous_scope: *void;
    borrowed: u32;
}

#assert size_of(Scope) == 80;

ensure :: () -> bool {
    if !handle handle = cast(*Domain_Handle) Bridge.domain_create();
    return handle != null;
}

enter :: (scope: *Scope) {
    if !handle abort();
    Bridge.scope_enter(cast(*void) handle, scope);
}

leave :: (scope: *Scope) {
    Bridge.scope_leave(scope);
}

select :: () -> *void {
    if !ensure() abort();
    return Bridge.select_domain(cast(*void) handle);
}

restore :: (previous: *void) {
    Bridge.restore_domain(previous);
}

destroy :: () {
    Bridge.domain_destroy(cast(*void) handle);
    handle = null;
}

#scope_file
Bridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = STATISTICS, RAW = RAW, PUBLIC_HEAPS = PUBLIC_HEAPS, GLOBAL_CACHE = GLOBAL_CACHE, THREAD_CACHE = THREAD_CACHE, ASSERTS = ASSERTS, VALIDATE_ARGS = VALIDATE_ARGS, ADAPTIVE = ADAPTIVE);

libc :: #library,system "c";
abort :: () #foreign libc;
