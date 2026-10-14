#import "Basic";
C :: #import "Shared_Memory_Channel";
P :: #import "POSIX";
F :: #import "File";
S :: #import "String";

Tracking :: struct {
    base: Allocator;
    outstanding: s64;
}

tracking_allocator :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracking) allocator_data;
    result := state.base.proc(mode, requested_size, old_size, old_memory, state.base.data);

    if mode == .ALLOCATE && result state.outstanding += 1;
    if mode == .RESIZE && !old_memory && result state.outstanding += 1;
    if mode == .FREE && old_memory state.outstanding -= 1;

    return result;
}

main :: () {
    name := sprint("portable-%-channel", P.getpid());

    defer free(name);
    journal_name := sprint("%/../../target/portable-tests/shared-owned-names.txt", S.path_strip_filename(#file));

    defer free(journal_name);
    journal, opened := F.file_open(journal_name, true, true);

    assert(opened && F.file_seek(journal, 0, .END));
    assert(F.file_write(*journal, name) && F.file_write(*journal, "\n") && F.file_flush(journal));
    F.file_close(*journal);
    tracking := Tracking.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ tracking_allocator, *tracking };
    ok: bool;
    reader: C.Read_Channel;

    push_context scoped ok, reader = C.reader_create_channel(name, 512);
    assert(tracking.outstanding > 0);

    assert(ok && reader.header.data_size_in_bytes == 256);
    connected, writer := C.writer_connect(name);

    assert(connected && writer.header != reader.header);
    duplicate_reader := C.reader_create_channel(name, 512);
    duplicate_writer := C.writer_connect(name);

    assert(!duplicate_reader && !duplicate_writer);
    queue := C.get_message_queue_header(reader.header);

    assert(queue.read_cursor == 0 && queue.write_cursor == 0);

    for sequence: 0..199 {
        length := sequence % 97;
        reserved_ok, reserved := C.writer_reserve_message(*writer, length);

        assert(reserved_ok);
        found_before, fatal_before := C.reader_poll_for_message(*reader);

        assert(!found_before && !fatal_before);
        for index: 0..length - 1 reserved.user_write_pointer[index] = cast(u8) ((sequence + index) % 256);

        assert(C.writer_commit_message(*writer, reserved));
        found, fatal, message := C.reader_poll_for_message(*reader);

        assert(found && !fatal && message.user_read_bytes == length);
        for index: 0..length - 1 assert(message.user_read_pointer[index] == cast(u8) ((sequence + index) % 256));

        assert(C.reader_done_with_message(*message));
        assert(!C.reader_done_with_message(*message));
    }

    committed: s64;

    while true {
        reserved_ok, reserved := C.writer_reserve_message(*writer, 31);
        if !reserved_ok break;

        memset(reserved.user_write_pointer, 0x5a, 31);
        assert(C.writer_commit_message(*writer, reserved));
        committed += 1;
    }

    assert(committed > 0);
    read: s64;

    while true {
        found, fatal, message := C.reader_poll_for_message(*reader);

        assert(!fatal);
        if !found break;

        assert(message.user_read_bytes == 31 && message.user_read_pointer[30] == 0x5a);
        assert(C.reader_done_with_message(*message));
        read += 1;
    }

    assert(read == committed);
    reserved_ok, reserved := C.writer_reserve_message(*writer, 240);
    if !reserved_ok {
        found, fatal := C.reader_poll_for_message(*reader);
        assert(!found && !fatal);
        reserved_ok, reserved = C.writer_reserve_message(*writer, 240);
    }

    assert(reserved_ok);
    memset(reserved.user_write_pointer, 0x3c, 240);
    assert(C.writer_commit_message(*writer, reserved));
    found_large, fatal_large, large := C.reader_poll_for_message(*reader);

    assert(found_large && !fatal_large && large.user_read_bytes == 240 && large.user_read_pointer[239] == 0x3c);
    assert(C.reader_done_with_message(*large));
    old_magic := reader.header.magic;

    reader.header.magic = 0;
    found, fatal := C.reader_poll_for_message(*reader);

    assert(!found && fatal);
    reader.header.magic = old_magic;
    assert(C.writer_disconnect(*writer));
    assert(C.reader_release_channel(*reader));
    assert(tracking.outstanding == 0);
    connected, writer = C.writer_connect(name, true);
    assert(!connected);
    print("ok\n");
}
