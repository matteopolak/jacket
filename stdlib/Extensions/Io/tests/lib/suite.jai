// The Io engine tests, shared by the test programs next to this directory. Every test runs on a
// fiber inside `io_run`, on whichever backend that scheduler has, so `backends.jai` can run all of
// them once per backend. Nothing here may depend on the machine's network beyond loopback.
#import "Basic";
Io :: #import "Extensions/Io";
Atomics :: #import "Atomics";

LOOPBACK :: "127.0.0.1:0";

now_ms :: () -> s64 {
    return to_milliseconds(current_time_monotonic());
}

pattern :: (index: int, round: int, position: int) -> u8 {
    return cast(u8) ((index * 31 + round * 7 + position) & 0xff);
}

// A connected pair on loopback: connect first (the kernel completes it from the backlog), then accept.
Pair :: struct {
    listener: Io.Tcp_Listener;
    client: Io.Tcp_Stream;
    server: Io.Tcp_Stream;
}

make_pair :: () -> Pair {
    pair: Pair;
    error: Io.Io_Error;

    pair.listener, error = Io.tcp_listen(LOOPBACK);
    assert(error == .NONE, "listen: %", error);

    address, address_error := Io.local_address(pair.listener);

    assert(address_error == .NONE && address.port != 0, "local_address");

    pair.client, error = Io.connect(address);
    assert(error == .NONE, "connect: %", error);

    pair.server, error = Io.accept(pair.listener);
    assert(error == .NONE, "accept: %", error);

    return pair;
}

close_pair :: (pair: Pair) {
    assert(Io.close(pair.client) == .NONE);
    assert(Io.close(pair.server) == .NONE);
    assert(Io.close(pair.listener) == .NONE);
}

// Waits for a fiber to report on a channel.
Report :: struct {
    channel: *Io.Channel(int);
}

// ---------------------------------------------------------------------------------------------
// Addresses

test_addresses :: () {
    address, ok := Io.parse_address("127.0.0.1:8080");

    assert(ok && address.family == .IPV4 && address.port == 8080);
    assert(address.bytes[0] == 127 && address.bytes[3] == 1);
    assert(Io.address_to_string(address) == "127.0.0.1:8080");

    address, ok = Io.parse_address(":9");
    assert(ok && address.family == .IPV4 && address.port == 9 && address.bytes[0] == 0);

    address, ok = Io.parse_address("10.0.0.255");
    assert(ok && address.port == 0 && address.bytes[3] == 255);

    address, ok = Io.parse_address("[::1]:443");
    assert(ok && address.family == .IPV6 && address.port == 443);
    assert(address.bytes[15] == 1 && address.bytes[0] == 0);
    assert(Io.address_to_string(address) == "[::1]:443", "got %", Io.address_to_string(address));

    address, ok = Io.parse_address("[2001:db8:0:0:1::7]:80");
    assert(ok && address.bytes[0] == 0x20 && address.bytes[1] == 0x01 && address.bytes[15] == 7);
    assert(Io.address_to_string(address) == "[2001:db8::1:0:0:7]:80", "got %", Io.address_to_string(address));

    address, ok = Io.parse_address("::");
    assert(ok && address.family == .IPV6);
    assert(Io.address_to_string(address) == "[::]:0");

    address, ok = Io.parse_address("[::ffff:1.2.3.4]:5");
    assert(ok && address.bytes[10] == 0xff && address.bytes[11] == 0xff && address.bytes[12] == 1 && address.bytes[15] == 4);

    bad := string.["", "1.2.3", "1.2.3.4.5", "256.1.1.1", "1.2.3.4:", "1.2.3.4:99999", "[::1", "[::1]x", "1::2::3", ":::", "12345::", "1.2.3.4:5:6", "host"];
    for bad {
        _, valid := Io.parse_address(it);
        if it == "" continue; // the empty string is "any address"

        assert(!valid, "'%' should not parse", it);
    }
}

// ---------------------------------------------------------------------------------------------
// TCP

Echo_Job :: struct {
    group: *Io.Wait_Group;
    conn: Io.Tcp_Stream;
}

echo_connection :: (job: Echo_Job) {
    defer Io.done(job.group);
    defer Io.close(job.conn);

    buffer: [2048] u8;

    while true {
        count, error := Io.read(job.conn, buffer);
        if error != .NONE || count == 0 return;

        _, write_error := Io.write(job.conn, .{ count, buffer.data });
        if write_error != .NONE return;
    }
}

Echo_Server :: struct {
    listener: Io.Tcp_Listener;
    group: Io.Wait_Group;
    accepted: int;
}

echo_accept_loop :: (server: *Echo_Server) {
    while true {
        conn, error := Io.accept(server.listener);
        if error != .NONE break;

        server.accepted += 1;
        Io.add(*server.group);
        Io.spawn(echo_connection, Echo_Job.{ *server.group, conn });
    }
}

