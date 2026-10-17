// HTTPS: the client and the server over TLS on loopback, with the test CA of Extensions/Tls
// (../../Tls/tests/data): a round trip, keep-alive across requests, certificate failures, client
// certificates, ALPN, handshakes that never finish, idle connections holding no read buffer, large
// bodies and graceful shutdown. Run from this directory, once per backend (JAI_IO_BACKEND).
#load "support/server_support.jai";

Tls :: #import "Extensions/Tls";

// The certificates of the Tls tests, as string constants (see ../../Tls/tests/data/make_certs.sh).
#load "../../Tls/tests/fixtures/certs.jai";

load :: (name: string) -> string {
    for DATA if it.name == name return it.text;

    assert(false, "no test file %", name);

    return "";
}

ping :: () -> string {
    return "pong";
}

echo_path :: (request: *Request) -> string {
    return request.path;
}

echo_size :: (body: Text) -> string {
    return tprint("%", body.value.count);
}

// Built once, before any server runs: filling 3 MB in the interpreter takes seconds on a slow machine,
// which must not count against a request.
big: string;

big_page :: () -> string {
    return big;
}

// A request with time to spare for the interpreter: the large bodies are checked byte by byte.
LONG_TIMEOUT :: 120_000;

fetch_long :: (api: *Client, method: Method, path: string, body := "", content_type := "") -> Reply {
    request: Client_Request;

    request.method = method;
    request.url = path;
    request.body = .{ body.count, body.data };
    request.timeout = LONG_TIMEOUT;

    headers: [1] Header_Field;

    if content_type.count {
        headers[0] = .{ "Content-Type", content_type };
        request.headers = headers;
    }

    response, error := send(api, request);

    check(error.kind == .NONE, "% % failed: %", method_name(method), path, error_text(error));

    return into_reply(*response);
}

BIG :: 3 * 1024 * 1024 + 17;

app: App;

make_server_config :: (certificate: string, roots := "", client_auth := false, alpn: [] string = string.["http/1.1"]) -> Tls.Tls_Config {
    config: Tls.Tls_Config;

    config.certificate_chain_pem = load(tprint("%.pem", certificate));
    config.private_key_pem = load(tprint("%.key", certificate));

    if roots.count config.roots_pem = load(tprint("%.pem", roots));

    config.client_auth = client_auth;
    config.alpn = alpn;

    error := Tls.tls_config_init(*config);

    check(error.kind == .NONE, "server config: %", Tls.error_text(error));

    return config;
}

make_client_config :: (certificate := "", roots := "ca", alpn: [] string = string.["http/1.1"], verify := true) -> Tls.Tls_Config {
    config: Tls.Tls_Config;

    config.verify = verify;

    if certificate.count {
        config.certificate_chain_pem = load(tprint("%.pem", certificate));
        config.private_key_pem = load(tprint("%.key", certificate));
    }

    if roots.count config.roots_pem = load(tprint("%.pem", roots));

    config.alpn = alpn;

    error := Tls.tls_config_init(*config);

    check(error.kind == .NONE, "client config: %", Tls.error_text(error));

    return config;
}

// 1. A round trip, keep-alive, large bodies.
round_trips :: (base: string, server: *Server) {
    check(begins_with(base, "https://127.0.0.1:"), "the server announces https: %", base);

    client_config := make_client_config();

    defer Tls.tls_config_free(*client_config);

    api: Client;

    api.base_url = base;
    api.tls = *client_config;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong", "GET over TLS: % '%'", reply.status, reply.body);

    reply = get_path(*api, "/echo/some");

    check(reply.body == "/echo/some", "the path: '%'", reply.body);

    // Five requests reuse one TLS connection: one handshake.
    before := http_statistics().connections_opened;

    for 1..5 {
        reply = get_path(*api, "/ping");

        check(reply.status == 200 && reply.body == "pong", "keep-alive request %", it);
    }

    check(http_statistics().connections_opened == before, "keep-alive: % connections opened for five requests", http_statistics().connections_opened - before);

    // Bodies bigger than a record and than the socket buffers, both ways.
    reply = fetch_long(*api, .GET, "/big");

    check(reply.status == 200 && reply.body.count == BIG, "a large response: % bytes", reply.body.count);

    ok := true;

    for reply.body {
        if it != #char "a" + it_index % 26 {
            ok = false;

            break;
        }
    }

    check(ok, "the large response arrived intact");

    upload := alloc_string(BIG);

    defer free(upload.data);

    for 0..BIG - 1 upload[it] = cast(u8) (it & 0x7f);

    reply = fetch_long(*api, .POST, "/size", upload, "text/plain");

    check(reply.status == 200 && reply.body == tprint("%", BIG), "a large request: '%'", reply.body);

    // The default client (no tls config) trusts the system's roots, which do not know the test CA.
    plain: Client;

    plain.base_url = base;

    response, error := send(*plain, .{ url = tprint("%/ping", base), timeout = 5000 });

    check(error.kind == .TLS, "the system roots do not trust the test CA: % %", error.kind, error_text(error));
    check(contains(error.detail, "certificate"), "and it says so: %", error.detail);
    check(response.status == 0, "no response");
    free_error(*error);
}

