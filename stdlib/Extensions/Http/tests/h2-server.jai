// test: skip_on(.WASM, reason = "needs the Http server, which is not built for wasm")
// The HTTP/2 server, driven by a hand-written peer over TLS so that every frame it sends and
// every rule it enforces can be checked on the wire: requests and responses, flow control in both
// directions, many streams at once, streamed responses, and the errors RFC 9113 names for
// malformed input. Run from this directory.
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

// ---- The app

add_bytes :: (to: *[..] u8, data: *u8, count: int) {
    if count <= 0 return;

    array_reserve(to, to.count + count);
    memcpy(to.data + to.count, data, count);

    to.count += count;
}

ping :: () -> string {
    return "pong";
}

size_of_body :: (body: *Body) -> string, Http_Error {
    bytes, error := read_all(body);

    if error.kind != .NONE return "", error;

    return tprint("%", bytes.count), .{};
}

describe :: (request: *Request) -> string {
    host, _ := header(request, "Host");
    cookie, _ := header(request, "Cookie");
    custom, _ := header(request, "X-Custom");

    return tprint("% % host=% cookie=% custom=% version=%", request.method_text, request.target, host, cookie, custom, request.version);
}

BIG :: 300 * 1024 + 5;

big_text: string;

big :: () -> string {
    return big_text;
}

produce :: (stream: *Stream, data: *void) {
    for 1..5 {
        if !send_chunk(stream, tprint("part %;", it)) return;

        Io.sleep(5);
    }
}

streamed :: () -> Response {
    return stream_response(produce, null, "text/plain");
}

sized_head :: () -> Response {
    return with_header(status_only(200), "Content-Length", "1234");
}

sleepy :: () -> string {
    Io.sleep(400);

    return "done";
}

missing :: () -> Http_Error {
    return not_found("nothing here");
}

echo_text :: (body: *Body) -> string, Http_Error {
    bytes, error := read_all(body);

    if error.kind != .NONE return "", error;

    return copy_string(string.{ bytes.count, bytes.data },, temp), .{};
}

app: App;
small_app: App;

// ---- The peer

Peer :: struct {
    tcp: Io.Tcp_Stream;
    tls: *Tls.Tls_Stream;
    config: Tls.Tls_Config;
    encoder: Hpack_Encoder;
    decoder: Hpack_Decoder;
    next_id: u32 = 1;

    // What the server allows us to send.
    stream_window: s64 = 65535;
    connection_window: s64 = 65535;
    server_settings: H2_Settings;
}

connect_peer :: (server: *Server, alpn: [] string = string.["h2"], preface := true) -> *Peer {
    peer := New(Peer);

    peer.config.certificate_chain_pem = "";
    peer.config.roots_pem = load("ca.pem");
    peer.config.alpn = alpn;

    error := Tls.tls_config_init(*peer.config);

    check(error.kind == .NONE, "client config: %", Tls.error_text(error));

    tcp, io_error := Io.connect(server.address);

    check(io_error == .NONE, "connect: %", io_error);

    peer.tcp = tcp;

    tls_error: Tls.Tls_Error;

    peer.tls, tls_error = Tls.tls_connect(*peer.config, Io.reader_of(tcp), Io.writer_of(tcp), "127.0.0.1");

    check(tls_error.kind == .NONE, "handshake: %", Tls.error_text(tls_error));

    hpack_encoder_init(*peer.encoder);
    hpack_decoder_init(*peer.decoder);

    if preface {
        out: [..] u8;

        defer array_reset(*out);

        add_bytes(*out, H2_PREFACE.data, H2_PREFACE.count);
        h2_put_settings(*out, .[]);

        send_bytes(peer, out);
    }

    return peer;
}

disconnect :: (peer: *Peer) {
    Tls.close(peer.tls);
    Tls.tls_config_free(*peer.config);
    hpack_encoder_free(*peer.encoder);
    hpack_decoder_free(*peer.decoder);
    free(peer);
}

send_bytes :: (peer: *Peer, bytes: [] u8) {
    _, error := Io.write(Tls.writer_of(peer.tls), bytes);

    check(error == .NONE, "send: %", error);
}

Frame :: struct {
    header: H2_Frame_Header;
    payload: [] u8;
    ok: bool;
}

