# Http server and test client (`Extensions/Http`)

Shared rules: [README](README.md#conventions-of-io-tls-http-and-json).

## What it is

A routing HTTP/1.1 server on fibers: register plain functions as handlers, declare what they need as parameter types, and the module checks them against the route at compile time and generates the request-reading code. Sits on the [codec](http-codec.md) and [Io](io-engine.md); the [client](http-client.md) is the other half. Not available on wasm. A built-in test client drives an `App` in memory ([Testing](#testing)).

```jai
#import "Extensions/Http";

User :: struct { id: int; name: string; }
New_User :: struct { name: string; }

get_user :: (id: Path("id", int), store: *State(Store)) -> User, Http_Error {
    for store.users if it.id == id.value return it, .{};
    return .{}, not_found("No user with id %", id.value);
}

create_user :: (body: Json(New_User), store: *State(Store)) -> User {   // POST: 201 with the JSON
    user := User.{ store.users.count + 1, copy_string(body.name) };
    array_add(*store.users, user);
    return user;
}

main :: () {
    store: Store;
    app: App;
    app.state = *store;
    get(*app, "/users/{id}", get_user);
    post(*app, "/users", create_user);
    serve(*app);       // :8080, one scheduler per core, Ctrl-C drains and exits
}
```

Runnable: `examples/http/` (`crud.jai`, `middleware.jai`, `client.jai`, `crud_test.jai`).

## How it works

**Registration.** `get post put patch delete head options (*app or group, template, handler)`. The template (`/users/{id}`, `/files/{path...}` last) is parsed at compile time. `register_route` (`app.jai`) inspects the handler's `Type_Info_Procedure`, calls `generate_prelude`/`generate_call` (`extract.jai`) and `#insert`s a thunk `(*Request) -> Response` into the router. Route mistakes are reported on the `get(...)` call in route terms (a `Path` name not in the template, two body extractors, unconvertible types, bad template); two routes matching the same requests are reported at startup (`serve` returns false).

**Extractors** (handler parameters; `value`, with `using value` for structs):

| Parameter | Reads | Failure |
| --- | --- | --- |
| `Path("id", T)`, `Params(T)`, `Rest("path")` | route parameters | 400 naming the parameter |
| `Query(T)`, `Form(T)` | query / urlencoded body via [`from_query`](json.md#query-and-form-text) | 400 / 415, 400, 422 |
| `Json(T)` | JSON body, `application/json` required | 415, 400, 422 with the JSON path |
| `Header("n", T)`, `Header_Optional("n", T)` | a header | 400 when missing (not `_Optional`) |
| `Bytes`, `Text`, `*Request`, `*State(T)` | raw body, whole request, `app.state` pointer | |

**Returns.** Nothing: 204. Struct/array/number: JSON, 200 (201 for POST). `string`: text/plain. `[]u8`: octet-stream. `Response`: as is (`json`, `text`, `html`, `bytes_response`, `redirect`, `no_content`, `status_only`, `with_header`). A second `Http_Error` return replaces the first when `kind != .NONE`. Errors (`bad_request`, `not_found`, ..., `http_error(status, detail)`) are RFC 9457 `application/problem+json` with `location`/`field`; extractor failures, 404, 405, 408, 413, 414, 431 use the same format. `app.on_error` replaces the format.

**Routing.** Segment trie: static beats `{param}` beats `{rest...}`, with backtracking. Other-method paths get 405 with `Allow`; `HEAD` runs `GET` without body; `OPTIONS` answers 204 with `Allow`; a trailing `/` is ignored; TRACE/CONNECT/unknown get 501. `use(*app, mw)` adds middleware `(request, next) -> Response` (wraps unrouted requests too); `group(*app, "/admin", mw)` adds a prefix (may contain parameters, checked against the whole route) and middleware, and nests.

**Connections.** One fiber per connection, handler calls in order; responses are written when nothing is left to read, so pipelining costs few writes. Bodies over 64 KiB are collected in a growing buffer, smaller ones are views into the read buffer. Temporary storage resets after each response, so copy strings that outlive a request. A crashing handler is not caught; return `Http_Error`.

**Memory.** An idle keep-alive connection holds no read buffer: with nothing pending the loop returns its buffer to a per-scheduler pool (64 kept) and waits with `Io.wait_readable`. A connection whose last two requests began less than 10 ms apart (`HOT_MS`) keeps its buffer. Response buffer and parser storage are trimmed when idle or oversized (`RETAIN_OUTPUT`, `RETAIN_HEAD`). About 16 KB per idle connection remains, mostly fiber stack. Anything held across a wait must not point into the read buffer.

**Serving.** `serve(*app)` runs `config.threads` schedulers (default one per core) each with its own listener (`SO_REUSEPORT`). SIGINT/SIGTERM (`handle_signals`) or `shutdown(*app)` close the listener and idle connections, finish requests in progress with `Connection: close`, and cut the rest after `shutdown_timeout`. `start(*app)` inside `Io.io_run` returns a `Server` (`server_url`, `stop`, `wait`) for tests.

**HTTPS.** `app.config.tls = *config` (a `Tls_Config` with chain and key that went through `tls_config_init` and outlives the server). Each connection handshakes on its own fiber under `header_timeout`, then the same loop runs on the TLS reader/writer. A client dropping without close_notify is an ordinary close; only failed handshakes are logged (with `log_requests`). ALPN: none or `http/1.1` runs HTTP/1.1, anything else drops the connection (`choose_protocol` in `server.jai`; `upgrade_connection` is the stub for HTTP/2). Costs about 34 KiB per connection and roughly 6 ms of CPU per handshake.

## Testing

`test_client(*app)` returns the ordinary [`Client`](http-client.md) wired to the app through an in-memory pipe (`Io.make_pipe`, `adopt` in `server.jai`). Parsing, framing, keep-alive, limits, middleware, routing and error mapping are production code; only the socket differs (a direct handler call would skip the connection loop, where most bugs are). Client and server fibers share one scheduler thread, so order depends only on the code and no test needs a sleep.

```jai
creates_and_reads :: () {
    store: Store;
    app: App;
    notes_routes(*app, *store);
    api := test_client(*app);

    created := expect_json(post(api, "/notes", New_Note.{ title = "milk" }), Note, status = 201);
    expect_equal(created.title, "milk");
    expect_equal(store.notes.count, 1);          // the store is the test's own variable
    expect_problem(post(api, "/notes", New_Note.{ title = "" }), 422, "title must not be empty");
}

main :: () { run_tests(.[ .{ "creates a note and reads it back", creates_and_reads } ]); }
```

- Outside `Io.io_run`, `send` runs a scheduler for that call, so test bodies are plain code (this works for any `Client`). Inside a scheduler, connections are kept and handlers can use `Io.sleep`, channels and other apps. Outside, temporary-storage strings a handler recorded are gone after the call: `copy_string` them.
- Relative URLs go to `base_url` or `http://localhost`. Redirects are not followed (`max_redirects` 0) unless `Test_Options.follow_redirects`. Default timeout 10 s.
- A failed call returns a response with `status` 0 and `reason` set, so `expect_*` prints why. Body-reading expectations buffer the body once so several can check one response and failures always show it.
- Expectations (each returns whether it held, counts a check, records a failure and continues): `expect_status`, `expect_success`, `expect_json(response, T, status)`, `expect_problem(response, status, detail_part)`, `expect_header`, `expect_no_header`, `expect_text`, `expect_body_contains`, `expect_redirect`, `expect`, `expect_equal`. Failures print file:line, the request, status, key headers and the body (400 bytes). `run_tests` exits 1 on any failure.
- `test_client(*app, .{ over_tcp = true })` serves on a free loopback port and uses the real socket client; sets `address` to `127.0.0.1:0` and `handle_signals` off. Socket-level behavior (keep-alive, timeouts, shutdown, pipelining) is tested on raw sockets in `tests/server_connections.jai`.

## How to change it

- **New extractor:** struct in `extract.jai`, a case in `classify` and `generate_prelude`/`generate_call`, a run-time `extract_*` function, failure wording, a row above, a test in `tests/server_extractors.jai`.
- **New return type:** a `respond` overload in `extract.jai`.
- **Compile-time errors** go through `report_*` in `extract.jai` in the form "route `GET /x`: what is wrong (what the route has)"; add a case to `crates/jaic-cli/tests/http_routes.rs`.
- **New expectation:** in `expect.jai` before `#scope_module`: `test_state.checks += 1;`, return early on success, report with `fail_response(loc, headline, response, focus)`, `loc := #caller_location` last, use `checked_body(response)` for bodies; add a case to `failure_messages` in `tests/testing.jai` (it compares printed text).
- **Another transport:** fill `Client.transport.open` with a `Link` (`tcp_link`, `pipe_link` in `link.jai`).
- Test the test tools with `begin_failure_capture()`/`end_failure_capture()`.
- Gotchas: everything after `#scope_module` in `server.jai`, `router.jai`, `url.jai` and the second half of `extract.jai` is private; put public procedures before it. Handlers are generated per route (compile time, not run time). No UFCS: `with_header(response, ...)`. A handler that blocks the thread (`Thread` sleep, busy wait) blocks the whole test: use `Io.sleep`. Routes must not change while clients use the app. `head_started` in `serve_connection` is set when a request's first byte arrives, including bytes that came with the previous one.

## Configuration

`app.config` (`Server_Config`), milliseconds; for `header_timeout`, `body_timeout`, `idle_timeout`, `write_timeout`, 0 or less means no limit; `shutdown_timeout` 0 or less means cut at once.

| Field | Default | Meaning |
| --- | --- | --- |
| `address` | `":8080"` | port 0 picks a free one |
| `threads` | 0 | 0 or less: one per processor |
| `header_timeout` / `body_timeout` | 10000 / 30000 | 408; header also covers the TLS handshake |
| `idle_timeout` / `write_timeout` | 60000 / 30000 | keep-alive wait / one response write |
| `shutdown_timeout` | 10000 | wait before cutting connections |
| `max_body_size` | 1 MiB | 413 |
| `max_head_size`, `max_headers`, `max_line_size` | 64 KiB, 100, 8 KiB | 431 / 414 |
| `max_requests_per_connection` | 10000 | then `Connection: close` |
| `read_buffer_size` | 16 KiB | per connection with bytes to read |
| `server_name`, `log_requests`, `handle_signals`, `tls` | `"jaic"`, false, true, null | |

Also `app.state`, `app.on_error`, `app.json_options`. `Test_Options`: `over_tcp`, `follow_redirects`, `timeout` (10000).

## Dependencies and tests

[Io](io-engine.md), [Tls](tls.md), [Json](json.md), the [codec](http-codec.md), `Atomics`. Tests in `stdlib/Extensions/Http/tests/`: `server_routing`, `server_extractors`, `server_responses`, `testing` (the test client and exact failure text) on the in-memory client; `server_connections` (raw sockets, memory rules), `https`, `client`. Benchmarks in `bench/` (`server_bench.jai`, `idle_bench.jai`, build with `-O2`). `python3 tools/http_conformance.py` also builds `tools/http_garden/server.jai`, an echo server to put in an http-garden image.
