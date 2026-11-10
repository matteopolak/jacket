#if REPLACEMENT_INTERFACE == Memory_Debugger_Interface {
    Memory_Debugger :: struct {
        check_alloc :: _check_alloc;
        check_free :: _check_free;
        check_create_heap :: _check_create_heap;
        check_destroy_heap :: _check_destroy_heap;
    }
} else {
    Memory_Debugger :: REPLACEMENT_INTERFACE;
}

_check_alloc :: inline (allocator: Allocator, memory: *void, size: s64) {
    if Debug_Context.is_active() return;

    if memory {
        debug_record(allocator, memory, size, false, context.stack_trace);
        #if MEMORY_DEBUGGER && VISUALIZE_MEMORY_DEBUGGER && REPLACEMENT_INTERFACE == Memory_Debugger_Interface vis_update_if_it_has_been_long_enough();
    }
}

_check_free :: inline (allocator: Allocator, memory: *void) {
    if Debug_Context.is_active() return;

    if memory {
        debug_release(allocator, memory, false);
        #if MEMORY_DEBUGGER && VISUALIZE_MEMORY_DEBUGGER && REPLACEMENT_INTERFACE == Memory_Debugger_Interface vis_update_if_it_has_been_long_enough();
    }
}

_check_create_heap :: (a: Allocator) {
    if Debug_Context.is_active() return;
    debug_record(a, a.data, 0, true, null);
    #if MEMORY_DEBUGGER && VISUALIZE_MEMORY_DEBUGGER && REPLACEMENT_INTERFACE == Memory_Debugger_Interface vis_update_if_it_has_been_long_enough();
}

_check_destroy_heap :: (a: Allocator) {
    if Debug_Context.is_active() return;
    debug_release(a, a.data, true);
    #if MEMORY_DEBUGGER && VISUALIZE_MEMORY_DEBUGGER && REPLACEMENT_INTERFACE == Memory_Debugger_Interface vis_update_if_it_has_been_long_enough();
}

memory_visualizer_per_frame_update :: () #expand #no_debug {
    #if MEMORY_DEBUGGER && VISUALIZE_MEMORY_DEBUGGER && REPLACEMENT_INTERFACE == Memory_Debugger_Interface visualization_per_frame();
}

create_heap :: (a: Allocator) -> Allocator {
    data := a.proc(.CREATE_HEAP, 0, 0, null, a.data);

    assert(data != null, "allocator failed to create a heap");
    result := Allocator.{ a.proc, data };

    #if MEMORY_DEBUGGER if data Memory_Debugger.check_create_heap(result);

    return result;
}

// Tells the memory debugger that `a` has given back everything it handed out at once, as a bump allocator does when it
// resets or is finalized: every range it owns stops being live, and later ones are tracked as new. Does nothing when the
// debugger is off. Call it from the reset and fini of an allocator that ignores `.FREE`.
//
// ```jai
// arena: int;
// arena_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
//     return null;
// }
// release_all_allocations(Allocator.{ arena_proc, *arena }); // call this from the arena's reset
// ```
release_all_allocations :: (a: Allocator) {
    Debug_Context.allocator_released_everything(a);
}

// Starts the debugger if it is not running yet, so it already hears of what an allocator does on its own, such as a pool
// taking its first block, while the call that led to it is still going. Does nothing when called from the debugger.
start_debugger :: () {
    if debug_initialized || Debug_Context.is_active() return;

    push_context, defer_pop;
    context.inside_memory_debugger = true;
    context.logger = runtime_support_default_logger;
    initialize_debugger();
}

destroy_heap :: (b: Allocator) {
    #if MEMORY_DEBUGGER Memory_Debugger.check_destroy_heap(b);
    b.proc(.DESTROY_HEAP, 0, 0, null, b.data);
}

#load "allocator-call.jai";

#scope_module
Debug_Context :: #import "internal/basic-context";

Allocation :: struct {
    next: *Allocation;
    previous: *Allocation;
    metadata_allocator: Allocator;
    owner: Allocator;
    owner_next: *Allocation;
    owner_previous: *Allocation;
    memory: *void;
    size: s64;
    heap: bool;
    ordinal: u64;
    exempt: bool;
    stops_ownership: bool;
    trace: *Packed_Stack_Trace;
    frame_allocations: s64;
    frame_bytes: s64;
}

