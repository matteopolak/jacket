#import "Basic";
C :: #import "Shared_Memory_Channel";

main :: () {
    ok, channel := C.writer_connect(CHANNEL_NAME);

    assert(ok);
    sequence: s64;
    while sequence < 2048 {
        length := sequence % 128;
        reserved, message := C.writer_reserve_message(*channel, length);

        if !reserved {
            yield_cpu();
            continue;
        }

        for index: 0..length - 1 message.user_write_pointer[index] = cast(u8) ((sequence + index) % 256);

        assert(C.writer_commit_message(*channel, message));
        sequence += 1;
    }

    assert(C.writer_disconnect(*channel));
    print("ok\n");
}

#scope_file
#if OS == .WINDOWS W :: #import "Windows";
else P :: #import "POSIX";

yield_cpu :: () {
    #if OS == .WINDOWS W.Sleep(0);
    else {
        status := P.sched_yield();
        assert(status == 0);
    }
}