Echo_Client :: struct {
    port: u16;
    index: int;
    group: *Io.Wait_Group;
    failures: *int;
}

echo_client :: (client: Echo_Client) {
    defer Io.done(client.group);

    conn, error := Io.connect(Io.address_v4(127, 0, 0, 1, client.port));
    if error != .NONE {
        client.failures.* += 1;

        return;
    }

    defer Io.close(conn);

    sizes := int.[1, 7, 300, 2048, 5000];

    for size, round: sizes {
        message := NewArray(size, u8, false);

        defer free(message.data);

        for 0..size - 1 message[it] = pattern(client.index, round, it);

        written, write_error := Io.write(conn, message);
        if write_error != .NONE || written != size {
            client.failures.* += 1;

            return;
        }

        reply := NewArray(size, u8, false);

        defer free(reply.data);

        count, read_error := Io.read_full(conn, reply);
        if read_error != .NONE || count != size {
            client.failures.* += 1;

            return;
        }

        for 0..size - 1 {
            if reply[it] != message[it] {
                client.failures.* += 1;

                return;
            }
        }
    }
}

test_echo_many_clients :: () {
    CLIENTS :: 40;

    server: Echo_Server;
    error: Io.Io_Error;

    server.listener, error = Io.tcp_listen(LOOPBACK);
    assert(error == .NONE, "listen: %", error);

    address, _ := Io.local_address(server.listener);

    Io.spawn(echo_accept_loop, *server);

    clients: Io.Wait_Group;
    failures := 0;

    Io.add(*clients, CLIENTS);
    for 1..CLIENTS Io.spawn(echo_client, Echo_Client.{ address.port, it, *clients, *failures });

    assert(Io.wait(*clients) == .NONE);
    assert(failures == 0, "% clients failed", failures);
    assert(server.accepted == CLIENTS, "accepted %", server.accepted);

    // Closing the listener ends the accept loop; the handlers end as their clients hang up.
    assert(Io.close(server.listener) == .NONE);
    assert(Io.wait(*server.group) == .NONE);
}

// A transfer much larger than the socket buffers: both sides see partial reads and writes.
Sink :: struct {
    listener: Io.Tcp_Listener;
    received: int;
    sum: int;
    chunks: int;
    finished: bool;
}

sink_run :: (sink: *Sink) {
    conn, error := Io.accept(sink.listener);

    assert(error == .NONE, "accept: %", error);

    buffer: [4093] u8;

    while true {
        count, read_error := Io.read(conn, buffer);

        assert(read_error == .NONE, "sink read: %", read_error);
        if count == 0 break;

        for 0..count - 1 sink.sum += buffer[it];

        sink.received += count;
        sink.chunks += 1;

        // Reading slower than the writer writes makes it wait for room.
        if sink.chunks % 16 == 0 Io.yield();
    }

    sink.finished = true;

    Io.close(conn);
}

test_large_transfer :: () {
    TOTAL :: 600_000;

    sink: Sink;
    error: Io.Io_Error;

    sink.listener, error = Io.tcp_listen(LOOPBACK);
    assert(error == .NONE);

    address, _ := Io.local_address(sink.listener);

    Io.spawn(sink_run, *sink);

    conn, connect_error := Io.connect(address);

    assert(connect_error == .NONE);

    data := NewArray(TOTAL, u8, false);

    defer free(data.data);

    expected := 0;

    for 0..TOTAL - 1 {
        data[it] = pattern(3, 4, it);
        expected += data[it];
    }

    written, write_error := Io.write(conn, data);

    assert(write_error == .NONE && written == TOTAL, "wrote % of % (%)", written, TOTAL, write_error);

    // The sink sees the end of the stream once the sending side is shut down, then closes: the
    // client reads the end of the stream too.
    assert(Io.shutdown_write(conn) == .NONE);

    buffer: [16] u8;
    count, read_error := Io.read(conn, buffer);

    assert(read_error == .NONE && count == 0, "expected the end of the stream, got % (%)", count, read_error);

    assert(sink.finished && sink.received == TOTAL, "sink got %", sink.received);
    assert(sink.sum == expected, "checksum");
    assert(sink.chunks > 10, "the transfer arrived in % pieces", sink.chunks);

    Io.close(conn);
    Io.close(sink.listener);
}

