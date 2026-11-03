#scope_module
Visualization_Event :: struct {
    next: *Visualization_Event;
    allocator: Allocator;
    allocation_trace: *Packed_Stack_Trace;
    free_trace: *Packed_Stack_Trace;
    allocator_proc: Allocator_Proc;
    memory: *void;
    ordinal: u64;
    bytes: s64;
    freed: bool;
}

visualization_events: *Visualization_Event;
visualization_events_tail: *Visualization_Event;
visualization_connected: u32;

visualization_note_locked :: (a: *Allocation, freed: bool, bytes: s64) {
    #if MEMORY_DEBUGGER && VISUALIZE_MEMORY_DEBUGGER && REPLACEMENT_INTERFACE == Memory_Debugger_Interface {
        if atomic_load(*visualization_connected) == 0 return;

        allocator := context.default_allocator;
        event := cast(*Visualization_Event) allocator.proc(.ALLOCATE, size_of(Visualization_Event), 0, null, allocator.data);

        if !event {
            context.assertion_failed(#caller_location, "memory visualization event allocation failed");
            debug_break();
        }

        event.* = .{};
        event.allocator = allocator;
        event.allocator_proc = a.owner.proc;
        event.memory = a.memory;
        event.ordinal = a.ordinal;
        event.bytes = bytes;
        event.freed = freed;
        event.allocation_trace = raw_trace_copy(allocator, ifx a.trace then a.trace.nodes.data else null);

        if freed {
            head: Stack_Trace_Node;
            if context.stack_trace head = context.stack_trace.*;
            event.free_trace = raw_trace_copy(allocator, ifx context.stack_trace then *head else null);
        }

        if visualization_events_tail visualization_events_tail.next = event;
        else visualization_events = event;

        visualization_events_tail = event;
    }
}

visualization_free_events :: (events: *Visualization_Event) {
    current := events;
    while current {
        next := current.next;
        allocator := current.allocator;

        raw_trace_free(allocator, current.allocation_trace);
        raw_trace_free(allocator, current.free_trace);
        allocator.proc(.FREE, 0, size_of(Visualization_Event), current, allocator.data);
        current = next;
    }
}

visualization_complete_events :: (events: *Visualization_Event, success: bool) {
    if !events return;

    debug_acquire();

    if !success {
        tail := events;
        while tail.next tail = tail.next;

        tail.next = visualization_events;
        visualization_events = events;
        if !visualization_events_tail visualization_events_tail = tail;
    } else {
        event := events;
        while event {
            if event.freed {
                frame_frees -= 1;
                frame_freed_bytes -= cast(u64) event.bytes;
            } else {
                frame_allocations -= 1;
                frame_allocated_bytes -= cast(u64) event.bytes;
                record := debug_records;
                while record {
                    if !record.heap && record.memory == event.memory && record.ordinal == event.ordinal {
                        record.frame_allocations -= 1;
                        record.frame_bytes -= event.bytes;
                        break;
                    }

                    record = record.next;
                }
            }

            event = event.next;
        }
    }

    debug_unlock();
    if success visualization_free_events(events);
}

visualization_array_add :: (array: *[..] $T, value: T) {
    if array.count == array.allocated {
        if !array.allocator.proc array.allocator = context.default_allocator;

        capacity := max(cast(s64) 8, array.allocated * 2);
        memory := array.allocator.proc(.RESIZE, capacity * size_of(T), array.allocated * size_of(T), array.data, array.allocator.data);

        if !memory {
            context.assertion_failed(#caller_location, "memory visualization metadata allocation failed");
            debug_break();
        }

        array.data = cast(*T) memory;
        array.allocated = capacity;
    }

    array.data[array.count] = value;
    array.count += 1;
}

visualization_array_free :: (array: *[..] $T) {
    if array.data array.allocator.proc(.FREE, 0, array.allocated * size_of(T), array.data, array.allocator.data);
    array.* = .{};
}
