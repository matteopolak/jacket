# Http client

An HTTP/1.1 client for fibers with a per-scheduler connection pool, timeouts, redirects and `https://` (over [Tls](tls.md)). A request returns a response holding status and headers; the body stays on the connection and is read from the response. Shares `#import "Extensions/Http"` with the [server](http-server.md) and [codec](http-codec.md). Not on wasm. Shared rules: [README](README.md#conventions-of-io-tls-http-and-json). `examples/http/client.jai` is a full program.

```jai
api: Client;                                   // zeroed works: TCP, 30 s, no base URL
api.base_url = "http://localhost:8080";

response, error := get(*api, "/users/1");
if error.kind != .NONE { print("%\n", error_text(error)); return; }
defer close(*response);

if response.status != 200 return;              // a 404 is a response, not an error
user, error2 := read_json(*response, User);    // the body, once
free_json(*user);

created := post(*api, "/users", New_User.{ name = "Ada" });   // struct body is sent as JSON
```

## How it works

- Requests: `send(client, .{ method, url, headers, body, timeout, ... })` or the verbs `get`, `head`, `options`, `delete`, `post`, `put`, `patch`. It adds `Host`, `User-Agent`, `Content-Length`, and `Content-Type: application/json` for a value body; headers you pass win.
- Bodies are read once through the response: `read_text`/`read_bytes` (refuse over `max_response_size`, 64 MiB), `read_json`, `read_some` (streaming), `error_for_status`. A second read is a `.BODY` error.
- A response holds its connection until the body is read or `close(*response)` runs. Small answers are buffered and release the connection at once. At most `max_open_responses` unread large responses are kept; the oldest is closed.
- Outside a scheduler `send` runs one for the call, so the whole body is read inside it.
- The pool is scheduler-local. The newest idle connection to the same scheme/host/port/TLS config is reused. A dead pooled connection is retried once, only for idempotent methods and only if no answer byte arrived.
- Redirects: 301/302/303/307/308 up to `max_redirects` (10). 301/302 on POST and 303 become GET; `Authorization` and `Cookie` are dropped when scheme, host or port changes.
- `timeout` (30 s, 0 none) covers the whole exchange, redirects and body reads included.
- `client.transport` decides how a connection is made; the default is TCP, and [`test_client`](http-server.md#testing) connects to an `App` through a pipe.
- Custom TLS configurations must outlive pooled connections, including idle ones. After closing every response, call `close_idle_connections()` before `Tls.tls_config_free(*config)`; changing `client.tls` does not close the old pool entries.
- `https://` runs TLS over whatever link the transport returns. The default config uses system roots and checks the host name; set `client.tls = *config` to change it. The default config offers `h2` and `http/1.1` through ALPN; a custom `client.tls` offers HTTP/2 only if its `alpn` lists `h2`. `client_settings.http2 = false` stops the client offering `h2` at all, whatever the config lists. `Client_Response.http2` says which protocol answered. A peer closing without close_notify ends a read-until-close body normally; a cut sized body is `.TRANSPORT`.
- **HTTP/2.** When the server picks `h2`, requests to an origin share one connection as concurrent streams; another connection opens when the server's `MAX_CONCURRENT_STREAMS` is reached. The API, redirects, timeouts and streaming are the same. `send` retries a request on a stream the server refused (`REFUSED_STREAM`, GOAWAY) once. `open_http2_exchange` (and so gRPC) never replays: the refusal is reported through `exchange_terminal`. Not supported: cleartext h2c and server push (the client sends `ENABLE_PUSH=0`).
- `Client_Error.kind`: `.TRANSPORT`, `.TIMED_OUT`, `.INVALID_URL`, `.INVALID_METHOD`, `.TLS`, `.REDIRECTS`, `.DECODE`, `.BODY`, `.TOO_LARGE`, `.STATUS`.

## How to change it

- HTTPS: `checkout`, `tls_config_of`, `default_tls_ready`, `tls_failure` in `client.jai`.
- `Body_Source` touches the connection only through its `Io.Reader` and `close(link)`; `fill_pending` is the one place that reads and parses. A new reader goes over it, calls `readable(response)` first, sets `source.done` at the end, and keeps `source.reading` set while it can block.
- A `Client_Response` is shared by value: copies point at one `Body_Source`, and `expect.jai` relies on that.
- `client_settings` (`max_idle_per_host`, `idle_timeout`, `user_agent`, `max_open_responses`) is a plain global; set it before starting threads.
- Tests: `stdlib/Extensions/Http/tests/client.jai` and `https.jai`.

Tests of a specific transport failure phase should inject a `Transport` whose writer accepts the request and whose reader returns the intended `Io_Error`. Keep wall-clock timeout tests separate: a deadline can expire during connecting, writing or reading.

The HTTP TLS fixtures share `tests/support/tls_support.jai`: `free_client_config` closes idle connections and asserts that no response still holds one before freeing its configuration. Raw test servers join their serving fiber before releasing its configuration or reusing the server state. Keep these cleanup boundaries when adding TLS cases.