test_end_of_stream :: () {
    pair := make_pair();

    _, error := Io.write(pair.server, "bye");

    assert(error == .NONE);
    assert(Io.close(pair.server) == .NONE);

    buffer: [16] u8;

    // The data first, then the end, and the end stays the end.
    count, read_error := Io.read(pair.client, buffer);

    assert(read_error == .NONE && count == 3 && buffer[0] == #char "b" && buffer[2] == #char "e");

    for 1..3 {
        count, read_error = Io.read(pair.client, buffer);
        assert(read_error == .NONE && count == 0, "end of stream: % %", count, read_error);
    }

    count, read_error = Io.read_full(pair.client, buffer);
    assert(count == 0 && read_error == .END_OF_FILE, "read_full at the end gave % %", count, read_error);

    // An empty read does nothing, even at the end.
    count, read_error = Io.read(pair.client, .[]);
    assert(count == 0 && read_error == .NONE);

    // Writing to a peer that went away fails with a broken pipe or a reset (Windows reports an
    // aborted connection), never hangs or kills the process.
    failed := false;

    for 1..50 {
        _, write_error := Io.write(pair.client, "still there?");
        if write_error != .NONE {
            assert(write_error == .BROKEN_PIPE || write_error == .CONNECTION_RESET || write_error == .CONNECTION_ABORTED, "write failed with %", write_error);
            failed = true;

            break;
        }

        Io.sleep(2);
    }

    assert(failed, "writing to a closed peer never failed");

    Io.close(pair.client);
    Io.close(pair.listener);
}

test_connection_reset :: () {
    pair := make_pair();

    // Closing a socket that still has unread data resets the connection.
    _, error := Io.write(pair.client, "unread");

    assert(error == .NONE);

    Io.sleep(20);
    Io.close(pair.server);

    buffer: [16] u8;
    count, read_error := Io.read(pair.client, buffer);

    assert(read_error == .CONNECTION_RESET || read_error == .BROKEN_PIPE || read_error == .CONNECTION_ABORTED, "read after a reset gave % % (code %)", count, read_error, Io.last_platform_code());
    assert(count == 0);
    assert(Io.last_platform_code() != 0);

    Io.close(pair.client);
    Io.close(pair.listener);
}

test_connection_refused :: () {
    // A port nothing listens on: listen, learn the number, close.
    listener, error := Io.tcp_listen(LOOPBACK);

    assert(error == .NONE);

    address, _ := Io.local_address(listener);

    assert(Io.close(listener) == .NONE);

    _, connect_error := Io.connect(address);

    assert(connect_error == .CONNECTION_REFUSED, "connect to a closed port gave %", connect_error);

    // A refused attempt leaves nothing behind: a new listener and connection on the same scheduler work.
    pair := make_pair();

    close_pair(pair);
}

test_address_in_use :: () {
    first, error := Io.tcp_listen(LOOPBACK);

    assert(error == .NONE);

    address, _ := Io.local_address(first);

    _, second_error := Io.tcp_listen(address);

    assert(second_error == .ADDRESS_IN_USE, "second listen gave %", second_error);

    Io.close(first);
}

test_local_and_peer_address :: () {
    pair := make_pair();

    server_address, error := Io.local_address(pair.server);

    assert(error == .NONE);

    peer, peer_error := Io.peer_address(pair.client);

    assert(peer_error == .NONE && peer.port == server_address.port);
    assert(peer.bytes[0] == 127 && peer.bytes[3] == 1);

    client_address, _ := Io.local_address(pair.client);
    seen, _ := Io.peer_address(pair.server);

    assert(seen.port == client_address.port);

    close_pair(pair);
}

Waiter_Job :: struct {
    stream: Io.Tcp_Stream;
    result: *Io.Io_Error;
    count: *int;
    group: *Io.Wait_Group;
}

blocked_reader :: (job: Waiter_Job) {
    defer Io.done(job.group);

    buffer: [8] u8;
    job.count.*, job.result.* = Io.read(job.stream, buffer);
}

// Closing a stream wakes the fibers waiting on it.
test_close_wakes_waiters :: () {
    pair := make_pair();

    group: Io.Wait_Group;
    result: Io.Io_Error;
    count := -1;

    Io.add(*group);
    Io.spawn(blocked_reader, Waiter_Job.{ pair.client, *result, *count, *group });
    Io.sleep(10);

    assert(count == -1, "the reader should still be waiting");

    assert(Io.close(pair.client) == .NONE);
    assert(Io.wait(*group) == .NONE);
    assert(result == .CLOSED, "closed under a reader gave %", result);

    Io.close(pair.server);
    Io.close(pair.listener);
}

test_resolve :: () {
    address, error := Io.resolve("127.0.0.1", 99);

    assert(error == .NONE && address.port == 99 && address.bytes[0] == 127);

    // A name goes through the system resolver on a helper thread.
    address, error = Io.resolve("localhost", 4242);
    assert(error == .NONE && address.port == 4242, "localhost: %", error);
    assert(address.family == .IPV4 || address.family == .IPV6);

    listener, listen_error := Io.tcp_listen(LOOPBACK);

    assert(listen_error == .NONE);

    bound, _ := Io.local_address(listener);
    connection_text := tprint("localhost:%", bound.port);

    conn, connect_error := Io.connect(connection_text);

    assert(connect_error == .NONE, "connect to '%': %", connection_text, connect_error);

    accepted, accept_error := Io.accept(listener);

    assert(accept_error == .NONE);

    Io.close(conn);
    Io.close(accepted);
    Io.close(listener);

    // Several lookups at once, from several fibers.
    group: Io.Wait_Group;
    failures := 0;

    Io.add(*group, 6);
    for 1..6 Io.spawn(lookup_fiber, Lookup_Job.{ *group, *failures });

    Io.wait(*group);
    assert(failures == 0);
}

Lookup_Job :: struct {
    group: *Io.Wait_Group;
    failures: *int;
}

lookup_fiber :: (job: Lookup_Job) {
    defer Io.done(job.group);

    address, error := Io.resolve("localhost", 7);
    if error != .NONE || address.port != 7 job.failures.* += 1;
}

// ---------------------------------------------------------------------------------------------
// Time: sleeps, deadlines, cancellation

test_sleep_uses_the_engine :: () {
    started := now_ms();

    assert(Io.sleep(40) == .NONE);

    elapsed := now_ms() - started;

    assert(elapsed >= 38 && elapsed < 1500, "slept % ms", elapsed);

    // Sleeping fibers and fibers waiting for IO share a thread.
    pair := make_pair();

    group: Io.Wait_Group;
    result: Io.Io_Error;
    count := 0;

    Io.add(*group);
    Io.spawn(blocked_reader, Waiter_Job.{ pair.client, *result, *count, *group });

    started = now_ms();
    Io.sleep(30);
    elapsed = now_ms() - started;
    assert(elapsed >= 28 && elapsed < 1500, "slept % ms next to a waiting reader", elapsed);

    Io.write(pair.server, "x");
    Io.wait(*group);
    assert(result == .NONE && count == 1);

    close_pair(pair);
}

test_read_deadline :: () {
    pair := make_pair();
    buffer: [16] u8;

    started := now_ms();
    count: s64;
    error: Io.Io_Error;

    Io.with_deadline(60, #code {
        count, error = Io.read(pair.client, buffer);
    });

    elapsed := now_ms() - started;

    assert(error == .TIMED_OUT && count == 0, "read gave % %", count, error);
    assert(elapsed >= 55 && elapsed < 1500, "timed out after % ms", elapsed);

    // The deadline ended that call only: the connection is fine, and a later read gets the data.
    assert(!Io.is_canceled());
    Io.write(pair.server, "later");

    count, error = Io.read(pair.client, buffer);
    assert(error == .NONE && count == 5, "read after a timeout gave % %", count, error);

    close_pair(pair);
}

