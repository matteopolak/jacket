// test: skip_on(.WASM, reason = "native HTTP/2 ownership")
// Model peer EOF while the connection writer is in cancellation-shielded cleanup.
// No wall-clock assumptions: explicit gates control when the writer can release TLS.
#load "../module.jai";

Channel_Probe :: struct {
    channel: Http2_Channel;
    conn: *H2_Conn;
    hold: Io.Condition;
    released: bool;
    link_closed: bool;
    returned: bool;
    canceled_scope: bool;
    done: Io.Wait_Group;
}

peer_eof :: (_data: *void, _buffer: [] u8) -> s64, Io.Io_Error {
    return 0, .NONE;
}

link_close :: (data: *void, _now: bool) -> Io.Io_Error {
    probe := cast(*Channel_Probe) data;
    probe.link_closed = true;
    return .NONE;
}

dial_attempts := 0;

count_dial :: (_data: *void, _url: Url) -> Link, Io.Io_Error {
    dial_attempts += 1;
    return .{}, .OTHER;
}

Pending_Probe :: struct {
    client: Client;
    channel: *Http2_Channel;
    url: Url;
    returned: bool;
    done: Io.Wait_Group;
    error: Client_Error;
}

open_pending :: (probe: *Pending_Probe) {
    exchange, error := open_http2_exchange(probe.channel, "/bounded", .[]);

    assert(!exchange.state);
    probe.error = error;
    probe.returned = true;
    Io.done(*probe.done);
}

held_writer :: (probe: *Channel_Probe) {
    defer h2_conn_release(probe.conn);
    defer Io.done(*probe.conn.tasks);
    Io.with_grace(Io.NO_DEADLINE, #code {
        while !probe.released Io.wait(*probe.hold);
    });
}

close_scope :: (probe: *Channel_Probe) {
    defer Io.done(*probe.done);

    if probe.canceled_scope {
        Io.with_deadline(0, #code {
            close_http2_channel(*probe.channel);
        });
    } else close_http2_channel(*probe.channel);

    probe.returned = true;
}

Saturated_Probe :: struct {
    client: Client;
    url: Url;
    done: Io.Wait_Group;
    returned: bool;
    error: Client_Error;
}

open_saturated :: (probe: *Saturated_Probe) {
    conn, _, _, error := open_connection(*probe.client, probe.url, 1, http2_only = true);
    if conn h2_conn_release(conn);

    probe.error = error;
    probe.returned = true;
    Io.done(*probe.done);
}

saturated_admission :: () {
    for canceled: bool.[false, true] {
        tls: Tls.Tls_Config;
        tls.alpn = string.["h2"];
        job: Saturated_Probe;
        job.client.tls = *tls;
        job.client.transport.data = *job;
        job.client.transport.open = count_dial;
        job.url, _ = parse_url("https://busy.invalid/");
        conn := h2_conn_new(.CLIENT, .{}, .{});
        conn.owner = *job;
        conn.host = copy_string(job.url.host);
        conn.port = job.url.port;
        conn.tls = *tls;
        conn.remote.max_concurrent_streams = 1;
        conn.idle_since = now_ms();
        stream := h2_stream_new(conn, 1);

        conn.next_stream = 3;
        pool := client_pool();

        array_add(*pool.h2, conn);

        defer {
            for pool.h2 if it == conn {
                array_ordered_remove_by_index(*pool.h2, it_index);
                break;
            }

            h2_conn_release(conn);
        }

        dial_attempts = 0;
        Io.add(*job.done);
        fiber := Io.spawn(open_saturated, *job);

        Io.yield();
        assert(!job.returned && dial_attempts == 0, "a saturated live connection waits for peer stream capacity");

        if canceled Io.cancel(fiber);
        else {
            stream.local_ended = true;
            stream.remote_ended = true;
            h2_check_closed(stream);
        }

        assert(Io.wait(*job.done) == .NONE);
        defer free_error(*job.error);
        assert(job.returned && dial_attempts == 0);
        if canceled assert(job.error.transport == .CANCELED);
        else assert(job.error.kind == .NONE);
    }
}

Header_Admission_Probe :: struct {
    channel: *Http2_Channel;
    exchange: Http2_Exchange;
    returned: bool;
    done: Io.Wait_Group;
}

open_after_queue :: (probe: *Header_Admission_Probe) {
    exchange, error := open_http2_exchange(probe.channel, "/admission", .[]);

    assert(error.kind == .NONE);
    probe.exchange = exchange;
    probe.returned = true;
    Io.done(*probe.done);
}