// 2. What a client does when the server's certificate is not good.
certificate_failures :: (base: string, server: *Server) {
    unknown := make_client_config("", "other_ca");

    defer Tls.tls_config_free(*unknown);

    api: Client;

    api.base_url = base;
    api.tls = *unknown;

    response, error := send(*api, .{ url = tprint("%/ping", base), timeout = 5000 });

    check(error.kind == .TLS && contains(error.detail, "trusted"), "an untrusted CA: % %", error.kind, error_text(error));
    free_error(*error);

    // Switching the check off connects.
    unchecked := make_client_config("", "", verify = false);

    defer Tls.tls_config_free(*unchecked);

    check(Tls.tls_config_init(*unchecked).kind == .NONE, "re-init is harmless");

    api.tls = *unchecked;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong", "verify = false connects: %", reply.status);

    // A client config nobody set up is an error, not a crash.
    forgotten: Tls.Tls_Config;

    api.tls = *forgotten;

    response, error = send(*api, .{ url = tprint("%/ping", base), timeout = 5000 });

    check(error.kind == .TLS && contains(error.detail, "tls_config_init"), "an uninitialised config: %", error_text(error));
    free_error(*error);

    // Plain HTTP to a TLS server: the server drops the connection, the client reports a failure.
    plain_url := tprint("http://%/ping", slice(base, 8, base.count - 8));

    api.tls = null;

    response, error = send(*api, .{ url = plain_url, timeout = 5000 });

    check(error.kind == .TRANSPORT || error.kind == .TIMED_OUT, "http to an https port: % %", error.kind, error_text(error));
    free_error(*error);
}

wrong_name_app: App;
impatient_app: App;
logging_app: App;

// 3. A server with a certificate for another name.
wrong_name :: (base: string, server: *Server) {
    client_config := make_client_config();

    defer Tls.tls_config_free(*client_config);

    api: Client;

    api.base_url = base;
    api.tls = *client_config;

    _, error := send(*api, .{ url = tprint("%/ping", base), timeout = 5000 });

    check(error.kind == .TLS && contains(error.detail, "Common Name"), "a certificate for another name: % %", error.kind, error_text(error));
    free_error(*error);
}

expired_app: App;

expired :: (base: string, server: *Server) {
    client_config := make_client_config();

    defer Tls.tls_config_free(*client_config);

    api: Client;

    api.base_url = base;
    api.tls = *client_config;

    _, error := send(*api, .{ url = tprint("%/ping", base), timeout = 5000 });

    check(error.kind == .TLS && contains(error.detail, "expired"), "an expired certificate: % %", error.kind, error_text(error));
    free_error(*error);
}

client_auth_app: App;

// 4. Client certificates.
client_certificates :: (base: string, server: *Server) {
    with := make_client_config("client");
    without := make_client_config();

    defer Tls.tls_config_free(*with);
    defer Tls.tls_config_free(*without);

    api: Client;

    api.base_url = base;
    api.tls = *with;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong", "a client with a certificate: %", reply.status);

    api.tls = *without;

    _, error := send(*api, .{ url = tprint("%/ping", base), timeout = 5000 });

    check(error.kind != .NONE, "a client with no certificate is refused");
    free_error(*error);
}

// 5. ALPN.
alpn :: (base: string, server: *Server) {
    h2_only := make_client_config("", "ca", string.["h2"]);
    none := make_client_config("", "ca", .[]);

    defer Tls.tls_config_free(*h2_only);
    defer Tls.tls_config_free(*none);

    api: Client;

    api.base_url = base;
    api.tls = *h2_only;

    _, error := send(*api, .{ url = tprint("%/ping", base), timeout = 5000 });

    check(error.kind == .TLS, "a client that only offers h2: % %", error.kind, error_text(error));
    free_error(*error);

    // No ALPN at all means HTTP/1.1.
    api.tls = *none;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong", "a client with no ALPN: %", reply.status);
}

