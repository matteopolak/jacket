// test: skip_on(.WASM, reason = "needs the Http server and router, which are not built for wasm")
// The HTTP/2 client against the HTTP/2 server of this library over TLS on loopback: one connection
// shared by many concurrent requests, bodies that need flow-control updates in both directions,
// streamed responses, and the fall back to HTTP/1.1 against a peer that does not offer h2 (and the
// other way round). Run from this directory.
#load "support/server_support.jai";

Tls :: #import "Extensions/Tls";

#load "../../Tls/tests/fixtures/certs.jai";

load :: (name: string) -> string {
    for DATA if it.name == name return it.text;

    assert(false, "no test file %", name);

    return "";
}

BIG :: 600 * 1024 + 3;

big_text: string;

big :: () -> string {
    return big_text;
}

ping :: () -> string {
    return "pong";
}

slow :: (request: *Request) -> string {
    Io.sleep(30);

    return request.target;
}

size_of_body :: (body: *Body) -> string, Http_Error {
    bytes, error := read_all(body);

    if error.kind != .NONE return "", error;

    return tprint("%", bytes.count), .{};
}

produce :: (stream: *Stream, data: *void) {
    for 1..5 {
        if !send_chunk(stream, tprint("tick %\n", it)) return;

        Io.sleep(10);
    }
}

streamed :: () -> Response {
    return stream_response(produce, null, "text/plain");
}

missing :: () -> Response {
    return status_only(404);
}

client_config :: (alpn: [] string) -> Tls.Tls_Config {
    config: Tls.Tls_Config;

    config.roots_pem = load("ca.pem");
    config.alpn = alpn;

    check(Tls.tls_config_init(*config).kind == .NONE, "client config");

    return config;
}

// The TLS configs of the tests are freed when they return, so no connection may be left holding one.
close_all :: () {
    close_idle_connections();

    left := http_statistics().idle_connections;

    check(left == 0, "close_idle_connections left % connections", left);
}

Fan :: struct {
    base: string;
    config: *Tls.Tls_Config;
    index: int;
    group: *Io.Wait_Group;
    bad: *int;
}

fan_out :: (fan: Fan) {
    defer Io.done(fan.group);

    api: Client;

    api.base_url = fan.base;
    api.tls = fan.config;

    path := tprint("/slow?i=%", fan.index);
    response, error := get(*api, path);

    if error.kind != .NONE {
        fan.bad.* += 1;

        return;
    }

    status := response.status;
    text, read_error := read_text(*response);

    close(*response);

    if status != 200 || read_error.kind != .NONE || text != path fan.bad.* += 1;
}

shared_connection :: (base: string, server: *Server) {
    config := client_config(string.["h2", "http/1.1"]);

    defer release_tls_config(*config);

    api: Client;

    api.base_url = base;
    api.tls = *config;

    before := http_statistics().connections_opened;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong", "a request: % '%'", reply.status, reply.body);
    check(reply.http2, "the answer came over HTTP/2");

    // Many requests at once, from several clients that share the scheduler's pool.
    group: Io.Wait_Group;
    bad := 0;

    Io.add(*group, 40);

    for 1..40 Io.spawn(fan_out, Fan.{ base, *config, it, *group, *bad });

    Io.wait(*group);

    check(bad == 0, "% of 40 concurrent requests failed", bad);

    opened := http_statistics().connections_opened - before;

    check(opened == 1, "40 concurrent requests shared one connection: % opened", opened);

    // A body bigger than the initial windows, both ways.
    reply = get_path(*api, "/big");

    check(reply.status == 200 && reply.body.count == BIG, "a large response: % bytes", reply.body.count);

    intact := true;

    for reply.body {
        if it != #char "a" + it_index % 26 {
            intact = false;

            break;
        }
    }

    check(intact, "the large response arrived intact");

    upload := alloc_string(BIG);

    defer free(upload.data);

    for *upload it.* = cast(u8) (it_index & 0x7f);

    reply = post_body(*api, "/size", upload, "text/plain");

    check(reply.status == 200 && reply.body == tprint("%", BIG), "a large request: '%'", reply.body);

    // Streaming and statuses.
    reply = get_path(*api, "/streamed");

    check(reply.status == 200 && reply.body == "tick 1\ntick 2\ntick 3\ntick 4\ntick 5\n", "a streamed response: '%'", reply.body);

    reply = get_path(*api, "/missing");

    check(reply.status == 404, "a 404: %", reply.status);

    reply = fetch(*api, .HEAD, "/ping");

    check(reply.status == 200 && reply.body.count == 0, "HEAD: % '%'", reply.status, reply.body);

    check(http_statistics().connections_opened - before == 1, "everything ran on one connection: % opened", http_statistics().connections_opened - before);

    close_all();
}

