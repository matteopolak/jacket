// test: skip_on(.WASM, reason = "needs the Http client, which runs on the Io scheduler")
// How the HTTP/2 client behaves when the server misbehaves in ways our own server never does, using a
// hand-written HTTP/2 server over TLS: a stream refused with REFUSED_STREAM is sent again, and a
// GOAWAY in the middle of a batch of requests lets the streams it covers finish while the ones above
// its last stream id are sent again on a new connection. A call made as an HTTP/2 exchange (what gRPC
// uses) is never sent again. Run from this directory.
#load "support/server_support.jai";
#load "../bytes.jai";
#load "../h2_frame.jai";
#load "../hpack.jai";

Tls :: #import "Extensions/Tls";

#load "../../Tls/tests/fixtures/certs.jai";

load :: (name: string) -> string {
    for DATA if it.name == name return it.text;

    assert(false, "no test file %", name);

    return "";
}

Mode :: enum {
    TRAILERS;
    REFUSE_FIRST;
    GOAWAY_AFTER_THREE;
}

// What the fake server does, and what it saw.
Script :: struct {
    listener: Io.Tcp_Listener;
    config: Tls.Tls_Config;
    mode: Mode;

    connections: int;
    requests: [..] int; // per connection: how many requests it received
    answered: [..] int; // per connection: how many it answered
}

accept_connections :: (script: *Script) {
    while true {
        tcp, error := Io.accept(script.listener);

        if error != .NONE break;

        script.connections += 1;

        array_add(*script.requests, 0);
        array_add(*script.answered, 0);

        Io.spawn(handle_connection, Handling.{ script, tcp, script.connections - 1 });
    }
}

Handling :: struct {
    script: *Script;
    tcp: Io.Tcp_Stream;
    index: int;
}

put :: (tls: *Tls.Tls_Stream, bytes: [] u8) {
    Io.write(Tls.writer_of(tls), bytes);
}

Peer_Frame :: struct {
    header: H2_Frame_Header;
    payload: [] u8;
    ok: bool;
}

next_frame :: (tls: *Tls.Tls_Stream, milliseconds: s64) -> Peer_Frame {
    frame: Peer_Frame;
    head: [H2_FRAME_HEADER_SIZE] u8;
    error: Io.Io_Error;

    Io.with_deadline(milliseconds, #code {
        _, error = Io.read_full(Tls.reader_of(tls), head);
    });

    if error != .NONE return frame;

    frame.header = h2_decode_header(head);

    payload := NewArray(frame.header.length, u8,, temp);

    if payload.count > 0 {
        Io.with_deadline(milliseconds, #code {
            _, error = Io.read_full(Tls.reader_of(tls), payload);
        });

        if error != .NONE return frame;
    }

    frame.payload = payload;
    frame.ok = true;

    return frame;
}

add_bytes :: (to: *[..] u8, data: *u8, count: int) {
    if count <= 0 return;

    array_reserve(to, to.count + count);
    memcpy(to.data + to.count, data, count);

    to.count += count;
}

respond :: (tls: *Tls.Tls_Stream, encoder: *Hpack_Encoder, id: u32, path: string) {
    block: [..] u8;
    block.allocator = temp;

    hpack_encode_start(encoder, *block);
    hpack_encode_field(encoder, *block, ":status", "200");
    if path == "/head-name" hpack_encode_field(encoder, *block, "bad name", "value");
    if path == "/head-value" hpack_encode_field(encoder, *block, "x-bad", "bad\r\nvalue");
    if path == "/trailer-length" hpack_encode_field(encoder, *block, "content-length", "100");

    body := tprint("ok %", path);
    out: [..] u8;
    out.allocator = temp;

    h2_put_headers(*out, id, block, false, 16384);
    if path != "/trailer-empty" h2_put_data(*out, id, .{ body.count, body.data }, false);

    block.count = 0;
    hpack_encode_start(encoder, *block);
    if path == "/trailer-pseudo" hpack_encode_field(encoder, *block, ":status", "200");
    if path == "/trailer-upper" hpack_encode_field(encoder, *block, "X-Bad", "value");
    if path == "/trailer-name" hpack_encode_field(encoder, *block, "bad name", "value");
    if path == "/trailer-value" hpack_encode_field(encoder, *block, "x-bad", "a\r\nb");
    if path == "/trailer-framing" hpack_encode_field(encoder, *block, "content-length", "0");

    hpack_encode_field(encoder, *block, "grpc-status", "0");
    hpack_encode_field(encoder, *block, "x-repeat", "first");
    hpack_encode_field(encoder, *block, "x-repeat", "second");
    h2_put_headers(*out, id, block, path != "/trailer-open", 16384);

    put(tls, out);
}

