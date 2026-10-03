vis_init :: () {
    #if VISUALIZATION_ENABLED {
        if !visualization_try_lock() return;

        defer visualization_finish();
        valid, now := visualization_clock();
        if valid visualization_connect_locked(now);
        else visualization_fail_once("Memory visualization transport/clock is unavailable on this target");
    }
}

maybe_add_string :: (s: string) {
    #if VISUALIZATION_ENABLED {
        while !visualization_try_lock() {}
        defer visualization_finish();
        visualization_intern_string(s);
    }
}

vis_update :: () {
    #if VISUALIZATION_ENABLED {
        if !visualization_try_lock() return;

        defer visualization_finish();
        valid, now := visualization_clock();
        if !valid {
            visualization_fail_once("Memory visualization transport/clock is unavailable on this target");
            return;
        }

        if !visualization_connect_locked(now) return;

        visualization_last_update = now;
        visualization_has_update = true;
        visualization_send_snapshot_locked();
    }
}

vis_update_if_it_has_been_long_enough :: () {
    #if VISUALIZATION_ENABLED {
        if atomic_load(*visualization_explicit_frames) != 0 || !visualization_try_lock() return;

        defer visualization_finish();
        valid, now := visualization_clock();
        if !valid {
            visualization_fail_once("Memory visualization transport/clock is unavailable on this target");
            return;
        }

        if !visualization_connect_locked(now) return;
        if visualization_has_update && now - visualization_last_update < 250000000 return;

        visualization_last_update = now;
        visualization_has_update = true;
        visualization_send_snapshot_locked();
    }
}

vis_fini :: () {
    #if VISUALIZATION_ENABLED {
        while !visualization_try_lock() {}

        defer visualization_finish();

        #if VISUALIZATION_NATIVE {
            if atomic_load(*visualization_connected) != 0 {
                success: bool;

                push_context {
                    context.logger = visualization_capture_log;
                    success = Visualization_Channel.writer_disconnect(*visualization_channel);
                }

                if !success {
                    visualization_queue_log("Memory visualization disconnect failed", .{ common_flags = .ERROR });
                    return;
                }
            }
        }

        atomic_store(*visualization_connected, cast(u32) 0);
        debug_acquire();
        events := visualization_events;

        visualization_events = null;
        visualization_events_tail = null;
        debug_unlock();
        visualization_free_events(events);
        for entry: visualization_strings entry.allocator.proc(.FREE, 0, entry.value.count, entry.value.data, entry.allocator.data);
        for entry: visualization_traces raw_trace_free(entry.allocator, entry.trace);

        visualization_array_free(*visualization_strings);
        visualization_array_free(*visualization_procedures);
        visualization_array_free(*visualization_traces);
        visualization_handshake = false;
        visualization_has_attempt = false;
        visualization_has_update = false;
        visualization_failure_logged = false;
    }
}

#scope_module
VISUALIZATION_NATIVE :: OS == .MACOS || OS == .LINUX;
VISUALIZATION_ENABLED :: MEMORY_DEBUGGER && VISUALIZE_MEMORY_DEBUGGER && REPLACEMENT_INTERFACE == Memory_Debugger_Interface;

Visualization_Protocol :: #import "Protocol_For_Memory_Visualization";

Visualization_String :: struct {
    value: string;
    allocator: Allocator;
    sent: bool;
}

Visualization_Procedure :: struct {
    info: *Stack_Trace_Procedure_Info;
    name: Visualization_Protocol.String_Index;
    filename: Visualization_Protocol.String_Index;
    sent: bool;
}

Visualization_Trace :: struct {
    trace: *Packed_Stack_Trace;
    allocator: Allocator;
    sent: bool;
}

Visualization_Log :: struct {
    message: string;
    allocator: Allocator;
    info: Log_Info;
}