// 6. Handshakes that go wrong or never finish.
bad_handshakes :: (base: string, server: *Server) {
    // Connects and says nothing: the handshake has header_timeout (400 ms here).
    stream := raw_connect(server);
    started := current_time_monotonic();

    check(closes_within(stream, 3000), "a client that never starts the handshake is dropped");

    elapsed := elapsed_milliseconds(started);

    check(elapsed >= 300 && elapsed < 1500, "after about header_timeout: % ms", elapsed);
    Io.close(stream);

    // Speaks HTTP instead of TLS.
    stream = raw_connect(server);
    send_text(stream, "GET /ping HTTP/1.1\r\nHost: t\r\n\r\n");

    text := read_until_closed(stream, 3000);

    check(!contains(text, "pong"), "plain HTTP on the TLS port gets no answer: '%'", text);
    Io.close(stream);

    // Starts a handshake and stops halfway.
    stream = raw_connect(server);
    send_text(stream, "\x16\x03\x01\x00\x30\x01\x00\x00");
    check(closes_within(stream, 3000), "a half handshake is dropped");
    Io.close(stream);

    // And the server still works.
    client_config := make_client_config();

    defer Tls.tls_config_free(*client_config);

    api: Client;

    api.base_url = base;
    api.tls = *client_config;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong", "the server is fine after bad handshakes: %", reply.status);
}

buffers_held :: () -> int {
    return http_statistics().read_buffers_in_use;
}

// 7. A connection that waits holds no read buffer, over TLS too.
idle_holds_no_buffer :: (base: string, server: *Server) {
    client_config := make_client_config();

    defer Tls.tls_config_free(*client_config);

    api: Client;

    api.base_url = base;
    api.tls = *client_config;

    COUNT :: 12;

    responses: [COUNT] Client_Response;

    // Responses that are still open keep their connections, so these are twelve connections.
    for 0..COUNT - 1 {
        error: Client_Error;

        responses[it], error = get(*api, "/ping");

        check(error.kind == .NONE, "request %: %", it, error_text(error));
    }

    for *responses {
        text, error := read_text(it);

        check(error.kind == .NONE && text == "pong", "body %: %", it_index, text);

        free(text.data);
        close(it);
    }

    Io.sleep(200);

    check(buffers_held() == 0, "% idle TLS connections hold % read buffers", COUNT, buffers_held());

    kept := http_statistics().read_buffers_free;

    check(kept >= 1, "the buffers went back to the pool: %", kept);

    // They all still work.
    for 1..COUNT {
        reply := get_path(*api, "/ping");

        check(reply.status == 200 && reply.body == "pong", "after idling: %", reply.status);
    }
}

// 8. Many clients at once.
Crowd :: struct {
    base: string;
    config: *Tls.Tls_Config;
    finished: int;
    failures: int;
}

crowd_client :: (crowd: *Crowd) {
    api: Client;

    api.base_url = crowd.base;
    api.tls = crowd.config;

    for 1..5 {
        response, error := get(*api, "/echo/crowd");

        if error.kind != .NONE {
            crowd.failures += 1;

            free_error(*error);

            continue;
        }

        text, read_error := read_text(*response);

        if read_error.kind != .NONE || text != "/echo/crowd" crowd.failures += 1;

        free(text.data);
        close(*response);
    }

    crowd.finished += 1;
}

many_clients :: (base: string, server: *Server) {
    client_config := make_client_config();

    defer Tls.tls_config_free(*client_config);

    crowd := Crowd.{ base = base, config = *client_config };

    COUNT :: 30;

    for 1..COUNT Io.spawn(crowd_client, *crowd);

    waited := 0;

    while crowd.finished < COUNT && waited < 20000 {
        Io.sleep(10);

        waited += 10;
    }

    check(crowd.finished == COUNT && crowd.failures == 0, "% of % clients finished, % failures", crowd.finished, COUNT, crowd.failures);
}

// 9. Shutdown with TLS connections open.
shutdown :: (base: string, server: *Server) {
    client_config := make_client_config();

    defer Tls.tls_config_free(*client_config);

    api: Client;

    api.base_url = base;
    api.tls = *client_config;

    reply := get_path(*api, "/ping");

    check(reply.status == 200, "a request before shutdown");

    // The pooled connection is idle on the server: stopping closes it quickly.
    started := current_time_monotonic();

    stop(server);
    wait(server);

    elapsed := elapsed_milliseconds(started);

    check(elapsed < 3000, "shutdown with an idle TLS connection took % ms", elapsed);
}

