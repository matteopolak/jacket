// The memory debugger streams its state to a visualizer over a shared-memory channel. With no
// visualizer listening, tracking still works; with this test acting as the visualizer (the
// channel's reader), a per-frame update sends the version message and one stack-trace record
// per live allocation site, each followed by its frames.
Basic :: #import "Basic"(MEMORY_DEBUGGER = true);

#if OS == .MACOS || OS == .LINUX {
    Wire :: #import "Protocol_For_Memory_Visualization";
    Channel :: #import "Shared_Memory_Channel";
    POSIX :: #import "POSIX";

    listen_and_check :: (kept: [] *u8) -> s64 {
        name := Wire.MEMORY_VISUALIZATION_CHANNEL_NAME;
        POSIX.shm_unlink(Basic.temp_c_string(name)); // a leftover from a crashed run
        created, reader := Channel.reader_create_channel(name, 1024 * 1024 - Channel.OFFSET_TO_MESSAGE_DATA);
        if !created return 20;
        defer Channel.reader_release_channel(*reader);

        // Connection attempts are rate-limited, and main already made one.
        Basic.sleep_milliseconds(600);
        Basic.memory_visualizer_per_frame_update();

        saw_version := false;
        traces := 0;
        while true {
            found, fatal, message := Channel.reader_poll_for_message(*reader);
            if fatal return 21;
            if !found break;
            tag := (cast(*Wire.Vis_Protocol_Tag) message.user_read_pointer).*;
            if tag == .RESET_AND_TELL_VERSION {
                info := cast(*Wire.Memory_Debugger_Version_Info) message.user_read_pointer;
                if info.version != Wire.VIS_PROTOCOL_VERSION return 22;
                saw_version = true;
            } else if tag == .ADD_STACK_TRACE {
                info := cast(*Wire.Single_Stack_Trace_Info) message.user_read_pointer;
                expected_bytes := size_of(Wire.Single_Stack_Trace_Info) + info.num_nodes * size_of(Stack_Trace_Node);
                if info.num_nodes <= 0 || !info.address || message.user_read_bytes != expected_bytes return 23;
                traces += 1;
            }
            if !Channel.reader_done_with_message(*message) return 24;
        }
        if !saw_version || traces == 0 return 25;
        return 0;
    }
}

main :: () -> s64 {
    blocks: [8] *u8;
    for *blocks it.* = Basic.alloc(32 + it_index);
    // No visualizer yet: this must be a harmless no-op.
    Basic.memory_visualizer_per_frame_update();
    result: s64;
    #if OS == .MACOS || OS == .LINUX result = listen_and_check(blocks);
    for blocks Basic.free(it);
    return result;
}