// A client that offers h2 to a server that only speaks HTTP/1.1.
http1_server :: (base: string, server: *Server) {
    config := client_config(string.["h2", "http/1.1"]);

    defer release_tls_config(*config);

    api: Client;

    api.base_url = base;
    api.tls = *config;

    for 1..3 {
        reply := get_path(*api, "/ping");

        check(reply.status == 200 && reply.body == "pong", "an HTTP/1.1 server answers: % '%'", reply.status, reply.body);
        check(!reply.http2, "and it is HTTP/1.1");
    }

    group: Io.Wait_Group;
    bad := 0;

    Io.add(*group, 10);

    for 1..10 Io.spawn(fan_out, Fan.{ base, *config, it, *group, *bad });

    Io.wait(*group);

    check(bad == 0, "% of 10 concurrent requests failed over HTTP/1.1", bad);

    close_all();
}

// A client that only offers http/1.1 to a server that would speak h2.
http1_client :: (base: string, server: *Server) {
    config := client_config(string.["http/1.1"]);

    defer release_tls_config(*config);

    api: Client;

    api.base_url = base;
    api.tls = *config;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong", "an HTTP/1.1 client is served: % '%'", reply.status, reply.body);
    check(!reply.http2, "over HTTP/1.1");

    reply = get_path(*api, "/big");

    check(reply.status == 200 && reply.body.count == BIG, "and gets a large body: %", reply.body.count);

    close_all();
}

// `http2 = false` on the server: it does not pick h2 even for a client that offers it, so the client
// falls back to HTTP/1.1 instead of being refused. And `client_settings.http2 = false` on the client
// does the same against a server that would speak h2.
switched_off :: (base: string, server: *Server) {
    config := client_config(string.["h2", "http/1.1"]);

    defer release_tls_config(*config);

    api: Client;

    api.base_url = base;
    api.tls = *config;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong" && !reply.http2, "the server with http2 off answers over HTTP/1.1: % '%'", reply.status, reply.body);

    close_all();
}

client_off :: (base: string, server: *Server) {
    config := client_config(string.["h2", "http/1.1"]);

    defer release_tls_config(*config);

    api: Client;

    api.base_url = base;
    api.tls = *config;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.http2, "with http2 on the client speaks it");

    close_all();

    client_settings.http2 = false;

    reply = get_path(*api, "/ping");

    check(reply.status == 200 && reply.body == "pong" && !reply.http2, "client_settings.http2 = false: HTTP/1.1 even though the config lists h2: % '%'", reply.status, reply.body);

    close_all();

    client_settings.http2 = true;
}

// Freeing a config closes the idle HTTP/2 connection made with it, and the `h2`-less copy that
// `client_settings.http2 = false` makes of it.
released_config :: (base: string, server: *Server) {
    config := client_config(string.["h2", "http/1.1"]);

    api: Client;

    api.base_url = base;
    api.tls = *config;

    reply := get_path(*api, "/ping");

    check(reply.status == 200 && reply.http2, "an HTTP/2 request");
    check(http_statistics().idle_connections == 1, "the connection is pooled: %", http_statistics().idle_connections);

    client_settings.http2 = false;

    reply = get_path(*api, "/ping");

    check(reply.status == 200 && !reply.http2, "an HTTP/1.1 request through the copy of the config");
    check(http_statistics().idle_connections == 2, "both connections are pooled: %", http_statistics().idle_connections);

    client_settings.http2 = true;

    release_tls_config(*config);

    check(http_statistics().idle_connections == 0, "releasing the config closes both: %", http_statistics().idle_connections);

    config = client_config(string.["h2", "http/1.1"]);

    before := http_statistics().connections_opened;

    reply = get_path(*api, "/ping");

    check(reply.status == 200 && reply.http2, "a new config at the same address");
    check(http_statistics().connections_opened == before + 1, "gets a connection of its own: %", http_statistics().connections_opened - before);

    release_tls_config(*config);
}

register :: (app: *App) {
    get(app, "/ping", ping);
    get(app, "/slow", slow);
    get(app, "/big", big);
    get(app, "/streamed", streamed);
    get(app, "/missing", missing);
    head(app, "/ping", ping);
    post(app, "/size", size_of_body, max_body = 8 * 1024 * 1024);
}

app: App;
legacy: App;
quiet: App;

main :: () {
    if !supported() return;

    big_text = alloc_string(BIG);

    for *big_text it.* = cast(u8) (#char "a" + it_index % 26);

    register(*app);
    register(*legacy);
    register(*quiet);

    h2_config: Tls.Tls_Config;

    h2_config.certificate_chain_pem = load("server.pem");
    h2_config.private_key_pem = load("server.key");
    h2_config.alpn = string.["h2", "http/1.1"];

    check(Tls.tls_config_init(*h2_config).kind == .NONE, "server config");

    old_config: Tls.Tls_Config;

    old_config.certificate_chain_pem = load("server.pem");
    old_config.private_key_pem = load("server.key");
    old_config.alpn = string.["http/1.1"];

    check(Tls.tls_config_init(*old_config).kind == .NONE, "server config");

    app.config.tls = *h2_config;
    defer app.config.tls = null;
    quiet.config.tls = *h2_config;
    defer quiet.config.tls = null;
    quiet.config.http2 = false;
    legacy.config.tls = *old_config;
    defer legacy.config.tls = null;

    run_server(*app, shared_connection);
    run_server(*app, released_config);
    run_server(*legacy, http1_server);
    run_server(*app, http1_client);
    run_server(*quiet, switched_off);
    run_server(*app, client_off);

    finish();
}
