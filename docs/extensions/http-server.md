# Http server (`Extensions/Http`)

Shared rules of Io, Tls, Http and Json (pointers or values, `close(x, now)`, errors, who frees, milliseconds): [conventions](extensions-conventions.md).

## What it is

A routing HTTP/1.1 server on fibers: you register plain functions as handlers, say what they need as parameter types, and the module checks them against the route at compile time and generates the code that reads the request. It sits on the [codec](http-codec.md) and on [Io](io-engine.md), and is a jaic extension, not official Jai. Not available on wasm (fibers cannot switch stacks there); the codec still is. The [client](http-client.md) is the other half, and the [test client](http-testing.md) drives an `App` in memory for end-to-end tests.

```jai
#import "Basic";
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

Runnable versions are in `examples/http/`: `crud.jai` (a notes API with in-memory state), `middleware.jai` (middleware and groups), `client.jai`, and `crud_test.jai`, end-to-end tests of the notes API (`notes.jai`, which `crud.jai` serves).

## How it works

**Registration.** `get`, `post`, `put`, `patch`, `delete`, `head` and `options` take `(*app or group, template, handler)`. The template is a string literal and is parsed at compile time (`#run`): `/users/{id}` is a parameter segment, `/files/{path...}` takes the rest of the path and must be last. `register_route` in `app.jai` inspects the handler's `Type_Info_Procedure`, calls `generate_prelude` and `generate_call` in `extract.jai`, and `#insert`s the result: a thunk `(*Request) -> Response` that runs the extractors, calls the handler and maps its return value. The thunk is stored in the router.

**Compile-time checks.** A route that cannot work is reported on the `get(...)` call, in terms of the route (`compiler_report` with the call site, which needed `$loc := #caller_location` to resolve to the caller):

