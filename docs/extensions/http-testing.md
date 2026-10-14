# Http testing (`Extensions/Http`)

## What it is

End-to-end tests for an `App` with no socket and no setup. `test_client(*app)` returns the ordinary [`Client`](http-client.md) wired to the app in memory, so every client call (`get`, `post`, `send`, `read_json`, ...) works on it with the same types, and `expect_*` procedures check the answers and print the request and response when something is not as expected. A jaic extension, not official Jai. Not available on wasm.

```jai
#import "Basic";
#import "Extensions/Http";
#load "notes.jai";                       // Store, Note, notes_routes(*app, *store)

main :: () {
    run_tests(.[
        .{ "creates a note and reads it back", creates_and_reads },
        .{ "rejects an empty title", rejects_an_empty_title },
    ]);
}

creates_and_reads :: () {
    store: Store;
    app: App;

    notes_routes(*app, *store);

    api := test_client(*app);

    created := expect_json(post(api, "/notes", New_Note.{ title = "milk" }), Note, status = 201);

    expect_equal(created.title, "milk");
    expect_equal(store.notes.count, 1);          // the store is the test's own variable
    expect_status(get(api, "/notes/1"), 200);
}

rejects_an_empty_title :: () {
    ...
    expect_problem(post(api, "/notes", New_Note.{ title = "" }), 422, "title must not be empty");
}
```

`examples/http/crud_test.jai` is the full version, testing `examples/http/notes.jai` (the app that `crud.jai` serves). A failed expectation looks like this, and the program exits with status 1:

```
FAIL  creates a note
  examples/http/crud_test.jai:20
    expected status 201 Created, got 422 Unprocessable Content
      request   POST /notes  {"title":"","done":false}
      response  422 Unprocessable Content
                Content-Type: application/problem+json
                {"type":"about:blank","title":"Unprocessable Content","status":422,"detail":"title must not be empty","instance":"/notes"}
  examples/http/crud_test.jai:22
    expected "milk", got ""
  ok    passes (1 check)

1 of 2 tests failed (3 failed checks of 4, 7 ms)
```

## How it works

**One client.** The client has a `Transport` (`data`, `base`, `open`): `open` makes the byte stream a request goes over, `base` says where a relative URL points. The default transport opens TCP connections; `test_client` sets one that opens an in-memory pipe (`Io.make_pipe`) and hands the other end to the app's real connection loop (`adopt` in `server.jai`, which is also what the accept loop calls for a socket). So parsing, framing, keep-alive, pipelining, limits, middleware, routing, extractors, error mapping and response writing are all the production code; only the socket differs. Reading and writing go through `Stream` (`stream.jai`), a TCP stream or a pipe end. The server side of a test client is a `Server` without a listener, kept per scheduler (scheduler-local storage) and dropped when the scheduler ends. Idle connections are pooled like any client's, keyed by transport, so a test client never reuses another client's connections.

**Why not call the handler directly.** A dispatch at the request level would skip the codec, the framing and the connection loop, which is where most real bugs are (a body over 64 KiB, `Connection: close`, `Expect: 100-continue`, a limit answered with 413). The pipe costs one allocation per connection and no system call, so there was nothing to save.

**Determinism.** Client and server fibers run on one scheduler thread and switch only when one blocks, so the order of events depends on the code alone. Writes to a pipe never block; a read waits for bytes, honours deadlines (`timeout` works) and cancellation. Nothing in a test needs a sleep to "let the server run". The `Date` header is the only wall-clock value.

**Zero setup.** `send` checks `Io.in_scheduler()`. Outside `Io.io_run` it runs a scheduler for that one call and returns when it is over, so a test body is plain code. Inside a scheduler the call runs on the calling fiber: connections are kept between calls (`http_statistics().connections_opened` counts them), handlers can use `Io.sleep`, channels, other fibers and the client against other apps, and a test can start many fibers that all make requests. This works for every `Client`, not only test clients (`api: Client; get(*api, "http://...")` in a `main` with no `io_run` works; the body is read inside that short scheduler, see the [client](http-client.md)).

What the outside-scheduler mode means for a test: connections do not outlive a call, and anything a handler allocated in temporary storage is gone when the call returns (as after every response), so a handler that records strings for the test to look at has to `copy_string` them.

**Requests.** Relative URLs (`"/notes/1"`) go to the client's `base_url`, or to `http://localhost` for a test client; an absolute URL with any host also reaches the app, with its `Host` header as written. Test clients do not follow redirects (`max_redirects` 0), so `expect_redirect` sees the 3xx; `send(api, .{ url = ..., max_redirects = 5 })` follows for one request, or set `follow_redirects = true` in `Test_Options`. The default request timeout is 10 s (`Test_Options.timeout` / `client.timeout`). `set_header(api, "Authorization", "Bearer x")` adds a header to every request that does not set it; `remove_header` takes it off.

The response calls return `(Client_Response, Client_Error)`; with the test client you usually write `response := get(api, ...)` and let the response carry any failure: a call that got no response returns one with `status` 0, `reason` saying why and `method`/`url` naming the request, so the `expect_*` that gets it prints "response none: timed out while reading the response" under the request line.

