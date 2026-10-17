B :: #import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true);
CH :: #import "Shared_Memory_Channel";
P :: #import "Protocol_For_Memory_Visualization";

State :: struct {
    resets: int;
    marker: int;
    retry_marker: int;
    empty_strings: int;
    updates: int;
    live: int;
    allocs: int;
    frees: int;
    bytes: s64;
    alloc_trace: *void;
    free_alloc_trace: *void;
}

drain :: (channel: *CH.Read_Channel, state: *State) {
    while true {
        found, fatal, message := CH.reader_poll_for_message(channel);

        B.assert(!fatal);
        if !found break;

        B.assert(message.user_read_bytes >= 4);
        tag := (cast(*P.Vis_Protocol_Tag) message.user_read_pointer).*;
        if tag == .RESET_AND_TELL_VERSION {
            packet := cast(*P.Memory_Debugger_Version_Info) message.user_read_pointer;
            B.assert(packet.version == 2 && packet.default_allocator_proc != null);
            state.resets += 1;
        } else if tag == .ADD_STRING {
            packet := cast(*P.String_Info) message.user_read_pointer;

            B.assert(packet.count >= 0 && size_of(P.String_Info) + packet.count <= message.user_read_bytes);
            text := B.to_string(message.user_read_pointer + size_of(P.String_Info), packet.count);
            if text == "deduplicated" state.marker += 1;
            if text == "retry after full" state.retry_marker += 1;
            if text.count == 0 state.empty_strings += 1;
        } else if tag == .UPDATE_USAGE {
            packet := cast(*P.Summary_Info) message.user_read_pointer;

            state.updates += 1;
            state.live = packet.num_live;
            state.allocs = packet.num_allocs_this_frame;
            state.frees = packet.num_frees_this_frame;
            expected := size_of(P.Summary_Info) + packet.num_live * size_of(P.Packed_Summary) + packet.num_allocs_this_frame * size_of(P.Allocs_This_Frame) + packet.num_frees_this_frame * size_of(P.Frees_This_Frame);

            B.assert(expected == message.user_read_bytes);
            if state.live > 0 state.bytes = (cast(*P.Packed_Summary) (message.user_read_pointer + size_of(P.Summary_Info))).bytes;
            if state.allocs > 0 state.alloc_trace = (cast(*P.Allocs_This_Frame) (message.user_read_pointer + size_of(P.Summary_Info) + packet.num_live * size_of(P.Packed_Summary))).alloc_site_trace;

            if state.frees > 0 {
                free_packet := cast(*P.Frees_This_Frame) (message.user_read_pointer + size_of(P.Summary_Info) + packet.num_live * size_of(P.Packed_Summary) + packet.num_allocs_this_frame * size_of(P.Allocs_This_Frame));
                B.assert(free_packet.free_site_trace != null && free_packet.bytes == 24);
                state.free_alloc_trace = free_packet.alloc_site_trace;
            }
        } else B.assert(tag == .ADD_PROCEDURE_INFO || tag == .ADD_STACK_TRACE);

        B.assert(CH.reader_done_with_message(*message));
    }
}

main :: () {
    success, reader := CH.reader_create_channel(P.MEMORY_VISUALIZATION_CHANNEL_NAME, 8192);

    B.assert(success);
    reader_alive := true;

    defer if reader_alive B.assert(CH.reader_release_channel(*reader));
    D.vis_init();
    defer D.vis_fini();
    D.memory_visualizer_per_frame_update();
    state: State;

    drain(*reader, *state);
    B.assert(state.resets == 1);
    D.maybe_add_string("deduplicated");
    D.maybe_add_string("deduplicated");
    D.maybe_add_string("");
    D.maybe_add_string("");
    D.vis_update();
    drain(*reader, *state);
    B.assert(state.marker == 1 && state.empty_strings == 1);
    memory := D.alloc(24);

    D.vis_update();
    drain(*reader, *state);
    B.assert(state.live == 1 && state.allocs == 1 && state.frees == 0 && state.bytes == 24);
    B.assert(state.alloc_trace != null);
    D.free(memory);
    D.vis_update();
    drain(*reader, *state);
    B.assert(state.live == 0 && state.allocs == 0 && state.frees == 1);
    B.assert(state.free_alloc_trace == state.alloc_trace);
    for i: 0..399 D.vis_update();

    D.maybe_add_string("retry after full");
    pending := D.alloc(24);

    D.vis_update();
    D.free(pending);
    D.vis_update();
    drain(*reader, *state);
    B.assert(state.retry_marker == 0);
    D.vis_update();
    drain(*reader, *state);
    B.assert(state.retry_marker == 1);
    B.assert(state.allocs == 1 && state.frees == 1 && state.live == 0);
    B.assert(state.free_alloc_trace == state.alloc_trace);
    D.vis_update();
    drain(*reader, *state);
    B.assert(state.retry_marker == 1);
    B.assert(CH.reader_release_channel(*reader));
    reader_alive = false;
    success, reader = CH.reader_create_channel(P.MEMORY_VISUALIZATION_CHANNEL_NAME, 8192);
    B.assert(success);
    reader_alive = true;
    B.sleep_milliseconds(550);
    D.vis_init();
    D.vis_update();
    reconnected: State;

    drain(*reader, *reconnected);
    B.assert(reconnected.resets == 1 && reconnected.marker == 1 && reconnected.retry_marker == 1);
    B.assert(reconnected.live == 0 && reconnected.allocs == 0 && reconnected.frees == 0);
    B.print("ok\n");
}
