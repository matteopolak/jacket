#import "Basic";
C :: #import "Shared_Memory_Channel";

main :: () {
    #if OS == .WINDOWS {
        #assert !#exists(C.Registration);
        #assert size_of(C.Read_Channel_Os_Specific) == size_of(W.HANDLE);
        #assert size_of(C.Write_Channel_Os_Specific) == size_of(W.HANDLE);
        name := sprint("Local\\portable-channel-%", W.GetCurrentProcessId());

        defer free(name);
        missing, absent := C.writer_connect(name, true);

        assert(!missing && !absent.header && !absent.os_specific.mapping);
        ok, reader := C.reader_create_channel(name, 512);

        assert(ok && reader.header && reader.os_specific.mapping);
        duplicate, ignored := C.reader_create_channel(name, 512);

        assert(!duplicate && !ignored.header && !ignored.os_specific.mapping);
        connected, writer := C.writer_connect(name);

        assert(connected && writer.header && writer.os_specific.mapping);
        second, extra := C.writer_connect(name);

        assert(!second && !extra.header && !extra.os_specific.mapping);

        for sequence: 0..2047 {
            length := 13 + (sequence % 3) * 9;
            reserved, packet := C.writer_reserve_message(*writer, length);

            assert(reserved);
            for i: 0..length - 1 packet.user_write_pointer[i] = cast,truncate(u8) (sequence + i);

            assert(C.writer_commit_message(*writer, packet));
            found, fatal, message := C.reader_poll_for_message(*reader);

            assert(found && !fatal && message.user_read_bytes == length);
            for i: 0..length - 1 assert(message.user_read_pointer[i] == cast,truncate(u8) (sequence + i));

            assert(C.reader_done_with_message(*message));
        }

        assert(C.writer_disconnect(*writer));
        assert(!writer.header && !writer.os_specific.mapping);
        connected, writer = C.writer_connect(name);
        assert(connected);
        assert(C.reader_release_channel(*reader));
        assert(!reader.header && !reader.os_specific.mapping);
        reserved, packet := C.writer_reserve_message(*writer, 1);

        assert(!reserved);
        assert(C.writer_disconnect(*writer));
        ok, reader = C.reader_create_channel(name, 512);
        assert(ok);
        protected := W.SetHandleInformation(reader.os_specific.mapping, W.HANDLE_FLAG_PROTECT_FROM_CLOSE, W.HANDLE_FLAG_PROTECT_FROM_CLOSE);

        assert(protected != 0);
        saved_logger := context.logger;

        context.logger = capture_cleanup;
        released := C.reader_release_channel(*reader);

        context.logger = saved_logger;
        assert(!released && cleanup_errors == 1);
        assert(!reader.header && reader.os_specific.mapping);
        cleared := W.SetHandleInformation(reader.os_specific.mapping, W.HANDLE_FLAG_PROTECT_FROM_CLOSE, 0);

        assert(cleared != 0);
        assert(C.reader_release_channel(*reader));
        print("ok\n");
    } else {
        assert(false, "Windows shared-memory execution requires an actual Windows host");
    }
}

#scope_file
cleanup_errors: s64;

capture_cleanup :: (message: string, _data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
    cleanup_errors += 1;
}

#if OS == .WINDOWS W :: #import "Windows";
