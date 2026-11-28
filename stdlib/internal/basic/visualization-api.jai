// Connects to the memory viewer. Does nothing unless `MEMORY_DEBUGGER` and
// `VISUALIZE_MEMORY_DEBUGGER` are both on, and logs once when the target has no way to connect.
vis_init :: () {
    #if VISUALIZATION_ENABLED {
        if !visualization_try_lock() return;

        defer visualization_finish();
        valid, now := visualization_clock();
        if valid visualization_connect_locked(now);
        else visualization_fail_once("Memory visualization transport/clock is unavailable on this target");
    }
}

// Sends `s` to the memory viewer's table of names, if it is not there yet.
maybe_add_string :: (s: string) {
    #if VISUALIZATION_ENABLED {
        while !visualization_try_lock() {}
        defer visualization_finish();
        visualization_intern_string(s);
    }
}

// Sends the memory viewer a snapshot of live allocations now.
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

// Sends the memory viewer a snapshot when the update interval has passed since the last one.
// Allocator calls do this; once [memory_visualizer_per_frame_update] has been called they stop.
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
        if visualization_has_update && now - visualization_last_update < UPDATE_INTERVAL return;

        visualization_last_update = now;
        visualization_has_update = true;
        visualization_send_snapshot_locked();
    }
}

// Disconnects from the memory viewer.
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
UPDATE_INTERVAL :: 250_000_000;
VISUALIZATION_NATIVE :: OS == .MACOS || OS == .LINUX;
VISUALIZATION_ENABLED :: MEMORY_DEBUGGER && VISUALIZE_MEMORY_DEBUGGER && REPLACEMENT_INTERFACE == Memory_Debugger_Interface;
