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
} @Releases

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

destroy_heap :: (b: Allocator) {
    #if MEMORY_DEBUGGER Memory_Debugger.check_destroy_heap(b);
    b.proc(.DESTROY_HEAP, 0, 0, null, b.data);
} @Releases

#load "allocator-call.jai";

#scope_module
Debug_Context :: #import "internal/basic-context";

Allocation :: struct {
    next: *Allocation;
    previous: *Allocation;
    metadata_allocator: Allocator;
    owner: Allocator;
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
debug_initialized: bool;
debug_ordinal: u64;
debug_baseline: u64;
debug_lock: u32;
frame_allocations: u64;
frame_allocated_bytes: u64;
frame_frees: u64;
frame_freed_bytes: u64;

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

    table := ifx record.heap then *debug_heaps else *debug_allocations;
    removed := Debug_Index.table_remove(table, record.memory);

    assert(removed, "memory debugger index and allocation list disagree");
    record.next = null;
    record.previous = null;
}

debug_record :: (owner: Allocator, memory: *void, size: s64, heap: bool, trace: *Stack_Trace_Node) {
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

    if heap {
        allocation := debug_records;
        while allocation {
            next := allocation.next;

            if !allocation.heap && allocation.owner.proc == owner.proc && allocation.owner.data == owner.data {
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
    }

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
} @Releases