```
case.jai:12:5: error: Http: route `GET /users/{id}`: the handler asks for Path("uid", ...), but the route has no `{uid}` parameter (the route has `{id}`)
        get(*app, "/users/{id}", get_user);
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

Checked: a `Path` or `Params` name that is not in the template, a `Path` type that text cannot become, two body extractors, a parameter that is not an extractor, more than two return values, a malformed template or a misplaced `{x...}`. Two routes that match the same requests are found when the router is built and reported at startup (`serve` prints them and returns false; `start` returns the error).

**Extractors.** Handler parameters; each has `value` and, for struct types, `using value` so `body.name` works.

| Parameter | Reads | Failure |
| --- | --- | --- |
| `Path("id", T)` | one `{id}` of the route (int, float, bool, string, enum) | 400 naming the parameter |
| `Params(T)` | every field of `T` from the route parameters of the same name | 400 |
| `Rest("path")` | the `{path...}` tail | |
| `Query(T)` | the query string, like JSON with the `@json` notes (`default`, `rename`, repeated keys fill arrays) | 400 |
| `Json(T)` | the body as JSON; `Content-Type: application/json` required | 415, 400 (not JSON), 422 (wrong shape, with the JSON path) |
| `Form(T)` | an `application/x-www-form-urlencoded` body | 415, 400, 422 |
| `Header("name", T)`, `Header_Optional("name", T)` | a header, parsed as `T` | 400 when missing (not for `_Optional`, which has `present`) |
| `Bytes`, `Text` | the raw body | |
| `*Request` | the whole request (`method`, `target`, `path`, `headers`, `body`, `peer`, `header(...)`, `query_param(...)`) | |
| `*State(T)` | `app.state`, the pointer you set (not a copy) | |

Query, form and path text is turned into typed values by [`from_query`/`from_param`](json.md#query-and-form-text) in Json, so the same field rules and error wording apply.

**Return values.** The first return value becomes the response, an optional second `Http_Error` replaces it when its `kind` is not `.NONE`:

| Returns | Response |
| --- | --- |
| nothing | 204 |
| `Http_Error` only | 204, or the error |
| a struct, array, number, ... | its JSON (`json`), 200, or 201 for POST |
| `string` | `text/plain; charset=utf-8` |
| `[]u8` | `application/octet-stream` |
| `Response` | as it is (build with `json`, `text`, `html`, `bytes_response`, `redirect`, `no_content`, `status_only`, `add_header`, `with_header`) |

**Errors.** `bad_request`, `unauthorized`, `forbidden`, `not_found`, `conflict`, `gone`, `unprocessable`, `too_many_requests`, `internal_error`, `unavailable` take a `tprint` format; `http_error(status, detail)` takes any status. They are sent as RFC 9457 `application/problem+json`:

```
{"type":"about:blank","title":"Not Found","status":404,"detail":"No user with id 7","instance":"/users/7"}
```

Extractor failures, 404, 405, 413 (body over the limit), 414, 431 and 408 use the same format and name `location` (`path`, `query`, `body`, `header`, `form`) and `field`. Set `app.on_error = (error: Http_Error, request: *Request) -> Response` to use your own format for all of them. `with_header(error, "Retry-After", "5")` adds headers to an error.

**Routing.** A segment trie: static segments beat `{param}` which beats `{rest...}`, with backtracking, so `/users/me` and `/users/{id}` coexist. A path that exists for other methods gets 405 with `Allow`; `HEAD` runs the `GET` route without the body (or its own route); `OPTIONS` answers 204 with `Allow` unless you register one. A trailing `/` is ignored (`/users/` is `/users`). TRACE, CONNECT and unknown methods get 501.

**Middleware and groups.** `use(*app, mw)` wraps every request including ones no route answers; middleware is `(request: *Request, next: Next) -> Response`, in the order added. `group(*app, "/admin", require_token)` returns a group whose routes get the prefix and the extra middleware (after the app's); groups nest (`group(admin, "/v1")`), and the prefix may contain parameters (`group(*app, "/orgs/{org}")`) that handlers below it extract. The prefix is part of the group's type, so `Path("org", ...)` is checked against the whole route.

**Connections.** One fiber per connection, one handler call per request, in order. The loop reads into a buffer taken from the scheduler's pool (see [Memory](#memory)), feeds the codec, and writes responses when it has nothing left to read, so pipelined requests are answered in order in few writes. Keep-alive follows HTTP/1.1 rules (HTTP/1.0 only with `Connection: keep-alive`). `Expect: 100-continue` is answered before the body is read, and a request whose declared body is over the limit is refused before reading it. Bodies over 64 KiB are collected in a growing buffer; smaller ones are views into the read buffer. `Date` comes from the codec's per-second cache. After each response temporary storage is reset, so handlers can use `tprint` and `temp` freely; strings that must outlive the request have to be copied. A handler that crashes the process is not caught (there are no exceptions); return an `Http_Error` instead.

**Memory.** A connection that is waiting holds no read buffer. When nothing is pending the loop gives its buffer back to a per-scheduler pool (`Io.scheduler_local`, at most 64 free buffers kept), waits with `Io.wait_readable` (no buffer, no data consumed), and takes a buffer only when the socket has bytes; idle timeout, header timeout and shutdown work on that wait as on a read. A connection whose reader cannot wait (the in-memory test client's pipes) keeps one buffer for its whole life, as before. One extra operation per request is the price, so a connection whose last two requests began less than 10 ms apart (`HOT_MS`) keeps its buffer for the next read and gives it up only if the client stays quiet for those 10 ms; a busy connection costs what it did, an idle one costs nothing, and the number of buffers in use follows the requests in flight. The rest of an idle connection is trimmed as well: when a connection is not busy it frees its response buffer and the parser's storage, and busy ones free any of them (the response buffer, the collected body, each parser array) that grew past 64 KiB (16 KiB for the parser) once the request is done. What is left is the fiber's stack (two pages: the live frames of the connection loop; the pages deeper calls touched once are given back when the connection starts its idle wait, see [Io](io-engine.md)) and a few small structures.

Server RSS with N idle keep-alive connections that each sent one request (`bench/idle_bench.jai`: the server in its own process, 4-core Linux VM, `-O2`; bytes are RSS growth per connection):

| Connections | Backend | Before | After |
|---|---|---|---|
| 1,000 | io_uring | 35.2 MB (34.8 KB each) | 17.3 MB (16.4 KB each) |
| 1,000 | epoll | 35.3 MB (34.9 KB each) | 17.1 MB (16.2 KB each) |
| 9,000 | io_uring | 306 MB (34.7 KB each) | 144 MB (16.2 KB each) |
| 9,000 | epoll | 307 MB (34.8 KB each) | 144 MB (16.3 KB each) |

Of the 16 KB left, about 12 KB are fiber stack pages the connection has touched (see Idle stacks in [io-engine](io-engine.md)) and about 4 KB the fiber record with its context, the temporary storage, the socket's registration and the connection state. Requests per second did not change beyond noise (`bench/server_bench.jai`, `-O2`, 4-core Linux VM, io_uring, before and after alternating, medians of 5 runs of 3 s; the run-to-run spread is about 10%): one thread, hello 97.6k to 95.0k and JSON 82.0k to 92.3k; four threads, hello 355k to 336k and JSON 303k to 353k. Earlier, when the buffer give-back was added: one thread pinned to a core, 15 runs of 2 s, hello 97.9k to 96.0k and JSON 90.5k to 94.2k. Without the `HOT_MS` rule the extra operation per request cost about 4 to 6% on one thread (9 runs), which is why it exists. On epoll, requests per second rose by 10% to 17% in the same series for an unrelated reason, the edge-triggered registration in [io-engine](io-engine.md#numbers).

**Serving.** `serve(*app)` runs one scheduler thread per core (`config.threads`), each with its own listener on the same port (`SO_REUSEPORT` when more than one), and returns when stopped. With `config.handle_signals`, SIGINT and SIGTERM begin a graceful stop: the listener closes, idle keep-alive connections close at once, requests in progress finish and get `Connection: close`, and after `config.shutdown_timeout` the rest are cut. `shutdown(*app)` does the same from any thread. For tests and programs that do more, `start(*app)` inside `Io.io_run` returns a `Server` (`server_url`, `stop`, `wait`) for the running scheduler only.

## HTTPS

`app.config.tls = *config` serves HTTPS: `config` is a [`Tls.Tls_Config`](tls.md) with a certificate chain and private key that has been through `Tls.tls_config_init` and outlives the server (`start` refuses a config that cannot accept, and `serve` says so). Every accepted connection then does its TLS handshake on its own fiber before anything else, under `header_timeout`, and the connection loop runs on the TLS stream's reader and writer, so routing, keep-alive, pipelining, limits and shutdown are the same code and the same settings: the idle wait still takes no read buffer (the TLS reader has a `wait_proc`), `idle_timeout` and `write_timeout` apply to the TLS reads and writes, and a graceful stop closes idle TLS connections with a close_notify. A client that drops its TLS connection without a close_notify, between requests or after one, is an ordinary close: no error and no log line, even with `log_requests` (only a failed handshake is logged). `server_url` says `https://`.