visualization_strings: [..] Visualization_String;
visualization_procedures: [..] Visualization_Procedure;
visualization_traces: [..] Visualization_Trace;
visualization_logs: [..] Visualization_Log;
visualization_lock: u32;
visualization_explicit_frames: u32;
visualization_handshake: bool;
visualization_failure_logged: bool;
visualization_has_attempt: bool;
visualization_last_attempt: u64;
visualization_has_update: bool;
visualization_last_update: u64;

#if VISUALIZATION_NATIVE {
    Visualization_Channel :: #import "Shared_Memory_Channel";
    Visualization_POSIX :: #import "POSIX";

    visualization_channel: Visualization_Channel.Write_Channel;
}

visualization_per_frame :: () {
    atomic_store(*visualization_explicit_frames, cast(u32) 1);
    vis_init();
    vis_update();
}

visualization_try_lock :: () -> bool {
    return atomic_exchange(*visualization_lock, cast(u32) 1) == 0;
}

visualization_finish :: () {
    logs := visualization_logs;

    visualization_logs = .{};
    atomic_store(*visualization_lock, cast(u32) 0);

    for entry: logs {
        context.logger(entry.message, context.logger_data, entry.info);
        entry.allocator.proc(.FREE, 0, entry.message.count, entry.message.data, entry.allocator.data);
    }

    visualization_array_free(*logs);
}

visualization_copy_string :: (value: string, allocator: Allocator) -> string {
    if value.count == 0 return .{};

    data := cast(*u8) allocator.proc(.ALLOCATE, value.count, 0, null, allocator.data);
    if !data {
        context.assertion_failed(#caller_location, "memory visualization string allocation failed");
        debug_break();
    }

    memcpy(data, value.data, value.count);

    return .{ value.count, data };
}

visualization_queue_log :: (message: string, info: Log_Info) {
    allocator := context.default_allocator;
    visualization_array_add(*visualization_logs, Visualization_Log.{ visualization_copy_string(message, allocator), allocator, info });
}

visualization_capture_log :: (message: string, data: *void, info: Log_Info) {
    visualization_queue_log(message, info);
}

visualization_fail_once :: (message: string) {
    if visualization_failure_logged return;
    visualization_failure_logged = true;
    visualization_queue_log(message, .{ source_identifier = context.log_source_identifier, common_flags = .ERROR });
}

visualization_clock :: () -> (success: bool, now: u64) {
    #if VISUALIZATION_NATIVE {
        stamp: Visualization_POSIX.timespec;
        if Visualization_POSIX.clock_gettime(.MONOTONIC, *stamp) != 0 return false, 0;
        return true, cast(u64) stamp.tv_sec * 1000000000 + cast(u64) stamp.tv_nsec;
    } else return false, 0;
}

