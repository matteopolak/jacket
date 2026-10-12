// Throughput of the Io engine over loopback: echo round trips on long-lived connections, and
// connections per second (connect, one round trip, close). Server and clients share one scheduler,
// so the numbers measure the engine and the fibers, not the machine's network.
//
//     jaic build -O2 stdlib/Extensions/Io/bench/bench.jai -o /tmp/io-bench
//     /tmp/io-bench                       # the default backend
//     JAI_IO_BACKEND=epoll /tmp/io-bench  # a forced one (io_uring, epoll or kqueue)
//
// With a thread count above 1 (`/tmp/io-bench 4`) the connection benchmark runs one scheduler per
// thread, each with its own `SO_REUSEPORT` listener and its own clients.
#import "Basic";
Io :: #import "Extensions/Io";
Atomics :: #import "Atomics";

CONNECTIONS :: 64;
ROUND_TRIPS :: 2000;
MESSAGE_BYTES :: 64;
CHURN_CLIENTS :: 16;
CHURN_PER_CLIENT :: 400;

echo_connection :: (conn: Io.Tcp_Stream) {
    defer Io.close(conn);

    buffer: [4096] u8;

    while true {
        count, error := Io.read(conn, buffer);
        if error != .NONE || count == 0 return;

        _, write_error := Io.write(conn, .{ count, buffer.data });
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

Ping :: struct {
    port: u16;
    group: *Io.Wait_Group;
    bytes: *s64;
}

ping_pong :: (ping: Ping) {
    defer Io.done(ping.group);

    conn, error := Io.connect(Io.address_v4(127, 0, 0, 1, ping.port));

    assert(error == .NONE, "connect: %", error);

    defer Io.close(conn);

    message: [MESSAGE_BYTES] u8;
    reply: [MESSAGE_BYTES] u8;

    for 0..MESSAGE_BYTES - 1 message[it] = cast(u8) it;

    for 1..ROUND_TRIPS {
        _, write_error := Io.write(conn, message);

        assert(write_error == .NONE);

        count, read_error := Io.read_full(conn, reply);

        assert(read_error == .NONE && count == MESSAGE_BYTES);

        ping.bytes.* += MESSAGE_BYTES * 2;
    }
}

Churn :: struct {
    port: u16;
    group: *Io.Wait_Group;
    done: *s64;
    count: int;
}

churn :: (job: Churn) {
    defer Io.done(job.group);

    byte: u8 = 1;
    reply: [1] u8;

    for 1..job.count {
        conn, error := Io.connect(Io.address_v4(127, 0, 0, 1, job.port));

        assert(error == .NONE, "connect: %", error);

        Io.write(conn, .{ 1, *byte });
        Io.read_full(conn, reply);
        Io.close(conn);

        job.done.* += 1;
    }
}

seconds :: (started: Apollo_Time) -> float64 {
    return cast(float64) to_nanoseconds(current_time_monotonic() - started) / 1_000_000_000.0;
}

run_single :: () {
    listener, error := Io.tcp_listen("127.0.0.1:0");

    assert(error == .NONE, "listen: %", error);

    address, _ := Io.local_address(listener);

    Io.spawn(accept_loop, listener);

    print("backend: %\n", Io.backend_name());

    // Echo throughput.
    group: Io.Wait_Group;
    bytes: s64;

    Io.add(*group, CONNECTIONS);

    started := current_time_monotonic();

    for 1..CONNECTIONS Io.spawn(ping_pong, Ping.{ address.port, *group, *bytes });

    Io.wait(*group);

    elapsed := seconds(started);
    round_trips := CONNECTIONS * ROUND_TRIPS;

    print("echo: % connections x % round trips of % bytes: % round trips/s, % MB/s\n", CONNECTIONS, ROUND_TRIPS, MESSAGE_BYTES, cast(s64) (cast(float64) round_trips / elapsed), cast(s64) (cast(float64) bytes / elapsed / 1_000_000));

    // Connection churn.
    done: s64;

    Io.add(*group, CHURN_CLIENTS);

    started = current_time_monotonic();
    for 1..CHURN_CLIENTS Io.spawn(churn, Churn.{ address.port, *group, *done, CHURN_PER_CLIENT });

    Io.wait(*group);

    elapsed = seconds(started);
    print("connections: % in % ms: % connections/s\n", done, cast(s64) (elapsed * 1000), cast(s64) (cast(float64) done / elapsed));

    Io.close(listener);
}

Threaded :: struct {
    port: u16;
    threads: int;
    done: s64;
}

shared: Threaded;

threaded_main :: () {
    listener, error := Io.tcp_listen(Io.address_v4(127, 0, 0, 1, shared.port), reuse_port = true);

    assert(error == .NONE, "listen: %", error);

    Io.spawn(accept_loop, listener);

    group: Io.Wait_Group;
    done: s64;

    Io.add(*group, CHURN_CLIENTS);
    for 1..CHURN_CLIENTS Io.spawn(churn, Churn.{ shared.port, *group, *done, CHURN_PER_CLIENT / shared.threads });

    Io.wait(*group);

    Atomics.atomic_add(*shared.done, done);
    Io.close(listener);
}

pick_port :: () {
    probe, error := Io.tcp_listen("127.0.0.1:0");

    assert(error == .NONE);

    address, _ := Io.local_address(probe);

    Io.close(probe);

    shared.port = address.port;
}

run_threads :: (count: int) {
    shared.threads = count;
    Io.io_run(pick_port);

    started := current_time_monotonic();

    Io.io_run_threads(count, threaded_main);

    elapsed := seconds(started);

    print("connections on % threads: % in % ms: % connections/s\n", count, shared.done, cast(s64) (elapsed * 1000), cast(s64) (cast(float64) shared.done / elapsed));
}

main :: () {
    arguments := get_command_line_arguments();
    threads := 1;

    if arguments.count > 1 {
        value, ok := to_integer(arguments[1]);
        if ok threads = value;
    }

    if threads > 1 {
        run_threads(threads);
    } else {
        Io.io_run(run_single);
    }
}