test_deadline_nesting :: () {
    pair := make_pair();
    buffer: [16] u8;

    count: s64;
    error: Io.Io_Error;
    started := now_ms();

    Io.with_deadline(5000, #code {
        Io.with_deadline(50, #code {
            count, error = Io.read(pair.client, buffer);
        });

        assert(error == .TIMED_OUT);
        assert(now_ms() - started < 1500, "the inner deadline was ignored");

        // The outer deadline is back in force: a short wait for data that arrives is fine.
        Io.spawn((stream: Io.Tcp_Stream) {
            Io.sleep(20);
            Io.write(stream, "ok");
        }, pair.server);
        count, error = Io.read(pair.client, buffer);
        assert(error == .NONE && count == 2, "outer: % %", count, error);
    });

    // Outside every block there is no deadline: waiting 80 ms for data works.
    Io.spawn((stream: Io.Tcp_Stream) {
        Io.sleep(80);
        Io.write(stream, "late");
    }, pair.server);
    count, error = Io.read(pair.client, buffer);
    assert(error == .NONE && count == 4, "no deadline: % %", count, error);

    // A deadline that has passed fails the call before it starts, even with data waiting.
    Io.write(pair.server, "ready");
    Io.sleep(10);

    Io.with_deadline(0, #code {
        count, error = Io.read(pair.client, buffer);
    });

    assert(error == .TIMED_OUT, "expired deadline gave %", error);

    count, error = Io.read(pair.client, buffer);
    assert(error == .NONE && count == 5);

    close_pair(pair);
}

test_deadline_sleep_and_accept :: () {
    started := now_ms();
    error: Io.Io_Error;

    Io.with_deadline(30, #code {
        error = Io.sleep(5000);
    });

    assert(error == .TIMED_OUT, "sleep under a deadline gave %", error);
    assert(now_ms() - started < 1500);

    listener, listen_error := Io.tcp_listen(LOOPBACK);

    assert(listen_error == .NONE);

    started = now_ms();

    Io.with_deadline(40, #code {
        _, error = Io.accept(listener);
    });

    assert(error == .TIMED_OUT, "accept under a deadline gave %", error);
    assert(now_ms() - started >= 35 && now_ms() - started < 1500);

    // The listener still works afterwards.
    address, _ := Io.local_address(listener);
    conn, connect_error := Io.connect(address);

    assert(connect_error == .NONE);

    accepted, accept_error := Io.accept(listener);

    assert(accept_error == .NONE);

    Io.close(conn);
    Io.close(accepted);
    Io.close(listener);
}