visualization_connect_locked :: (now: u64) -> bool {
    push_allocator(context.default_allocator);

    #if VISUALIZATION_NATIVE {
        if atomic_load(*visualization_connected) != 0 && visualization_channel.header && atomic_load(*visualization_channel.header.reader_status) == .DISCONNECTED {
            success: bool;

            push_context {
                context.logger = visualization_capture_log;
                success = Visualization_Channel.writer_disconnect(*visualization_channel);
            }

            if !success {
                visualization_queue_log("Memory visualization disconnect failed", .{ common_flags = .ERROR });
                return false;
            }

            atomic_store(*visualization_connected, cast(u32) 0);
            visualization_handshake = false;
            visualization_has_update = false;
            debug_acquire();
            old_events := visualization_events;

            visualization_events = null;
            visualization_events_tail = null;
            debug_unlock();
            visualization_free_events(old_events);
        }

        if atomic_load(*visualization_connected) == 0 {
            if visualization_has_attempt && now - visualization_last_attempt < 500000000 return false;

            visualization_has_attempt = true;
            visualization_last_attempt = now;
            before := visualization_logs.count;
            success: bool;

            push_context {
                context.logger = visualization_capture_log;
                success, visualization_channel = Visualization_Channel.writer_connect(Visualization_Protocol.MEMORY_VISUALIZATION_CHANNEL_NAME, name_not_found_is_quiet = true);
            }

            if !success {
                if visualization_failure_logged {
                    for index: before..visualization_logs.count - 1 {
                        entry := visualization_logs[index];
                        entry.allocator.proc(.FREE, 0, entry.message.count, entry.message.data, entry.allocator.data);
                    }

                    visualization_logs.count = before;
                } else {
                    if visualization_logs.count == before visualization_fail_once("Unable to connect to memory visualization channel");
                    visualization_failure_logged = true;
                }

                return false;
            }

            debug_acquire();
            old_events := visualization_events;

            visualization_events = null;
            visualization_events_tail = null;
            frame_allocations = 0;
            frame_allocated_bytes = 0;
            frame_frees = 0;
            frame_freed_bytes = 0;
            record := debug_records;
            while record {
                record.frame_allocations = 0;
                record.frame_bytes = 0;
                record = record.next;
            }

            atomic_store(*visualization_connected, cast(u32) 1);
            debug_unlock();
            visualization_free_events(old_events);
            visualization_handshake = false;
            for *entry: visualization_strings entry.sent = false;
            for *entry: visualization_procedures entry.sent = false;
            for *entry: visualization_traces entry.sent = false;
        }

        if !visualization_handshake {
            packet: Visualization_Protocol.Memory_Debugger_Version_Info;

            memset(*packet, 0, size_of(packet));
            packet.tag = .RESET_AND_TELL_VERSION;
            packet.version = Visualization_Protocol.VIS_PROTOCOL_VERSION;
            packet.default_allocator_proc = context.default_allocator.proc;
            if !visualization_send_packet_locked(*packet, size_of(packet)) return false;

            visualization_handshake = true;
        }

        return true;
    } else return false;
}

visualization_send_packet_locked :: (prefix: *void, prefix_size: s64, payload: *void = null, payload_size: s64 = 0) -> bool {
    #if VISUALIZATION_NATIVE {
        success, reserved := Visualization_Channel.writer_reserve_message(*visualization_channel, prefix_size + payload_size);
        if !success return false;

        memcpy(reserved.user_write_pointer, prefix, prefix_size);
        memcpy(reserved.user_write_pointer + prefix_size, payload, payload_size);

        return Visualization_Channel.writer_commit_message(*visualization_channel, reserved);
    } else return false;
}

visualization_intern_string :: (value: string) -> Visualization_Protocol.String_Index {
    for entry, index: visualization_strings if entry.value == value return cast(Visualization_Protocol.String_Index) index;

    assert(visualization_strings.count <= 0x7fffffff);
    allocator := context.default_allocator;
    index := visualization_strings.count;

    visualization_array_add(*visualization_strings, Visualization_String.{ visualization_copy_string(value, allocator), allocator, false });

    return cast(Visualization_Protocol.String_Index) index;
}

visualization_intern_procedure :: (info: *Stack_Trace_Procedure_Info) {
    if !info return;
    for entry: visualization_procedures if entry.info == info return;

    name := visualization_intern_string(info.name);
    filename := visualization_intern_string(info.location.fully_pathed_filename);

    visualization_array_add(*visualization_procedures, Visualization_Procedure.{ info, name, filename, false });
}

visualization_intern_trace :: (trace: *Packed_Stack_Trace) -> *void {
    if !trace return null;

    for entry: visualization_traces {
        if entry.trace.nodes.count != trace.nodes.count continue;

        equal := true;

        for node, index: trace.nodes {
            other := entry.trace.nodes[index];
            if node.info != other.info || node.line_number != other.line_number || node.hash != other.hash || node.call_depth != other.call_depth {
                equal = false;
                break;
            }
        }

        if equal return entry.trace;
    }

    for node: trace.nodes visualization_intern_procedure(node.info);

    allocator := context.default_allocator;
    copy := raw_trace_copy(allocator, trace.nodes.data);

    visualization_array_add(*visualization_traces, Visualization_Trace.{ copy, allocator, false });

    return copy;
}
