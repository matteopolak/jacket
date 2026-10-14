#import "Basic";
C :: #import "Shared_Memory_Channel";
P :: #import "POSIX";

main :: () {
    ok, channel := C.reader_create_channel(CHANNEL_NAME, 1024);

    assert(ok);
    print("ready\n");
    sequence: s64;
    while sequence < 2048 {
        found, fatal, message := C.reader_poll_for_message(*channel);

        assert(!fatal);

        if !found {
            assert(P.sched_yield() == 0);
            continue;
        }

        length := sequence % 128;

        assert(message.user_read_bytes == length);
        for index: 0..length - 1 assert(message.user_read_pointer[index] == cast(u8) ((sequence + index) % 256));

        assert(C.reader_done_with_message(*message));
        sequence += 1;
    }

    assert(C.reader_release_channel(*channel));
    print("ok\n");
}
