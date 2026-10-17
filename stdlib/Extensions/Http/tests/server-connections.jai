// Connections: keep-alive, pipelining, bodies in pieces and chunked, limits (body, head, request
// line), timeouts (head, body, idle), Expect: 100-continue, malformed requests, the request cap
// per connection and graceful shutdown.
#load "support/server_support.jai";

ping :: () -> string {
    return "pong";
}

echo_path :: (request: *Request) -> string {
    return request.path;
}

echo_text :: (body: Text) -> string {
    return tprint("%:%", body.value.count, body.value);
}

slow_entered := 0;

// Waits (up to 5 s) until `count` slow requests have reached their handler, so a slow machine
// does not make a test stop the server before the request is in flight.
wait_for_slow :: (count: int) {
    waited := 0;

    while slow_entered < count && waited < 5000 {
        Io.sleep(10);
        waited += 10;
    }
}

slow :: (ms: Path("ms", int)) -> string {
    slow_entered += 1;
    Io.sleep(ms.value);
    return "slow done";
}

app: App;

// 1. Keep-alive, Connection: close, HTTP/1.0.
plain_client: Client;

keep_alive :: (base: string, server: *Server) {
    stream := raw_connect(server);

    send_text(stream, "GET /echo/one HTTP/1.1\r\nHost: t\r\n\r\n");
    responses := read_responses(stream, 1, 2000);

    check(responses.count == 1 && responses[0].body == "/echo/one", "first request: %", responses.count);

    send_text(stream, "GET /echo/two HTTP/1.1\r\nHost: t\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].body == "/echo/two", "second request on the same connection");
    check(raw_header(responses[0], "Connection") == "", "keep-alive is the default: %", raw_header(responses[0], "Connection"));

    send_text(stream, "GET /echo/three HTTP/1.1\r\nHost: t\r\nConnection: close\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && raw_header(responses[0], "Connection") == "close", "Connection: close is answered with it");
    check(closes_within(stream, 1000), "and the server closes");
    Io.close(stream);

    stream = raw_connect(server);
    send_text(stream, "GET /echo/old HTTP/1.0\r\n\r\n");
    text := read_until_closed(stream, 2000);

    check(begins_with(text, "HTTP/1.0 200") && contains(text, "/echo/old"), "HTTP/1.0 gets an HTTP/1.0 answer that ends the connection: %", text);
    Io.close(stream);

    stream = raw_connect(server);
    send_text(stream, "GET /echo/old HTTP/1.0\r\nConnection: keep-alive\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && raw_header(responses[0], "Connection") == "keep-alive", "HTTP/1.0 keep-alive");
    send_text(stream, "GET /echo/again HTTP/1.0\r\nConnection: keep-alive\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].body == "/echo/again", "HTTP/1.0 keep-alive carries a second request");
    Io.close(stream);
}

// 2. Pipelining and pieces.
pipelining :: (base: string, server: *Server) {
    stream := raw_connect(server);

    batch := "GET /echo/a HTTP/1.1\r\nHost: t\r\n\r\nPOST /text HTTP/1.1\r\nHost: t\r\nContent-Length: 5\r\n\r\nhelloGET /echo/c HTTP/1.1\r\nHost: t\r\n\r\nGET /echo/d HTTP/1.1\r\nHost: t\r\n\r\nGET /echo/e HTTP/1.1\r\nHost: t\r\n\r\n";

    send_text(stream, batch);

    responses := read_responses(stream, 5, 3000);

    if responses.count == 5 {
        check(responses[0].body == "/echo/a", "pipelined 1: %", responses[0].body);
        check(responses[1].body == "5:hello", "pipelined 2 (with a body): %", responses[1].body);
        check(responses[2].body == "/echo/c", "pipelined 3: %", responses[2].body);
        check(responses[3].body == "/echo/d", "pipelined 4: %", responses[3].body);
        check(responses[4].body == "/echo/e", "pipelined 5: %", responses[4].body);
    }

    // The same bytes one at a time.
    for 0..batch.count - 1 {
        send_text(stream, slice(batch, it, 1));
    }

    responses = read_responses(stream, 5, 3000);
    check(responses.count == 5 && responses[1].body == "5:hello" && responses[4].body == "/echo/e", "a pipelined batch sent byte by byte: %", responses.count);

    // HEAD responses have no body, whatever Content-Length says.
    send_text(stream, "HEAD /echo/a HTTP/1.1\r\nHost: t\r\n\r\nHEAD /echo/b HTTP/1.1\r\nHost: t\r\n\r\nHEAD /echo/c HTTP/1.1\r\nHost: t\r\n\r\n");
    responses = read_responses(stream, 3, 2000, .HEAD);
    check(responses.count == 3 && responses[2].status == 200 && raw_header(responses[1], "Content-Length") == "7", "pipelined HEAD requests: %", responses.count);

    // A chunked body.
    send_text(stream, "POST /text HTTP/1.1\r\nHost: t\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n4;ext=1\r\ndefg\r\n0\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].body == "7:abcdefg", "a chunked request body: %", responses.count);

    // A body arriving in two parts.
    send_text(stream, "POST /text HTTP/1.1\r\nHost: t\r\nContent-Length: 10\r\n\r\n01234");
    Io.sleep(30);
    send_text(stream, "56789");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].body == "10:0123456789", "a body in two parts: %", responses.count);

    Io.close(stream);
}

// 3. Limits.
limits :: (base: string, server: *Server) {
    stream := raw_connect(server);

    send_text(stream, "POST /text HTTP/1.1\r\nHost: t\r\nContent-Length: 1000\r\n\r\n");
    send_text(stream, "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    Io.close(stream);

    // Exactly at the limit is fine.
    body: [..] u8;
    for 1..1000 array_add(*body, #char "y");

    response := post_body(*plain_client, tprint("%/text", base), string.{ body.count, body.data }, "text/plain");

    check(response.status == 200 && starts_with_count(body_of(response), "1000:"), "a body of exactly the limit: % %", response.status, response.body.count);

    // One byte over: refused when the head says so, and the connection closes.
    stream = raw_connect(server);
    send_text(stream, "POST /text HTTP/1.1\r\nHost: t\r\nContent-Length: 1001\r\n\r\n");
    responses := read_responses(stream, 1, 2000);

    check(responses.count == 1 && responses[0].status == 413, "declared body over the limit: %", responses.count);
    check(closes_within(stream, 1000), "413 closes the connection");
    Io.close(stream);

    // A chunked body is counted as it arrives.
    stream = raw_connect(server);
    send_text(stream, "POST /text HTTP/1.1\r\nHost: t\r\nTransfer-Encoding: chunked\r\n\r\n");
    chunk_data: [..] u8;
    for 1..400 array_add(*chunk_data, #char "x");

    chunk := tprint("190\r\n%\r\n", string.{ chunk_data.count, chunk_data.data });
    for 1..3 send_text(stream, chunk);

    text := read_until_closed(stream, 2000);

    check(contains(text, "413"), "a chunked body over the limit: %", text);
    Io.close(stream);

    // A head over the limit.
    stream = raw_connect(server);
    long_value: [..] u8;
    for 1..5000 array_add(*long_value, #char "v");

    send_text(stream, tprint("GET /ping HTTP/1.1\r\nHost: t\r\nX-Long: %\r\n\r\n", string.{ long_value.count, long_value.data }));
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 431, "a header line over the limit: %", responses.count);
    Io.close(stream);

    stream = raw_connect(server);
    long_target := tprint("GET /%s HTTP/1.1\r\nHost: t\r\n\r\n", string.{ long_value.count, long_value.data });

    send_text(stream, long_target);
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 414, "a request line over the limit: %", responses.count);
    Io.close(stream);

    // Too many headers.
    stream = raw_connect(server);
    builder: String_Builder;

    append(*builder, "GET /ping HTTP/1.1\r\nHost: t\r\n");
    for 1..30 print(*builder, "X-H%: v\r\n", it);

    append(*builder, "\r\n");
    send_text(stream, builder_to_string(*builder,, temp));
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 431, "too many headers: %", responses.count);
    Io.close(stream);
}

starts_with_count :: (text: string, prefix: string) -> bool {
    return begins_with(text, prefix);
}

// 4. Timeouts.
timeouts :: (base: string, server: *Server) {
    // A head that never finishes.
    stream := raw_connect(server);

    send_text(stream, "GET /ping HTTP/1.1\r\nHost: t\r\nX-Slow:");
    started := current_time_monotonic();
    responses := read_responses(stream, 1, 3000);
    elapsed := elapsed_milliseconds(started);

    check(responses.count == 1 && responses[0].status == 408, "an unfinished head is a 408: %", responses.count);
    check(elapsed >= 200 && elapsed < 1500, "the header timeout was about 300 ms: % ms", elapsed);
    check(closes_within(stream, 1000), "and closes");
    Io.close(stream);

    // A trickling head does not keep the connection longer than the header timeout.
    stream = raw_connect(server);
    started = current_time_monotonic();
    sent := 0;

    for 1..40 {
        _, error := Io.write(stream, "X");
        if error != .NONE break;
        sent += 1;
        Io.sleep(20);
    }

    elapsed = elapsed_milliseconds(started);
    check(elapsed < 1500, "a trickled head was cut off at the header timeout: % ms, % bytes sent", elapsed, sent);
    Io.close(stream);

    // A body that stops.
    stream = raw_connect(server);
    send_text(stream, "POST /text HTTP/1.1\r\nHost: t\r\nContent-Length: 10\r\n\r\n123");
    responses = read_responses(stream, 1, 3000);
    check(responses.count == 1 && responses[0].status == 408, "a stalled body is a 408: %", responses.count);
    Io.close(stream);

    // An idle connection (no request at all).
    stream = raw_connect(server);
    started = current_time_monotonic();
    check(closes_within(stream, 3000), "an idle connection is closed");
    elapsed = elapsed_milliseconds(started);
    check(elapsed >= 300 && elapsed < 1500, "the idle timeout was about 500 ms: % ms", elapsed);
    Io.close(stream);

    // An idle keep-alive connection after a request.
    stream = raw_connect(server);
    send_text(stream, "GET /ping HTTP/1.1\r\nHost: t\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1, "request before idling");
    check(closes_within(stream, 3000), "an idle keep-alive connection is closed");
    Io.close(stream);
}

// 4b. Memory: a connection that waits holds no read buffer, whatever it was doing before.
buffers_held :: () -> int {
    return http_statistics().read_buffers_in_use;
}

// How long a hot connection (a request just now) may still hold its buffer is `HOT_MS` in the
// server; waiting this long is enough for every connection to be waiting without one.
SETTLE :: 120;

idle_holds_no_buffer :: (base: string, server: *Server) {
    COUNT :: 20;

    streams: [COUNT] Io.Tcp_Stream;

    for 0..COUNT - 1 {
        streams[it] = raw_connect(server);

        send_text(streams[it], "GET /echo/idle HTTP/1.1\r\nHost: t\r\n\r\n");
    }

    for 0..COUNT - 1 {
        responses := read_responses(streams[it], 1, 3000);

        check(responses.count == 1 && responses[0].body == "/echo/idle", "request % before idling", it);
    }

    Io.sleep(SETTLE);
    check(buffers_held() == 0, "% idle keep-alive connections hold % read buffers", COUNT, buffers_held());

    kept := http_statistics().read_buffers_free;

    check(kept >= 1, "the buffers went back to the pool: %", kept);

    // The next request on each of them works, buffer or not, and they go idle again.
    for 0..COUNT - 1 {
        send_text(streams[it], "GET /echo/again HTTP/1.1\r\nHost: t\r\n\r\n");
    }

    for 0..COUNT - 1 {
        responses := read_responses(streams[it], 1, 3000);

        check(responses.count == 1 && responses[0].body == "/echo/again", "second request % after idling", it);
    }

    Io.sleep(SETTLE);
    check(buffers_held() == 0, "idle again: % read buffers held", buffers_held());

    // Half a head: the parser holds what came, the connection holds no buffer while it waits for
    // the rest, and the request is answered when it arrives.
    send_text(streams[0], "GET /echo/split HTTP/1.1\r\nHo");
    Io.sleep(SETTLE);
    check(buffers_held() == 0, "a connection waiting for the rest of a head holds % read buffers", buffers_held());
    send_text(streams[0], "st: t\r\n\r\n");

    responses := read_responses(streams[0], 1, 3000);

    check(responses.count == 1 && responses[0].body == "/echo/split", "a head split across an idle wait: %", responses.count);

    // Pipelined requests with the buffer given back in the middle: the first is answered from one
    // segment, the second completes after a pause.
    send_text(streams[1], "GET /echo/p1 HTTP/1.1\r\nHost: t\r\n\r\nGET /echo/p2 HTTP/1.1\r\nHost: t\r\n");
    responses = read_responses(streams[1], 1, 3000);
    check(responses.count == 1 && responses[0].body == "/echo/p1", "the first of two pipelined requests");

    Io.sleep(SETTLE);
    check(buffers_held() == 0, "between pipelined requests: % read buffers held", buffers_held());

    send_text(streams[1], "\r\nGET /echo/p3 HTTP/1.1\r\nHost: t\r\n\r\n");
    responses = read_responses(streams[1], 2, 3000);
    check(responses.count == 2 && responses[0].body == "/echo/p2" && responses[1].body == "/echo/p3", "pipelined requests after the pause: %", responses.count);

    // A big head and a big response, then small ones: the connection shrinks back and still works.
    builder: String_Builder;

    append(*builder, "GET /echo/big HTTP/1.1\r\nHost: t\r\n");

    // 90 header lines of about 250 bytes: a 22 KB head, past what the parser keeps between requests.
    filler: [240] u8;

    for *filler it.* = #char "a";

    for 1..90 print(*builder, "X-H%: %\r\n", it, string.{ filler.count, filler.data });

    append(*builder, "\r\n");

    big := builder_to_string(*builder,, temp);

    send_text(streams[2], big);
    responses = read_responses(streams[2], 1, 3000);
    check(responses.count == 1 && responses[0].body == "/echo/big", "a request with a 22 KB head");

    send_text(streams[2], "GET /echo/small HTTP/1.1\r\nHost: t\r\n\r\n");
    responses = read_responses(streams[2], 1, 3000);
    check(responses.count == 1 && responses[0].body == "/echo/small", "a small request after a big head");

    // A body in pieces keeps working across the wait (the body is read with the buffer held).
    send_text(streams[3], "POST /text HTTP/1.1\r\nHost: t\r\nContent-Length: 10\r\n\r\n01234");
    Io.sleep(SETTLE);
    send_text(streams[3], "56789");
    responses = read_responses(streams[3], 1, 3000);
    check(responses.count == 1 && responses[0].body == "10:0123456789", "a body in two parts after a pause");

    for streams Io.close(it);

    // Closing returns every buffer.
    Io.sleep(SETTLE);
    check(buffers_held() == 0, "after closing every connection: % read buffers held", buffers_held());
}

// A connection that was busy a moment ago and then goes quiet still times out at idle_timeout, not
// later: the time it held its buffer counts.
busy_then_idle :: (base: string, server: *Server) {
    stream := raw_connect(server);

    for 1..3 {
        send_text(stream, "GET /ping HTTP/1.1\r\nHost: t\r\n\r\n");

        responses := read_responses(stream, 1, 2000);

        check(responses.count == 1, "busy request %", it);
    }

    started := current_time_monotonic();

    check(closes_within(stream, 3000), "a connection that went quiet is closed");

    elapsed := elapsed_milliseconds(started);

    check(elapsed >= 300 && elapsed < 1500, "the idle timeout was about 500 ms: % ms", elapsed);
    Io.close(stream);
}

// 5. Expect: 100-continue and bad requests.
expectations :: (base: string, server: *Server) {
    stream := raw_connect(server);

    send_text(stream, "POST /text HTTP/1.1\r\nHost: t\r\nContent-Length: 5\r\nExpect: 100-continue\r\n\r\n");
    responses := read_responses(stream, 1, 2000);

    check(responses.count == 1 && responses[0].status == 100, "100 Continue before the body: %", responses.count);

    send_text(stream, "hello");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 200 && responses[0].body == "5:hello", "then the answer: %", responses.count);
    Io.close(stream);

    // Too large: refused without asking for the body.
    stream = raw_connect(server);
    send_text(stream, "POST /text HTTP/1.1\r\nHost: t\r\nContent-Length: 99999\r\nExpect: 100-continue\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 413, "Expect with a body over the limit is refused: %", responses.count);
    Io.close(stream);

    // An expectation the server does not know.
    stream = raw_connect(server);
    send_text(stream, "GET /ping HTTP/1.1\r\nHost: t\r\nExpect: magic\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 417, "unknown Expect: %", responses.count);
    Io.close(stream);

    // Garbage.
    stream = raw_connect(server);
    send_text(stream, "NOT HTTP AT ALL\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 400, "garbage is a 400: %", responses.count);
    check(closes_within(stream, 1000), "and closes");
    Io.close(stream);

    // Request smuggling shapes.
    stream = raw_connect(server);
    send_text(stream, "POST /text HTTP/1.1\r\nHost: t\r\nContent-Length: 4\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 400, "Content-Length with Transfer-Encoding: %", responses.count);
    Io.close(stream);

    stream = raw_connect(server);
    send_text(stream, "GET /ping HTTP/1.1\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 400, "HTTP/1.1 without Host: %", responses.count);
    Io.close(stream);

    stream = raw_connect(server);
    send_text(stream, "GET /ping HTTP/2.0\r\nHost: t\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 505, "HTTP/2.0 request line: % ", responses.count);
    Io.close(stream);
}

// 6. The cap on requests per connection.
capped :: (base: string, server: *Server) {
    stream := raw_connect(server);

    for 1..3 send_text(stream, "GET /ping HTTP/1.1\r\nHost: t\r\n\r\n");

    responses := read_responses(stream, 3, 2000);

    check(responses.count == 3, "three responses: %", responses.count);

    if responses.count == 3 {
        check(raw_header(responses[1], "Connection") == "", "the second keeps the connection");
        check(raw_header(responses[2], "Connection") == "close", "the third closes it: %", raw_header(responses[2], "Connection"));
    }

    check(closes_within(stream, 1000), "closed after the cap");
    Io.close(stream);
}

// 7. Many clients at once on one scheduler.
Burst :: struct {
    base: string;
    group: *Io.Wait_Group;
    ok: *int;
}

burst_client :: (burst: Burst) {
    defer Io.done(burst.group);

    for 1..5 {
        response, error := send(*plain_client, .{ url = tprint("%/echo/x", burst.base), timeout = 3000 });

        text := "";

        if error.kind == .NONE text, _ = read_text(*response);

        if error.kind == .NONE && response.status == 200 && text == "/echo/x" burst.ok.* += 1;
        else check(false, "burst request failed: %", error_text(error));

        close(*response);
    }
}

crowd :: (base: string, server: *Server) {
    group: Io.Wait_Group;
    ok := 0;

    Io.add(*group, 30);

    for 1..30 Io.spawn(burst_client, Burst.{ base, *group, *ok });

    Io.wait(*group);

    check(ok == 150, "30 clients x 5 requests: % answered", ok);
    check(server.served >= 150, "the server counted them: %", server.served);
}

// 8. Graceful shutdown.
Shutdown_Probe :: struct {
    server: *Server;
    base: string;
    status: int;
    connection_header: string;
    body: string;
    finished: Io.Wait_Group;
}

in_flight :: (probe: *Shutdown_Probe) {
    stream := raw_connect(probe.server);

    send_text(stream, "GET /slow/300 HTTP/1.1\r\nHost: t\r\n\r\n");
    responses := read_responses(stream, 1, 4000);

    if responses.count == 1 {
        probe.status = responses[0].status;
        probe.body = responses[0].body;
        probe.connection_header = raw_header(responses[0], "Connection");
    }

    Io.close(stream);
    Io.done(*probe.finished);
}

shutdown_test :: (base: string, server: *Server) {
    probe := Shutdown_Probe.{ server = server, base = base };

    // A keep-alive connection that is idle, and one with a request in flight.
    idle := raw_connect(server);

    send_text(idle, "GET /ping HTTP/1.1\r\nHost: t\r\n\r\n");
    responses := read_responses(idle, 1, 2000);

    check(responses.count == 1, "idle connection set up");

    entered := slow_entered;

    Io.add(*probe.finished, 1);
    Io.spawn(in_flight, *probe);
    wait_for_slow(entered + 1);

    started := current_time_monotonic();

    stop(server);

    check(closes_within(idle, 1000), "an idle connection closes at once when the server stops");

    // New connections are refused.
    _, error := Io.connect(server.address);

    check(error != .NONE, "a stopped server accepts no more connections");

    wait(server);

    elapsed := elapsed_milliseconds(started);

    // The server is done, but the client fiber may not have read its answer yet: wait for it.
    Io.wait(*probe.finished);

    check(probe.status == 200 && probe.body == "slow done", "the request in flight finished: % %", probe.status, probe.body);
    check(probe.connection_header == "close", "and asked the client to close: %", probe.connection_header);
    check(elapsed < 3000, "the drain was quick: % ms", elapsed);

    Io.close(idle);
}

// A request that outlasts the shutdown timeout is cut off.
forced_shutdown :: (base: string, server: *Server) {
    stream := raw_connect(server);
    entered := slow_entered;

    send_text(stream, "GET /slow/5000 HTTP/1.1\r\nHost: t\r\n\r\n");
    wait_for_slow(entered + 1);

    started := current_time_monotonic();

    stop(server);
    wait(server);

    elapsed := elapsed_milliseconds(started);

    check(elapsed >= 150 && elapsed < 2500, "shutdown waited for the timeout (200 ms), not for the handler: % ms", elapsed);
    check(closes_within(stream, 1000), "the connection was cut");
    Io.close(stream);
}

run_with :: (config: Server_Config, test: Server_Test) {
    local: App;

    local.config = config;

    get(*local, "/ping", ping);
    get(*local, "/echo/{rest...}", echo_rest);
    post(*local, "/text", echo_text);
    get(*local, "/slow/{ms}", slow);

    run_server(*local, test);
}

echo_rest :: (request: *Request) -> string {
    return request.path;
}

main :: () {
    if !supported() return;

    plain: Server_Config;

    run_with(plain, keep_alive);
    run_with(plain, pipelining);
    run_with(plain, idle_holds_no_buffer);
    run_with(plain, crowd);

    small: Server_Config;

    small.max_body_size = 1000;
    small.max_head_size = 2048;
    small.max_line_size = 1024;
    small.max_headers = 20;

    run_with(small, limits);
    run_with(small, expectations);

    quick: Server_Config;

    quick.header_timeout = 300;
    quick.body_timeout = 300;
    quick.idle_timeout = 500;

    run_with(quick, timeouts);
    run_with(quick, busy_then_idle);

    cap: Server_Config;

    cap.max_requests_per_connection = 3;

    run_with(cap, capped);
    run_with(plain, shutdown_test);

    forced: Server_Config;

    forced.shutdown_timeout = 200;

    run_with(forced, forced_shutdown);

    finish();
}
