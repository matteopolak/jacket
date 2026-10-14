#import "Basic";
C :: #import "Shared_Memory_Channel";
P :: #import "POSIX";

main :: () {
    ok, channel := C.writer_connect(CHANNEL_NAME);

    assert(ok);
    sequence: s64;
    while sequence < 2048 {
        length := sequence % 128;
        reserved, message := C.writer_reserve_message(*channel, length);

        if !reserved {
            assert(P.sched_yield() == 0);
            continue;
        }

        for index: 0..length - 1 message.user_write_pointer[index] = cast(u8) ((sequence + index) % 256);

        assert(C.writer_commit_message(*channel, message));
        sequence += 1;
    }

    assert(C.writer_disconnect(*channel));
    print("ok\n");
}
