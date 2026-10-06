// Shared_Memory_Channel: a reader creates a named ring, a writer in the same process connects,
// and messages make the round trip in order, including after the ring wraps.
#import "Basic";

#if OS == .MACOS || OS == .LINUX {
    Channel :: #import "Shared_Memory_Channel";
    POSIX :: #import "POSIX";

    round_trip :: () -> s64 {
        name := tprint("/jaic-smc-%", POSIX.getpid());
        // A whole number of pages: macOS reports shared-memory sizes rounded up to the page
        // size, and the writer insists that the size it sees matches the header.
        RING :: 16384 - Channel.OFFSET_TO_MESSAGE_DATA;
        created, reader := Channel.reader_create_channel(name, RING);
        if !created return 1;
        header := reader.header;
        if header.version != 1 || header.data_size_in_bytes != RING return 2;
        if header.full_memory_region_size_in_bytes != 16384 return 3;
        if header.reader_status != .PAUSED || header.writer_status != .DISCONNECTED return 4;
        // Nothing has been written yet.
        found, fatal := Channel.reader_poll_for_message(*reader);
        if found || fatal return 5;

        connected, writer := Channel.writer_connect(name);
        if !connected return 6;
        // Twenty 3000-byte payloads through a ring of under 16 KiB force it to wrap.
        PAYLOAD :: 3000;
        for round: 0..19 {
            reserved, message := Channel.writer_reserve_message(*writer, PAYLOAD);
            if !reserved return 100 + round;
            for i: 0..PAYLOAD - 1 message.user_write_pointer[i] = cast,trunc(u8) (round * 7 + i);
            if !Channel.writer_commit_message(*writer, message) return 200 + round;

            got, broken, incoming := Channel.reader_poll_for_message(*reader);
            if !got || broken || incoming.user_read_bytes != PAYLOAD return 300 + round;
            for i: 0..PAYLOAD - 1 if incoming.user_read_pointer[i] != cast,trunc(u8) (round * 7 + i) return 400 + round;
            if !Channel.reader_done_with_message(*incoming) return 500 + round;
        }
        // A message larger than the ring can never fit.
        too_big := Channel.writer_reserve_message(*writer, RING);
        if too_big return 60;
        if !Channel.writer_disconnect(*writer) return 61;
        if !Channel.reader_release_channel(*reader) return 62;
        return 0;
    }
}

main :: () -> s64 {
    #if OS == .MACOS || OS == .LINUX return round_trip();
    return 0;
}