debug_records: *Allocation;

Debug_Index :: #import "Hash_Table";

debug_allocations: Debug_Index.Table(*void, *Allocation);
debug_heaps: Debug_Index.Table(*void, *Allocation);
// Each owner's live ranges, as a list threaded through `owner_next`, keyed by the owner's data pointer.
debug_owners: Debug_Index.Table(*void, *Allocation);
debug_initialized: bool;
debug_ordinal: u64;
debug_baseline: u64;
debug_lock: u32;
frame_allocations: u64;
frame_allocated_bytes: u64;
frame_frees: u64;
frame_freed_bytes: u64;

// This debugger's entry in the registry of `internal/basic-context`, which tells it what allocators do on their own.
debug_hooks: Debug_Context.Debug_Hooks;

debug_record_block :: (owner: Allocator, memory: *void, size: s64) {
    debug_record(owner, memory, size, false, context.stack_trace);
}

debug_release_block :: (owner: Allocator, memory: *void) {
    debug_release(owner, memory, false);
}

debug_acquire :: () {
    while atomic_exchange(*debug_lock, cast(u32) 1) != 0 {}
}

debug_unlock :: () {
    atomic_store(*debug_lock, cast(u32) 0);
}

initialize_debugger :: () {
    debug_acquire();
    defer debug_unlock();
    initialize_debugger_locked();
}

initialize_debugger_locked :: () {
    if debug_initialized return;

    debug_allocations.allocator = context.default_allocator;
    debug_heaps.allocator = context.default_allocator;
    debug_hooks = .{ acquired = debug_record_block, released = debug_release_block, released_everything = debug_release_owned };
    Debug_Context.register_hooks(*debug_hooks);
    debug_owners.allocator = context.default_allocator;
    Debug_Index.table_ensure_space(*debug_allocations, 1);
    Debug_Index.table_ensure_space(*debug_heaps, 1);
    debug_initialized = true;
}

find_debug_allocation_locked :: (memory: *void, heap: bool) -> *Allocation {
    table := ifx heap then *debug_heaps else *debug_allocations;
    value := Debug_Index.table_find_pointer(table, memory);
    if !value return null;
    return value.*;
}

unlink_debug_allocation_locked :: (record: *Allocation) {
    if record.previous record.previous.next = record.next;
    else debug_records = record.next;
    if record.next record.next.previous = record.previous;

    if !record.heap unlink_owned_locked(record);

    table := ifx record.heap then *debug_heaps else *debug_allocations;
    removed := Debug_Index.table_remove(table, record.memory);

    assert(removed, "memory debugger index and allocation list disagree");
    record.next = null;
    record.previous = null;
}

link_owned_locked :: (record: *Allocation) {
    head := Debug_Index.table_find_pointer(*debug_owners, record.owner.data);

    if head {
        record.owner_next = head.*;
        head.*.owner_previous = record;
        head.* = record;
    } else Debug_Index.table_add(*debug_owners, record.owner.data, record);
}

unlink_owned_locked :: (record: *Allocation) {
    if record.owner_previous record.owner_previous.owner_next = record.owner_next;
    else if record.owner_next Debug_Index.table_set(*debug_owners, record.owner.data, record.owner_next);
    else Debug_Index.table_remove(*debug_owners, record.owner.data);

    if record.owner_next record.owner_next.owner_previous = record.owner_previous;

    record.owner_next = null;
    record.owner_previous = null;
}

