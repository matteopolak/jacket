// Private lifecycle/access protocol shared by every Bindings_Generator instance.
// This stores ownership and activation only; canonical model state is Generator.
Owned_Allocation :: struct {
    memory: *void;
    bytes: s64;
}

Owned_Array :: struct {
    array: *Resizable_Array;
}

Scope_State :: struct {
    declaration: *void;
    stage: enum u8 {
        SHELL;
        BUILDING;
        TYPES_ONLY;
        FULL;
        REJECTED;
    };
}

Source_Overlay :: struct {
    filename: string;
    contents: string;
}

Parse_Input :: struct {
    filename: string;
    arguments: [] string;
    overlays: [] Source_Overlay;
}

Session :: struct {
    activate :: session_activate;
    active_generator :: session_active_generator;
    active_options :: session_active_options;
    open :: session_open;
    current :: session_current;
    new_owned :: session_new_owned;
    copy_text :: session_copy_text;
    append_owned :: session_append_owned;
    append_movable_owned :: session_append_movable_owned;
    copy_array :: session_copy_array;
    close :: session_close;
    generator: *void;
    target_os: Operating_System_Tag;
    allocator: Allocator;
    allocations: [..] Owned_Allocation;
    arrays: [..] Owned_Array;
    scope_states: [..] Scope_State;
    parse_input: *Parse_Input;
    lifecycle_owned: bool;
    // Sticky native/input failure is separate from local semantic rejection.
    failed: bool;
    model_errors: u64;
    macro_lookup_depth: int;
    live: bool;
}

Canonical_Generator :: #import "Bindings_Generator";

#add_context generator: *Canonical_Generator.Generator;
#add_context _bindings_generator_options: *void;
#add_context _bindings_generator_session: *Session;

activate :: session_activate;

session_activate :: (generator: *void, options: *void = null, owner: *Session = null) {
    assert(generator != null);
    assert(!owner || (owner.live && owner.generator == generator));
    context.generator = cast(*Canonical_Generator.Generator) generator;
    context._bindings_generator_options = options;
    context._bindings_generator_session = owner;
}

active_generator :: session_active_generator;

session_active_generator :: () -> *void {
    return context.generator;
}

active_options :: session_active_options;

session_active_options :: () -> *void {
    return context._bindings_generator_options;
}

open :: session_open;

session_open :: (generator: *void, target_os: Operating_System_Tag) -> Session {
    assert(generator != null, "Bindings_Generator: session needs a canonical Generator");
    active := context._bindings_generator_session;

    assert(!active || active.generator != generator, "Bindings_Generator: reuse the existing ownership ledger for reentrant callbacks");

    result: Session;
    result.generator = generator;
    result.target_os = target_os;
    result.allocator = context.allocator;
    result.allocations.allocator = result.allocator;
    result.arrays.allocator = result.allocator;
    result.scope_states.allocator = result.allocator;
    result.live = true;

    return result;
}

current :: session_current;

session_current :: (target_os: Operating_System_Tag) -> *Session {
    session := context._bindings_generator_session;
    if !session || !session.live || !session.generator || context.generator != session.generator || session.target_os != target_os {
        log_error("Bindings_Generator: no live generator session for the requested target\n");
        return null;
    }

    return session;
}

new_owned :: session_new_owned;

session_new_owned :: (session: *Session, $T: Type) -> *T {
    assert(session != null && session.live);
    push_allocator(session.allocator);
    result := New(T);

    array_add(*session.allocations, Owned_Allocation.{ result, size_of(T) });

    return result;
}

copy_text :: session_copy_text;

session_copy_text :: (session: *Session, text: string) -> string {
    assert(session != null && session.live);
    push_allocator(session.allocator);
    result := copy_string(text);
    if result.data array_add(*session.allocations, Owned_Allocation.{ result.data, result.count });

    return result;
}

append_owned :: session_append_owned;

append_movable_owned :: session_append_movable_owned;

// Movable cache values cannot register a stack/hash-table array header address.
// The canonical cache owner must clear these headers before closing the ledger.
session_append_movable_owned :: (session: *Session, array: *[..] $T, item: T) {
    assert(session != null && session.live && array != null);
    if array.allocated == 0 array.allocator = .{ session_array_allocator, session };
    assert(array.allocator.proc == session_array_allocator && array.allocator.data == session);
    array_add(array, item);
}

session_append_owned :: (session: *Session, array: *[..] $T, item: T) {
    assert(session != null && session.live && array != null);
    raw := cast(*Resizable_Array) array;
    known := false;

    for resource: session.arrays if resource.array == raw known = true;

    if !known {
        if array.allocated > 0 {
            array_add(array, item);
            return;
        }

        array_add(*session.arrays, Owned_Array.{ raw });
    }

    if array.allocated == 0 array.allocator = .{ session_array_allocator, session };

    array_add(array, item);
}

copy_array :: session_copy_array;

session_copy_array :: (session: *Session, input: [] $T) -> [] T {
    #assert size_of(T) > 0;
    assert(session != null && session.live);
    assert(input.count >= 0 && (input.count == 0 || input.data != null));
    assert(input.count <= 0x7fffffffffffffff / size_of(T));
    if input.count == 0 return .{};

    bytes := input.count * size_of(T);
    memory := session.allocator.proc(.ALLOCATE, bytes, 0, null, session.allocator.data);

    assert(memory != null && cast(u64) memory % align_of(T) == 0);
    memcpy(memory, input.data, bytes);
    array_add(*session.allocations, Owned_Allocation.{ memory, bytes });

    return .{ input.count, cast(*T) memory };
}

session_array_allocator :: (mode: Allocator_Mode, bytes: s64, old_bytes: s64, memory: *void, data: *void) -> *void {
    session := cast(*Session) data;

    assert(session != null && session.live);
    index := -1;

    if memory {
        for allocation, i: session.allocations if allocation.memory == memory index = i;
        if mode == .FREE || mode == .RESIZE assert(index >= 0, "Bindings_Generator: resizing/freeing an unowned array buffer");
    }

    result := session.allocator.proc(mode, bytes, old_bytes, memory, session.allocator.data);

    if mode == .FREE && memory {
        session.allocations[index] = session.allocations[session.allocations.count - 1];
        session.allocations.count -= 1;
    }

    if (mode == .ALLOCATE || mode == .RESIZE) && result {
        if index >= 0 {
            session.allocations[index].memory = result;
            session.allocations[index].bytes = bytes;
        } else array_add(*session.allocations, Owned_Allocation.{ result, bytes });
    }

    return result;
}

close :: session_close;

session_close :: (session: *Session) {
    assert(session != null && session.live, "Bindings_Generator: duplicate or invalid session close");
    assert(context._bindings_generator_session != session, "Bindings_Generator: close after leaving activation scope");

    // Detach only headers still using our allocator. Caller replacements stay
    // caller-owned; old buffers and callback-driven resizes are in the ledger.
    for resource: session.arrays {
        array := resource.array;
        if array.allocator.proc == session_array_allocator && array.allocator.data == session array.* = .{};
    }

    array_free(session.arrays);
    array_free(session.scope_states);
    for allocation: session.allocations session.allocator.proc(.FREE, 0, allocation.bytes, allocation.memory, session.allocator.data);

    array_free(session.allocations);
    session.allocations = .{};
    session.arrays = .{};
    session.scope_states = .{};
    session.generator = null;
    session.parse_input = null;
    session.lifecycle_owned = false;
    session.live = false;
}

#scope_file
#import "Basic";
