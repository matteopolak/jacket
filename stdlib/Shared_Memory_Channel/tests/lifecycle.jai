// All outputs and channel names are assigned by the worker-local runner.
#import "Basic";
C :: #import "Shared_Memory_Channel";
F :: #import "File";

main :: () {
    args := get_command_line_arguments();
    if args.count != 5 exit(2);

    calls: s64;

    assert(mark(*calls));

    if args[4] == "enabled" {
        if calls != 1 exit(31);
    } else if args[4] == "disabled" {
        if calls != 0 exit(32);
    } else exit(33);

    if args[1] == "writer" {
        send_messages(args[2]);
        return;
    }

    direct := args[1] == "local";
    if !direct && args[1] != "reader" exit(3);

    ok, reader := C.reader_create_channel(args[2], 512);
    if !ok exit(4);

    defer if reader.header {
        if !C.reader_release_channel(*reader) exit(5);
    }

    if reader.header.version != C.PROTOCOL_VERSION || reader.header.data_size_in_bytes != 256 exit(6);
    if reader.header.offset_to_message_data != C.OFFSET_TO_MESSAGE_DATA exit(7);
    if C.get_message_queue_header(reader.header).read_cursor != 0 exit(8);

    if direct {
        connected, writer := C.writer_connect(args[2]);
        if !connected exit(9);

        defer if writer.header {
            if !C.writer_disconnect(*writer) exit(10);
        }

        // Zero-length packets and repeated wraparound obey reserve/commit/done order.
        for sequence: 0..255 {
            length := sequence % 97;
            reserved, packet := C.writer_reserve_message(*writer, length);

            if !reserved exit(11);

            found, fatal := C.reader_poll_for_message(*reader);
            if found || fatal exit(12);

            fill(packet.user_write_pointer, length, sequence);
            if !C.writer_commit_message(*writer, packet) exit(13);

            found_after, fatal_after, message := C.reader_poll_for_message(*reader);
            if !found_after || fatal_after || message.user_read_bytes != length exit(14);

            check(message.user_read_pointer, length, sequence);
            if !C.reader_done_with_message(*message) exit(15);
            if message.channel || message.user_read_pointer exit(16);
        }

        if !C.writer_disconnect(*writer) exit(17);
        if !C.reader_release_channel(*reader) exit(18);

        write_string("ok\n");

        return;
    }

    if !F.write_entire_file(args[3], "ready") exit(19);

    for sequence: 0..255 {
        received := false;

        for attempt: 0..4999 {
            found, fatal, message := C.reader_poll_for_message(*reader);
            if fatal exit(20);

            if found {
                if message.user_read_bytes != sequence % 97 exit(21);

                check(message.user_read_pointer, message.user_read_bytes, sequence);
                if !C.reader_done_with_message(*message) exit(22);

                received = true;
                break;
            }

            sleep_milliseconds(1);
        }

        if !received exit(23);
    }

    if !C.reader_release_channel(*reader) exit(24);

    write_string("ok\n");
}

send_messages :: (name: string) {
    connected, writer := C.writer_connect(name);
    if !connected exit(25);

    defer if writer.header {
        if !C.writer_disconnect(*writer) exit(26);
    }

    for sequence: 0..255 {
        sent := false;

        for attempt: 0..4999 {
            reserved, packet := C.writer_reserve_message(*writer, sequence % 97);
            if reserved {
                fill(packet.user_write_pointer, sequence % 97, sequence);
                if !C.writer_commit_message(*writer, packet) exit(27);
                sent = true;
                break;
            }

            sleep_milliseconds(1);
        }

        if !sent exit(28);
    }

    if !C.writer_disconnect(*writer) exit(29);

    write_string("ok\n");
}

fill :: (memory: *u8, bytes: s64, sequence: s64) {
    for index: 0..bytes - 1 memory[index] = cast,truncate(u8) (sequence + index);
}

check :: (memory: *u8, bytes: s64, sequence: s64) {
    for index: 0..bytes - 1 if memory[index] != cast,truncate(u8) (sequence + index) exit(30);
}

mark :: (calls: *s64) -> bool {
    calls.* += 1;
    return true;
}