debug_record :: (owner: Allocator, memory: *void, size: s64, heap: bool, trace: *Stack_Trace_Node) {
    // Empty ranges need not have distinct addresses, even from a later nonempty allocation.
    // Heap records use size zero too, but their handles still have identity.
    if !heap && size == 0 return;

    push_context, defer_pop;
    context.inside_memory_debugger = true;
    context.logger = runtime_support_default_logger;
    head: Stack_Trace_Node;

    if trace {
        head = trace.*;
        trace = *head;
    }

    if !heap {
        caps := cast(Allocator_Caps) cast(u64) owner.proc(.CAPS, 0, 0, null, owner.data);
        if (caps & .HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE) != 0 return;
    }

    debug_acquire();
    initialize_debugger_locked();
    record := find_debug_allocation_locked(memory, heap);

    if record {
        debug_unlock();
        context.assertion_failed(#caller_location,"memory debugger: duplicate live allocation");
        return;
    }

    if !record {
        metadata := context.default_allocator;

        record = cast(*Allocation) metadata.proc(.ALLOCATE, size_of(Allocation), 0, null, metadata.data);
        if !record {
            debug_unlock();
            context.assertion_failed(#caller_location,"memory debugger: metadata allocation failed");
            return;
        }

        record.* = .{};
        record.metadata_allocator = metadata;
        if !heap record.trace = raw_trace_copy(metadata, trace);

        record.next = debug_records;
        if debug_records debug_records.previous = record;

        debug_records = record;
        table := ifx heap then *debug_heaps else *debug_allocations;

        Debug_Index.table_add(table, memory, record);
    }

    record.owner = owner;
    record.memory = memory;
    record.size = size;
    record.heap = heap;
    if !heap link_owned_locked(record);

    debug_ordinal += 1;
    record.ordinal = debug_ordinal;

    if !heap {
        record.frame_allocations = 1;
        record.frame_bytes = size;
    }

    #if VISUALIZE_MEMORY_DEBUGGER if !heap {
        frame_allocations += 1;
        frame_allocated_bytes += cast(u64) size;
    }

    if !heap visualization_note_locked(record, false, size);

    debug_unlock();
}

debug_release :: (owner: Allocator, memory: *void, heap: bool) {
    push_context, defer_pop;
    context.inside_memory_debugger = true;
    context.logger = runtime_support_default_logger;

    if !heap {
        caps := cast(Allocator_Caps) cast(u64) owner.proc(.CAPS, 0, 0, null, owner.data);
        if (caps & .FREE) == 0 return;
    }

    debug_acquire();
    record := find_debug_allocation_locked(memory, heap);

    // External/raw allocations are outside this registry's instrumentation.
    if !record {
        debug_unlock();
        return;
    }

    if record.owner.proc != owner.proc || record.owner.data != owner.data {
        debug_unlock();
        context.assertion_failed(#caller_location,"memory debugger: invalid free or allocator ownership mismatch");
        return;
    }

    unlink_debug_allocation_locked(record);
    released := record;

    #if VISUALIZE_MEMORY_DEBUGGER if !heap {
        frame_frees += 1;
        frame_freed_bytes += cast(u64) record.size;
    }

    if heap released = take_owned_locked(owner, released);

    finish_release_locked(released);
}

// Unlinks every live range `owner` holds onto the front of `released` and returns the new head. Costs the number
// of ranges the owner holds, whatever else is live. An owner whose proc differs but whose data matches is skipped.
take_owned_locked :: (owner: Allocator, released: *Allocation) -> *Allocation {
    head := Debug_Index.table_find_pointer(*debug_owners, owner.data);
    if !head return released;

    allocation := head.*;

    while allocation {
        next := allocation.owner_next;

        if allocation.owner.proc == owner.proc {
            unlink_debug_allocation_locked(allocation);
            allocation.next = released;
            released = allocation;

            #if VISUALIZE_MEMORY_DEBUGGER {
                frame_frees += 1;
                frame_freed_bytes += cast(u64) allocation.size;
            }
        }

        allocation = next;
    }

    return released;
}

// Forgets every live range `owner` has handed out, as when a bump allocator resets or shuts down. Ranges handed out
// afterwards are tracked as new ones.
debug_release_owned :: (owner: Allocator) {
    push_context, defer_pop;
    context.inside_memory_debugger = true;
    context.logger = runtime_support_default_logger;

    if owner.data == null {
        context.assertion_failed(#caller_location,"memory debugger: releasing every range needs an allocator with instance data");
        return;
    }

    debug_acquire();
    released := take_owned_locked(owner, null);

    finish_release_locked(released);
}

// Reports the unlinked records to the visualizer, drops the lock and frees their metadata.
finish_release_locked :: (list: *Allocation) {
    released := list;
    event_record := released;

    while event_record {
        if !event_record.heap visualization_note_locked(event_record, true, event_record.size);
        event_record = event_record.next;
    }

    debug_unlock();

    while released {
        next := released.next;
        metadata := released.metadata_allocator;

        raw_trace_free(metadata, released.trace);
        metadata.proc(.FREE, 0, size_of(Allocation), released, metadata.data);
        released = next;
    }
}