Cancel_Job :: struct {
    listener: Io.Tcp_Listener;
    stream: Io.Tcp_Stream;
    result: *Io.Io_Error;
    count: *int;
    after: *Io.Io_Error;
    group: *Io.Wait_Group;
}

accept_until_canceled :: (job: Cancel_Job) {
    defer Io.done(job.group);

    _, job.result.* = Io.accept(job.listener);

    // Cancellation is sticky: later IO fails the same way, but a close still works.
    buffer: [4] u8;

    _, job.after.* = Io.read(job.stream, buffer);
}

read_until_canceled :: (job: Cancel_Job) {
    defer Io.done(job.group);

    buffer: [4] u8;
    job.count.*, job.result.* = Io.read(job.stream, buffer);

    job.after.* = Io.close(job.stream);
}

test_cancel_accept :: () {
    pair := make_pair();

    group: Io.Wait_Group;
    result, after: Io.Io_Error;
    count := 0;

    Io.add(*group);
    handle := Io.spawn(accept_until_canceled, Cancel_Job.{ pair.listener, pair.client, *result, *count, *after, *group });

    Io.sleep(10);
    assert(result == .NONE, "still waiting");

    started := now_ms();

    Io.cancel(handle);
    Io.wait(*group);

    assert(result == .CANCELED, "accept after cancel gave %", result);
    assert(after == .CANCELED, "read after cancel gave %", after);
    assert(now_ms() - started < 1000, "cancel took % ms", now_ms() - started);

    close_pair(pair);
}

test_cancel_read :: () {
    pair := make_pair();

    group: Io.Wait_Group;
    result, after: Io.Io_Error;
    count := 7;

    Io.add(*group);
    handle := Io.spawn(read_until_canceled, Cancel_Job.{ pair.listener, pair.client, *result, *count, *after, *group });

    Io.sleep(10);
    Io.cancel(handle);
    Io.wait(*group);

    assert(result == .CANCELED && count == 0, "read after cancel gave % %", count, result);
    assert(after == .NONE, "close in a canceled fiber gave %", after);

    Io.close(pair.server);
    Io.close(pair.listener);
}

// More than the kernel will buffer for a peer that never reads (Linux loopback takes about 10 MB).
UNREAD_BYTES :: 64 * 1024 * 1024;

Writer_Job :: struct {
    stream: Io.Tcp_Stream;
    written: *int;
    result: *Io.Io_Error;
    group: *Io.Wait_Group;
}

write_until_canceled :: (job: Writer_Job) {
    defer Io.done(job.group);

    data := NewArray(UNREAD_BYTES, u8, false);
    defer free(data.data);

    job.written.*, job.result.* = Io.write(job.stream, data);
}

// A write to a peer that never reads blocks once the buffers are full; cancel ends it and reports
// how much went out.
test_cancel_write :: () {
    pair := make_pair();

    group: Io.Wait_Group;
    written := -1;
    result: Io.Io_Error;

    Io.add(*group);
    handle := Io.spawn(write_until_canceled, Writer_Job.{ pair.client, *written, *result, *group });

    Io.sleep(50);
    assert(written == -1, "the write cannot have finished");

    Io.cancel(handle);
    Io.wait(*group);

    assert(result == .CANCELED, "write after cancel gave %", result);
    assert(written > 0 && written < UNREAD_BYTES, "wrote %", written);

    // The bytes that did go out arrive intact.
    total := 0;
    buffer: [65536] u8;

    while total < written {
        count, error := Io.read(pair.server, buffer);
        assert(error == .NONE && count > 0, "reading what was sent: % %", count, error);

        total += count;
    }

    assert(total == written);

    close_pair(pair);
}

Resolve_Cancel :: struct {
    result: *Io.Io_Error;
    group: *Io.Wait_Group;
}

resolving_fiber :: (job: Resolve_Cancel) {
    defer Io.done(job.group);

    _, job.result.* = Io.resolve("localhost", 1);
}

// Starts a lookup in another fiber and cancels it once that fiber waits, and returns what the
// lookup gave. A helper thread can answer before the cancel lands (on a busy machine), and then
// the lookup succeeds instead, so either result is fine here.
cancel_a_lookup :: () -> Io.Io_Error {
    group: Io.Wait_Group;
    result := Io.Io_Error.NONE;

    Io.add(*group);
    handle := Io.spawn(resolving_fiber, Resolve_Cancel.{ *result, *group });

    Io.yield();
    Io.cancel(handle);
    Io.wait(*group);

    assert(result == .CANCELED || result == .NONE, "resolve after cancel gave %", result);

    return result;
}