- **Failed handshakes** (a client that speaks plain HTTP to the port, stops halfway, presents no client certificate when `client_auth` is on, or is too slow) close the connection without an answer, since there is no HTTP yet; with `log_requests` they print one line with the peer's address and the reason.
- **ALPN.** The connection loop is chosen after the handshake by `choose_protocol` in `server.jai`: no ALPN or `http/1.1` runs HTTP/1.1; anything else (`h2`) drops the connection. Give the config `alpn = string.["http/1.1"]`, or none: a config that lists `h2` today would have clients negotiate a protocol the server does not speak.
- **Client certificates:** `client_auth = true` with `roots_pem` makes the handshake fail for a client without a certificate that chains to those roots.
- **Cost.** A TLS connection holds about 34 KiB more than a plain one for as long as it is open, idle or not ([tls.md](tls.md#how-it-works)); plain HTTP connections are unaffected, and throughput of plain HTTP did not change (bench below).
- The handshake costs a signature and a key exchange: measured 5.7 ms for both ends together on one core (TLS 1.3, ECDSA P-256 certificate, the client checking the chain; 6.7 ms with RSA-2048, 5.2 ms for TLS 1.2), so a server does very roughly 300 new connections a second per core. Keep-alive and the client's pooling amortise it.

**HTTP/2.** `upgrade_connection` in `server.jai` is where an `Upgrade: h2c` request or a prior-knowledge preface would hand the connection over. It is a stub that returns false until the HTTP/2 milestone.

## How to change it

- A new extractor: add the struct to `extract.jai`, a case to `classify` and to `generate_prelude`/`generate_call` (the arguments are built there), a run-time `extract_*` function in the module-scoped half of the file, and the failure wording. Add a row to the table above and a test in `tests/server_extractors.jai`.
- A new return type: add a `respond` overload in `extract.jai` (the thunk calls `respond(value, request)`).
- Compile-time errors go through `report_*` in `extract.jai`; keep the message in the form "route `GET /x`: what is wrong (what the route has)" and add a case to `crates/jaic-cli/tests/http_routes.rs`.
- Gotchas: everything after `#scope_module` in `server.jai`, `router.jai`, `url.jai` and the second half of `extract.jai` is private to the module; put new public procedures before it. Handler thunks are generated per route, so a big handler set costs compile time, not run time. Jai has no UFCS: use `with_header(response, ...)`, not `response.with_header`.
- Memory in the connection loop: the read buffer is taken and given back in `serve_connection` (`take_buffer`, `give_buffer`, `HOT_MS`), the response buffer, collected body and parser storage are trimmed with `trim` and `parser_trim` (`RETAIN_OUTPUT`, `RETAIN_HEAD`). Anything the loop holds across a wait must not point into the read buffer, since the buffer goes back to the pool when nothing is pending.
- Timing in the connection loop is in `serve_connection`: `head_started` is set when the first byte of a request arrives, including when it came with the tail of the previous one.

## Configuration

`app.config` (`Server_Config`). Every time is in milliseconds. For `header_timeout`, `body_timeout`, `idle_timeout` and `write_timeout`, 0 or a negative number means no limit (the code turns it into `Io.NO_DEADLINE`). `shutdown_timeout` is the exception: 0 or less is not "wait forever" but "do not wait", so connections are cut at once; there is no unlimited setting for it.

| Field | Default | Meaning |
| --- | --- | --- |
| `address` | `":8080"` | `":8080"`, `"127.0.0.1:3000"`, `"[::1]:8080"`; port 0 picks a free one |
| `threads` | 0 | schedulers; 0 or less is one per processor |
| `header_timeout` | 10000 | from the first byte of a request to the end of its head (408); also the TLS handshake of an https connection; 0 or less: no limit |
| `body_timeout` | 30000 | silence while the body is sent (408); 0 or less: no limit |
| `idle_timeout` | 60000 | waiting for the next request on a keep-alive connection; 0 or less: wait until the client sends or hangs up (`stop` still closes it) |
| `write_timeout` | 30000 | one write of a response; 0 or less: no limit |
| `shutdown_timeout` | 10000 | how long `stop` waits before cutting connections; 0 or less: cut at once, never "forever" |
| `max_body_size` | 1 MiB | 413 over it |
| `max_head_size`, `max_headers`, `max_line_size` | 64 KiB, 100, 8 KiB | 431 / 414 |
| `max_requests_per_connection` | 10000 | then `Connection: close`; 0 or less: no limit |
| `read_buffer_size` | 16 KiB | per connection that has bytes to read, not per connection ([Memory](#memory)) |
| `server_name` | `"jaic"` | the `Server` header; empty sends none |
| `log_requests` | false | one line per request on standard output |
| `handle_signals` | true | SIGINT/SIGTERM stop `serve` |
| `tls` | null | a `Tls.Tls_Config` to serve HTTPS ([HTTPS](#https)); null serves plain HTTP |

Also `app.state`, `app.on_error` and `app.json_options` (limits for `Json(T)` bodies).

## Dependencies

[Io](io-engine.md) (fibers, TCP, deadlines, `io_run_threads`, scheduler-local storage, the interrupt flag), [Tls](tls.md) (HTTPS), [Json](json.md) (bodies, query and form decoding, responses), the [codec](http-codec.md), and `Atomics`.

## Tests and benchmark

`stdlib/Extensions/Http/tests/`: `server_routing.jai` (templates, priority, 404/405, HEAD/OPTIONS, conflicts), `server_extractors.jai`, `server_responses.jai` (return mapping, errors, `on_error`, middleware, groups; these three use the [test client](http-testing.md), no socket), `testing.jai` (the test client itself), `server_connections.jai` (on raw sockets) (keep-alive, pipelining, limits, timeouts, 100-continue, graceful and forced shutdown, a 150-request crowd) and `client.jai`, sharing `tests/lib/server_support.jai`. `https.jai` runs the server and the client over TLS ([HTTPS](#https)). `server_connections.jai` also checks the memory rules: twenty idle keep-alive connections hold no read buffer (`http_statistics().read_buffers_in_use`), neither does one waiting for the rest of a head or between pipelined requests, a 22 KB head and a body in two parts still work across the give-back, a connection that was busy and went quiet still times out at `idle_timeout`, and every buffer is back when the connections close; `codec.jai` covers `parser_trim`. Compile errors are in `crates/jaic-cli/tests/http_routes.rs`. `python3 tools/http_conformance.py` also builds `tools/http_garden/server.jai` (an echo server) and probes it with smuggling shapes, limits and a SIGINT shutdown; it is the program to put in a [http-garden](https://github.com/narfindustries/http-garden) image.

`stdlib/Extensions/Http/bench/idle_bench.jai` measures the server's resident memory with many idle keep-alive connections (Linux; see [Memory](#memory)), and `stdlib/Extensions/Http/bench/server_bench.jai` measures requests per second over loopback with 32 keep-alive connections per thread, for a text route and a JSON route (build with `-O2`; arguments are threads and milliseconds per route, and optionally a rate in requests per second for an open load that times each request from when it was due). It prints p50, p90, p99, p99.9 and the largest latency of each route; the numbers are in [io-engine](io-engine.md#latency). Every thread is an independent server with its own load clients, so the numbers include the load generator's cost. On a 10-core M-series Mac, with other work running: about 96k requests/s for hello and 82k for the JSON route on one thread, 140k and 136k on two, 126k and 124k on four. More threads stop helping because the client and server share the machine and the loopback stack; the single-thread figure is the one to compare.
