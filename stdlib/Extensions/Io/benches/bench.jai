#scope_file
// Run with jacket bench --path stdlib/Extensions/Io.
#import "Extensions/Testing";
#import "Basic";
Io :: #import "Extensions/Io";

CONNECTIONS :: 64;
MESSAGE_BYTES :: 64;

echo_connection :: (conn: Io.Tcp_Stream) {
    defer Io.close(conn);

    buffer: [4096] u8;

    while true {
        count, error := Io.read(conn, buffer);
        if error != .NONE || count == 0 return;

        _, write_error := Io.write(conn, ([] u8).{ count, buffer.data });
        if write_error != .NONE return;
    }
}

accept_loop :: (listener: Io.Tcp_Listener) {
    while true {
        conn, error := Io.accept(listener);
        if error != .NONE return;

        Io.spawn(echo_connection, conn);
    }
}

Io_Bench :: struct {
    b: *Bench;
    churn: bool;
}

io_case :: (b: *Bench, churn: bool) {
    state := Io_Bench.{ b, churn };

    Io.io_run((state: *Io_Bench) {
        b := state.b;
        listener, error := Io.tcp_listen("127.0.0.1:0");

        assert(error == .NONE);
        defer Io.close(listener);
        address, _ := Io.local_address(listener);

        Io.spawn(accept_loop, listener);
        streams: [CONNECTIONS] Io.Tcp_Stream;
        if !state.churn for *streams {
            failure: Io.Io_Error;
            it.*, failure = Io.connect(address);
            assert(failure == .NONE, "connect %: %", Io.address_to_string(address), failure);
        }

        defer if !state.churn for streams Io.close(it);
        message: [MESSAGE_BYTES] u8;
        for *message it.* = cast(u8) it_index;
        if !state.churn set_bytes(b, CONNECTIONS * MESSAGE_BYTES * 2);
        else set_items(b, 16);

        if state.churn {
            group: Io.Wait_Group;

            Io.add(*group, 16);
            for 1..16 Io.spawn(churn_worker, Churn_Job.{ address.port, b.iterations, *group });

            reset_timer(b);
            Io.wait(*group);
        } else {
            reset_timer(b);

            for 0..b.iterations - 1 {
                for streams {
                    _, failure := Io.write(it, black_box(message));
                    assert(failure == .NONE);
                }

                for streams {
                    reply: [MESSAGE_BYTES] u8;
                    got, failure := Io.read_full(it, reply);
                    assert(failure == .NONE && got == MESSAGE_BYTES);
                    black_box(reply);
                }
            }
        }
    }, *state);
}

Atomics :: #import "Atomics";

Thread_Bench :: struct {
    b: *Bench;
    port: u16;
    ready: s32;
    go: s32;
    finished: s32;
}

Churn_Job :: struct {
    port: u16;
    iterations: int;
    group: *Io.Wait_Group;
}

churn_worker :: (job: Churn_Job) {
    defer Io.done(job.group);

    for 0..job.iterations - 1 {
        stream, error := Io.connect(black_box(Io.address_v4(127, 0, 0, 1, job.port)));

        assert(error == .NONE, "connect %: %", job.port, error);
        byte: [1] u8 = .[1];

        _, error = Io.write(stream, black_box(byte));
        assert(error == .NONE);
        got, failure := Io.read_full(stream, byte);

        assert(failure == .NONE && got == 1);
        black_box(byte);
        Io.close(stream);
    }
}

thread_churn :: (state: *Thread_Bench) {
    listener, error := Io.tcp_listen(Io.address_v4(127, 0, 0, 1, state.port), reuse_port = true);

    assert(error == .NONE);
    defer Io.close(listener);
    Io.spawn(accept_loop, listener);
    slot := Atomics.atomic_add(*state.ready, 1);
    if slot == 3 {
        reset_timer(state.b);
        Atomics.atomic_write(*state.go, 1);
    }

    while Atomics.atomic_read(*state.go) == 0 Io.sleep(1);

    group: Io.Wait_Group;

    Io.add(*group, 16);
    for 1..16 Io.spawn(churn_worker, Churn_Job.{ state.port, state.b.iterations, *group });

    Io.wait(*group);
    Atomics.atomic_add(*state.finished, 1);
    while Atomics.atomic_read(*state.finished) != 4 Io.sleep(1);
}

#scope_export
loopback_echo :: (b: *Bench) {
    io_case(b, false);
} @bench
loopback_connections :: (b: *Bench) {
    io_case(b, true);
} @bench

loopback_connections_four_threads :: (b: *Bench) {
    state := Thread_Bench.{ b = b };

    Io.io_run((state: *Thread_Bench) {
        probe, error := Io.tcp_listen("127.0.0.1:0");

        assert(error == .NONE);
        address, failure := Io.local_address(probe);

        assert(failure == .NONE);
        state.port = address.port;
        Io.close(probe);
    }, *state);
    set_items(b, 64);
    Io.io_run_threads(4, thread_churn, *state);
} @bench
