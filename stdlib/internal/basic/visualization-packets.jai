#scope_module
Visualization_Change :: struct {
    common: Visualization_Protocol.Summary_Common;
    free_trace: *void;
}

visualization_send_definitions_locked :: () -> bool {
    for *entry, index: visualization_strings {
        if entry.sent continue;

        packet: Visualization_Protocol.String_Info;

        memset(*packet, 0, size_of(packet));
        packet.tag = .ADD_STRING;
        packet.index = cast(Visualization_Protocol.String_Index) index;
        packet.count = cast(s32) entry.value.count;
        if !visualization_send_packet_locked(*packet, size_of(packet), entry.value.data, entry.value.count) return false;

        entry.sent = true;
    }

    for *entry: visualization_procedures {
        if entry.sent continue;

        packet: Visualization_Protocol.Packed_Stack_Trace_Procedure_Info;

        memset(*packet, 0, size_of(packet));
        packet.tag = .ADD_PROCEDURE_INFO;
        packet.name = entry.name;
        packet.filename = entry.filename;
        packet.line_number = cast(s32) entry.info.location.line_number;
        packet.character_number = cast(s32) entry.info.location.character_number;
        packet.remote_address = entry.info;
        if !visualization_send_packet_locked(*packet, size_of(packet)) return false;

        entry.sent = true;
    }

    for *entry: visualization_traces {
        if entry.sent continue;

        packet: Visualization_Protocol.Single_Stack_Trace_Info;

        memset(*packet, 0, size_of(packet));
        packet.tag = .ADD_STACK_TRACE;
        packet.num_nodes = cast(s32) entry.trace.nodes.count;
        if entry.trace.nodes.count > 0 packet.hash = entry.trace.nodes[0].hash;

        packet.address = entry.trace;
        if !visualization_send_packet_locked(*packet, size_of(packet), entry.trace.nodes.data, entry.trace.nodes.count * size_of(Stack_Trace_Node)) return false;

        entry.sent = true;
    }

    return true;
}

visualization_send_snapshot_locked :: () {
    push_allocator(context.default_allocator);
    debug_acquire();
    report := build_leak_report_locked(.{}, true);
    events := visualization_events;

    visualization_events = null;
    visualization_events_tail = null;
    debug_unlock();
    defer deinit(*report);
    committed := false;

    defer visualization_complete_events(events, committed);
    live: [..] Visualization_Protocol.Packed_Summary;
    allocs: [..] Visualization_Change;
    frees: [..] Visualization_Change;

    defer visualization_array_free(*live);
    defer visualization_array_free(*allocs);
    defer visualization_array_free(*frees);

    for summary: report.sorted_summaries {
        packet: Visualization_Protocol.Packed_Summary;

        memset(*packet, 0, size_of(packet));
        packet.count = summary.count;
        packet.bytes = summary.bytes;
        packet.alloc_site_trace = visualization_intern_trace(summary.alloc_site_trace);
        packet.allocator_proc = summary.allocator_proc;
        packet.allocations_since_last_visualizer_update = summary.allocations_since_last_visualizer_update;
        packet.bytes_allocated_since_last_visualizer_update = summary.bytes_allocated_since_last_visualizer_update;
        packet.grouped_from_this_many_stack_traces = 1;
        visualization_array_add(*live, packet);
    }

    current := events;
    while current {
        allocation_trace := visualization_intern_trace(current.allocation_trace);
        free_trace := visualization_intern_trace(current.free_trace);
        change := Visualization_Change.{ common = .{ count = 1, bytes = current.bytes, alloc_site_trace = allocation_trace, allocator_proc = current.allocator_proc }, free_trace = free_trace };

        if current.freed visualization_add_change(*frees, change);
        else visualization_add_change(*allocs, change);

        current = current.next;
    }

    if !visualization_send_definitions_locked() return;

    packet: Visualization_Protocol.Summary_Info;

    memset(*packet, 0, size_of(packet));
    packet.tag = .UPDATE_USAGE;
    packet.num_live = cast(s32) live.count;
    packet.num_allocs_this_frame = cast(s32) allocs.count;
    packet.num_frees_this_frame = cast(s32) frees.count;
    bytes := live.count * size_of(Visualization_Protocol.Packed_Summary) + allocs.count * size_of(Visualization_Protocol.Allocs_This_Frame) + frees.count * size_of(Visualization_Protocol.Frees_This_Frame);

    #if VISUALIZATION_NATIVE {
        success, reserved := Visualization_Channel.writer_reserve_message(*visualization_channel, size_of(packet) + bytes);
        if !success return;

        cursor := reserved.user_write_pointer;

        memcpy(cursor, *packet, size_of(packet));
        cursor += size_of(packet);
        memcpy(cursor, live.data, live.count * size_of(Visualization_Protocol.Packed_Summary));
        cursor += live.count * size_of(Visualization_Protocol.Packed_Summary);

        for change: allocs {
            allocation: Visualization_Protocol.Allocs_This_Frame;

            memset(*allocation, 0, size_of(allocation));
            allocation.common = change.common;
            memcpy(cursor, *allocation, size_of(allocation));
            cursor += size_of(allocation);
        }

        for change: frees {
            released: Visualization_Protocol.Frees_This_Frame;

            memset(*released, 0, size_of(released));
            released.common = change.common;
            released.free_site_trace = change.free_trace;
            memcpy(cursor, *released, size_of(released));
            cursor += size_of(released);
        }

        committed = Visualization_Channel.writer_commit_message(*visualization_channel, reserved);
    }
}

visualization_add_change :: (changes: *[..] Visualization_Change, change: Visualization_Change) {
    for *entry: changes.* {
        if entry.common.alloc_site_trace == change.common.alloc_site_trace && entry.common.allocator_proc == change.common.allocator_proc && entry.free_trace == change.free_trace {
            entry.common.count += change.common.count;
            entry.common.bytes += change.common.bytes;
            return;
        }
    }

    visualization_array_add(changes, change);
}
