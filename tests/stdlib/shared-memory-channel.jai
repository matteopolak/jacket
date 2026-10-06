// Shared_Memory_Channel: a reader creates a named ring, a writer in the same process connects,
// and messages make the round trip in order, including after the ring wraps.
#import "Basic";

#if OS == .MACOS || OS == .LINUX || OS == .WINDOWS {
    Channel :: #import "Shared_Memory_Channel";
    #if OS == .WINDOWS {
        Windows :: #import "Windows";
        process_id :: () -> u32 {
            return Windows.GetCurrentProcessId();
        }
    } else {
        POSIX :: #import "POSIX";
        process_id :: () -> s32 {
            return POSIX.getpid();
        }
    }

    round_trip :: () -> s64 {
        name := tprint("/jaic-smc-%", process_id());
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
        // The ring follows the fixed-size header block.
        data := Channel.get_data_area(header);
        if data != cast(*u8) header + Channel.OFFSET_TO_MESSAGE_DATA return 7;
        // The name is taken while the reader holds it; a missing name has no channel.
        taken := Channel.reader_create_channel(name, RING);
        if taken return 8;
        missing := Channel.writer_connect(tprint("/jaic-smc-missing-%", process_id()), name_not_found_is_quiet = true);
        if missing return 9;

        connected, writer := Channel.writer_connect(name);
        if !connected return 6;
        // Only one writer at a time.
        second := Channel.writer_connect(name);
        if second return 10;
        // Twenty 3000-byte payloads through a ring of under 16 KiB force it to wrap.
        PAYLOAD :: 3000;
        for round: 0..19 {
            reserved, message := Channel.writer_reserve_message(*writer, PAYLOAD);
            if !reserved return 100 + round;
            for i: 0..PAYLOAD - 1 message.user_write_pointer[i] = cast,trunc(u8) (round * 7 + i);
            if !Channel.writer_commit_message(*writer, message) return 200 + round;

            got, broken, incoming := Channel.reader_poll_for_message(*reader);
            if !got || broken || incoming.user_read_bytes != PAYLOAD return 300 + round;
            // Every message lives inside the ring.
            if incoming.user_read_pointer < data || incoming.user_read_pointer + PAYLOAD > data + RING return 600 + round;
            for i: 0..PAYLOAD - 1 if incoming.user_read_pointer[i] != cast,trunc(u8) (round * 7 + i) return 400 + round;
            if !Channel.reader_done_with_message(*incoming) return 500 + round;
        }
        // A message larger than the ring can never fit.
        too_big := Channel.writer_reserve_message(*writer, RING);
        if too_big return 60;
        // The reader cannot go while a writer is connected.
        if Channel.reader_release_channel(*reader) return 63;
        if !Channel.writer_disconnect(*writer) return 61;
        if !Channel.reader_release_channel(*reader) return 62;
        // Released: the name is free again, and nobody can connect to the old ring.
        if Channel.writer_connect(name, name_not_found_is_quiet = true) return 64;
        again, fresh := Channel.reader_create_channel(name, RING);
        if !again return 65;
        if !Channel.reader_release_channel(*fresh) return 66;
        return 0;
    }
}

main :: () -> s64 {
    #if OS == .MACOS || OS == .LINUX || OS == .WINDOWS return round_trip();
    return 0;
}