// The next frame from the server, or `ok = false` when the connection ended or nothing came.
next_frame :: (peer: *Peer, milliseconds := 5000) -> Frame {
    frame: Frame;
    head: [H2_FRAME_HEADER_SIZE] u8;
    error: Io.Io_Error;

    Io.with_deadline(milliseconds, #code {
        _, error = Io.read_full(Tls.reader_of(peer.tls), head);
    });

    if error != .NONE return frame;

    frame.header = h2_decode_header(head);

    payload := NewArray(frame.header.length, u8,, temp);

    if payload.count > 0 {
        Io.with_deadline(milliseconds, #code {
            _, error = Io.read_full(Tls.reader_of(peer.tls), payload);
        });

        if error != .NONE return frame;
    }

    frame.payload = payload;
    frame.ok = true;

    return frame;
}

// Sends HEADERS for a new request; returns the stream id.
request_on :: (peer: *Peer, id: u32, method: string, path: string, extra: [] Header_Field = .[], end_stream := true, authority := "localhost") -> u32 {
    fields: [..] Header_Field;
    fields.allocator = temp;

    array_add(*fields, .{ name = ":method", value = method });
    array_add(*fields, .{ name = ":scheme", value = "https" });
    array_add(*fields, .{ name = ":path", value = path });
    array_add(*fields, .{ name = ":authority", value = authority });

    for extra array_add(*fields, it);

    send_headers(peer, id, fields, end_stream);

    return id;
}

send_headers :: (peer: *Peer, id: u32, fields: [] Header_Field, end_stream: bool) {
    block: [..] u8;
    block.allocator = temp;

    hpack_encode_start(*peer.encoder, *block);

    for fields hpack_encode_field(*peer.encoder, *block, it.name, it.value);

    out: [..] u8;
    out.allocator = temp;

    h2_put_headers(*out, id, block, end_stream, 16384);

    send_bytes(peer, out);
}

request :: (peer: *Peer, method: string, path: string, extra: [] Header_Field = .[], end_stream := true) -> u32 {
    id := peer.next_id;

    peer.next_id += 2;

    return request_on(peer, id, method, path, extra, end_stream);
}