header_queue_admission :: () {
    channel: Http2_Channel;
    channel.base_url = "https://queue.invalid/";
    channel.max_connections = 1;
    tls: Tls.Tls_Config;
    tls.alpn = string.["h2"];
    channel.tls = *tls;
    conn := h2_conn_new(.CLIENT, .{}, .{});
    conn.owner = *channel;
    conn.host = copy_string("queue.invalid");
    conn.port = 443;
    conn.tls = *tls;
    conn.remote.max_concurrent_streams = 1;
    conn.idle_since = now_ms();
    pool := client_pool();

    array_add(*pool.h2, conn);

    defer {
        for pool.h2 if it == conn {
            array_ordered_remove_by_index(*pool.h2, it_index);
            break;
        }

        h2_conn_release(conn);
    }

    array_reserve(*conn.spare, H2_QUEUE_LIMIT + 1);
    conn.spare.count = H2_QUEUE_LIMIT + 1;
    jobs: [2] Header_Admission_Probe;
    for *jobs {
        it.channel = *channel;
        Io.add(*it.done);
        Io.spawn(open_after_queue, it);
    }

    Io.yield();
    assert(!jobs[0].returned && !jobs[1].returned && conn.open_local == 0);
    conn.spare.count = 0;
    h2_notify_drained(conn);
    Io.yield();
    count := cast(int) jobs[0].returned + cast(int) jobs[1].returned;

    assert(count == 1 && conn.open_local == 1, "queue wakeup cannot admit two requests into one peer slot");
    for *jobs if it.returned close_exchange(*it.exchange);
    for *jobs assert(Io.wait(*it.done) == .NONE);
    for *jobs close_exchange(*it.exchange);

    assert(conn.open_local == 0 && channel.active_exchanges == 0);
}

goaway_retirement :: () {
    for already_ended: bool.[false, true] {
        conn := h2_conn_new(.CLIENT, .{}, .{});

        defer h2_conn_release(conn);
        stream: *H2_Stream;
        if !already_ended stream = h2_stream_new(conn, 1);

        payload := u8.[0, 0, 0, 1, 0, 0, 0, 0];
        error := h2_on_goaway(conn, .{ length = 8, type = .GOAWAY }, payload);

        assert(error == .NO_ERROR);

        if !already_ended {
            assert(!conn.closing, "GOAWAY must preserve accepted unfinished streams");
            stream.local_ended = true;
            stream.remote_ended = true;
            h2_check_closed(stream);
        }

        assert(conn.closing, "a drained GOAWAY connection retires without waiting for peer EOF");
    }
}

main :: () {
    Io.io_run(() {
        saturated_admission();
        header_queue_admission();
        goaway_retirement();

        for canceled: bool.[false, true] {
            probe: Channel_Probe;
            probe.canceled_scope = canceled;
            link: Link;
            link.reader = .{ data = *probe, read_proc = peer_eof, close_proc = link_close };
            conn := h2_conn_new(.CLIENT, link, .{});

            probe.conn = conn;
            conn.owner = *probe.channel;
            // Pool, reader, held writer and this test each hold one reference.
            conn.refs = 4;
            pool := client_pool();

            array_add(*pool.h2, conn);
            Io.add(*conn.tasks, 1);
            Io.spawn(held_writer, *probe);
            conn.reader = Io.spawn(h2_client_reader, conn);
            Io.yield();
            assert(conn.dead && !probe.link_closed);
            client: Client;
            client.transport.data = *probe.channel;
            client.transport.open = count_dial;
            url, parsed := parse_url("https://held.invalid/");

            assert(parsed == .NONE);
            dial_attempts = 0;
            _, _, _, capacity := open_connection(*client, url, 1);

            defer free_error(*capacity);
            assert(capacity.kind == .TOO_LARGE && dial_attempts == 0, "a dead connection retains its resource admission until TLS teardown completes");
            Io.add(*probe.done, 1);
            Io.spawn(close_scope, *probe);
            Io.yield();
            assert(!probe.returned, "channel close cannot lose peer-ended connections or abandon TLS join on caller expiry");
            probe.released = true;
            Io.notify_all(*probe.hold);
            Io.wait(*probe.done);
            assert(probe.returned && probe.link_closed && conn.link_closed);
            _, _, _, available := open_connection(*client, url, 1);

            defer free_error(*available);
            assert(available.kind != .TOO_LARGE && dial_attempts == 1, "closed TLS resources release connection admission");
            h2_conn_release(conn);
        }

        channel: Http2_Channel;
        tls: Tls.Tls_Config;
        tls.alpn = string.["h2"];
        pending := New(H2_Connecting);

        defer free(pending);
        pending.owner = *channel;
        pending.host = "other.invalid";
        pending.port = 443;
        pending.tls = *tls;
        job: Pending_Probe;
        job.client.tls = *tls;
        job.client.transport.data = *channel;
        job.client.transport.open = count_dial;
        job.url, _ = parse_url("https://held.invalid/");
        pool := client_pool();

        assert(pool.connecting.count == 0);
        array_add(*pool.connecting, pending);
        dial_attempts = 0;
        _, _, _, limit := open_connection(*job.client, job.url, 1);

        defer free_error(*limit);
        assert(limit.kind == .TOO_LARGE && dial_attempts == 0, "an in-progress connection consumes owner resource admission");
        pending.host = job.url.host;
        channel.base_url = "https://held.invalid";
        channel.tls = *tls;
        channel.max_connections = 1;
        channel.client.transport.open = count_dial;
        job.channel = *channel;
        array_add(*pool.http1_only, H2_Known_Origin.{ owner = *channel, host = copy_string(job.url.host), port = job.url.port, tls = *tls, since = now_ms() });
        Io.add(*job.done, 1);
        Io.spawn(open_pending, *job);
        Io.yield();
        assert(!job.returned && dial_attempts == 0, "same-origin callers share the existing connecting gate before admission");
        array_unordered_remove_by_index(*pool.connecting, 0);
        Io.notify_all(*pending.done);
        Io.with_deadline(1000, #code {
            assert(Io.wait(*job.done) == .NONE);
        });
        defer free_error(*job.error);
        assert(job.returned && job.error.kind != .TOO_LARGE && dial_attempts == 1);
    });
    print("ok\n");
}
