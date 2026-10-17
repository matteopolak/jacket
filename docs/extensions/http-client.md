# Http client (`Extensions/Http`)

Shared rules: [README](README.md#conventions-of-io-tls-http-and-json).

## What it is

An HTTP/1.1 client for fibers with a per-scheduler connection pool, timeouts, redirects and `https://` (over [Tls](tls.md)). A request returns a response holding status and headers; the body stays on the connection and is read from the response (`read_text`, `read_bytes`, `read_json`, `read_some`). Shares `#import "Extensions/Http"` with the [server](http-server.md) and [codec](http-codec.md). Not available on wasm. `examples/http/client.jai` is a full program.

```jai
api: Client;                                   // zeroed works: TCP, 30 s, no base URL
api.base_url = "http://localhost:8080";

response, error := get(*api, "/users/1");
if error.kind != .NONE { print("%\n", error_text(error)); return; }   // no response at all
defer close(*response);

if response.status != 200 return;              // a 404 is a response, not an error
user, error2 := read_json(*response, User);    // the body, once
free_json(*user);

created := post(*api, "/users", New_User.{ name = "Ada" });           // struct body is sent as JSON
```

## How it works

- **Requests.** `send(client, .{ method, url, headers, body, timeout, max_redirects, max_response_size })`, or the verbs `get`, `head`, `options`, `delete` `(client, url, headers)` and `post`, `put`, `patch` `(client, url, [value], headers)`. A failed call still returns a response with `status` 0 and `reason` holding the error text. It adds `Host`, `User-Agent`, `Content-Length`, and `Content-Type: application/json` for a value body, never `Accept`; headers you pass win; hand-set framing headers are refused.
- **Bodies.** Read once through the response: `read_text`/`read_bytes` (refuse bodies over `max_response_size`, 64 MiB, with `.TOO_LARGE`), `read_json` (any status; failed decode is `.DECODE`), `read_some` (streams, no size limit; 0 at the end), `error_for_status` (2xx or a `.STATUS` error with problem+json fields). A second read, or a read after `close`, is a `.BODY` error. A failed read closes the connection.
- **Connection lifetime.** A response holds its connection until the body is read to the end or `close(*response)` runs (safe any time, more than once; unread short bodies are drained up to 64 KiB / 500 ms so the connection can be reused, `now = true` never drains). Small answers that arrive with their head, and bodiless ones, are taken into memory and the connection is released at once. A larger unread response keeps its connection out of the pool; at most `max_open_responses` (64) such connections are kept, the oldest is closed and reading it gives a `.BODY` error. Same lifetime as Go's `Response.Body`, without a finalizer.
- **Outside a scheduler** `send` runs one for the call, so the whole body is read inside it (up to `max_response_size`); streaming needs a scheduler. Connections are not kept between such calls.
- **Pool.** Scheduler-local (no locks). The newest idle connection to the same scheme, host, port (and TLS config) is reused; idle ones older than `idle_timeout` close; `max_idle_per_host` are kept. A pooled connection that died is retried once on a new one only for idempotent methods and only if no answer byte arrived. A response's state and 16 KiB read buffer are recycled through the same pool (64 each).
- **Redirects.** 301/302/303/307/308 up to `max_redirects` (10; 0 returns the 3xx). 301/302 on POST and 303 become GET without body; 307/308 repeat. `Authorization` and `Cookie` are dropped when scheme, host or port changes.
- **Timeouts.** `timeout` (30 s, 0 none) covers the whole exchange including redirects and body reads. A timed-out connection is never pooled. Set `timeout = 0` on the request for long streams. DNS runs on [Io](io-engine.md)'s resolver threads.
- **Transport.** `client.transport` (`open(data, url) -> Link`, `base(data)`) decides how a connection is made; the default is TCP. [`test_client`](http-server.md#testing) sets one that connects to an `App` through a pipe. For `https://` the client runs TLS over whatever `Link` the transport returns.
- **Errors.** `Client_Error.kind`: `.TRANSPORT` (with the `Io_Error`), `.TIMED_OUT`, `.INVALID_URL`, `.TLS` (handshake or certificate; `detail` says why), `.REDIRECTS`, `.DECODE`, `.BODY`, `.TOO_LARGE`, `.STATUS`.

### HTTPS

`https://` (port 443) runs the handshake in `checkout` via `secure_link` under the request's `timeout`; pooled connections are already TLS. With `client.tls` unset the default config uses the system roots, checks the host name, TLS 1.2/1.3, ALPN `http/1.1`; there is no client switch that disables certificate checks (a `Tls_Config` with `verify = false` does). Set `client.tls = *config` (a `Tls_Config` that went through `tls_config_init` and outlives the connections; `alpn` empty or `http/1.1`, there is no HTTP/2 client).

A peer that closes without close_notify is treated like browsers do: a read-until-close body ends normally, a cut Content-Length or chunked body is still `.TRANSPORT`. Over TLS a read-until-close body cannot tell the end from an attacker's cut, so prefer servers that frame bodies. A TLS connection costs about 34 KiB more than a plain one.

## How to change it

- HTTPS: `checkout` in `client.jai`, `tls_config_of`, `default_tls_ready`, `tls_failure` (maps `Tls_Error` to `Client_Error`).
- Body side: `Body_Source` in `client.jai` touches the connection only through its `Io.Reader` and `close(link)`; `fill_pending` is the one place that reads and parses. A new reader goes over `fill_pending` (leaves bytes in `source.pending`), calls `readable(response)` first, sets `source.done` at the end, and keeps `source.reading` set while it can block so eviction skips it.
- A `Client_Response` is shared by value: copies point at one `Body_Source`; `expect.jai` relies on that and on `peek_body`.
- Strings in a response are one allocation (`build_response`); errors with `owned` strings go through `free_error`. Request bodies are buffers, not streams.
- `client_settings` is a plain global; set it before starting threads.

## Configuration

`client_settings`: `max_idle_per_host` (8), `idle_timeout` (30000 ms), `user_agent` (`"jaic-http"`), `max_open_responses` (64). `Client`: `base_url`, `headers` (`set_header`, `remove_header`), `timeout`, `max_redirects`, `transport`, `tls`. Per request (-1 = the client's): `timeout`, `max_redirects`, `max_response_size`. Constants `DRAIN_LIMIT` (64 KiB), `DRAIN_TIMED_OUT` (500 ms). `http_statistics()` exposes pool counters for tests.

## Dependencies and tests

[Io](io-engine.md), [Tls](tls.md), [Json](json.md), the [codec](http-codec.md), `url.jai`. Tests: `stdlib/Extensions/Http/tests/client.jai` (readers over each framing, close semantics, pool reuse, redirects, timeouts, stale connections) and `https.jai` (against the Tls test CA, including no-close_notify cases).
