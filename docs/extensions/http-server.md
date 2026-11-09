# Http server and test client

A routing HTTP/1.1 server on fibers: register plain functions as handlers, declare what they need as parameter types, and the module checks them against the route at compile time and generates the request-reading code. Sits on the [codec](http-codec.md) and [Io](io-engine.md); the [client](http-client.md) is the other half. Not on wasm. Shared rules: [README](README.md#conventions-of-io-tls-http-and-json). Runnable: `examples/http/`.

```jai
#import "Extensions/Http";

User :: struct { id: int; name: string; }
New_User :: struct { name: string; }

get_user :: (id: Path("id", int), store: *State(Store)) -> User, Http_Error {
    for store.users if it.id == id.value return it, .{};
    return .{}, not_found("No user with id %", id.value);
}

create_user :: (body: Json(New_User), store: *State(Store)) -> Response {
    user := User.{ store.users.count + 1, copy_string(body.temporary.name) };
    array_add(*store.users, user);
    return created(user, tprint("/users/%", user.id));     // 201 with a Location header
}

main :: () {
    store: Store;
    app: App;
    app.state = *store;
    get(*app, "/users/{id}", get_user);
    post(*app, "/users", create_user);

    error := serve(*app);       // :8080, one scheduler per core, Ctrl-C drains and exits
    if error.kind != .NONE { print("%\n", error_text(error)); exit(1); }
}
```

## How it works

- **Registration.** `get post put patch delete head options (app or group, template, handler, max_body = 0)`. The template (`/users/{id}`, `/files/{path...}` last) is parsed at compile time. `register_route` (`app.jai`) inspects the handler's type, calls `generate_prelude`/`generate_call` (`extract.jai`) and `#insert`s a thunk `(*Request) -> Response`. Route mistakes are reported on the `get(...)` call in route terms. Two routes matching the same requests make `serve` return `.INVALID_ROUTES`.
- **Extractors** are handler parameters. `Path("id", T)`, `Rest("path")`, `Header("n", T)` and `Header_Optional` keep one scalar in `value`. `Params(T)`, `Query(T)`, `Form(T)` and `Json(T)` decode a struct into `temporary`. `*Body` streams the body. `*Request` gives method, target, path, query, headers and peer; `*State(T)` gives `app.state`. Failures become 400/413/415/422 problem responses.
- **Returns.** Nothing is 204; a struct, array or number is JSON; `string` is text/plain; `[] u8` is octet-stream; `Response` is as is. A second `Http_Error` return replaces the first when `kind != .NONE`. Builders: `json`, `created`, `text`, `html`, `bytes_response`, `redirect`, `status_only`, `with_header`, `file_response`, `stream_response`. A POST that returns a value is a 200; say `created(value)` for 201.
- **Errors** (`bad_request`, `not_found`, `http_error(status, detail)`, and extractor failures, 404, 405, 408, 413, 414, 431) are RFC 9457 `application/problem+json`. `app.on_error` replaces the format.
- **Routing.** Segment trie: static beats `{param}` beats `{rest...}`, with backtracking. Wrong-method paths get 405 with `Allow`; `HEAD` runs `GET`; `OPTIONS` answers 204. `use(*app, mw)` adds middleware `(request, next) -> Response`; `group(*app, "/admin", mw)` adds a prefix and middleware, and nests.
- **Connections.** One fiber per connection, handlers in order. Temporary storage resets after each response. A crashing handler isn't caught; return `Http_Error`.
- **Memory.** An idle keep-alive connection gives its read buffer back to a per-scheduler pool and waits with `Io.wait_readable`; a connection that has served a request also releases the stack pages its handler touched. Cost is about 22 KiB per idle connection (`bench/idle_bench.jai`). Don't hold pointers into the read buffer across a wait.
- **Serving.** `serve(*app)` runs `config.threads` schedulers, each with its own `SO_REUSEPORT` listener. It returns a `Serve_Error` (`.INVALID_ROUTES`, `.INVALID_TLS`, `.CANNOT_LISTEN`) or kind `.NONE` after a clean stop; nothing is printed. SIGINT/SIGTERM or `shutdown(*app)` close the listener, finish in-flight requests with `Connection: close` and cut the rest after `shutdown_timeout`. `start(*app)` inside `Io.io_run` returns a `Server` for tests.
- **HTTPS.** Set `app.config.tls = *config` (a `Tls_Config` that went through `tls_config_init` and outlives the server). ALPN none or `http/1.1` is served as HTTP/1.1; `h2` is served as HTTP/2 when `tls.alpn` lists it (`string.["h2", "http/1.1"]`); anything else drops the connection. About 34 KiB per connection.
- **HTTP/2.** Same handlers, middleware and streaming responses as HTTP/1.1; a handler cannot tell which protocol carried the request. To turn it off set `config.http2 = false`: `h2` is dropped from the ALPN list the server accepts (it uses a copy of `tls`), and clients fall back to HTTP/1.1; `h2_max_streams` (100), `h2_window` (256 KiB per stream) and `h2_connection_window` (1 MiB) set the advertised limits. Violations get GOAWAY or RST_STREAM with the RFC 9113 code. Not supported: cleartext h2c, server push, and priorities (parsed, ignored).

### Request bodies

The server doesn't read a body before the handler runs. Take `Json(T)`/`Form(T)` for a whole body, or `*Body` (`read`, `read_all`, `read_json`, `content_length`) to stream:

```jai
upload :: (name: Path("name", string), body: *Body) -> Http_Error {
    file, _ := Io.open_file(name.value, .WRITE);
    buffer: [16 * 1024] u8;
    while true {
        count, error := read(body, buffer);
        if error.kind != .NONE return error;      // 413, or 408 when the client stalls
        if count == 0 break;
        Io.write(*file, .{ count, buffer.data });
    }
    return .{};
}

post(*app, "/files/{name}", upload, max_body = 1 << 30);
```

- A route reads at most `max_body` bytes (0 means `config.max_body_size`, -1 no limit). An oversize `Content-Length` is a 413 before anything is read; chunked bodies are counted as they arrive. The parser itself has no body limit.
- After the handler returns, an unread body up to 64 KiB is dropped so the connection can continue; larger closes it.
- `Expect: 100-continue` is answered when the handler first asks for body bytes, so a handler that refuses without reading never triggers the upload.

### Temporary storage

`Query`, `Params`, `Form`, `Json` and `read_json` build their `T` in temporary storage, reset after each response. A handler that keeps a string or array must copy it. That's why these extractors keep the value in a member called `temporary` instead of `using` it: `store.users[0] = body.temporary` looks wrong in review. `Path`, `Rest` and `Header` keep `value`; it too lives until the response is written.

### Streaming responses

- `file_response(path, content_type = "", status = 200)` sends a file in 64 KiB pieces. The type comes from the extension. Missing file is 404. No range or caching support.
- `stream_response(produce, data, content_type, status = 200)` calls `produce(stream, data)` after the head. The producer calls `send_chunk` as data appears (false means the client left) and may wait between calls. Server-sent events are a producer writing `data: ...\n\n`.

Headers, status and `data` must stay valid until the producer returns.

## Testing

`test_client(*app)` returns the ordinary [`Client`](http-client.md) wired to the app through an in-memory pipe. Parsing, framing, keep-alive, limits, middleware and error mapping are production code; only the socket differs. Client and server fibers share one thread, so no test needs a sleep.

```jai
creates_and_reads :: () {
    store: Store;
    app: App;
    notes_routes(*app, *store);
    api := test_client(*app);

    created := expect_json(post(api, "/notes", New_Note.{ title = "milk" }), Note, status = 201);
    expect_equal(created.title, "milk");
    expect_problem(post(api, "/notes", New_Note.{ title = "" }), 422, "title must not be empty");
}

main :: () { run_tests(.[ .{ "creates a note and reads it back", creates_and_reads } ]); }
```

- Outside `Io.io_run`, `send` runs a scheduler per call, so temporary strings a handler recorded are gone afterwards; `copy_string` them.
- Relative URLs go to `base_url` or `http://localhost`. Redirects aren't followed unless `Test_Options.follow_redirects`. Default timeout 10 s.
- A failed call returns `status` 0 with `reason` set. Expectations (`expect_status`, `expect_json`, `expect_problem`, `expect_header`, `expect_text`, `expect_redirect`, `expect_equal`, ...) record a failure with the request and body and continue; `run_tests` exits 1 on any failure.
- `.{ over_tcp = true }` serves on a free loopback port with the real socket client.

## How to change it

- New extractor: a struct in `extract.jai`, a case in `classify` and `generate_prelude`/`generate_call`, an `extract_*` function, and a test in `tests/server-extractors.jai`. One that decodes a struct names its member `temporary`.
- New return type: a `respond` overload in `extract.jai`.
- Something a handler may read from the request: a field of `Request` in `message.jai` or an accessor. Internals go in the private `Request_Frame`.
- Body reading is all in `body.jai` (`next_piece` is the only place that parses body events). Streamed bodies: `stream.jai`.
- Compile-time errors go through `report_*` in `extract.jai` ("route `GET /x`: what is wrong"); add a case to `crates/jaic-cli/tests/http_routes.rs`.
- New expectation: in `expect.jai` before `#scope_module`; count the check, return early on success, report with `fail_response`, and add a case to `failure_messages` in `tests/testing.jai`.
- Another transport: fill `Client.transport.open` with a `Link` (`tcp_link`, `pipe_link` in `link.jai`).
- Gotchas: everything after `#scope_module` in `server.jai`, `router.jai`, `url.jai` and the second half of `extract.jai` is private. No UFCS: `with_header(response, ...)`. A handler that blocks the thread blocks the whole test; use `Io.sleep`. Routes must not change while clients use the app.

## Configuration

`app.config` (`Server_Config`), in milliseconds; 0 or less means no limit (for `shutdown_timeout`, cut at once).

| Field | Default |
| --- | --- |
| `address` | `":8080"` (port 0 picks a free one) |
| `threads` | 0 (one per processor) |
| `header_timeout` / `body_timeout` | 10000 / 30000 |
| `idle_timeout` / `write_timeout` | 60000 / 30000 |
| `shutdown_timeout` | 10000 |
| `max_body_size` | 1 MiB |
| `max_head_size`, `max_headers`, `max_line_size` | 64 KiB, 100, 8 KiB |
| `max_requests_per_connection` | 10000 |
| `read_buffer_size` | 16 KiB |
| `server_name`, `log_requests`, `handle_signals`, `tls` | `"jaic"`, false, true, null |

Also `app.state`, `app.on_error`, `app.json_options`.

Tests: `server-routing`, `server-extractors`, `server-responses`, `testing`, `server-bodies`, `server-connections`, `https` and `client` in `stdlib/Extensions/Http/tests/`. Run parser benchmarks with `jacket bench --path stdlib/Extensions/Http`; the network experiments remain standalone in `bench/`. The server load needs a peer that persists across samples, and the client/idle experiments report RSS or stack residency. The supplied runner provides neither per-run fixture teardown nor those memory metrics. `tools/stdlib/http_conformance.py` also builds `tools/stdlib/http_garden/server.jai`, an echo server for an http-garden image.

## Compared with the community servers

`benchmarks/compare/http/` (`PYTHONPATH=tools python3 tools/bench/http_compare.py`) serves the same small JSON API (`GET /items/{id}` over 1,000 items) from Jacket and from the hyperserve and simplehttp libraries, with 1, 64 and 1,024 keep-alive loopback connections (Apple M5, `-o` builds, in the bench sandbox). Read the ratios, not the absolute numbers.

| Server | Connections | Requests/s | p99 |
| --- | --- | --- | --- |
| Jacket | 1 / 64 / 1024 | 51,945 / 175,958 / 149,674 | 35 us / 506 us / 7.81 ms |
| hyperserve | 1 / 64 / 1024 | 32,794 / 94,869 / 87,134 | 48 us / 887 us / 14.1 ms |
| simplehttp | 1 / 64 / 1024 | 16,360 / 1,808 / 30,659 | 73 us / 1.76 ms / 97.3 ms |

Idle connections cost about 22 KiB each in Jacket and hyperserve, 68 KiB in simplehttp. Jacket also has streaming bodies, per-route limits, chunked request bodies, streaming responses, slow-client timeouts, HTTPS and an in-memory test client, which neither community server has.

## Route callbacks with independent state

`register_handler(app, method, path, callback, data)` registers a callback taking
`(*Request, *void) -> Response`, without extractors. It uses the same router and app
middleware as typed routes. The route stores a borrowed `data` pointer; the caller
keeps that state alive for the registration lifetime and every in-flight handler.
This lets multiple generated services carry independent state without replacing
`App.state`.

```jai,ignore
register_handler(*app, .POST, "/example.Service/Method", dispatch, *service);
```

There is no unregister or replacement operation. Register before serving and keep
the app's routing table fixed while handlers run. A duplicate route records a setup
error and leaves the first route intact; invalid paths likewise record setup errors.
Paths are copied at registration; caller path storage may be released immediately.
A null callback is diagnosed before path validation. Errors accumulate in registration order, and server startup reports setup errors
before opening a listener. Freeing the app releases route records, not caller state;
first stop the server and wait for its handlers before releasing either.

### Releasing a server handle

`start` returns an owned heap handle. `stop` begins shutdown; `wait` waits for
connections and internal fibers to release their server references while keeping
the handle available for diagnostics. `close_server(*server)` performs stop/wait,
frees that handle and clears the pointer. Repeating close on the cleared pointer
is safe; other aliases must not be used afterwards. Keep caller-owned App, route
state and TLS configurations alive until close returns. Cooperative handlers must
observe cancellation; close cannot preempt CPU work that does not yield.

The completion group includes accept, drain/watch fibers and each HTTP/2
connection through its final reference. Changing this accounting requires lifetime
tests where callbacks finish after shutdown begins, not simply waiting for the
listener to close. The ordinary `serve` convenience owns and closes its handles.

### Routed protocol upgrades

`upgrade_response(protocol, handler, data)` returns a 101 response with Upgrade and
Connection headers. Example: `return upgrade_response("example", run_protocol, state);`
The `Upgrade_Handler` receives `(Link, pending_bytes, data)` on the connection fiber
only after the complete response head has been written. Middleware still sees and
may replace the response. The request must offer that protocol with HTTP/1.1 upgrade
headers and no body; invalid handoffs return 400. HTTP/2 returns 400 for this response.
Cleartext h2c remains unsupported.

The server owns the link and closes it after the callback returns; do not close it
again in the callback. Pending bytes are borrowed from the read buffer and must be
consumed before reading the link. The active TLS link is handed out for HTTPS.
Keep callback data alive until it returns, including across fiber waits. Shutdown
accounts for this fiber as an active connection. Implement new protocols through
this callback; `upgrade_connection` in `server.jai` is the shared ownership boundary,
not a protocol dispatcher. See [WebSocket](websocket.md) for one implementation.