**Bodies.** A response holds only the status and headers; its body is read with `read_text`, `read_bytes`, `read_json` and `read_some` ([client](http-client.md)). The expectations that need the body (`expect_json`, `expect_problem`, `expect_text`, `expect_body_contains`) and every failure message read it into memory the first time they look and keep it there, so several expectations can check one response, a failure always shows the body, and the test can still call `read_json(*response, T)` afterwards. A body the test already read is shown in a failure from the first 512 bytes that were kept. An expectation that only looks at the status or headers does not touch the body, so a test that drops such a response leaves its connection to the client's usual handling of unread responses; small answers never hold one.

**Expectations.** Each returns whether it held (`expect_json` and `expect_problem` return the decoded value, empty when the check failed), counts one check and, on failure, records the failure and lets the test go on. Use `if !expect_status(...) return;` when the rest depends on it.

| Call | Checks |
| --- | --- |
| `expect_status(response, 201)` | the exact status |
| `expect_success(response)` | any 2xx |
| `expect_json(response, T, status = 0)` | status (2xx when 0), a JSON content type, then decodes a `T` |
| `expect_problem(response, 422, "title")` | status, `application/problem+json`, the document's own `status`, and a `detail` containing the text; returns the `Problem` (`title`, `detail`, `instance`, `location`, `field`) |
| `expect_header(response, "content-type", "application/json")` | a header's value; `Content-Type` without parameters matches the media type |
| `expect_header(response, name)`, `expect_no_header(response, name)` | presence or absence |
| `expect_text(response, "...")`, `expect_body_contains(response, "...")` | the body |
| `expect_redirect(response, "/login", status = 0)` | 3xx (or `status`) and `Location` |
| `expect(condition, "what")`, `expect_equal(actual, expected, "what")` | anything else: state a handler left behind, counts |

A failure prints the file and line of the caller, what was expected, and the request line (with the start of the request body), the status, `Content-Type`, `Location`, `Allow`, `WWW-Authenticate`, `Retry-After`, `ETag`, `Set-Cookie` and the header the check was about, and the body cut at 400 bytes (a binary body is only counted). Paths are relative to the working directory when they are below it.

Why plain procedures and not a fluent chain: Jai has no methods, so a chain would read inside out. Procedures that return what they check nest instead: `expect_json(post(api, "/notes", body), Note, status = 201)`.

**Running.** `run_tests(.[ .{ name, proc }, ... ])` runs the procedures in order on the calling thread, frees the test clients after each, prints `ok` lines, the failures as they happen, and a summary; it exits with status 1 when any expectation failed. Plain `expect_*` calls outside `run_tests` still print and count (`test_failure_count()`), for a program with its own loop.

**State.** Tests build the `App` with state they own (`app.state = *store`, handlers take `*State(Store)`) and read or prepare it directly; requests run on the same thread, so no locking.

**Over TCP.** `test_client(*app, .{ over_tcp = true })` serves the app on a free loopback port instead (started on first use, closed when the scheduler ends) and connects with the real socket client. The same test code runs unchanged; use it where the network path matters. It sets `app.config.address` to `127.0.0.1:0` and turns `handle_signals` off. Tests that are about the socket itself (keep-alive over TCP, timeouts, shutdown, pipelining) stay on raw sockets: see `tests/server_connections.jai`.

## How to change it

- A new expectation: add it to `expect.jai` (before `#scope_module`), start with `test_state.checks += 1;`, return early on success, and report with `fail_response(loc, headline, response, focus)` so the output has the same shape; take `loc := #caller_location` last. If it needs the body use `checked_body(response)`, not the connection. Add a row above, a case to `failure_messages` in `tests/testing.jai` (it compares the printed text) and use it in `examples/http/crud_test.jai` if it reads well there.
- A different transport (a Unix socket, TLS): fill `Client.transport` with `open` returning a `Link` (an `Io.Reader` and `Io.Writer` of your own, or `tcp_link`/`pipe_link`; see `link.jai`). The pool keeps connections per `transport.data`.
- Test the test tools themselves with `begin_failure_capture()` / `end_failure_capture()`: failures inside are collected as text and do not count against the run.
- The pipe is `Extensions/Io/pipe.jai` (`make_pipe`, `read`, `write`, `shutdown_write`, `close`, all taking `*Pipe_Stream`); close each end once, the pair frees itself after the second.
- Gotchas: `test_client` returns a pointer owned by the test run (`free_test_client` releases it sooner). An app's routes must not change while clients use it. A handler that blocks the thread (`sleep` from `Thread`, a busy wait) blocks the whole test: use `Io.sleep`.

## Configuration

`Test_Options`: `over_tcp` (false), `follow_redirects` (false), `timeout` (10000 ms). `Client` fields: `base_url`, `timeout`, `max_redirects`, `headers` (`set_header`), `transport`. Failures print to standard output.

## Dependencies

The [client](http-client.md) and [server](http-server.md) of this module, [Io](io-fibers.md) (`make_pipe`, `in_scheduler`, `io_run`, scheduler-local storage, deadlines) and [Json](json.md) for `expect_json` and `expect_problem`.

## Tests

`stdlib/Extensions/Http/tests/testing.jai` covers the test client (outside and inside a scheduler, concurrent fibers, keep-alive, large bodies, limits, interim responses, handlers that sleep, use channels or call another app, timeouts, default headers, TCP, conflicting routes) and the exact text of failures. `tests/server_routing.jai`, `server_extractors.jai` and `server_responses.jai` run on the test client. `stdlib/Extensions/Io/tests/pipe.jai` tests the pipe.