handle_connection :: (handling: Handling) {
    script := handling.script;
    index := handling.index;

    defer Io.close(handling.tcp);

    tls, tls_error := Tls.tls_accept(*script.config, Io.reader_of(handling.tcp), Io.writer_of(handling.tcp));

    if tls_error.kind != .NONE return;

    defer Tls.close(tls);

    preface: [24] u8;
    error: Io.Io_Error;

    Io.with_deadline(5000, #code {
        _, error = Io.read_full(Tls.reader_of(tls), preface);
    });

    if error != .NONE return;

    out: [..] u8;
    out.allocator = temp;

    h2_put_settings(*out, .[]);
    put(tls, out);

    encoder: Hpack_Encoder;
    decoder: Hpack_Decoder;

    hpack_encoder_init(*encoder);
    hpack_decoder_init(*decoder);

    defer hpack_encoder_free(*encoder);
    defer hpack_decoder_free(*decoder);

    Pending :: struct {
        id: u32;
        path: string;
    }

    held: [..] Pending;

    while true {
        frame := next_frame(tls, 3000);

        if !frame.ok return;

        if frame.header.type == .SETTINGS && (frame.header.flags & H2_ACK) == 0 {
            reply: [..] u8;
            reply.allocator = temp;

            h2_put_settings_ack(*reply);
            put(tls, reply);

            continue;
        }

        if frame.header.type != .HEADERS continue;

        block, block_error := h2_headers_payload(frame.header.flags, frame.payload);

        if block_error != .NO_ERROR return;

        decoded: Hpack_Block;

        defer hpack_block_free(*decoded);

        if hpack_decode(*decoder, block, 1 << 20, *decoded) != .NO_ERROR return;

        path: string;

        for decoded.fields if it.name == ":path" path = copy_string(it.value,, temp);

        script.requests[index] += 1;

        id := frame.header.stream;

        if script.mode == {
            case .TRAILERS;
                respond(tls, *encoder, id, path);
                script.answered[index] += 1;

            case .REFUSE_FIRST;
                if index == 0 && script.requests[index] == 1 {
                    reply: [..] u8;
                    reply.allocator = temp;

                    h2_put_rst_stream(*reply, id, .REFUSED_STREAM);
                    put(tls, reply);
                } else {
                    respond(tls, *encoder, id, path);

                    script.answered[index] += 1;
                }

            case .GOAWAY_AFTER_THREE;
                if index != 0 {
                    respond(tls, *encoder, id, path);

                    script.answered[index] += 1;
                } else {
                    // Hold the requests until three are in flight; answer the first and say that
                    // nothing above it was processed.
                    array_add(*held, .{ id, path });

                    if held.count == 3 {
                        respond(tls, *encoder, held[0].id, held[0].path);

                        script.answered[index] += 1;

                        reply: [..] u8;
                        reply.allocator = temp;

                        h2_put_goaway(*reply, held[0].id, .NO_ERROR);
                        put(tls, reply);

                        // Stay around until the client has moved on and closes.
                        while true {
                            rest := next_frame(tls, 3000);

                            if !rest.ok return;
                        }
                    }
                }
        }
    }
}

Job :: struct {
    base: string;
    config: *Tls.Tls_Config;
    path: string;
    group: *Io.Wait_Group;
    body: *string;
    status: *int;
}

fetch_one :: (job: Job) {
    defer Io.done(job.group);

    api: Client;

    api.base_url = job.base;
    api.tls = job.config;

    response, error := get(*api, job.path);

    defer close(*response);
    defer free_error(*error);

    if job.path == "/head-name" || job.path == "/head-value" {
        check(error.kind == .TRANSPORT && response.status == 0, "malformed initial fields fail the head: %", job.path);
        return;
    }

    if error.kind != .NONE {
        job.body.* = sprint("error: %", error_text(error));

        return;
    }

    job.status.* = response.status;

    text, read_error := read_text(*response);

    if job.path == "/trailer-pseudo" || job.path == "/trailer-open" || job.path == "/trailer-upper" || job.path == "/trailer-length" || job.path == "/trailer-name" || job.path == "/trailer-value" || job.path == "/trailer-framing" {
        check(read_error.kind != .NONE, "malformed trailers fail the body read: %", job.path);
        _, complete := response_trailers(*response);

        check(!complete, "malformed trailers do not establish completion");
        close(*response);

        return;
    }

    check(read_error.kind == .NONE, "valid body read");

    job.body.* = copy_string(text);

    // Reading the body releases the H2 stream; terminal fields must remain owned by
    // the response, including repeated names and their original order.
    fields, complete := response_trailers(*response);

    check(complete, "terminal headers available after body consumption");
    check(fields.count == 3, "all trailers retained: %", fields.count);

    if fields.count == 3 {
        check(fields[0].name == "grpc-status" && fields[0].value == "0", "RPC completion trailer");
        check(fields[1].value == "first" && fields[2].value == "second", "repeated trailer order");
    }

    close(*response);
}

