// test: skip_on(.WASM, reason = "native Io timers") skip_on(.WINDOWS, reason = "Io has no Windows backend")
#import "Basic";
Io :: #import "Extensions/Io";

main :: () {
    Io.io_run(() {
        group: Io.Wait_Group;

        Io.add(*group);
        Io.with_deadline(1, #code {
            assert(Io.wait(*group) == .TIMED_OUT);
        });
        Io.done(*group);
        assert(Io.wait(*group) == .NONE);
        channel := Io.make_channel(int, 0);

        defer Io.free_channel(channel);
        Io.with_deadline(1, #code {
            assert(Io.send(channel, 7) == .TIMED_OUT);
            _, error := Io.recv(channel);
            assert(error == .TIMED_OUT);
        });
        assert(!Io.try_send(channel, 8));
        _, found := Io.try_recv(channel);

        assert(!found);
        mutex: Io.Fiber_Mutex;

        assert(Io.lock(*mutex) == .NONE);
        Io.with_deadline(1, #code {
            assert(Io.lock(*mutex) == .TIMED_OUT);
        });
        Io.unlock(*mutex);
        assert(Io.try_lock(*mutex));
        Io.unlock(*mutex);
    });
    print("ok\n");
}
