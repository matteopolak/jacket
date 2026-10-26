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

create_user :: (body: Json(New_User), store: *State(Store)) -> Response {
    // `temporary`: the decoded value lives in temporary storage until the response is written.
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

Runnable: `examples/http/` (`crud.jai`, `middleware.jai`, `client.jai`, `crud_test.jai`).

## How it works

**Registration.** `get post put patch delete head options (*app or group, template, handler, max_body = 0)`. The template (`/users/{id}`, `/files/{path...}` last) is parsed at compile time. `register_route` (`app.jai`) inspects the handler's `Type_Info_Procedure`, calls `generate_prelude`/`generate_call` (`extract.jai`) and `#insert`s a thunk `(*Request) -> Response` into the router. Route mistakes are reported on the `get(...)` call in route terms (a `Path` name not in the template, two body extractors, unconvertible types, bad template); two routes matching the same requests are reported at startup (`serve` returns `.INVALID_ROUTES`).

**Extractors** (handler parameters). Those that parse one scalar keep it in `value`; those that decode a whole struct keep it in `temporary` (see [Temporary storage](#temporary-storage)).

| Parameter | Reads | Failure |
| --- | --- | --- |
| `Path("id", T)` (`.value`), `Rest("path")` (`.value`) | route parameters | 400 naming the parameter |
| `Params(T)` (`.temporary`) | several route parameters into a struct | 400 |
| `Query(T)` (`.temporary`) | the query via [`from_query`](json.md#query-and-form-text) | 400 |
| `Form(T)` (`.temporary`) | the whole urlencoded body, within the route's limit | 415, 400, 413 |
| `Json(T)` (`.temporary`) | the whole JSON body, within the route's limit; `application/json` required | 415, 400, 413, 422 with the JSON path |
| `*Body` | the body as a stream: `read`, `read_all`, `read_json` | what the reads return |
| `Header("n", T)`, `Header_Optional("n", T)` (`.value`, `.present`) | a header | 400 when missing (not `_Optional`) |
| `*Request`, `*State(T)` | the request's line, headers and peer; the `app.state` pointer | |

`*Request` has only what a handler needs: `method`, `method_text`, `target`, `path`, `query`, `version`, `headers`, `peer`, and the accessors `header(request, name)`, `query_param(request, name)`, `peer_text(request)` and (for middleware) `app_state(request, T)`. Everything else about a request (the route's parameters, the middleware position, the body) is private to the module: `Request` is the first member of a private `Request_Frame` (`message.jai`) that the server allocates, so a `*Request` casts to it inside the module and nobody else can reach the rest. There is no `request.body` and no `body_text`; ask for the body with `*Body`, `Json(T)` or `Form(T)`.

**Returns.** Nothing: 204. Struct/array/number: JSON, 200. `string`: text/plain. `[] u8`: octet-stream. `Response`: as is. A second `Http_Error` return replaces the first when `kind != .NONE`. Builders, one for each job:

| Builder | Response |
| --- | --- |
| `json(value, status = 200)` | JSON |
| `created(value, location = "")` | 201 with the JSON of `value` and, when given, a `Location` header |
| `text(s)`, `html(s)` | `text/plain` and `text/html`, UTF-8 |
| `bytes_response(bytes, content_type, status = 200)` | any `[] u8` or `string` |
| `redirect(location, status = 302)` | a redirect |
| `status_only(status)` | no body (`status_only(204)`) |
| `with_header(response, name, value)` | the same response with one more header; also on an `Http_Error` |
| `file_response(path, content_type = "", status = 200)` | a file, sent in pieces ([Streaming responses](#streaming-responses)) |
| `stream_response(produce, data, content_type, status = 200)` | a body the handler pushes ([Streaming responses](#streaming-responses)) |

A POST that returns a value is a 200 like any other: say `created(value)` for a 201. Errors (`bad_request`, `not_found`, ..., `http_error(status, detail)`) are RFC 9457 `application/problem+json` with `location`/`field`; extractor failures, 404, 405, 408, 413, 414, 431 use the same format. `app.on_error` replaces the format.

**Routing.** Segment trie: static beats `{param}` beats `{rest...}`, with backtracking. Other-method paths get 405 with `Allow`; `HEAD` runs `GET` without body; `OPTIONS` answers 204 with `Allow`; a trailing `/` is ignored; TRACE/CONNECT/unknown get 501. `use(*app, mw)` adds middleware `(request, next) -> Response` (wraps unrouted requests too); `group(*app, "/admin", mw)` adds a prefix (may contain parameters, checked against the whole route) and middleware, and nests. Middleware change a response with `response = with_header(response, ...)`.

**Connections.** One fiber per connection, handler calls in order; responses are written when nothing is left to read, so pipelining costs few writes. Temporary storage resets after each response. A crashing handler is not caught; return `Http_Error`.

**Memory.** An idle keep-alive connection holds no read buffer: with nothing pending the loop returns its buffer to a per-scheduler pool (64 kept) and waits with `Io.wait_readable`. A connection whose last two requests began less than 10 ms apart (`HOT_MS`) keeps its buffer. Response buffer and parser storage are trimmed when idle or oversized (`RETAIN_OUTPUT`, `RETAIN_HEAD`). What remains is mostly the fiber's stack, so a connection that has served a request also gives back the stack pages its handler reached into before it waits (`Io.release_unused_stack`, once per idle period; connections that are hot do not call it, and neither does a new connection). Measured on macOS arm64 (16 KiB pages, `-o` build), 1,024 connections that each made a request and stayed open, resident set of the server process before and after: **about 22 KiB per connection on both a plain-text route and a route that extracts a path parameter and answers with JSON** (without the release the JSON route cost about 38 KiB, because its handler's frames cross into a second 16 KiB stack page). What is left is the top stack page and the connection's own state. `bench/idle_bench.jai` measures it on Linux (`/proc`) and macOS (`proc_pidinfo`): `idle_bench 1024 http` for the plain route, `idle_bench 1024 http json` for the JSON one. On Linux the release is `madvise(MADV_DONTNEED)`; it was not measured there. Anything held across a wait must not point into the read buffer.

**Serving.** `serve(*app)` runs `config.threads` schedulers (default one per core) each with its own listener (`SO_REUSEPORT`). It returns a `Serve_Error` when the server could not start and `.{}` (kind `.NONE`) after a clean stop:

| `kind` | Meaning |
| --- | --- |
| `.INVALID_ROUTES` | two routes match the same requests; `message` names them |
| `.INVALID_TLS` | `config.tls` has no certificate and key, or `Tls.tls_config_init` was not called on it |
| `.CANNOT_LISTEN` | the address is taken, malformed or not allowed; `io_error` has the reason and `message` the address |

`error_text(error)` is `message`. Nothing is printed on failure, so the caller decides (`print`, `exit(1)`). There is one way to start a server: set `app.config.address` and call `serve(*app)`. SIGINT/SIGTERM (`handle_signals`) or `shutdown(*app)` close the listener and idle connections, finish requests in progress with `Connection: close`, and cut the rest after `shutdown_timeout`. `start(*app)` inside `Io.io_run` returns a `Server` (`server_url`, `stop`, `wait`) and the same `Serve_Error`, for tests.

**HTTPS.** `app.config.tls = *config` (a `Tls_Config` with chain and key that went through `tls_config_init` and outlives the server). Each connection handshakes on its own fiber under `header_timeout`, then the same loop runs on the TLS reader/writer. A client dropping without close_notify is an ordinary close; only failed handshakes are logged (with `log_requests`). ALPN: none or `http/1.1` runs HTTP/1.1, anything else drops the connection (`choose_protocol` in `server.jai`; `upgrade_connection` is the stub for HTTP/2). Costs about 34 KiB per connection and roughly 6 ms of CPU per handshake.

## Request bodies

The server does not read a body before the handler runs. A handler that wants it takes `Json(T)` or `Form(T)` (the whole body, decoded) or a `*Body` (`body.jai`):

| Procedure | Does |
| --- | --- |
| `read(body, buffer) -> int, Http_Error` | copies the next bytes into `buffer`; 0 means the end |
| `read_all(body) -> [] u8, Http_Error` | the rest of the body; a view of the connection's buffer when it arrived in one piece |
| `read_json(body, T) -> T, Http_Error` | the rest as JSON, like `Json(T)` |
| `body.content_length` | the declared length, -1 for a chunked body |

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

post(*app, "/files/{name}", upload, max_body = 1 << 30);     // gigabytes are fine: nothing is buffered
```

**Limits.** A route reads at most `max_body` bytes: the last argument of `get`, `post`... (0, the default, is `config.max_body_size`; -1 is no limit). It applies to `Json`, `Form`, `read` and `read_all`: a declared `Content-Length` over it is refused with 413 before anything is read, and a chunked body is counted as it arrives. The first error is sticky, so a handler that ignores it keeps getting it, and the connection closes after the response. The parser itself has no body limit (`config.max_body_size` only supplies the default), so the limit is always the route's.

**What the handler leaves unread.** After the handler returns, a small unread body (up to 64 KiB, `UNREAD_LIMIT`) is read and dropped so the connection can carry the next request; a larger one, or one the client has not started to send, closes the connection after the response.

**`Expect: 100-continue`** is answered when the handler first asks for body bytes, not before. A handler that refuses without reading (`return forbidden(...)`) never sends `100 Continue`, so the client never uploads, and the connection closes after the answer.

## Temporary storage

`Query(T)`, `Params(T)`, `Form(T)`, `Json(T)` and `read_json` build their `T` in the temporary allocator, which the server resets after each response. Strings and arrays inside it are gone once the response is written, so a handler that keeps one (in a store, a channel, a closure) must `copy_string` it or copy the array.

The API makes that visible instead of relying on the reader knowing it: these extractors do not `using` their value; it sits in a member called `temporary`, so the handler reads `body.temporary.name` and the word is in every use. A `Json(T)` that is stored without a copy looks wrong in review (`store.users[0] = body.temporary` is plainly a temporary going into a store), where `body.name` looked like any other field. `Path`, `Rest` and `Header` keep `value`: their strings point into the request itself, are short, and are mostly parsed into integers and enums; the same rule (valid until the response is written) applies to them and is stated on each type. Everything a `Response` points at has the same lifetime, which is why the builders use temporary storage too.

## Streaming responses

Both go out after the head, from the connection's fiber, so a slow client costs one fiber and no buffer.

- `file_response(path, content_type = "", status = 200)` sends a file in 64 KiB pieces with `Content-Length` set from its size. The type comes from the extension when not given (`application/octet-stream` when unknown). A missing file is a 404 and any other failure a 500, as the handler's response; there is no range or caching support (add `ETag` and friends with `with_header`). A file that shrinks while it is sent cuts the connection.
- `stream_response(produce, data, content_type, status = 200)` calls `produce(stream, data)` once the head is out. The producer calls `send_chunk(stream, bytes_or_text)` as it has data (each call is written at once, as one chunk) and may wait between calls (`Io.sleep`, channels). `send_chunk` returns false when the client has gone or the write timed out; the producer should then return. On HTTP/1.1 the body is chunked and the connection stays usable; for an HTTP/1.0 client it ends with the connection. HEAD requests and 204/304 get the head only and the producer is not called. Server-sent events are a producer that writes `data: ...\n\n`.

The headers, status and `data` pointer must stay valid until the producer returns: use string literals or temporary storage (which only resets after the whole response).

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
- `test_client(*app, .{ over_tcp = true })` serves on a free loopback port and uses the real socket client; sets `address` to `127.0.0.1:0` and `handle_signals` off. Socket-level behavior (keep-alive, timeouts, shutdown, pipelining) is tested on raw sockets in `tests/server-connections.jai`.

## How to change it

- **New extractor:** struct in `extract.jai`, a case in `classify` and `generate_prelude`/`generate_call`, a run-time `extract_*` function, failure wording, a row above, a test in `tests/server-extractors.jai`. One that decodes a struct names its member `temporary`.
- **New return type:** a `respond` overload in `extract.jai`.
- **Something a handler may read from the request:** a field of `Request` in `message.jai` (set in `serve_connection`) or an accessor procedure. Anything the server needs for itself goes in `Request_Frame`, not in `Request`.
- **Body reading:** `body.jai`. `next_piece` is the only place that parses body events and reads the socket; `take_piece` adds the limit; `finish_body` is the connection loop's cleanup. A view into the read buffer is only valid until the next `next_piece`, which is why `read_all` records the piece it holds (`hold`) and `fill` copies it out before reading.
- **Streamed bodies:** `stream.jai`; `send_source` is called by the connection loop right after the head.
- **Compile-time errors** go through `report_*` in `extract.jai` in the form "route `GET /x`: what is wrong (what the route has)"; add a case to `crates/jaic-cli/tests/http_routes.rs`.
- **New expectation:** in `expect.jai` before `#scope_module`: `test_state.checks += 1;`, return early on success, report with `fail_response(loc, headline, response, focus)`, `loc := #caller_location` last, use `checked_body(response)` for bodies; add a case to `failure_messages` in `tests/testing.jai` (it compares printed text).
- **Another transport:** fill `Client.transport.open` with a `Link` (`tcp_link`, `pipe_link` in `link.jai`).
- Test the test tools with `begin_failure_capture()`/`end_failure_capture()`.
- Gotchas: everything after `#scope_module` in `server.jai`, `router.jai`, `url.jai` and the second half of `extract.jai` is private; put public procedures before it. Handlers are generated per route (compile time, not run time). No UFCS: `with_header(response, ...)`. Struct members cannot be private, which is why `Request` and `Body` keep their internals in private frames that hold them as the first member. A handler that blocks the thread (`Thread` sleep, busy wait) blocks the whole test: use `Io.sleep`. Routes must not change while clients use the app. `head_started` in `serve_connection` is set when a request's first byte arrives, including bytes that came with the previous one.

## Configuration

`app.config` (`Server_Config`), milliseconds; for `header_timeout`, `body_timeout`, `idle_timeout`, `write_timeout`, 0 or less means no limit; `shutdown_timeout` 0 or less means cut at once.

| Field | Default | Meaning |
| --- | --- | --- |
| `address` | `":8080"` | port 0 picks a free one |
| `threads` | 0 | 0 or less: one per processor |
| `header_timeout` / `body_timeout` | 10000 / 30000 | 408; header also covers the TLS handshake |
| `idle_timeout` / `write_timeout` | 60000 / 30000 | keep-alive wait / one response write |
| `shutdown_timeout` | 10000 | wait before cutting connections |
| `max_body_size` | 1 MiB | the default route `max_body`; 413 |
| `max_head_size`, `max_headers`, `max_line_size` | 64 KiB, 100, 8 KiB | 431 / 414 |
| `max_requests_per_connection` | 10000 | then `Connection: close` |
| `read_buffer_size` | 16 KiB | per connection with bytes to read |
| `server_name`, `log_requests`, `handle_signals`, `tls` | `"jaic"`, false, true, null | |

Also `app.state`, `app.on_error`, `app.json_options` (limits for `Json` and `read_json`). Per route: `max_body` (see [Request bodies](#request-bodies)). `Test_Options`: `over_tcp`, `follow_redirects`, `timeout` (10000).

## Dependencies and tests

[Io](io-engine.md), [Tls](tls.md), [Json](json.md), the [codec](http-codec.md), `Atomics`. Tests in `stdlib/Extensions/Http/tests/`: `server-routing`, `server-extractors`, `server-responses`, `testing` (the test client and exact failure text) on the in-memory client; `server-bodies` (streamed reads, route limits, unread bodies, `Expect`, streamed responses, files), `server-connections` (raw sockets, memory rules), `https`, `client`. Benchmarks in `bench/` (`server_bench.jai`, `idle_bench.jai`, build with `-O2`). `PYTHONPATH=tools python3 tools/stdlib/http_conformance.py` also builds `tools/stdlib/http_garden/server.jai`, an echo server to put in an http-garden image.

## Compared with the community servers

Jacket's server next to [`rluba--hyperserve`](https://github.com/rluba/hyperserve), [`smari--jai-simplehttp`](https://github.com/smari/jai-simplehttp) and [`farzher--Jai-HTTP-Server`](https://github.com/farzher/Jai-HTTP-Server), the HTTP servers in the upstream corpus. The cells come from reading each library's source and README; ⚠ has a note, and a `❌` means the source or README says so (hyperserve lists several of these as its own to-do items).

| Feature | Jacket | hyperserve | simplehttp | farzher |
| --- | --- | --- | --- | --- |
| Builds and runs on macOS | ✅ | ✅ | ✅ | ❌ [1] |
| Typed handler parameters (path, query, JSON, form) | ✅ extractors | ✅ typed `path` and `query` members [2] | ❌ | ❌ |
| Route patterns with path parameters | ✅ | ✅ | ⚠ [3] | ✅ |
| Middleware | ✅ | ⚠ authentication hooks only | ✅ [4] | ❌ |
| Streaming request body | ✅ `*Body` | ❌ [5] | ❌ | ❌ |
| Body size limit | ✅ per route | ❌ [5] | ⚠ one fixed 64 MiB constant | ❌ |
| Chunked request bodies | ✅ | ❌ [5] | ❌ | ❌ |
| Streaming and file responses | ✅ | ❌ | ❌ | ❌ |
| Timeouts for slow clients | ✅ header, body, idle | ⚠ [6] | ❌ | ❌ |
| HTTPS | ✅ [`Tls`](tls.md) | ❌ closes TLS clients | ❌ | ⚠ OpenSSL module [1] |
| WebSockets | ❌ | ❌ | ❌ [8] | ✅ |
| Test client without a socket | ✅ | ❌ | ❌ | ❌ |
| Errors as values (`Serve_Error`, `Http_Error`) | ✅ | ❌ | ⚠ `HTTPServerError` | ❌ |
| Multi-threaded | ✅ schedulers | ✅ request threads | ❌ [7] | ✅ epoll threads |

1. farzher has an `epoll` backend (Linux) and a Windows backend and no macOS one, so the benchmark program stops at `http_listen` (unknown identifier). It was not measured; its numbers on Linux are not in this table.
2. A handler takes a struct whose `path` and `query` members are filled and checked before the call.
3. Routes are registered by path with `http_register_url`; the benchmark handler reads the id from the path by hand.
4. Request and response middleware, switched off with `USE_MIDDLEWARE`.
5. The source lists "decode bodies", "limit body size", "stream the body" and "chunked" as to-do items, and only reads bodies for GET.
6. Fixed constants (30 s for header, body and send) that are not configurable.
7. `http_server_run_single_threaded` is the only way the library runs a server.
8. `http_register_websocket_url` has the same body as `http_register_url`: it adds a plain route, and nothing in the library answers an upgrade or reads frames (`USE_WEBSOCKETS` is declared and never used).

### How it was measured

`benchmarks/compare/http/`, run by `PYTHONPATH=tools python3 tools/bench/http_compare.py`. Every server serves the same small JSON API: `GET /items/{id}` answers a 1,000-item in-memory store with one JSON object (`servers/store.jai` holds the shared data; each `servers/<name>.jai` is the same handler written on that library). `loadgen.jai` opens 1, 64 and 1,024 keep-alive connections over loopback and requests random ids for 5 s after a 1 s warm-up, three rounds, the median by requests per second reported. Latency is per request, from write to the last byte of the response. Memory is the resident set of the server at rest and with 1,024 idle connections held open, divided by 1,024. Jacket and hyperserve run with 4 threads; the load generator uses 2. Failures are counted by kind: a connect that was refused or reset (`Connect errors`), a request on a connected socket that failed, mostly by the server resetting or closing it (`Request errors`), and anything that ran into the 5 s request timeout, connecting or waiting for the answer (`Timeouts`). The load generator also reports the first failure of each kind.

All servers and the load generator are compiled with `-o` and run inside the `Sandbox` of `tools/bench/sandbox.py` with loopback-only networking, because the community servers are unreviewed code.

Apple M5 (10 cores: 4 Super and 6 Efficiency), 16 GB, macOS 27.0 arm64. The machine was a working desktop: the load average when the run began was 4.97, and the load generator shares the cores with the server, so read the ratios, not the absolute numbers.

| Server | Connections | Requests/s | p50 | p99 | p99.9 | Connect errors | Request errors | Timeouts |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Jacket | 1 | 52,441 | 18 us | 36 us | 69 us | 0 | 0 | 0 |
| Jacket | 64 | 151,347 | 410 us | 674 us | 823 us | 0 | 0 | 0 |
| Jacket | 1024 | 137,675 | 6.99 ms | 12.5 ms | 22.9 ms | 0 | 0 | 0 |
| hyperserve | 1 | 33,977 | 27 us | 58 us | 115 us | 0 | 0 | 0 |
| hyperserve | 64 | 82,894 | 764 us | 1.17 ms | 1.46 ms | 0 | 0 | 0 |
| hyperserve | 1024 | 74,469 | 13.8 ms | 18.5 ms | 19.5 ms | 234 | 176 | 0 |
| simplehttp | 1 | 16,251 | 55 us | 107 us | 163 us | 0 | 0 | 0 |
| simplehttp | 64 | 0 | 0 ns | 0 ns | 0 ns | 15,553 | 1,245 | 0 |
| simplehttp | 1024 | 5,868 | 29 ms | 95 ms | 95.7 ms | 134,610 | 11,441 | 0 |
| farzher | n/a | does not build on macOS | | | | | | |

| Server | Resident at rest | With 1,024 idle connections | Per connection |
| --- | --- | --- | --- |
| Jacket | 2.5 MiB | 24.8 MiB | 22.3 KiB |
| hyperserve | 6.4 MiB | 29.1 MiB | 22.7 KiB |
| simplehttp | 1.9 MiB | 69.9 MiB | 67.9 KiB |
| farzher | n/a | n/a | n/a |

Jacket handles about 1.8 times the requests per second of hyperserve, with lower latency at every percentile, and both take about 22 KiB per idle connection. simplehttp is killed by `SIGPIPE` (signal 13) when 64 or more connections write to clients that have gone, which is why its error counts are high and its numbers are those of the runs that survived (its 64-connection row is the median of runs that did not finish).

hyperserve loses 400 to 800 connections at 1,024 and none to a timeout: every failure is a reset (`CONNECTION_RESET`), about a quarter while connecting and the rest on the first read of a connection that had been accepted. The listen backlog does not explain it. hyperserve listens with `SOMAXCONN` and Jacket asks for 1,024, but macOS caps every backlog at `kern.ipc.somaxconn` (128 here), so both queues hold the same number of connections, and Jacket with a single listener still lost none. What differs is how fast the queue drains: Jacket's accept fiber only accepts and spawns, while hyperserve accepts one connection per pass of its one event loop, which also handles finished requests, so the burst of 1,024 connects outruns it and the kernel resets what does not fit. Jacket's four listeners add three more queues.