// Starts the fake server for `script` and returns the base URL clients reach it at.
begin_script :: (script: *Script, client_config: *Tls.Tls_Config) -> string {
    script.config.certificate_chain_pem = load("server.pem");
    script.config.private_key_pem = load("server.key");
    script.config.alpn = string.["h2"];

    check(Tls.tls_config_init(*script.config).kind == .NONE, "server config");

    listener, error := Io.tcp_listen("127.0.0.1:0");

    check(error == .NONE, "listen");

    script.listener = listener;

    address, _ := Io.local_address(listener);

    Io.spawn(accept_connections, script);

    client_config.roots_pem = load("ca.pem");
    client_config.alpn = string.["h2", "http/1.1"];

    check(Tls.tls_config_init(client_config).kind == .NONE, "client config");

    return tprint("https://%", Io.address_to_string(address));
}

run_script :: (mode: Mode, paths: [] string, checks: #type (script: *Script, bodies: [] string, statuses: [] int)) {
    script: Script;
    client_config: Tls.Tls_Config;

    script.mode = mode;

    base := begin_script(*script, *client_config);

    group: Io.Wait_Group;
    bodies := NewArray(paths.count, string);
    statuses := NewArray(paths.count, int);

    Io.add(*group, paths.count);

    for paths Io.spawn(fetch_one, Job.{ base, *client_config, it, *group, *bodies[it_index], *statuses[it_index] });

    Io.wait(*group);

    checks(*script, bodies, statuses);

    close_idle_connections();
    Io.close(script.listener);
}

// A call made as an HTTP/2 exchange, the way gRPC makes one, is never sent again: the stream the
// server refuses ends the call with the refusal reported, and the server sees the request once.
exchange_not_replayed :: () {
    script: Script;
    client_config: Tls.Tls_Config;

    script.mode = .REFUSE_FIRST;

    base := begin_script(*script, *client_config);

    channel: Http2_Channel;

    channel.base_url = base;
    channel.tls = *client_config;

    exchange, failure := open_http2_exchange(*channel, "/refused", .[]);

    check(failure.kind == .NONE, "the exchange opens");

    _, _, head_error := exchange_head(*exchange);

    check(head_error.kind != .NONE, "the refused stream fails the exchange");

    terminal := exchange_terminal(*exchange);

    check(terminal.kind == .RESET && terminal.code == cast(u32) H2_Error.REFUSED_STREAM && terminal.not_processed, "the refusal is reported, not hidden");

    close_exchange(*exchange);
    free_error(*head_error);
    close_http2_channel(*channel);

    total := 0;

    for script.requests total += it;

    check(total == 1 && script.connections == 1, "the server saw the request once, on one connection: % requests, % connections", total, script.connections);

    Io.close(script.listener);
}

trailers :: (script: *Script, bodies: [] string, statuses: [] int) {
    check(bodies[0] == "", "empty DATA body retains terminal fields");
    check(bodies[4] == "ok /valid", "unrelated stream survives malformed trailers");
}

refused :: (script: *Script, bodies: [] string, statuses: [] int) {
    check(statuses[0] == 200 && bodies[0] == "ok /refused", "a refused stream is sent again: % '%'", statuses[0], bodies[0]);

    total := 0;

    for script.requests total += it;

    check(total == 2, "the server saw the request twice: %", total);
}

goaway :: (script: *Script, bodies: [] string, statuses: [] int) {
    complete := true;

    for bodies {
        if statuses[it_index] != 200 || it != tprint("ok /r%", it_index) complete = false;
    }

    check(complete, "every request finished: % | % | %", bodies[0], bodies[1], bodies[2]);
    check(script.connections == 2, "the retried requests used one new connection: % connections", script.connections);
    check(script.requests.count == 2 && script.requests[0] == 3, "the first connection carried all three");
    check(script.requests.count == 2 && script.requests[1] == 2, "and the two above the last stream id were sent again: % requests", ifx script.requests.count == 2 then script.requests[1] else -1);
}

main :: () {
    entry :: (_: *void) {
        run_script(.TRAILERS, string.["/trailer-empty", "/trailer-pseudo", "/trailer-open", "/trailer-upper", "/valid", "/trailer-length", "/trailer-name", "/trailer-value", "/trailer-framing", "/head-name", "/head-value"], trailers);
        run_script(.REFUSE_FIRST, string.["/refused"], refused);
        run_script(.GOAWAY_AFTER_THREE, string.["/r0", "/r1", "/r2"], goaway);
        exchange_not_replayed();
    }

    Io.io_run(entry, null);

    finish();
}