// A fiber that waits for a name lookup stops waiting when it is canceled; the lookup's own answer
// arrives later and is dropped.
test_cancel_resolve :: () {
    // Quick answers can beat the cancel, but not five times in a row.
    canceled := false;

    for 1..5 {
        if cancel_a_lookup() == .CANCELED {
            canceled = true;

            break;
        }
    }

    assert(canceled, "no lookup was canceled in five tries");

    // Let the abandoned answer arrive, then check lookups still work.
    Io.sleep(30);

    address, error := Io.resolve("localhost", 5);

    assert(error == .NONE && address.port == 5, "resolve after an abandoned lookup: %", error);

    Io.with_deadline(0, #code {
        _, error = Io.resolve("localhost", 5);
    });

    assert(error == .TIMED_OUT, "resolve past the deadline gave %", error);
}

// A scheduler that ends right after a lookup was given up: it waits for the helper's answer before
// it releases the engine, so the late answer cannot touch freed memory.
test_run_ends_with_lookup_under_way :: () {
    cancel_a_lookup();
}

acceptor_started: bool;
acceptor_finished: bool;

parked_acceptor :: (listener: Io.Tcp_Listener) {
    acceptor_started = true;
    Io.accept(listener);
    acceptor_finished = true;
}

// `io_run` returns even when fibers are still blocked in IO: the root fiber just returns here, and
// `run_group` checks afterwards that the blocked accept was canceled and ended.
test_run_ends_blocked_fibers :: () {
    listener, error := Io.tcp_listen(LOOPBACK);

    assert(error == .NONE);

    acceptor_started = false;
    acceptor_finished = false;
    Io.spawn(parked_acceptor, listener);

    while !acceptor_started Io.yield();
}

// ---------------------------------------------------------------------------------------------
// Threads: one scheduler each, listeners sharing a port

Shared_Port :: struct {
    port: u16;
    ready: s32;
    claimed: s32;
    done: s32;
    accepted: s32;
    failures: s32;
}

shared: Shared_Port;

ping_connection :: (conn: Io.Tcp_Stream) {
    defer Io.close(conn);

    buffer: [4] u8;
    count, error := Io.read(conn, buffer);

    if error != .NONE || count != 1 return;

    Io.write(conn, .{ 1, buffer.data });
}

shared_accept_loop :: (listener: Io.Tcp_Listener) {
    while true {
        conn, error := Io.accept(listener);
        if error != .NONE break;

        Atomics.atomic_add(*shared.accepted, 1);
        Io.spawn(ping_connection, conn);
    }
}

thread_main :: () {
    listener, error := Io.tcp_listen(Io.address_v4(127, 0, 0, 1, shared.port), reuse_port = true);

    assert(error == .NONE, "listen with reuse_port: %", error);

    Io.spawn(shared_accept_loop, listener);

    Atomics.atomic_add(*shared.ready, 1);

    me := Atomics.atomic_add(*shared.claimed, 1);

    if me == 0 {
        while Atomics.atomic_load(*shared.ready) < 2 Io.sleep(1);

        for 1..20 {
            conn, connect_error := Io.connect(Io.address_v4(127, 0, 0, 1, shared.port));
            if connect_error != .NONE {
                Atomics.atomic_add(*shared.failures, 1);

                continue;
            }

            byte: u8 = 7;
            buffer: [4] u8;

            Io.write(conn, .{ 1, *byte });

            count, read_error := Io.read(conn, buffer);
            if read_error != .NONE || count != 1 || buffer[0] != 7 Atomics.atomic_add(*shared.failures, 1);

            Io.close(conn);
        }

        Atomics.atomic_store(*shared.done, 1);
    } else {
        while Atomics.atomic_load(*shared.done) == 0 Io.sleep(2);
    }

    Io.close(listener);
}

test_threads_share_a_port :: () {
    // Pick a free port, then let two schedulers listen on it at once.
    probe, error := Io.tcp_listen(LOOPBACK);

    assert(error == .NONE);

    address, _ := Io.local_address(probe);

    Io.close(probe);

    shared = .{};
    shared.port = address.port;

    // io_run_threads starts its own schedulers and cannot run inside one, so this only prepares
    // the port; `run_group` calls `run_threads_here` after io_run has ended.
    pending_threads = true;
}

pending_threads: bool;

run_threads_here :: () {
    if !pending_threads return;

    Io.io_run_threads(2, thread_main);

    assert(shared.failures == 0, "% clients failed", shared.failures);
    assert(shared.accepted == 20, "accepted % connections", shared.accepted);
}

// ---------------------------------------------------------------------------------------------
// Files

SCRATCH :: ".scratch/io-engine";

make_scratch :: () {
    #if OS == .WINDOWS {
        kernel32 :: #system_library "kernel32";
        CreateDirectoryA :: (path: *u8, security: *void) -> s32 #foreign kernel32;

        CreateDirectoryA(".scratch", null);
        CreateDirectoryA(SCRATCH, null);
    } else {
        libc :: #system_library "libc";
        mkdir :: (path: *u8, mode: u32) -> s32 #foreign libc;

        mkdir(".scratch", 0x1ed);
        mkdir(SCRATCH, 0x1ed);
    }
}

