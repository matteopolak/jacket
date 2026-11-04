// test: skip_on(.WASM, reason = "native Io timers") skip_on(.WINDOWS, reason = "Io has no Windows backend")
#import "Basic";
Io :: #import "Extensions/Io";

Budget_Accept :: struct {
    listener: Io.Tcp_Listener;
    done: Io.Wait_Group;
}

accept_one :: (job: *Budget_Accept) {
    defer Io.done(*job.done);
    stream, error := Io.accept(job.listener);

    assert(error == .NONE);
    defer Io.close(stream);
    sent, write_error := Io.write(stream, u8.[42]);

    assert(write_error == .NONE && sent == 1);
}

main :: () {
    Io.io_run(() {
        listener, listen_error := Io.tcp_listen("127.0.0.1:0");

        assert(listen_error == .NONE);
        defer Io.close(listener);
        address, _ := Io.local_address(listener);
        job := Budget_Accept.{ listener = listener };

        Io.add(*job.done, 1);
        Io.spawn(accept_one, *job);
        Io.with_deadline(0x7fffffffffffffff, #code {
            stream, connect_error := Io.connect(address);

            assert(connect_error == .NONE);
            defer Io.close(stream);
            buffer: [1] u8;
            count, read_error := Io.read(stream, buffer);

            assert(read_error == .NONE && count == 1 && buffer[0] == 42);
        });
        Io.wait(*job.done);
        Io.with_deadline(0x7fffffffffffffff, #code {
            error := Io.sleep(1);
            assert(error == .NONE, "a large positive deadline must not wrap into the past");
        });
        Io.with_grace(0x7fffffffffffffff, #code {
            error := Io.sleep(1);
            assert(error == .NONE, "a large grace budget must not wrap into the past");
        });
        Io.with_deadline(1, #code {
            assert(Io.sleep(0x7fffffffffffffff) == .TIMED_OUT, "a large sleep still obeys its enclosing deadline");
        });
        Io.with_deadline(0, #code {
            Io.with_deadline(0x7fffffffffffffff, #code {
                assert(Io.sleep(1) == .TIMED_OUT, "an enclosing expired budget still wins");
            });
        });
    });
    print("ok\n");
}
