B :: #import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true);
CH :: #import "Shared_Memory_Channel";
P :: #import "Protocol_For_Memory_Visualization";
Native :: #import "POSIX";

failures: int;

capture :: (message: string, data: *void, info: Log_Info) {
    B.assert((info.common_flags & .ERROR) != 0 && message.count > 0);
    failures += 1;
}

clock :: () -> u64 {
    value: Native.timespec;
    B.assert(Native.clock_gettime(.MONOTONIC, *value) == 0);
    return cast(u64) value.tv_sec * 1000000000 + cast(u64) value.tv_nsec;
}

drain :: (channel: *CH.Read_Channel) -> int {
    updates: int;

    while true {
        found, fatal, message := CH.reader_poll_for_message(channel);

        B.assert(!fatal);
        if !found break;

        tag := (cast(*P.Vis_Protocol_Tag) message.user_read_pointer).*;
        if tag == .UPDATE_USAGE updates += 1;
        else B.assert(tag == .RESET_AND_TELL_VERSION);

        B.assert(CH.reader_done_with_message(*message));
    }

    return updates;
}

main :: () {
    push_context {
        context.logger = capture;
        initial := clock();

        D.vis_init();
        B.assert(failures == 1);
        success, reader := CH.reader_create_channel(P.MEMORY_VISUALIZATION_CHANNEL_NAME, 8192);

        B.assert(success);
        defer B.assert(CH.reader_release_channel(*reader));
        defer D.vis_fini();

        for i: 0..19 {
            D.vis_init();
            if reader.header.writer_status == .ACTIVELY_WRITING B.assert(clock() - initial >= 500000000);
        }

        B.assert(failures == 1);
        B.sleep_milliseconds(550);
        D.vis_init();
        B.assert(reader.header.writer_status == .ACTIVELY_WRITING);
        drain(*reader);
        previous := clock();

        D.vis_update_if_it_has_been_long_enough();
        B.assert(drain(*reader) == 1);

        for i: 0..19 {
            before := clock();

            D.vis_update_if_it_has_been_long_enough();
            after := clock();
            updates := drain(*reader);

            if updates != 0 {
                B.assert(updates == 1 && after - previous >= 250000000);
                previous = before;
            }
        }

        B.sleep_milliseconds(270);
        D.vis_update_if_it_has_been_long_enough();
        B.assert(drain(*reader) == 1);
        D.memory_visualizer_per_frame_update();
        B.assert(drain(*reader) == 1);
        B.sleep_milliseconds(270);
        D.vis_update_if_it_has_been_long_enough();
        B.assert(drain(*reader) == 0);
        D.vis_update();
        B.assert(drain(*reader) == 1 && failures == 1);
    }

    B.print("ok\n");
}
