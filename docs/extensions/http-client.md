# Http client (`Extensions/Http`)

Shared rules of Io, Tls, Http and Json (pointers or values, `close(x, now)`, errors, who frees, milliseconds): [conventions](extensions-conventions.md).

## What it is

A small HTTP/1.1 client for fibers. A request returns a response that holds the status and headers; the body stays on the connection and is read from the response afterwards (`read_text`, `read_bytes`, `read_json`, `read_some`). There is a per-scheduler connection pool, timeouts and redirects. `http://` and `https://` (see [HTTPS](#https)). It shares `#import "Extensions/Http"` with the [server](http-server.md) and the [codec](http-codec.md) and is a jaic extension. Not available on wasm. A client can also be wired to an `App` in memory for [tests](http-testing.md).

```jai
main :: () {
    api: Client;
    api.base_url = "http://localhost:8080";

    response, error := get(*api, "/users/1");
    if error.kind != .NONE { print("%\n", error_text(error)); return; }     // could not get a response at all
    defer close(*response);

    if response.status != 200 { print("status %\n", response.status); return; }

    user, error2 := read_json(*response, User);                              // the body, once
    if error2.kind != .NONE { print("%\n", error_text(error2)); return; }
    free_json(*user);

    created := post(*api, "/users", New_User.{ name = "Ada" });               // a struct body is sent as JSON
    text, _ := read_text(*created);
    free(text.data);
    close(*created);
}
```

A `Client` holds what requests have in common (`base_url`, default `headers`, `timeout`, `max_redirects`, the `transport`). Every call takes the client it goes through. A zeroed `Client` works (no base URL, TCP, 30 seconds), so a one-off script writes `api: Client;` and passes `*api`; absolute URLs need no `base_url`. `examples/http/client.jai` is a complete program.

## How it works

**Making a request.** One shape: `send(client, .{ method, url, headers, body, timeout, max_redirects, max_response_size })`, or a verb that fills in the request: `get`, `head`, `options`, `delete` `(client, url, headers)` and `post`, `put`, `patch` `(client, url, [value], headers)`, where `value` is sent as JSON (`Content-Type: application/json`). Any other body is a `send` with `body` set; `bytes_of(text)` turns a string into one. The verbs share their names with the server's route registration; the first argument, a `*Client` or an `*App`, tells them apart. They all return `(Client_Response, Client_Error)` as soon as the status line and headers have arrived. A 404 is a response, not an error; the error is only about not getting one (URL, connection, timeout, a reply that is not HTTP, too many redirects). When there is no response the call still returns one, with `status` 0 and `reason` holding the error text, so code that keeps only the response can still name the request; it has no body.

**Reading the body.** The response has `status`, `reason`, `headers` (read one with `header(*response, "Content-Type")`) and the request it answers (`method`, `url`, the start of `request_body`). Its body is read through the response, and only once:

| Call | Returns |
| --- | --- |
| `read_text(*response)` | the whole body as a string (release with `free`) |
| `read_bytes(*response)` | the whole body as `[] u8` (release with `free(bytes.data)`) |
| `read_json(*response, T)` | the body decoded as a `T`; `free_json` releases it |
| `read_some(*response, buffer)` | the next part of the body in `buffer`, `(count, error)`; 0 when the body has ended, and on every call after that |
| `error_for_status(*response)` | `.NONE` for 2xx, else a `.STATUS` error with `status`, `title`, `detail` (from a problem+json body when there is one) and the `body`; reads the body |

The readers handle Content-Length, chunked (trailers dropped) and read-until-close framing. The whole-body readers refuse a body longer than the request's `max_response_size` (64 MiB) with `.TOO_LARGE`, early when Content-Length says so; `read_some` has no such limit, because the caller never holds the body: that is how a large download is streamed. Reading a body a second time (after any whole-body reader, or after `read_some` ended it) is a `.BODY` error, not an empty result, and so is reading a response after `close`. A read that fails (connection lost, timeout, malformed chunk) closes the connection and returns the error; reading again says the earlier read failed. `read_json` decodes whatever the status; check `response.status` first, or use `error_for_status`. A failed decode is `.DECODE` with the `Json_Error` in `error.json`.

**Connection lifetime (the one rule).** A response holds its connection until its body has been read to the end, or `close(*response)` is called, whichever is first.

- Reading to the end (any whole-body reader, or `read_some` returning data of a body of known length / 0) returns the connection to the pool at once, when the framing allows another request on it (not for read-until-close, `Connection: close`, or leftover bytes).
- `close(*response, now := false)` releases whatever is still held and frees the response (it leaves `response` empty). It is safe at any time and more than once, so `defer close(*response)` is always right. If the body was not read, `close` reads and drops a short rest (at most 64 KiB, for at most 500 ms) so the connection can be reused; a longer or unknown rest, an error or a timeout closes the connection instead. `close(*response, now = true)` never drains: it closes the connection at once. Do not use a response, or a copy of it, after closing it.
- Answers with no body (HEAD, 204, 304, `Content-Length: 0`) and small ones that arrived with their head (up to what one 16 KiB read holds, and within `max_response_size`) are taken into memory at once and the connection is released at once; they are still read with the same calls. So the common case, a dropped small response, costs nothing.
- A larger response that is dropped unread keeps its connection checked out, so it can never be mistaken for an idle one; it does not block the pool (the next request opens another connection). Such connections are listed per scheduler and bounded by `client_settings.max_open_responses` (64): past it the oldest is closed and reading it returns a `.BODY` error that names the setting. They are also closed when the scheduler ends. `open_response_count()` shows the number, for tests.
- Redirects close the intermediate answers the same way, so a chain reuses one connection and holds nothing but the final response.

Why this rule: it is the same lifetime as Go's `http.Response.Body` and reqwest's `Response` (the body is a stream owned by the response, `close`/drop frees it), without a finalizer, which Jai does not have. Reading to the end releases by itself so the common program needs no cleanup call for its connections, while the stored headers still need `close` to free their memory.

**Calls outside a scheduler.** Inside `Io.io_run` (or a handler or a spawned fiber) a call runs on the calling fiber and only that fiber waits; the body is read later, on demand, by whichever fiber calls `read_*`. Outside a scheduler `send` runs one for that call, and the connection cannot outlive it, so the whole body is read inside it (up to `max_response_size`; a failure is returned by the first `read_*`). Streaming with `read_some` therefore wants a scheduler. Connections are not kept between such calls.

**Errors.** `Client_Error` (separate from the server's status-based `Http_Error`): `kind` is `.TRANSPORT` (connect, send or receive failed; `transport` is the `Io_Error`, such as `.CONNECTION_REFUSED`), `.TIMED_OUT`, `.INVALID_URL`, `.TLS` (the TLS handshake or the server's certificate failed; `detail` says why), `.REDIRECTS` (too many), `.DECODE` (the body was not the JSON asked for; `json` says where and why), `.BODY` (already read, closed or evicted), `.TOO_LARGE`, `.STATUS` (from `error_for_status`). `error_text(error)` makes one line from any of them and `free_error(*error)` releases its strings.

**The pool.** Idle connections wait in a pool owned by the scheduler (it uses Io's scheduler-local storage, so each thread has its own and nothing is locked). A request takes the newest idle connection to the same scheme (and, for `https`, the same TLS config), host and port, or opens one. Connections older than `idle_timeout` are closed when the pool is next used, and at most `max_idle_per_host` are kept per host. A connection that dies while idle is only noticed on use: if it fails before any answer byte arrived and the method is idempotent (GET, HEAD, PUT, DELETE, OPTIONS), the request is retried once on a new connection; a POST is not repeated, you get the error. `close_idle_connections()` closes the pool at shutdown, and `http_statistics()` (below) counts it for tests. The pool is released when the scheduler ends.

**Memory per response.** A response's state (`Body_Source`, with the parser) and its 16 KiB read buffer are recycled through the same scheduler-local pool: at most 64 of each are kept, and `close` (or reading to the end) puts them back. A response whose body is complete when the head arrives (the small-answer case above) gives its buffer back at once, so holding such responses open costs their state, not 16 KiB each: 500 open responses of a 13-byte body took 10.5 MB (21.5 KB each, client and an idle server together) and now take 2.4 MB (4.9 KB each, `bench/client_bench.jai 0`). A call outside a scheduler (see above) allocates every time, because its response outlives the scheduler whose pool would own the memory. `http_statistics()` shows the pools (`body_sources_free`, `body_buffers_free`, `body_sources_created`), for tests. Requests per second did not change (about 62,000 to 66,000 on one thread either way, `bench/client_bench.jai`, noise is larger than the difference).

**Redirects.** 301, 302, 303, 307 and 308 are followed up to `max_redirects` (the request's, else the client's, 10 by default); 0 returns the 3xx. 301 and 302 on a POST and 303 on anything but HEAD become a GET without a body; 307 and 308 repeat the method and body. A `Location` may be absolute or relative. `Authorization` and `Cookie` headers are dropped when the scheme, host or port changes (a redirect from `https` to `http` is followed, without them).

**Transport.** `client.transport` decides how a connection is made: `open(data, url)` returns a `Link` (an `Io.Reader` and `Io.Writer` over a TCP stream or an in-memory pipe end, plus the peer's address; `tcp_link`, `pipe_link` in `link.jai`) and `base(data)` gives the URL a relative one is joined to. The default, with both null, connects over TCP and needs a `base_url` for relative URLs. [`test_client`](http-testing.md) sets a transport that connects to an `App` in memory. Pooled connections are only reused by the transport that opened them.

**Timeouts.** `timeout` (the request's, else the client's 30 s, 0 for none) covers the whole exchange: the request, the redirects and reading the body, however many `read_*` calls that takes. It uses Io's deadlines, so the fiber stops waiting and the connection is closed (never pooled after a timeout, because a late answer would be read as the next one). For a body that is streamed for longer than that, set `timeout = 0` on the request. DNS goes through Io's resolver, on helper threads, so a slow lookup does not block the scheduler.

**What it adds to a request.** `Host`, `User-Agent` (`client_settings.user_agent`, default `jaic-http`), `Content-Length`, and for `post`/`put`/`patch` with a value `Content-Type: application/json`. It sends no `Accept`; pass one in `headers` if the server needs it. A header you pass wins. Headers that set the framing yourself (`Content-Length`, `Transfer-Encoding`) or are malformed are refused before anything is sent. Responses are read through the codec: Content-Length, chunked, read-until-close, `100 Continue` skipped, HEAD and 204/304 without bodies.

## HTTPS

An `https://` URL (port 443 unless it says otherwise) is the same request over a [TLS](tls.md) connection: `checkout` opens the TCP connection (or asks the `transport` for one), then `secure_link` runs the handshake on the calling fiber, with the server name from the URL, under the request's `timeout`, and the connection's `Link` becomes the TLS stream's reader and writer. Everything after that, framing, the pool, redirects and body readers, is the same code, with one rule about a peer that leaves without a TLS close_notify (below). The handshake happens only for a new connection; pooled ones are TLS already, and are only reused for the same host, port and TLS config.

**A TLS peer that closes without close_notify.** The Tls layer reports it as an error (`.CONNECTION_RESET`, `last_error(stream).kind == .TRUNCATED`); the client treats it like browsers and Python's `ssl` (`suppress_ragged_eofs`) do. A response body framed by read-until-close (HTTP/1.0, or `Connection: close` without Content-Length or chunked) ends normally when the connection ends this way, exactly like a TCP end on plain http. A body framed by Content-Length or chunked is unchanged: a cut one is still a `.TRANSPORT` error, and a missing close_notify after the full body is never read. An idle pooled TLS connection the server dropped this way is just a dead pooled connection: the next request fails before any response byte with `.TRANSPORT` and the one retry on a fresh connection (for idempotent methods) replaces it. Security note: over TLS a read-until-close body cannot tell the end of the data from a connection cut by an attacker, so truncation goes unnoticed; prefer servers that frame their bodies (Content-Length or chunked).

- **Trust.** With `client.tls` unset the program's default config is used: the system's roots (the certificate bundles on Linux, `Security.framework` on macOS, CryptoAPI on Windows: see [tls.md](tls.md#how-it-works)), the host name checked against the certificate, TLS 1.2 or 1.3 and ALPN `http/1.1`. It is built the first time any scheduler needs it. There is no client setting that turns certificate checking off; a `Tls_Config` with `verify = false` does, and says so in its name.
- **Your own settings.** `client.tls = *config` where `config` is a `Tls.Tls_Config` that went through `Tls.tls_config_init` (private roots in `roots_pem`, a client certificate, a narrower `max_version`) and outlives the client's connections. Its `alpn` should be empty or `.["http/1.1"]`: there is no HTTP/2 client.
- **Errors.** A handshake that fails or a certificate that is not accepted is `.TLS`, with the reason in `detail` (`TLS with 127.0.0.1:8443: certificate verification failed: The certificate is not correctly signed by the trusted CA`). A handshake that ends in a timeout or a cancel is `.TIMED_OUT`, like any other wait of the request, and a connection that fails during it `.TRANSPORT`. A `.TLS` error needs no new connection handling: the connection is already closed.
- **A custom `transport`** still only provides the byte stream: for an `https://` URL the client runs TLS over whatever `Link` it returns, so a transport can never send an `https` request in the clear. (`test_client` connects to an `App` that speaks no TLS, so an `https://` URL through it is a `.TLS` error.)
- **Costs.** A TLS connection holds about 34 KiB more than a plain one (see [tls.md](tls.md#how-it-works)); a program that makes requests links about 1 MB of mbedTLS.

## How to change it

- HTTPS: `checkout` in `client.jai` opens the connection and calls `secure_link` for an `https` URL; `tls_config_of` picks the config, `default_tls_ready` builds the default one under an atomic state, `tls_failure` maps a `Tls_Error` to a `Client_Error`.
- The body side is `Body_Source` in `client.jai`: the connection, the parser positioned at the start of the body, the bytes read past the head, and where the reader is. It touches the connection only through the connection's `Io.Reader` (`Io.read` under `Io.with_deadline`) and `close(link)`, so it does not care what the connection is. `fill_pending` is the one place that reads and parses; every reader (`read_all`, `read_some`, the drain in `release_source`) goes through it.
- A new reader (a file download, a line reader): write it over `fill_pending` (it leaves body bytes in `source.pending`), call `readable(response)` first and set `source.done` at the end. Keep `source.reading` set while a call can block, so eviction skips the body.
- A response is shared by value: `Client_Response` copies point at one `Body_Source`. The expectations in `expect.jai` rely on that and on `peek_body`, which buffers the body without using it up.
- New pooled settings: add to `Client_Settings` and read them in `checkout`/`checkin`.
- Strings in a `Client_Response` are one allocation (`build_response`); `close` frees it with the `Body_Source`.
- Errors that hold strings (`owned`) must go through `free_error`.
- Request bodies are not streamed: `Client_Request.body` is a buffer. A streaming upload would add a body callback to the request and write it in `converse`.
- `client_settings` is a plain global read by every scheduler. Set it before starting threads.

## Configuration

`client_settings` (`Client_Settings`): `max_idle_per_host` (8), `idle_timeout` (30000 ms), `user_agent` (`"jaic-http"`), `max_open_responses` (64). `Client`: `base_url`, `headers` (`set_header(client, name, value)`, `remove_header`), `timeout` (30000 ms), `max_redirects` (10), `transport`, `tls` (the `Tls_Config` for `https://`; null: the system's roots). Per request, where -1 means the client's: `timeout`, `max_redirects`; and `max_response_size` (64 MiB, whole-body readers only). Compile-time constants in `client.jai`: `DRAIN_LIMIT` (64 KiB) and `DRAIN_TIMED_OUT` (500 ms) for `close`.

`http_statistics()` returns an `Http_Statistics` (diagnostic, for tests: `read_buffers_in_use`, `read_buffers_free`, `idle_connections`, `connections_opened`, `open_responses`, `body_sources_free`, `body_buffers_free`, `body_sources_created` for the running scheduler's pools).

## Dependencies

[Io](io-engine.md) (TCP, `resolve`, deadlines, scheduler-local storage, pipes), [Tls](tls.md) for `https://`, [Json](json.md), the [codec](http-codec.md) and `url.jai` for URL parsing and percent encoding.

## Tests

`stdlib/Extensions/Http/tests/client.jai` runs the client against the in-process server and against hand-made servers: each reader (`read_text`, `read_bytes`, `read_json`, `read_some` in small pieces) over Content-Length, chunked and read-until-close bodies, the read-once errors, `close` before, after and twice, pool reuse per framing, `error_for_status`, a body larger than `max_response_size` (refused by `read_bytes`, streamed by `read_some`), an unread response followed by another request on the same client, `close` of short and long unread bodies, the recycled response states and buffers and their caps, the open-response cap, redirects of each status (and that the chain holds nothing), credential stripping across hosts, redirect loops, timeouts, a refused connection (also over `https`), URL errors, a stale pooled connection (retried for GET, not for POST), and short, garbage and ambiguous responses.

`stdlib/Extensions/Http/tests/https.jai` runs the client against an `https` server on the test CA of the [Tls tests](tls.md#tests): a round trip, five requests on one TLS connection, a 3 MB body each way, the default client refusing the test CA, an untrusted CA, a certificate for another name and an expired one, `verify = false`, an uninitialised config, plain http to a TLS port, client certificates, ALPN, a client that never completes the handshake, idle connections holding no read buffer, 30 clients at once and shutdown with a TLS connection open. It also runs hand-made TLS servers that drop the connection without close_notify: a read-until-close body that is complete, a Content-Length body that is cut (an error), and a pooled connection killed that way (replaced by the retry); and clients that leave the server that way (no error, no log).