test_file_round_trip :: () {
    make_scratch();

    path := tprint("%/round-trip.bin", SCRATCH);

    small := "hello, files";

    written := Io.write_file(path, small);

    assert(written == .NONE, "writing the small file: %", written);

    content, error := Io.read_file(path);

    assert(error == .NONE && string.{ content.count, content.data } == small, "small file: % %", error, content.count);
    free(content.data);

    // Larger than any read buffer the engine might use.
    SIZE :: 300_001;

    data := NewArray(SIZE, u8, false);

    defer free(data.data);

    for 0..SIZE - 1 data[it] = pattern(9, 1, it);

    assert(Io.write_file(path, data) == .NONE);

    content, error = Io.read_file(path);
    assert(error == .NONE && content.count == SIZE, "large file: % %", error, content.count);

    same := true;

    for 0..SIZE - 1 {
        if content[it] != data[it] {
            same = false;

            break;
        }
    }

    assert(same, "large file content differs");
    free(content.data);

    // An empty file.
    assert(Io.write_file(path, "") == .NONE);

    content, error = Io.read_file(path);
    assert(error == .NONE && content.count == 0);
    free(content.data);
}

test_file_modes_and_offsets :: () {
    make_scratch();

    path := tprint("%/modes.txt", SCRATCH);

    file, error := Io.open_file(path, .WRITE);

    assert(error == .NONE, "open for writing: %", error);

    count, write_error := Io.write(*file, "0123456789");

    assert(write_error == .NONE && count == 10);
    assert(file.position == 10);

    // Positional writes leave the position alone and may extend the file.
    count, write_error = Io.write_at(*file, "ab", 3);
    assert(write_error == .NONE && count == 2 && file.position == 10);

    count, write_error = Io.write_at(*file, "Z", 12);
    assert(write_error == .NONE && count == 1);

    size, size_error := Io.file_size(*file);

    assert(size_error == .NONE && size == 13, "size %", size);

    assert(Io.close(*file) == .NONE);

    file, error = Io.open_file(path, .READ);
    assert(error == .NONE);

    buffer: [32] u8;

    count, error = Io.read(*file, .{ 4, buffer.data });
    assert(error == .NONE && count == 4 && string.{ 4, buffer.data } == "012a", "read: '%'", string.{ 4, buffer.data });
    assert(file.position == 4);

    count, error = Io.read(*file, buffer);
    assert(error == .NONE && count == 9, "read the rest: %", count);
    assert(buffer[0] == #char "b" && buffer[8] == #char "Z");

    // At the end: 0 bytes, no error, and again.
    for 1..2 {
        count, error = Io.read(*file, buffer);
        assert(error == .NONE && count == 0);
    }

    count, error = Io.read_at(*file, .{ 3, buffer.data }, 7);
    assert(error == .NONE && count == 3 && string.{ 3, buffer.data } == "789");

    count, error = Io.read_at(*file, buffer, 1000);
    assert(error == .NONE && count == 0);

    // A file opened for reading cannot be written.
    _, write_error = Io.write_at(*file, "x", 0);
    assert(write_error != .NONE, "writing to a read-only file succeeded");
    assert(Io.last_platform_code() != 0);

    assert(Io.close(*file) == .NONE);

    // Append adds to the end of what is there.
    file, error = Io.open_file(path, .APPEND);
    assert(error == .NONE);
    Io.write(*file, "++");
    Io.close(*file);

    content, read_error := Io.read_file(path);

    assert(read_error == .NONE && content.count == 15);
    assert(string.{ 2, content.data + 13 } == "++", "appended: '%'", string.{ content.count, content.data });
    free(content.data);

    // Read-write keeps the content.
    file, error = Io.open_file(path, .READ_WRITE);
    assert(error == .NONE);

    Io.write_at(*file, "XY", 0);
    count, error = Io.read_at(*file, .{ 4, buffer.data }, 0);
    assert(error == .NONE && string.{ 4, buffer.data } == "XY2a");

    Io.close(*file);
}

test_file_errors :: () {
    make_scratch();

    _, error := Io.open_file(tprint("%/does-not-exist", SCRATCH), .READ);

    assert(error == .NOT_FOUND, "missing file gave %", error);
    assert(Io.last_platform_code() != 0);

    _, error = Io.read_file(tprint("%/also-missing", SCRATCH));
    assert(error == .NOT_FOUND);

    _, error = Io.open_file(SCRATCH, .WRITE);
    assert(error == .IS_A_DIRECTORY, "opening a directory for writing gave %", error);

    _, error = Io.open_file(tprint("%/no-such-directory/file", SCRATCH), .WRITE);
    assert(error == .NOT_FOUND, "missing directory gave %", error);
}

File_Job :: struct {
    index: int;
    group: *Io.Wait_Group;
    failures: *int;
}

file_fiber :: (job: File_Job) {
    defer Io.done(job.group);

    path := tprint("%/parallel-%.bin", SCRATCH, job.index);

    data := NewArray(20_000 + job.index * 100, u8, false);

    defer free(data.data);

    for *data it.* = pattern(job.index, 0, it_index);

    if Io.write_file(path, data) != .NONE {
        job.failures.* += 1;

        return;
    }

    Io.yield();

    content, error := Io.read_file(path);

    defer free(content.data);

    if error != .NONE || content.count != data.count {
        job.failures.* += 1;

        return;
    }

    for 0..data.count - 1 {
        if content[it] != data[it] {
            job.failures.* += 1;

            return;
        }
    }
}

test_files_from_many_fibers :: () {
    make_scratch();

    group: Io.Wait_Group;
    failures := 0;

    Io.add(*group, 12);
    for 1..12 Io.spawn(file_fiber, File_Job.{ it, *group, *failures });

    Io.wait(*group);
    assert(failures == 0, "% file fibers failed", failures);
}

Big_Read :: struct {
    file: Io.Io_File;
    result: *Io.Io_Error;
    count: *int;
    group: *Io.Wait_Group;
}

big_reader :: (job: Big_Read) {
    defer Io.done(job.group);

    BYTES :: 1_500_000;

    buffer := NewArray(BYTES, u8, false);

    defer free(buffer.data);

    job.count.*, job.result.* = Io.read_at(*job.file, buffer, 0);
}

// A large read can be out (on the thread pool, in an IoRing, in the kernel) when another fiber
// cancels the reader or closes the file. Neither may crash or lose the buffer: the reader ends
// with its data, or with the cancel or close error, and the scheduler stays usable.
test_file_cancel_and_close_under_read :: () {
    make_scratch();

    path := tprint("%/big-read.bin", SCRATCH);
    data := NewArray(1_500_000, u8, false);

    defer free(data.data);

    memset(data.data, 7, data.count);

    assert(Io.write_file(path, data) == .NONE);

    for variant: 0..1 {
        file, error := Io.open_file(path, .READ);

        assert(error == .NONE, "open: %", error);

        group: Io.Wait_Group;
        result: Io.Io_Error;
        count := -1;

        Io.add(*group);
        handle := Io.spawn(big_reader, Big_Read.{ file, *result, *count, *group });

        // The reader runs until it waits for its read.
        Io.yield();

        if variant == 0 {
            Io.cancel(handle);
            Io.wait(*group);

            assert(result == .NONE || result == .CANCELED, "read canceled under way gave %", result);
            assert(Io.close(*file) == .NONE);
        } else {
            assert(Io.close(*file) == .NONE);
            Io.wait(*group);

            assert(result == .NONE || result == .CLOSED || result == .INVALID || result == .OTHER, "read closed under way gave %", result);
        }

        if result == .NONE assert(count == data.count, "read % of %", count, data.count);
    }

    // The scheduler and the files still work.
    content, read_error := Io.read_file(path);

    assert(read_error == .NONE && content.count == data.count);
    free(content.data);
}

// ---------------------------------------------------------------------------------------------
// Groups

last_backend: string;

run_tcp :: () {
    last_backend = Io.backend_name();

    test_addresses();
    test_echo_many_clients();
    test_large_transfer();
    test_end_of_stream();
    test_connection_reset();
    test_connection_refused();
    test_address_in_use();
    test_local_and_peer_address();
    test_close_wakes_waiters();
    test_resolve();
}

run_timeouts :: () {
    last_backend = Io.backend_name();

    test_sleep_uses_the_engine();
    test_read_deadline();
    test_deadline_nesting();
    test_deadline_sleep_and_accept();
    test_cancel_accept();
    test_cancel_read();
    test_cancel_write();
    test_cancel_resolve();
    test_run_ends_blocked_fibers();
}

run_files :: () {
    last_backend = Io.backend_name();

    test_file_round_trip();
    test_file_modes_and_offsets();
    test_file_errors();
    test_files_from_many_fibers();
    test_file_cancel_and_close_under_read();
}

run_all :: () {
    last_backend = Io.backend_name();

    run_tcp();
    run_timeouts();
    run_files();
    test_threads_share_a_port();
}

// Runs `group` on a new scheduler and the checks that need the scheduler to be over.
run_group :: (group: () -> ()) {
    Io.io_run(group);
    Io.io_run(test_run_ends_with_lookup_under_way);

    if acceptor_started {
        assert(acceptor_finished, "the blocked accept never returned");
        acceptor_started = false;
    }

    if pending_threads {
        run_threads_here();
        pending_threads = false;
    }
}