parse_int :: (text: string) -> int {
    value := 0;

    for text value = value * 10 + (it - #char "0");

    return value;
}

Result :: struct {
    id: u32;
    status: int;
    headers: [..] Header_Field;
    body: [..] u8;
    ended: bool;
    reset: bool;
    reset_code: u32;
}

text_of :: (result: *Result) -> string {
    return string.{ result.body.count, result.body.data };
}

header_of :: (result: *Result, name: string) -> string {
    value, _ := find_header(result.headers, name);

    return value;
}

// Results by stream, for the exchange `read_results` is running.
Exchange :: struct {
    results: [..] *Result;
    goaway: bool;
    goaway_code: u32;
    goaway_last: u32;
    closed: bool;
    pings: int;
}

result_for :: (exchange: *Exchange, id: u32) -> *Result {
    for exchange.results if it.id == id return it;

    result := New(Result,, temp);

    result.id = id;

    array_add(*exchange.results, result);

    return result;
}

// Reads frames until every stream in `wait_for` has ended or been reset, the server said GOAWAY, or the connection
// closed. Window updates for what arrives are sent as it arrives, as a client that reads everything would.
read_results :: (peer: *Peer, exchange: *Exchange, wait_for: [] u32, milliseconds := 5000, acknowledge := true) {
    for wait_for result_for(exchange, it);

    while true {
        remaining := 0;

        for wait_for {
            result := result_for(exchange, it);

            if !result.ended && !result.reset remaining += 1;
        }

        if remaining == 0 && wait_for.count > 0 break;

        frame := next_frame(peer, milliseconds);

        if !frame.ok {
            exchange.closed = true;

            break;
        }

        header := frame.header;
        out: [..] u8;
        out.allocator = temp;

        if header.type == {
            case .SETTINGS;
                if (header.flags & H2_ACK) == 0 {
                    old := peer.server_settings;

                    check(h2_read_settings(frame.payload, *peer.server_settings) == .NO_ERROR, "the server's SETTINGS are valid");

                    peer.stream_window += peer.server_settings.initial_window_size - old.initial_window_size;

                    if acknowledge {
                        h2_put_settings_ack(*out);
                        send_bytes(peer, out);
                    }
                }

            case .WINDOW_UPDATE;
                increment := cast(s64) (h2_read_u32(frame.payload, 0) & 0x7fffffff);

                if header.stream == 0 peer.connection_window += increment;
                else result_for(exchange, header.stream).id = header.stream; // stream windows are tracked by the senders

            case .PING;
                if (header.flags & H2_ACK) == 0 {
                    h2_put_ping(*out, frame.payload, true);
                    send_bytes(peer, out);
                } else {
                    exchange.pings += 1;
                }

            case .GOAWAY;
                last, code, _, _ := h2_read_goaway(frame.payload);

                exchange.goaway = true;
                exchange.goaway_code = code;
                exchange.goaway_last = last;

                return;

            case .RST_STREAM;
                result := result_for(exchange, header.stream);

                result.reset = true;
                result.reset_code = h2_read_u32(frame.payload, 0);

            case .HEADERS;
                result := result_for(exchange, header.stream);
                block, _ := h2_headers_payload(header.flags, frame.payload);

                // Only a header block that fits one frame; the tests keep responses small.
                check((header.flags & H2_END_HEADERS) != 0, "the response head fits one frame");

                decoded: Hpack_Block;

                error := hpack_decode(*peer.decoder, block, 1 << 20, *decoded);

                check(error == .NO_ERROR, "the response head decodes");

                for decoded.fields {
                    if it.name == ":status" {
                        result.status = parse_int(it.value);
                    } else {
                        copy: Header_Field;

                        copy.name = copy_string(it.name,, temp);
                        copy.value = copy_string(it.value,, temp);
                        copy.id = it.id;

                        array_add(*result.headers, copy,, temp);
                    }
                }

                hpack_block_free(*decoded);

                if (header.flags & H2_END_STREAM) != 0 result.ended = true;

            case .DATA;
                result := result_for(exchange, header.stream);
                data, _ := h2_data_payload(header.flags, frame.payload);

                add_bytes(*result.body, data.data, data.count);

                if frame.payload.count > 0 {
                    h2_put_window_update(*out, 0, cast(u32) frame.payload.count);

                    if (header.flags & H2_END_STREAM) == 0 h2_put_window_update(*out, header.stream, cast(u32) frame.payload.count);

                    send_bytes(peer, out);
                }

                if (header.flags & H2_END_STREAM) != 0 result.ended = true;
            case; // Other members deliberately do nothing.
        }
    }
}

// One request and its response.
fetch :: (peer: *Peer, method: string, path: string, extra: [] Header_Field = .[]) -> *Result, Exchange {
    id := request(peer, method, path, extra);
    exchange: Exchange;

    exchange.results.allocator = temp;

    read_results(peer, *exchange, .[id]);

    return result_for(*exchange, id), exchange;
}

// The server's SETTINGS and the acknowledgement of ours.
handshake :: (peer: *Peer) -> Exchange {
    exchange: Exchange;

    exchange.results.allocator = temp;

    seen_settings := false;
    acknowledged := false;

    while !seen_settings || !acknowledged {
        frame := next_frame(peer);

        if !frame.ok break;

        if frame.header.type == .SETTINGS {
            if (frame.header.flags & H2_ACK) != 0 {
                acknowledged = true;
            } else {
                seen_settings = true;

                check(h2_read_settings(frame.payload, *peer.server_settings) == .NO_ERROR, "the server's SETTINGS are valid");

                peer.stream_window = peer.server_settings.initial_window_size;

                out: [..] u8;
                out.allocator = temp;

                h2_put_settings_ack(*out);
                send_bytes(peer, out);
            }
        } else if frame.header.type == .WINDOW_UPDATE && frame.header.stream == 0 {
            peer.connection_window += cast(s64) (h2_read_u32(frame.payload, 0) & 0x7fffffff);
        }
    }

    check(seen_settings && acknowledged, "the server sends SETTINGS and acknowledges ours");

    return exchange;
}

// The next frame that is not SETTINGS or WINDOW_UPDATE, which are the server's own business.
next_interesting :: (peer: *Peer, milliseconds := 5000) -> Frame {
    while true {
        frame := next_frame(peer, milliseconds);

        if !frame.ok return frame;

        if frame.header.type == .SETTINGS || frame.header.type == .WINDOW_UPDATE continue;

        return frame;
    }

    return .{};
}

// ---- Scenarios

basic_requests :: (base: string, server: *Server) {
    peer := connect_peer(server);

    defer disconnect(peer);

    handshake(peer);

    check(peer.server_settings.max_concurrent_streams == 100, "the server announces its stream limit: %", peer.server_settings.max_concurrent_streams);
    check(peer.server_settings.initial_window_size == 256 * 1024, "and its window: %", peer.server_settings.initial_window_size);
    check(!peer.server_settings.enable_push, "push is not offered");

    result, exchange := fetch(peer, "GET", "/ping");

    check(result.status == 200 && text_of(result) == "pong" && result.ended, "GET /ping: % '%'", result.status, text_of(result));
    check(header_of(result, "content-length") == "4", "content-length: %", header_of(result, "content-length"));
    check(header_of(result, "server") == "jaic" && header_of(result, "date").count > 0, "server and date headers");
    check(contains(header_of(result, "content-type"), "text/plain"), "content-type: %", header_of(result, "content-type"));

    // Header names are lowercase and nothing HTTP/1 specific is sent.
    for result.headers {
        check(it.name == to_lower_copy(it.name,, temp), "a response header is lowercase: %", it.name);
        check(it.name != "connection" && it.name != "keep-alive" && it.name != "transfer-encoding" && it.name != "upgrade", "no HTTP/1 only header: %", it.name);
    }

    // The request as the handler sees it: Host from :authority, cookies joined, same version.
    extra := Header_Field.[.{ name = "cookie", value = "a=1" }, .{ name = "x-custom", value = "yes" }, .{ name = "cookie", value = "b=2" }];

    result, exchange = fetch(peer, "GET", "/describe?x=1", extra);

    check(text_of(result) == "GET /describe?x=1 host=localhost cookie=a=1; b=2 custom=yes version=HTTP_1_1", "the request as the handler sees it: %", text_of(result));

    // Errors come out of the same machinery.
    result, exchange = fetch(peer, "GET", "/nowhere");

    check(result.status == 404 && contains(header_of(result, "content-type"), "problem+json"), "404: % %", result.status, header_of(result, "content-type"));

    result, exchange = fetch(peer, "GET", "/missing");

    check(result.status == 404 && contains(text_of(result), "nothing here"), "a handler's error: %", text_of(result));

    result, exchange = fetch(peer, "DELETE", "/ping");

    check(result.status == 405 && header_of(result, "allow").count > 0, "405 with Allow: %", result.status);

    // HEAD: the headers of the GET and no body, and a stated length is sent as it is.
    result, exchange = fetch(peer, "HEAD", "/ping");

    check(result.status == 200 && result.ended && result.body.count == 0 && header_of(result, "content-length") == "4", "HEAD /ping: % '%'", result.status, header_of(result, "content-length"));

    result, exchange = fetch(peer, "HEAD", "/sized");

    check(result.status == 200 && result.ended && result.body.count == 0 && header_of(result, "content-length") == "1234", "HEAD with a stated length: %", header_of(result, "content-length"));

    // The server answers a PING.
    out: [..] u8;

    defer array_reset(*out);

    h2_put_ping(*out, .[1, 2, 3, 4, 5, 6, 7, 8], false);
    send_bytes(peer, out);

    frame := next_interesting(peer);

    check(frame.ok && frame.header.type == .PING && (frame.header.flags & H2_ACK) != 0 && frame.payload.count == 8 && frame.payload[7] == 8, "PING is acknowledged with its data");

    // Frames of a type nobody knows, PRIORITY and a padded DATA frame are all fine.
    out.count = 0;

    h2_put_frame_header(*out, 3, .UNKNOWN, 0, 0);
    add_bytes(*out, "abc".data, 3);
    h2_put_frame_header(*out, 5, .PRIORITY, 0, 3);
    add_bytes(*out, "\0\0\0\0\x10".data, 5);

    send_bytes(peer, out);

    result, exchange = fetch(peer, "GET", "/ping");

    check(result.status == 200 && text_of(result) == "pong", "the connection carries on after frames it ignores");
}

large_response :: (base: string, server: *Server) {
    peer := connect_peer(server);

    defer disconnect(peer);

    handshake(peer);

    // 300 KB with our connection window at 64 KB: the server must wait for the updates.
    result := fetch(peer, "GET", "/big");

    check(result.status == 200 && result.body.count == BIG, "a large response: % bytes", result.body.count);
    check(text_of(result) == big_text, "arrives intact");

    streamed_result := fetch(peer, "GET", "/streamed");

    check(streamed_result.status == 200 && text_of(streamed_result) == "part 1;part 2;part 3;part 4;part 5;", "a streamed response: '%'", text_of(streamed_result));
    check(header_of(streamed_result, "content-length").count == 0, "has no length");
}

// A request body larger than the window the server gives: the peer has to wait for the server's updates.
large_request :: (base: string, server: *Server) {
    peer := connect_peer(server);

    defer disconnect(peer);

    handshake(peer);

    total := 1500 * 1024;
    id := request(peer, "POST", "/size", .[.{ name = "content-type", value = "text/plain" }], false);

    exchange: Exchange;

    exchange.results.allocator = temp;

    sent := 0;
    stream_window := peer.stream_window;
    waited := 0;

    chunk := NewArray(16384, u8,, temp);

    for *chunk it.* = cast(u8) (it_index & 0x7f);

    while sent < total {
        count := min(min(total - sent, 16384), min(stream_window, peer.connection_window));

        if count <= 0 {
            // Wait for window.
            frame := next_frame(peer);

            check(frame.ok, "the server keeps the connection while we wait for window");

            if !frame.ok break;

            if frame.header.type == .WINDOW_UPDATE {
                increment := cast(s64) (h2_read_u32(frame.payload, 0) & 0x7fffffff);

                if frame.header.stream == 0 peer.connection_window += increment;
                else stream_window += increment;

                waited += 1;
            }

            continue;
        }

        out: [..] u8;
        out.allocator = temp;

        h2_put_data(*out, id, .{ count, chunk.data }, sent + count == total);
        send_bytes(peer, out);

        sent += count;
        stream_window -= count;
        peer.connection_window -= count;
    }

    check(waited > 0, "the peer had to wait for window updates: % waits", waited);

    read_results(peer, *exchange, .[id]);

    result := result_for(*exchange, id);

    check(result.status == 200 && text_of(result) == tprint("%", total), "a large request body: '%'", text_of(result));
}

many_streams :: (base: string, server: *Server) {
    peer := connect_peer(server);

    defer disconnect(peer);

    handshake(peer);

    count := 60;
    ids: [..] u32;
    ids.allocator = temp;

    // All the requests go out before any answer is read.
    for 1..count {
        id := request(peer, "POST", "/echo", .[.{ name = "content-type", value = "text/plain" }], false);

        array_add(*ids, id);
    }

    for ids {
        out: [..] u8;
        out.allocator = temp;

        h2_put_data(*out, it, bytes_of(tprint("stream %", it)), true);
        send_bytes(peer, out);
    }

    exchange: Exchange;

    exchange.results.allocator = temp;

    read_results(peer, *exchange, ids);

    ok := 0;

    for ids {
        result := result_for(*exchange, it);

        if result.status == 200 && text_of(result) == tprint("stream %", it) ok += 1;
    }

    check(ok == count, "% of % concurrent streams were answered correctly", ok, count);
}

// Rules a server must enforce.
protocol_errors :: (base: string, server: *Server) {
    // Uppercase in a header name: the request is malformed, the connection is fine.
    peer := connect_peer(server);

    handshake(peer);

    id := request(peer, "GET", "/ping", .[.{ name = "X-Upper", value = "1" }]);
    exchange: Exchange;

    exchange.results.allocator = temp;

    read_results(peer, *exchange, .[id]);

    check(result_for(*exchange, id).reset && result_for(*exchange, id).reset_code == cast(u32) H2_Error.PROTOCOL_ERROR, "an uppercase header name is a stream error");

    result := fetch(peer, "GET", "/ping");

    check(result.status == 200, "and the connection keeps working");

    // Connection-specific headers, a missing :path, and a body of another length than announced.
    bad_cases := Header_Field.[.{ name = "connection", value = "close" }, .{ name = "transfer-encoding", value = "chunked" }, .{ name = "te", value = "gzip" }];

    for bad_cases {
        id = request(peer, "GET", "/ping", .[it]);
        exchange = .{};
        exchange.results.allocator = temp;

        read_results(peer, *exchange, .[id]);

        check(result_for(*exchange, id).reset, "the header `%` makes a request malformed", it.name);
    }

    fields := Header_Field.[.{ name = ":method", value = "GET" }, .{ name = ":scheme", value = "https" }];

    id = peer.next_id;
    peer.next_id += 2;

    send_headers(peer, id, fields, true);

    exchange = .{};
    exchange.results.allocator = temp;

    read_results(peer, *exchange, .[id]);

    check(result_for(*exchange, id).reset, "a request without :path is malformed");

    id = request(peer, "POST", "/size", .[.{ name = "content-length", value = "10" }], false);

    out: [..] u8;
    out.allocator = temp;

    h2_put_data(*out, id, bytes_of("abc"), true);
    send_bytes(peer, out);

    exchange = .{};
    exchange.results.allocator = temp;

    read_results(peer, *exchange, .[id]);

    check(result_for(*exchange, id).reset, "a body shorter than content-length is malformed");

    // Stream-level WINDOW_UPDATE of 0.
    id = request(peer, "POST", "/size", .[], false);
    out.count = 0;
    h2_put_window_update(*out, id, 0);
    send_bytes(peer, out);

    exchange = .{};
    exchange.results.allocator = temp;

    read_results(peer, *exchange, .[id]);

    check(result_for(*exchange, id).reset && result_for(*exchange, id).reset_code == cast(u32) H2_Error.PROTOCOL_ERROR, "a zero window update is a stream error");

    disconnect(peer);

    // Each of these ends the connection with a GOAWAY.
    expect_goaway :: (server: *Server, what: string, code: H2_Error, build: #type (out: *[..] u8)) {
        peer := connect_peer(server);

        defer disconnect(peer);

        handshake(peer);

        out: [..] u8;
        out.allocator = temp;

        build(*out);
        send_bytes(peer, out);

        exchange: Exchange;

        exchange.results.allocator = temp;

        read_results(peer, *exchange, .[]);

        check(exchange.goaway && exchange.goaway_code == cast(u32) code, "% -> GOAWAY %: got % (goaway %)", what, code, exchange.goaway_code, exchange.goaway);

        // And the connection is closed after it.
        frame := next_frame(peer, 2000);

        check(!frame.ok, "% closes the connection", what);
    }

    expect_goaway(server, "DATA on stream 0", .PROTOCOL_ERROR, (out: *[..] u8) {
        h2_put_data(out, 0, bytes_of("x"), false);
    });
    expect_goaway(server, "WINDOW_UPDATE of 0 on the connection", .PROTOCOL_ERROR, (out: *[..] u8) {
        h2_put_window_update(out, 0, 0);
    });
    expect_goaway(server, "a frame above the size limit", .FRAME_SIZE_ERROR, (out: *[..] u8) {
        h2_put_frame_header(out, 20000, .DATA, 0, 1);
        add_bytes(out, NewArray(20000, u8,, temp).data, 20000);
    });
    expect_goaway(server, "SETTINGS with an odd size", .FRAME_SIZE_ERROR, (out: *[..] u8) {
        h2_put_frame_header(out, 5, .SETTINGS, 0, 0);
        add_bytes(out, "12345".data, 5);
    });
    expect_goaway(server, "a window above 2^31 - 1", .FLOW_CONTROL_ERROR, (out: *[..] u8) {
        h2_put_settings(out, .[.{ cast(u16) H2_Setting.INITIAL_WINDOW_SIZE, 0x80000000 }]);
    });
    expect_goaway(server, "ENABLE_PUSH of 2", .PROTOCOL_ERROR, (out: *[..] u8) {
        h2_put_settings(out, .[.{ cast(u16) H2_Setting.ENABLE_PUSH, 2 }]);
    });
    expect_goaway(server, "a frame size below 16384", .PROTOCOL_ERROR, (out: *[..] u8) {
        h2_put_settings(out, .[.{ cast(u16) H2_Setting.MAX_FRAME_SIZE, 100 }]);
    });
    expect_goaway(server, "PING of the wrong size", .FRAME_SIZE_ERROR, (out: *[..] u8) {
        h2_put_frame_header(out, 4, .PING, 0, 0);
        add_bytes(out, "abcd".data, 4);
    });
    expect_goaway(server, "PING on a stream", .PROTOCOL_ERROR, (out: *[..] u8) {
        h2_put_frame_header(out, 8, .PING, 0, 1);
        add_bytes(out, "abcdefgh".data, 8);
    });
    expect_goaway(server, "PUSH_PROMISE from a client", .PROTOCOL_ERROR, (out: *[..] u8) {
        h2_put_frame_header(out, 4, .PUSH_PROMISE, H2_END_HEADERS, 1);
        add_bytes(out, "\0\0\0\x02".data, 4);
    });
    expect_goaway(server, "CONTINUATION with no HEADERS", .PROTOCOL_ERROR, (out: *[..] u8) {
        h2_put_frame_header(out, 0, .CONTINUATION, H2_END_HEADERS, 1);
    });
    expect_goaway(server, "a stream id the client may not use", .PROTOCOL_ERROR, (out: *[..] u8) {
        encoder: Hpack_Encoder;

        hpack_encoder_init(*encoder);
        block: [..] u8;
        block.allocator = temp;
        hpack_encode_field(*encoder, *block, ":method", "GET");
        hpack_encode_field(*encoder, *block, ":scheme", "https");
        hpack_encode_field(*encoder, *block, ":path", "/");
        h2_put_headers(out, 2, block, true, 16384);
        hpack_encoder_free(*encoder);
    });
    expect_goaway(server, "a header block that cannot be decoded", .COMPRESSION_ERROR, (out: *[..] u8) {
        h2_put_frame_header(out, 1, .HEADERS, H2_END_HEADERS | H2_END_STREAM, 1);
        add_bytes(out, "\xbe".data, 1);
    });
    expect_goaway(server, "RST_STREAM on a stream never opened", .PROTOCOL_ERROR, (out: *[..] u8) {
        h2_put_rst_stream(out, 9, .CANCEL);
    });

    // A header block that continues in other frames.
    peer = connect_peer(server);

    defer disconnect(peer);

    handshake(peer);

    big_value := alloc_string(40000,, temp);

    for *big_value it.* = cast(u8) (#char "a" + it_index % 26);

    id = request(peer, "GET", "/describe", .[.{ name = "x-custom", value = big_value }]);
    exchange = .{};
    exchange.results.allocator = temp;

    read_results(peer, *exchange, .[id]);

    result = result_for(*exchange, id);

    // 40 KB of header is over the 64 KiB limit?  No: it fits, and arrives split in CONTINUATION frames.
    check(result.status == 200 && contains(text_of(result), tprint("custom=%", big_value)), "a header block in HEADERS and CONTINUATION frames");
}

// The stream limit: the third stream of a server that allows two is refused.
stream_limit :: (base: string, server: *Server) {
    peer := connect_peer(server);

    defer disconnect(peer);

    handshake(peer);

    check(peer.server_settings.max_concurrent_streams == 2, "the limit is announced");

    first := request(peer, "GET", "/sleepy");
    second := request(peer, "GET", "/sleepy");
    third := request(peer, "GET", "/ping");

    exchange: Exchange;

    exchange.results.allocator = temp;

    read_results(peer, *exchange, .[first, second, third]);

    check(result_for(*exchange, third).reset && result_for(*exchange, third).reset_code == cast(u32) H2_Error.REFUSED_STREAM, "the third stream is refused");
    check(result_for(*exchange, first).status == 200 && result_for(*exchange, second).status == 200, "the first two are answered");

    result := fetch(peer, "GET", "/ping");

    check(result.status == 200, "and a stream can be opened again once they have ended");
}

// An HTTP/1.1 client on the same server, and a client that offers only h2 or nothing.
negotiation :: (base: string, server: *Server) {
    config: Tls.Tls_Config;

    config.roots_pem = load("ca.pem");
    config.alpn = string.["http/1.1"];

    check(Tls.tls_config_init(*config).kind == .NONE, "config");

    defer Tls.tls_config_free(*config);

    api: Client;

    api.base_url = base;
    api.tls = *config;

    response, error := get(*api, "/ping");

    check(error.kind == .NONE && response.status == 200, "an HTTP/1.1 client is served on the same port: %", error_text(error));

    text, _ := read_text(*response);

    check(text == "pong", "with the same answer");
    close(*response);

    // No ALPN at all is HTTP/1.1 too.
    none: Tls.Tls_Config;

    none.roots_pem = load("ca.pem");

    check(Tls.tls_config_init(*none).kind == .NONE, "config");

    defer Tls.tls_config_free(*none);

    api.tls = *none;

    response, error = get(*api, "/ping");

    check(error.kind == .NONE && response.status == 200, "a client with no ALPN is served: %", error_text(error));
    close(*response);

    // A client that prefers h2 and also offers http/1.1 gets h2.
    peer := connect_peer(server, string.["h2", "http/1.1"]);

    defer disconnect(peer);

    check(Tls.alpn_protocol(peer.tls) == "h2", "h2 is chosen when offered first: '%'", Tls.alpn_protocol(peer.tls));
}

// Bytes that are not the preface close the connection without an answer.
bad_preface :: (base: string, server: *Server) {
    peer := connect_peer(server, string.["h2"], false);

    defer disconnect(peer);

    send_bytes(peer, bytes_of("GET / HTTP/1.1\r\nHost: x\r\n\r\n"));

    ended := false;

    // The server may have sent its SETTINGS and a GOAWAY before it closes; no response to the HTTP/1 request.
    for 1..4 {
        frame := next_frame(peer, 2000);

        if !frame.ok {
            ended = true;

            break;
        }

        check(frame.header.type == .SETTINGS || frame.header.type == .WINDOW_UPDATE || frame.header.type == .GOAWAY, "only HTTP/2 frames come back");
    }

    check(ended, "the connection is closed");
}

// A server shut down while a connection is idle tells it so.
shutdown :: (base: string, server: *Server) {
    peer := connect_peer(server);

    defer disconnect(peer);

    handshake(peer);

    result := fetch(peer, "GET", "/ping");

    check(result.status == 200, "a request before the shutdown");

    stop(server);

    exchange: Exchange;

    exchange.results.allocator = temp;

    read_results(peer, *exchange, .[]);

    check(exchange.goaway && exchange.goaway_code == 0, "the server says GOAWAY with NO_ERROR: %", exchange.goaway_code);
}

main :: () {
    if !supported() return;

    big_text = alloc_string(BIG);

    for *big_text it.* = cast(u8) (#char "a" + it_index % 26);

    get(*app, "/ping", ping);
    get(*app, "/describe", describe);
    get(*app, "/big", big);
    get(*app, "/streamed", streamed);
    get(*app, "/missing", missing);
    get(*app, "/sized", ping);
    head(*app, "/sized", sized_head);
    post(*app, "/size", size_of_body, max_body = 8 * 1024 * 1024);
    post(*app, "/echo", echo_text);

    server_config: Tls.Tls_Config;

    server_config.certificate_chain_pem = load("server.pem");
    server_config.private_key_pem = load("server.key");
    server_config.alpn = string.["h2", "http/1.1"];

    check(Tls.tls_config_init(*server_config).kind == .NONE, "server config");

    app.config.tls = *server_config;
    defer app.config.tls = null;
    app.config.max_head_size = 128 * 1024;

    run_server(*app, basic_requests);
    run_server(*app, large_response);
    run_server(*app, large_request);
    run_server(*app, many_streams);
    run_server(*app, protocol_errors);
    run_server(*app, negotiation);
    run_server(*app, bad_preface);
    run_server(*app, shutdown);

    get(*small_app, "/sleepy", sleepy);
    get(*small_app, "/ping", ping);

    small_app.config.tls = *server_config;
    defer small_app.config.tls = null;
    small_app.config.h2_max_streams = 2;

    run_server(*small_app, stream_limit);

    finish();
}