// 10. A server config that cannot serve.
bad_server_config :: () {
    bad: App;

    get(*bad, "/ping", ping);

    config: Tls.Tls_Config;

    bad.config.tls = *config;
    bad.config.address = "127.0.0.1:0";
    bad.config.handle_signals = false;

    print("(the server's refusal on the next line is the expected result of this check)\n");

    Io.io_run((app: *App) {
        server, error := start(app);

        check(server == null && error == .INVALID, "a config that was never set up cannot serve: %", error);
    }, *bad);
}

// 11. Peers that leave without close_notify. The Tls layer reports it (`TRUNCATED`); HTTP decides
// whether it matters: not for a body that ends with the connection, nor for an idle connection.

// A TLS server that answers one way and then drops the connection without close_notify.
Ragged_Mode :: enum {
    UNTIL_CLOSE; // HTTP/1.0, no Content-Length: the body ends with the connection
    CUT_LENGTH; // promises 100000 bytes and sends 20000
    KEEP_ALIVE; // a complete keep-alive answer, then the connection is dropped
}

Ragged :: struct {
    listener: Io.Tcp_Listener;
    config: *Tls.Tls_Config;
    mode: Ragged_Mode;
    accepted: int;
}

RAGGED_SIZE :: 200_000;

ragged_body :: (count: int) -> string {
    text := alloc_string(count,, temp);

    for *text it.* = cast(u8) (#char "a" + it_index % 26);

    return text;
}

// Reads until the end of a request head; false when the client went away first.
read_request_head :: (stream: *Tls.Tls_Stream) -> bool {
    seen: [..] u8;

    seen.allocator = temp;

    buffer: [512] u8;

    while true {
        count, error := Tls.read(stream, buffer);

        if error != .NONE || count == 0 return false;

        for 0..count - 1 array_add(*seen, buffer[it]);

        if contains(string.{ seen.count, seen.data }, "\r\n\r\n") return true;
    }

    return false;
}

ragged_server :: (state: *Ragged) {
    while true {
        tcp, accept_error := Io.accept(state.listener);

        if accept_error != .NONE break;

        state.accepted += 1;

        stream, tls_error := Tls.tls_accept(state.config, Io.reader_of(tcp), Io.writer_of(tcp));

        if tls_error.kind != .NONE {
            Io.close(tcp);

            continue;
        }

        if read_request_head(stream) {
            if state.mode == {
                case .UNTIL_CLOSE;
                    Tls.write(stream, "HTTP/1.0 200 OK\r\nContent-Type: text/plain\r\n\r\n");
                    Tls.write(stream, ragged_body(RAGGED_SIZE));

                case .CUT_LENGTH;
                    Tls.write(stream, "HTTP/1.1 200 OK\r\nContent-Length: 100000\r\n\r\n");
                    Tls.write(stream, ragged_body(20_000));

                case .KEEP_ALIVE;
                    Tls.write(stream, "HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\npong");
            }
        }

        // Closes the connection and sends no close_notify.
        Tls.close(stream, true);
    }
}

start_ragged :: (state: *Ragged, mode: Ragged_Mode, config: *Tls.Tls_Config) -> string {
    error: Io.Io_Error;

    state.* = .{};
    state.config = config;
    state.mode = mode;

    state.listener, error = Io.tcp_listen("127.0.0.1:0");

    check(error == .NONE, "listen: %", error);

    address, _ := Io.local_address(state.listener);

    Io.spawn(ragged_server, state);

    return tprint("https://%", Io.address_to_string(address));
}

stop_ragged :: (state: *Ragged) {
    Io.close(state.listener);
    Io.sleep(20);
}

ragged_eof :: (base: string, server: *Server) {
    client_config := make_client_config();

    defer Tls.tls_config_free(*client_config);

    server_config := make_server_config("server");

    defer Tls.tls_config_free(*server_config);

    state: Ragged;

    // A body that ends with the connection is complete when the peer leaves without close_notify.
    ragged_base := start_ragged(*state, .UNTIL_CLOSE, *server_config);

    api: Client;

    api.base_url = ragged_base;
    api.tls = *client_config;

    reply := get_path(*api, "/");

    check(reply.status == 200, "read until close, status: %", reply.status);
    check(reply.body == ragged_body(RAGGED_SIZE), "read until close, the whole body without an error: % bytes", reply.body.count);

    // The same through the streaming reader.
    response, error := get(*api, "/");

    check(error.kind == .NONE, "read until close, streaming: %", error_text(error));

    total := 0;
    buffer: [4096] u8;

    while error.kind == .NONE {
        count: int;

        count, error = read_some(*response, buffer);

        if count == 0 && error.kind == .NONE break;

        total += count;
    }

    check(error.kind == .NONE && total == RAGGED_SIZE, "read until close, streamed: % bytes, %", total, error_text(error));

    close(*response);
    stop_ragged(*state);

    // A body with a Content-Length that is cut short is still an error.
    base_cut := start_ragged(*state, .CUT_LENGTH, *server_config);

    api.base_url = base_cut;

    response, error = get(*api, "/");

    failed := error.kind != .NONE;

    if !failed {
        _, error = read_text(*response);

        failed = error.kind != .NONE;

        close(*response);
    }

    check(failed, "a cut Content-Length body is an error");
    check(error.kind == .TRANSPORT, "and a transport error: %", error.kind);

    stop_ragged(*state);

    // A pooled connection the server dropped without close_notify is replaced by a retry.
    base_pooled := start_ragged(*state, .KEEP_ALIVE, *server_config);

    api.base_url = base_pooled;

    for 1..4 {
        reply = get_path(*api, "/");

        check(reply.status == 200 && reply.body == "pong", "request % over a dropped pooled connection: % '%'", it, reply.status, reply.body);

        Io.sleep(50);
    }

    check(state.accepted == 4, "each request needed a new connection: %", state.accepted);

    stop_ragged(*state);
}

// A client that drops its TLS connection without close_notify is an ordinary end for the server.
ragged_clients :: (base: string, server: *Server) {
    client_config := make_client_config();

    defer Tls.tls_config_free(*client_config);

    // Handshakes and leaves, with and without a request.
    for 0..3 {
        tcp := raw_connect(server);

        stream, error := Tls.tls_connect(*client_config, Io.reader_of(tcp), Io.writer_of(tcp), "127.0.0.1");

        check(error.kind == .NONE, "handshake: %", Tls.error_text(error));

        if error.kind != .NONE {
            Io.close(tcp);

            continue;
        }

        if it == 1 || it == 2 Tls.write(stream, "GET /ping HTTP/1.1\r\nHost: t\r\n\r\n");

        if it == 2 {
            answer: [256] u8;
            text: string;

            while !contains(text, "pong") {
                count, read_error := Tls.read(stream, answer);

                check(read_error == .NONE && count > 0, "answer: % %", count, read_error);

                if read_error != .NONE || count == 0 break;

                text = tprint("%1%2", text, string.{ count, answer.data });
            }
        }

        if it == 3 Tls.write(stream, "GET /pi");

        Tls.close(stream, true);
    }

    started := current_time_monotonic();

    while server.count > 0 && elapsed_milliseconds(started) < 3000 Io.sleep(10);

    check(server.count == 0, "the server dropped the connections: % left", server.count);

    api: Client;

    api.base_url = base;
    api.tls = *client_config;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong", "the server is fine after clients that left: %", reply.status);
}

main :: () {
    if !supported() return;

    big = alloc_string(BIG);

    for *big it.* = cast(u8) (#char "a" + it_index % 26);

    get(*app, "/ping", ping);
    get(*app, "/echo/{rest}", echo_path);
    get(*app, "/big", big_page);
    post(*app, "/size", echo_size);

    app.config.max_body_size = 8 * 1024 * 1024;

    server_config := make_server_config("server");

    app.config.tls = *server_config;

    run_server(*app, round_trips);
    run_server(*app, certificate_failures);
    run_server(*app, alpn);
    run_server(*app, idle_holds_no_buffer);
    run_server(*app, many_clients);
    run_server(*app, ragged_eof);
    run_server(*app, shutdown);

    // The handshake timeout is short here.
    get(*impatient_app, "/ping", ping);

    impatient_app.config.header_timeout = 400;
    impatient_app.config.tls = *server_config;

    run_server(*impatient_app, bad_handshakes);

    get(*wrong_name_app, "/ping", ping);

    wrong_config := make_server_config("wrong_name");

    wrong_name_app.config.tls = *wrong_config;

    run_server(*wrong_name_app, wrong_name);

    get(*expired_app, "/ping", ping);

    expired_config := make_server_config("expired");

    expired_app.config.tls = *expired_config;

    run_server(*expired_app, expired);

    get(*client_auth_app, "/ping", ping);

    auth_config := make_server_config("server", "ca", true);

    client_auth_app.config.tls = *auth_config;

    run_server(*client_auth_app, client_certificates);

    // Logs every request: leaving without close_notify must add nothing (see the output).
    get(*logging_app, "/ping", ping);

    logging_app.config.log_requests = true;
    logging_app.config.tls = *server_config;

    run_server(*logging_app, ragged_clients);

    bad_server_config();

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*wrong_config);
    Tls.tls_config_free(*expired_config);
    Tls.tls_config_free(*auth_config);

    finish();
}
