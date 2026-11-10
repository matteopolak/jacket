Canonical_Builder :: #import "internal/basic-builder-types"(ENABLE_ASSERT = true);

String_Builder :: Canonical_Builder.String_Builder;

#load "print-types.jai";

#add_context print_style: Print_Style;

#add_context inside_memory_debugger: bool;

is_active :: inline () -> bool #no_debug {
    return context.inside_memory_debugger;
}

// What a memory debugger registers so an allocator module can reach it whatever `Basic` it imports. Every debugger
// that is tracking keeps its own, so debuggers from different import parameters coexist.
Debug_Hooks :: struct {
    // An allocator handed out `size` bytes at `memory`.
    acquired: #type (owner: Allocator, memory: *void, size: s64);
    // An allocator took back the range at `memory`.
    released: #type (owner: Allocator, memory: *void);
    // `owner` gave back every range it handed out.
    released_everything: #type (owner: Allocator);
    next: *Debug_Hooks;
    linked: bool;
}

// Adds a debugger's hooks to the ones notified. Safe to call again for the same `hooks`, which must stay valid for the
// rest of the program.
register_hooks :: (hooks: *Debug_Hooks) {
    while atomic_exchange(*hooks_lock, cast(u32) 1) != 0 {}

    if !hooks.linked {
        hooks.next = hooks_head;
        hooks.linked = true;
        atomic_store(*hooks_head, hooks);
    }

    atomic_store(*hooks_lock, cast(u32) 0);
}

// Tells every memory debugger that is tracking, if any, that `owner` gave back every range it handed out.
allocator_released_everything :: (owner: Allocator) {
    if is_active() return;

    hooks := atomic_load(*hooks_head);

    while hooks {
        hooks.released_everything(owner);
        hooks = hooks.next;
    }
}

// Tells every memory debugger that is tracking, if any, that `owner` handed out `size` bytes at `memory` on its own,
// as an allocator does for the blocks it takes from another allocator. Not for ranges a caller got through `alloc`.
allocator_acquired :: (owner: Allocator, memory: *void, size: s64) {
    if is_active() return;

    hooks := atomic_load(*hooks_head);

    while hooks {
        hooks.acquired(owner, memory, size);
        hooks = hooks.next;
    }
}

// The counterpart of `allocator_acquired`: `owner` is about to take back `memory`.
allocator_released :: (owner: Allocator, memory: *void) {
    if is_active() return;

    hooks := atomic_load(*hooks_head);

    while hooks {
        hooks.released(owner, memory);
        hooks = hooks.next;
    }
}

#scope_file
hooks_head: *Debug_Hooks;
hooks_lock: u32;
