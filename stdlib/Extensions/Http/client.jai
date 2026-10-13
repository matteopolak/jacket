// The client: requests over HTTP/1.1, plain or over TLS, on fibers, with a connection pool per
// scheduler.
//
//     api: Client;
//     api.base_url = "http://localhost:8080";
//
//     response, error := get(*api, "/users/1");
//     if error.kind != .NONE print("%\n", error_text(error));
//     defer close(*response);
//
//     if response.status == 200 user, error2 := read_json(*response, User);
//
//     response2 := post(*api, "/users", New_User.{ name = "Ada" });   // a struct is sent as JSON
//
// A request has one shape: `send(*client, request)`, or a verb (`get`, `head`, `options`, `delete`,
// `post`, `put`, `patch`) that fills in the request. It returns once the status line and headers
// are in, whatever the status (a 404 is a response, not an error). The response holds the status
// and headers; the body is still on the connection and is read from the response afterwards, once:
// `read_text`, `read_bytes`, `read_json` or, for a body too large to hold, `read_some`.
//
// A connection goes back to the pool when its body has been read to the end. A response whose body
// is not wanted is released with `close(*response)`, which is safe to call at any time and more
// than once. See docs/stdlib/http-client.md for the exact rule.
//
// Every call takes the `Client` it goes through. A zeroed `Client` is a working one (no base URL,
// TCP, 30 seconds), so `api: Client;` is all a one-off script needs. A call made outside
// `Io.io_run` runs a scheduler of its own for that one call and reads the body before it returns
// (a connection cannot outlive its scheduler), so a script needs no setup; inside a scheduler it
// runs on the calling fiber and only that fiber waits. Idle connections are kept for the next
// request to the same host, so repeated calls on one scheduler reuse a connection.
//
// `https://` URLs work like `http://` ones: the connection is a TLS 1.2 or 1.3 session, the server's
// certificate is checked against the system's trusted roots (macOS and Windows ask the operating
// system) and the name in the URL, and ALPN offers `http/1.1`. `client.tls` replaces that with a
// `Tls_Config` of your own (private roots, a client certificate); see docs/stdlib/tls.md. A failed
// handshake or certificate check is a `.TLS` error. There is no way to turn certificate
// checking off from the client except a config with `verify = false`.
//
// A client's `transport` decides how it reaches a server. The default opens TCP connections;
// `test_client` (test_client.jai) connects to an `App` in memory, so the same calls drive an
// application without a socket.

// A request. Only `url` is needed; `.{ url = "http://..." }` is a GET.
Client_Request :: struct {
    method := Method.GET;

    // `http://host[:port]/path?query` (or `https://`), or `/path?query` relative to the client's
    // `base_url`.
    url: string;
    headers: [] Header_Field; // `Host`, `Content-Length` and `User-Agent` are added when missing
    body: [] u8;

    // Milliseconds for the whole exchange, redirects and reading the body included (0: no limit,
    // -1: the client's `timeout`).
    timeout := -1;

    // Follow 301, 302, 303, 307 and 308 up to this many times (more is an error), or not at all
    // when 0: the 3xx response is returned. -1 is the client's `max_redirects`. 301, 302 and 303
    // turn a POST into a GET without a body; 307 and 308 repeat the request. `Authorization`
    // and `Cookie` are not sent to another host.
    max_redirects := -1;

    // The largest body `read_text`, `read_bytes` and `read_json` accept, in bytes. `read_some`
    // has no limit: it hands the body over in pieces.
    max_response_size := 64 * 1024 * 1024;
}

// A response: the status and headers, and the body still waiting on its connection. `close`
// releases it.
//
// A call that got no response at all (the connection failed, a timeout) still returns one beside
// its error: `status` is 0, `reason` says what went wrong and there is no body, so code that only
// keeps the response can still name the request.
//
// Copies of a response share one body: reading it through any copy reads it for all.
Client_Response :: struct {
    status: int;
    reason: string;
    headers: [] Header_Field;

    // The request this answers, as it was given: the method, the URL (possibly relative) and the
    // first bytes of its body (`request_body_size` is the full size).
    method: Method;
    url: string;
    request_body: string;
    request_body_size: int;

    storage: *void;
    source: *Body_Source;
}

Client_Error_Kind :: enum u8 {
    NONE; // no error: `.{}`
    TRANSPORT; // connecting, sending or receiving failed (`transport` says how)
    TIMED_OUT; // the request took longer than its timeout
    INVALID_URL; // the URL cannot be requested
    TLS; // the TLS handshake or the check of the server's certificate failed (`detail` says why)
    REDIRECTS; // too many redirects
    DECODE; // the body was not what `read_json` asked for (`json` says why)
    BODY; // the body cannot be read: it was read already, or the response was closed
    TOO_LARGE; // the body is longer than the request's `max_response_size`
    STATUS; // `error_for_status`: the response was not 2xx
}

// What the client returns when something went wrong. A handler's `Http_Error` is about answering
// with a status; this is about not getting, or not reading, an answer. `error_text` makes one
// line of it.
Client_Error :: struct {
    kind: Client_Error_Kind;
    detail: string; // what happened
    transport: Io.Io_Error; // for TRANSPORT and TIMED_OUT: the Io error behind it
    json: Json_Error; // for DECODE: where and why the JSON did not fit

    // For DECODE and STATUS: the response that was being read.
    status: int;

    // For STATUS: the reason phrase, or the `title` of a problem+json body, and the body text.
    title: string;
    body: string;

    // Set when the strings above were allocated for this error: release them with `free_error`.
    owned: bool;
}

// A one-line description of any client error, for logs and messages.
error_text :: (error: Client_Error) -> string {
    if error.kind == .NONE return "no error";

    if error.kind == .STATUS {
        if error.detail.count return tprint("% %: %", error.status, error.title, error.detail);

        return tprint("% %", error.status, error.title);
    }

    if error.kind == .DECODE {
        return tprint("%: %", error.detail, error_text(error.json));
    }

    if error.kind == .TRANSPORT && error.transport != .NONE return tprint("%: %", error.detail, error.transport);

    return error.detail;
}

// Releases the strings an error from the client holds.
free_error :: (error: *Client_Error) {
    if error.owned {
        if error.title.count free(error.title.data);
        if error.detail.count free(error.detail.data);
        if error.body.count free(error.body.data);

        free_json(*error.json);
    }

    error.* = .{};
}

// How a client reaches a server. Both procedures run on a fiber of the running scheduler.
Transport :: struct {
    // Passed to both procedures; connections made through one transport are only reused by it.
    data: *void;

    // Where a relative URL points (`"http://localhost:8080"`); null: the client needs a
    // `base_url` for relative URLs.
    base: #type (data: *void) -> string;

    // A new connection for `url` (its scheme, host and port); null: a TCP connection to the host.
    open: #type (data: *void, url: Url) -> Link, Io.Io_Error;
}

// What requests have in common.
Client :: struct {
    // Relative request URLs (`/users/1`) are joined to this: `"http://localhost:8080"`.
    base_url: string;

    // Sent with every request that does not set the header itself (an `Authorization` token).
    headers: [..] Header_Field;

    // Milliseconds for a request that does not set its own `timeout`.
    timeout := 30_000;

    // Redirects followed for a request that does not set its own `max_redirects` (0: none).
    max_redirects := 10;

    transport: Transport;

    // The TLS settings for `https://` URLs; null: trust the system's roots and offer `http/1.1`
    // (a config the client makes once for the whole program). A config of your own must have gone
    // through `Tls.tls_config_init` and must outlive the client's connections.
    tls: *Tls.Tls_Config;
}

// Pool settings, for the whole program. Idle connections are closed when they have waited
// `idle_timeout` milliseconds or when more than `max_idle_per_host` pile up for one host.
Client_Settings :: struct {
    max_idle_per_host := 8;
    idle_timeout := 30_000;
    user_agent := "jaic-http";

    // Responses whose body is neither read to the end nor closed may hold this many
    // connections per scheduler; past it the oldest is closed (its body can no longer be read).
    max_open_responses := 64;
}

client_settings: Client_Settings;

// A header sent with every request of `client` that does not set it: `set_header(*api,
// "Authorization", "Bearer abc")`. Setting a name again replaces the value.
set_header :: (client: *Client, name: string, value: string) {
    for client.headers {
        if equal_nocase(it.name, name) {
            free(it.value.data);

            client.headers[it_index].value = copy_string(value);

            return;
        }
    }

    header: Header_Field;

    header.name = copy_string(name);
    header.value = copy_string(value);
    header.id = lookup_header_name(name);

    array_add(*client.headers, header);
}

// Takes a default header off again.
remove_header :: (client: *Client, name: string) {
    for client.headers {
        if equal_nocase(it.name, name) {
            free(it.name.data);
            free(it.value.data);

            array_ordered_remove_by_index(*client.headers, it_index);

            return;
        }
    }
}

// The value of the first header called `name` (any case).
header :: (response: *Client_Response, name: string) -> string, bool {
    return find_header(response.headers, name);
}

// The bytes of `text` (the same memory), for a request's `body`.
bytes_of :: (text: string) -> [] u8 {
    return .{ text.count, text.data };
}

// Releases a response: its connection and its memory. If the body was read to the end the
// connection is already back in the pool. Otherwise a short rest of the body is read and thrown
// away so the connection can serve the next request, and a connection with more left, or one that
// fails while draining, is closed. Safe to call at any time and more than once (it leaves
// `response` empty), so `defer close(*response)` is always right. After it the body cannot be read.
close :: (response: *Client_Response) {
    if response.source {
        release_source(response.source);
        discard_source(response.source);
    }

    if response.storage free(response.storage);

    response.* = .{};
}

// Sends `request` and returns once the status line and headers have arrived. The response is
// whatever the server answered (a 404 is a response, not an error); the body is read from it with
// `read_text`, `read_bytes`, `read_json` or `read_some`. Errors are only about not getting a
// response: the URL, the connection, a timeout, a reply that is not HTTP, too many redirects.
send :: (client: *Client, request: Client_Request) -> Client_Response, Client_Error {
    if Io.in_scheduler() return send_on_fiber(client, request);

    // No scheduler yet: run one for this call. It ends with the call, and a connection cannot
    // outlive it, so the body is read inside it.
    call: Outside_Call;

    call.client = client;
    call.request = request;

    Io.io_run(run_outside_call, *call);

    return call.response, call.error;
}

// `GET` with no body.
get :: (client: *Client, url: string, headers: [] Header_Field = .[]) -> Client_Response, Client_Error {
    return send(client, .{ method = .GET, url = url, headers = headers });
}

head :: (client: *Client, url: string, headers: [] Header_Field = .[]) -> Client_Response, Client_Error {
    return send(client, .{ method = .HEAD, url = url, headers = headers });
}

options :: (client: *Client, url: string, headers: [] Header_Field = .[]) -> Client_Response, Client_Error {
    return send(client, .{ method = .OPTIONS, url = url, headers = headers });
}

delete :: (client: *Client, url: string, headers: [] Header_Field = .[]) -> Client_Response, Client_Error {
    return send(client, .{ method = .DELETE, url = url, headers = headers });
}

// `POST` with `value` as a JSON body (`Content-Type: application/json`). `post(client, url)`
// sends no body. A request with any other body is a `send`.
post :: (client: *Client, url: string, value: $V, headers: [] Header_Field = .[]) -> Client_Response, Client_Error {
    return send_with_json(client, .POST, url, value, headers);
}

post :: (client: *Client, url: string, headers: [] Header_Field = .[]) -> Client_Response, Client_Error {
    return send(client, .{ method = .POST, url = url, headers = headers });
}

put :: (client: *Client, url: string, value: $V, headers: [] Header_Field = .[]) -> Client_Response, Client_Error {
    return send_with_json(client, .PUT, url, value, headers);
}

patch :: (client: *Client, url: string, value: $V, headers: [] Header_Field = .[]) -> Client_Response, Client_Error {
    return send_with_json(client, .PATCH, url, value, headers);
}

// ---------------------------------------------------------------------------------------------
// Reading the body. It can be read once: by one of the whole-body calls, or in pieces with
// `read_some` until it returns 0. A second read is a `.BODY` error. Reading to the end puts the
// connection back in the pool; an error while reading closes it.

// The whole body as text, in memory from the context allocator (release it with `free`). A body
// longer than the request's `max_response_size` is a `.TOO_LARGE` error and is not read.
read_text :: (response: *Client_Response) -> string, Client_Error {
    bytes, error := read_bytes(response);

    return string.{ bytes.count, bytes.data }, error;
}

// The whole body, in memory from the context allocator (release it with `free`).
read_bytes :: (response: *Client_Response) -> [] u8, Client_Error {
    source, error := readable(response);

    if error.kind != .NONE return .[], error;

    data: [..] u8;

    if source.has_stored {
        data = take_stored(source);
    } else {
        source.reading = true;
        defer source.reading = false;

        error = read_all(source, *data);

        if error.kind != .NONE {
            array_reset(*data);

            return .[], error;
        }
    }

    source.done = true;

    return .{ data.count, data.data }, .{};
}

// The body decoded as a `T` (whatever the status: check `response.status` first). It is
// `Io.read_json` over the response's body, so the body is decoded as it arrives; the limit of
// `max_response_size` and the read-once rule are the same as for `read_text`. Strings in the
// value are from the context allocator; `free_json` releases them.
read_json :: (response: *Client_Response, $T: Type) -> T, Client_Error {
    status := response.status;
    source, error := readable(response);

    if error.kind != .NONE return .{}, error;

    if source.has_stored == false && source.kind == .LENGTH && source.length > source.max_size {
        drop_connection(source);

        source.failed = true;

        return .{}, client_error(.TOO_LARGE, .NONE, tprint("the body is % bytes, more than the limit of % (max_response_size)", source.length, source.max_size));
    }

    body := Body_Reader.{ response = response, source = source };

    value, failure := Io.read_json(Io.Reader.{ *body, body_read_proc, null }, T);

    source.streamed = false;

    if body.failure.kind != .NONE {
        free_json(*failure);

        return .{}, body.failure;
    }

    if failure.kind != .NONE {
        failed := Client_Error.{ kind = .DECODE, status = status, owned = true };

        failed.detail = copy_string("the response is not the JSON expected");
        failed.json = failure;

        return .{}, failed;
    }

    return value, .{};
}

// The body of a response as an `Io.Reader`, for `read_json`: each read is a `read_some` that stops
// at the request's `max_response_size` and keeps the client error it ran into, which the reader
// can only report as an `Io_Error`.
Body_Reader :: struct {
    response: *Client_Response;
    source: *Body_Source;
    total: int;
    failure: Client_Error;
}

body_read_proc :: (data: *void, buffer: [] u8) -> s64, Io.Io_Error {
    body := cast(*Body_Reader) data;

    count, error := read_some(body.response, buffer);

    if error.kind != .NONE {
        body.failure = error;

        return 0, .OTHER;
    }

    body.total += count;

    if body.total > body.source.max_size {
        drop_connection(body.source);

        body.source.failed = true;
        body.failure = client_error(.TOO_LARGE, .NONE, tprint("the body is longer than the limit of % bytes (max_response_size)", body.source.max_size));

        return 0, .OTHER;
    }

    return count, .NONE;
}

// Reads the next part of the body into `buffer` and returns how many bytes it wrote: at least one
// while there is more, then 0 once the body has ended (and on every call after that). It waits
// for data when none has arrived. For a body that should not be held in memory:
//
//     buffer: [16384] u8;
//
//     while true {
//         count, error := read_some(*response, buffer);
//         if error.kind != .NONE || count == 0 break;
//         write(file, buffer, count);
//     }
//
// The request's `max_response_size` does not apply: the caller decides when to stop (and `close`
// the response to give up).
read_some :: (response: *Client_Response, buffer: [] u8) -> int, Client_Error {
    existing := response.source;

    if existing && existing.done && existing.streamed return 0, .{};

    source, error := readable(response);

    if error.kind != .NONE return 0, error;

    source.streamed = true;

    if buffer.count == 0 return 0, .{};

    if source.has_stored {
        count := min(buffer.count, source.stored.count - source.stored_at);

        memcpy(buffer.data, source.stored.data + source.stored_at, count);
        note_sample(source, .{ count, source.stored.data + source.stored_at });

        source.stored_at += count;

        if source.stored_at >= source.stored.count {
            array_reset(*source.stored);

            source.stored_at = 0;
            source.has_stored = false;
            source.done = true;
        }

        return count, .{};
    }

    source.reading = true;
    defer source.reading = false;

    eof, failure := fill_pending(source);

    if failure.kind != .NONE return 0, fail_source(source, failure);

    if eof {
        source.done = true;

        return 0, .{};
    }

    count := min(buffer.count, source.pending.count);

    memcpy(buffer.data, source.pending.data, count);
    note_sample(source, .{ count, source.pending.data });

    source.pending = advance(source.pending, count);

    // A body of known length ends with its last byte: put the connection back now, not on the
    // next call.
    if source.pending.count == 0 && source.kind == .LENGTH && source.received >= source.length {
        _, ignored := fill_pending(source);

        free_error(*ignored);
    }

    return count, .{};
}

// A non-2xx response as an error, for code that treats those as failures: `.NONE` for a 2xx. The
// error carries `status`, `title` and `detail` (from an RFC 9457 problem body, the kind the
// server sends, when there is one) and the `body` text, so this reads the body. Release the error
// with `free_error`.
//
//     response, error := get(*api, "/users/1");
//     defer close(*response);
//     error = error_for_status(*response);
error_for_status :: (response: *Client_Response) -> Client_Error {
    if response.status >= 200 && response.status < 300 return .{};

    error: Client_Error;

    error.kind = .STATUS;
    error.owned = true;
    error.status = response.status;
    error.title = copy_string(status_reason(response.status));

    text, read_error := read_text(response);

    if read_error.kind == .NONE {
        error.body = text;
    } else {
        free_error(*read_error);
    }

    content_type, found := header(response, "Content-Type");

    if found && error.body.count && (media_type_is(content_type, "application/problem+json") || media_type_is(content_type, "application/json")) {
        problem, failure := from_json(Problem_Body, error.body);

        if failure.kind == .NONE {
            if problem.detail.count error.detail = copy_string(problem.detail);

            if problem.title.count {
                free(error.title.data);

                error.title = copy_string(problem.title);
            }

            free_json(*problem);
        } else {
            free_json(*failure);
        }
    }

    if error.detail.count == 0 error.detail = copy_string(tprint("the server answered % %", response.status, status_reason(response.status)));

    return error;
}

// Closes every idle connection of the running scheduler (they are also closed when it ends).
close_idle_connections :: () {
    pool := client_pool();

    while pool.idle.count drop_entry(pool, pool.idle.count - 1);
}

// What the running scheduler's pools hold right now, for tests and diagnostics (nothing here is
// needed to use the module). Call it on a scheduler: inside `Io.io_run` or a test client call.
Http_Statistics :: struct {
    read_buffers_in_use: int; // server read buffers held by connections that are reading a request
    read_buffers_free: int; // read buffers kept for reuse (a connection between requests holds none)
    idle_connections: int; // client connections waiting in the pool
    connections_opened: int; // client connections opened over the scheduler's life
    open_responses: int; // responses that still hold their connection (body neither read to the end nor closed)
    body_sources_free: int; // response states kept for reuse
    body_buffers_free: int; // response read buffers kept for reuse
    body_sources_created: int; // response states allocated over the scheduler's life
}

http_statistics :: () -> Http_Statistics {
    statistics: Http_Statistics;

    buffers := read_buffers();
    pool := client_pool();

    statistics.read_buffers_in_use = buffers.held;
    statistics.read_buffers_free = buffers.free.count;
    statistics.idle_connections = pool.idle.count;
    statistics.connections_opened = pool.opened;
    statistics.open_responses = pool.open.count;
    statistics.body_sources_free = pool.sources.count;
    statistics.body_buffers_free = pool.buffers.count;
    statistics.body_sources_created = pool.sources_created;

    return statistics;
}

#scope_module

with_json_headers :: (headers: [] Header_Field) -> [] Header_Field {
    list: [..] Header_Field;

    for headers array_add(*list, it,, temp);

    content: Header_Field;

    content.name = "Content-Type";
    content.value = "application/json";
    content.id = .CONTENT_TYPE;

    array_add(*list, content,, temp);

    return list;
}

send_with_json :: (client: *Client, method: Method, url: string, value: $V, headers: [] Header_Field) -> Client_Response, Client_Error {
    request: Client_Request;

    request.method = method;
    request.url = url;
    request.headers = with_json_headers(headers);

    body := to_json(value,, temp);

    request.body = .{ body.count, body.data };

    return send(client, request);
}

Problem_Body :: struct @json(default) {
    title: string;
    detail: string;
}

client_error :: (kind: Client_Error_Kind, io: Io.Io_Error, what: string) -> Client_Error {
    error: Client_Error;

    error.kind = kind;
    error.owned = true;
    error.transport = io;
    error.detail = copy_string(what);

    return error;
}

// What a call was asked to do, kept for the response that answers it.
Request_Note :: struct {
    method: Method;
    url: string;
    body: [] u8;
}

// A call made with no scheduler running: `send` runs one for it.
Outside_Call :: struct {
    client: *Client;
    request: Client_Request;
    response: Client_Response;
    error: Client_Error;
}

run_outside_call :: (call: *Outside_Call) {
    // The response outlives this scheduler, so its memory must not come from the scheduler's pool.
    client_pool().recycle = false;

    call.response, call.error = send_on_fiber(call.client, call.request);

    // The scheduler ends with this call, and so does the connection: take the body now. A body
    // that cannot be taken is reported by the first read.
    if call.error.kind == .NONE && call.response.source {
        source := call.response.source;

        if !source.has_stored && !source.finished {
            source.reading = true;

            failure := read_all(source, *source.stored);

            source.reading = false;

            if failure.kind != .NONE {
                source.deferred = failure;
                source.failed = true;
            } else {
                source.has_stored = true;
            }
        }
    }
}

send_on_fiber :: (client: *Client, request: Client_Request) -> Client_Response, Client_Error {
    note: Request_Note;

    note.method = request.method;
    note.url = request.url;
    note.body = request.body;

    timeout := ifx request.timeout < 0 then client.timeout else request.timeout;
    started := now_ms();

    response: Client_Response;
    error: Client_Error;

    Io.with_deadline(within(timeout), #code {
        response, error = send_following(client, request, note);
    });

    if error.kind != .NONE return build_response(0, error_text(error), .[], note), error;

    // Reading the body is part of the exchange: it shares the request's time.
    if timeout > 0 && response.source response.source.deadline_ms = started + timeout;

    return response, .{};
}

// One exchange plus the redirects it leads to.
send_following :: (client: *Client, request: Client_Request, note: Request_Note) -> Client_Response, Client_Error {
    url_text, problem := absolute_url(client, request.url);

    if problem.kind != .NONE return .{}, problem;

    method := request.method;
    body := request.body;
    headers := with_default_headers(client, request.headers);
    redirects := 0;
    limit := ifx request.max_redirects < 0 then client.max_redirects else request.max_redirects;

    while true {
        response, error := exchange(client, method, url_text, headers, body, request.max_response_size, note);

        if error.kind != .NONE return .{}, error;

        redirect := limit > 0 && (response.status == 301 || response.status == 302 || response.status == 303 || response.status == 307 || response.status == 308);

        location, has_location := header(*response, "Location");

        if !redirect || !has_location return response, .{};

        if redirects >= limit {
            detail := tprint("more than % redirects, the last to %", limit, location);

            close(*response);

            return .{}, client_error(.REDIRECTS, .NONE, detail);
        }

        redirects += 1;

        next := resolve_location(url_text, location);

        if response.status == 303 || ((response.status == 301 || response.status == 302) && method == .POST) {
            if method != .HEAD method = .GET;

            body = .{};
        }

        if !same_origin(url_text, next) headers = without_credentials(headers);

        // An answer we are not going to read: drain it if it is short, so its connection is reused.
        close(*response);

        url_text = next;
    }

    return .{}, .{};
}

// `text` as an absolute URL: a relative one is joined to the client's base URL.
absolute_url :: (client: *Client, text: string) -> string, Client_Error {
    if find_index_from_left(text, "://") > 0 return text, .{};

    base := client.base_url;

    if base.count == 0 && client.transport.base base = client.transport.base(client.transport.data);

    if base.count == 0 {
        return "", client_error(.INVALID_URL, .NONE, tprint("`%`: it is not an absolute URL such as http://host/path, and the client has no base_url to join it to", text));
    }

    while base.count > 0 && base[base.count - 1] == #char "/" base.count -= 1;

    if text.count == 0 || text[0] == #char "/" return tprint("%%", base, text), .{};

    return tprint("%/%", base, text), .{};
}

// The request's headers and, for every name it does not set, the client's default.
with_default_headers :: (client: *Client, headers: [] Header_Field) -> [] Header_Field {
    if client.headers.count == 0 return headers;

    list: [..] Header_Field;

    for headers array_add(*list, it,, temp);

    for client.headers {
        _, found := find_header(headers, it.name);

        if !found array_add(*list, it,, temp);
    }

    return list;
}

same_origin :: (a: string, b: string) -> bool {
    first, problem_a := parse_url(a);
    second, problem_b := parse_url(b);

    if problem_a != .NONE || problem_b != .NONE return false;

    return equal_nocase(first.scheme, second.scheme) && equal_nocase(first.host, second.host) && first.port == second.port;
}

without_credentials :: (headers: [] Header_Field) -> [] Header_Field {
    list: [..] Header_Field;

    for headers {
        if equal_nocase(it.name, "Authorization") || equal_nocase(it.name, "Cookie") continue;

        array_add(*list, it,, temp);
    }

    return list;
}

// The URL a `Location` header means, relative to the URL it came with.
resolve_location :: (base: string, location: string) -> string {
    if find_index_from_left(location, "://") > 0 return copy_string(location,, temp);

    url, problem := parse_url(base);

    if problem != .NONE return copy_string(location,, temp);

    origin := tprint("%://%", url.scheme, host_with_port(url));

    if begins_with(location, "//") return tprint("%:%", url.scheme, location);
    if begins_with(location, "/") return tprint("%%", origin, location);

    slash := find_index_from_right(url.path, #char "/");
    directory := slice(url.path, 0, slash + 1);

    return tprint("%%%", origin, directory, location);
}

host_with_port :: (url: Url) -> string {
    host := ifx url.ipv6 then tprint("[%]", url.host) else url.host;

    if url.port == default_port(url) return host;

    return tprint("%:%", host, url.port);
}

// One request on one connection (a pooled one when there is one), and its response head.
exchange :: (client: *Client, method: Method, url_text: string, extra: [] Header_Field, body: [] u8, max_response_size: int, note: Request_Note) -> Client_Response, Client_Error {
    url, problem := parse_url(url_text);

    if problem != .NONE {
        if problem == .UNSUPPORTED_SCHEME {
            return .{}, client_error(.INVALID_URL, .NONE, tprint("`%`: the scheme `%` is not supported, use http or https", url_text, url.scheme));
        }

        what := ifx problem == .NOT_A_URL then "it is not an absolute URL such as http://host/path"
                else ifx problem == .NO_HOST then "it names no host"
                else ifx problem == .BAD_PORT then "the port is not a number up to 65535"
                else "the host is not valid";

        return .{}, client_error(.INVALID_URL, .NONE, tprint("`%`: %", url_text, what));
    }

    headers: [..] Header_Field;

    host_given := false;
    agent_given := false;

    for extra {
        if it.id == .HOST host_given = true;
        if it.id == .USER_AGENT agent_given = true;

        array_add(*headers, it,, temp);
    }

    if !host_given {
        host: Header_Field;

        host.name = "Host";
        host.value = host_with_port(url);
        host.id = .HOST;

        array_add(*headers, host,, temp);
    }

    if !agent_given {
        agent: Header_Field;

        agent.name = "User-Agent";
        agent.value = client_settings.user_agent;
        agent.id = .USER_AGENT;

        array_add(*headers, agent,, temp);
    }

    method_text := method_name(method);

    idempotent := method == .GET || method == .HEAD || method == .PUT || method == .DELETE || method == .OPTIONS;

    attempt := 0;

    while true {
        stream, reused, error := checkout(client, url);

        if error.kind != .NONE return .{}, error;

        response, reply_started, failure := converse(client, stream, method, method_text, url, headers, body, max_response_size, note);

        if failure.kind == .NONE return response, .{};

        close(stream);

        // A pooled connection the server closed while it sat idle fails before any answer came:
        // that is not the request's fault, so try once more on a fresh connection.
        if reused && !reply_started && idempotent && attempt == 0 && failure.kind == .TRANSPORT {
            free_error(*failure);

            attempt += 1;

            continue;
        }

        return .{}, failure;
    }

    return .{}, .{};
}

// Writes the request on `stream` and reads the response up to the end of its head. The body is
// left on the stream, in the response's `source`. `reply_started` says whether any byte of an
// answer arrived.
converse :: (client: *Client, stream: Link, method: Method, method_text: string, url: Url, headers: [] Header_Field, body: [] u8, max_response_size: int, note: Request_Note) -> Client_Response, reply_started: bool, Client_Error {
    out: [..] u8;

    defer array_reset(*out);

    _, ok := write_request_head(*out, method_text, url.target, headers, body.count);

    if !ok {
        return .{}, false, client_error(.INVALID_URL, .NONE, "a request header is not valid (a bad name or value, or one that sets the framing itself)");
    }

    append_bytes(*out, body.data, body.count);

    _, write_error := Io.write(stream.writer, out);

    if write_error != .NONE return .{}, false, io_failure(write_error, "sending the request");

    source := take_source();
    success := false;

    defer {
        if !success discard_source(source);
    }

    source.stream = stream;
    source.owner = client.transport.data;
    source.host = copy_string(url.host);
    source.port = url.port;
    source.tls = tls_config_of(client, url);
    source.max_size = max_response_size;

    // The parser does not limit the body: `read_text` and friends do, and `read_some` streams.
    limits: Limits;

    limits.max_body_size = -1;

    parser_init(*source.parser, .RESPONSE, limits);

    source.parser.response_to = method;

    status := 0;
    reason: string;
    headers_list: [..] Header_Field;
    started := false;
    got_head := false;

    defer array_reset(*headers_list);

    while !got_head {
        count, read_error := Io.read(stream.reader, source.buffer);

        if read_error != .NONE return .{}, started, io_failure(read_error, "reading the response");

        if count == 0 {
            return .{}, started, client_error(.TRANSPORT, .END_OF_FILE, "the server closed the connection before the response was complete");
        }

        started = true;

        data: [] u8 = .{ count, source.buffer.data };

        while !got_head {
            used, event := parse(*source.parser, data);

            data = advance(data, used);

            if event.kind == .NEED_MORE break;

            if event.kind == .HEAD {
                if source.parser.head.informational continue;

                status = source.parser.head.status;
                reason = source.parser.head.reason;

                // The head's strings die with the parser's next step: keep copies.
                for source.parser.head.headers array_add(*headers_list, it);

                got_head = true;

                continue;
            }

            if event.kind == .ERROR {
                return .{}, started, client_error(.TRANSPORT, .INVALID, tprint("the response is not valid HTTP: %", source.parser.error.message));
            }

            if event.kind == .SWITCHED {
                return .{}, started, client_error(.TRANSPORT, .INVALID, "the server switched protocols, which the client does not do");
            }
        }

        source.input = data;
    }

    source.kind = source.parser.head.body;
    source.length = source.parser.head.content_length;
    source.keep_alive = source.parser.head.keep_alive;
    source.held = true;

    response := build_response(status, reason, headers_list, note);

    response.source = source;

    success = true;

    // No body to wait for (HEAD, 204, 304, Content-Length: 0), or all of it came with the head
    // (a small answer): take it into memory, and the connection is free at once. A response
    // that is dropped unread then costs nothing.
    arrived := source.kind == .NONE || (source.kind == .LENGTH && source.length <= source.input.count && source.length <= source.max_size);

    if arrived {
        failure := read_all(source, *source.stored);

        if failure.kind != .NONE {
            source.deferred = failure;
            source.failed = true;
        } else {
            source.has_stored = true;
        }
    }

    // Everything that arrived is in memory (or the read failed): the buffer is not needed.
    if arrived release_buffer(source);

    if source.held register_source(source);

    return response, true, .{};
}

io_failure :: (error: Io.Io_Error, what: string) -> Client_Error {
    if error == .TIMED_OUT || error == .CANCELED {
        return client_error(.TIMED_OUT, error, tprint("timed out while %", what));
    }

    return client_error(.TRANSPORT, error, tprint("%: %", what, error));
}

REQUEST_BODY_KEPT :: 256;

// One allocation for the headers, their text, the reason and the request's summary.
build_response :: (status: int, reason: string, headers: [] Header_Field, note: Request_Note) -> Client_Response {
    kept := min(note.body.count, REQUEST_BODY_KEPT);
    size := headers.count * size_of(Header_Field) + reason.count + note.url.count + kept + 16;

    for headers size += it.name.count + it.value.count;

    memory := cast(*u8) alloc(size);

    response: Client_Response;

    response.status = status;
    response.storage = memory;

    cursor := memory;

    list := cast(*Header_Field) cursor;

    cursor += headers.count * size_of(Header_Field);

    take :: (text: string, cursor: **u8) -> string #expand {
        result: string;

        result.data = cursor.*;
        result.count = text.count;

        memcpy(cursor.*, text.data, text.count);

        cursor.* += text.count;

        return result;
    }

    for headers {
        list[it_index].name = take(it.name, *cursor);
        list[it_index].value = take(it.value, *cursor);
        list[it_index].id = it.id;
    }

    response.headers = .{ headers.count, list };
    response.reason = take(reason, *cursor);
    response.method = note.method;
    response.url = take(note.url, *cursor);
    response.request_body = take(string.{ kept, note.body.data }, *cursor);
    response.request_body_size = note.body.count;

    return response;
}

// ---------------------------------------------------------------------------------------------
// The body of a response
//
// A `Body_Source` is what is left of a response once its head is parsed: the connection, the
// parser positioned at the start of the body, and the bytes read past the head. It sits on the
// heap so every copy of the `Client_Response` shares it. It reads through the connection's `Io.Reader`
// procedures only, so it does not care what the connection is.

Body_Source :: struct {
    // The connection, and what it is pooled under.
    stream: Link;
    owner: *void;
    host: string;
    port: u16;
    tls: *Tls.Tls_Config; // null: plain http
    held := false; // the connection is checked out: this body is still on the wire

    parser: Parser;
    kind: Body_Kind;
    length: s64;
    keep_alive: bool;
    buffer: [] u8; // what the stream is read into
    input: [] u8; // the part of it the parser has not seen
    pending: [] u8; // body bytes the parser returned and the reader has not taken
    received: s64; // body bytes the parser has returned
    finished := false; // the parser reached the end of the message

    // The body held in memory (read ahead by an expectation, or by a call outside a scheduler).
    stored: [..] u8;
    stored_at: int;
    has_stored := false;

    // Where the reader is.
    done := false; // the body was handed over completely
    streamed := false; // `read_some` was used
    reading := false; // a read is waiting on the stream
    failed := false; // a read failed: the connection is gone
    deferred: Client_Error; // the failure to report on the first read
    released := false; // closed or evicted before the end
    why: string;

    sample: [..] u8; // the first bytes handed over, for failure messages
    sample_total: int;

    // The scheduler's pool gave it out and takes it back (`take_source`, `discard_source`).
    pooled: bool;

    max_size: int;
    deadline_ms: s64; // an absolute `now_ms()` (0: none)
}

SAMPLE_KEPT :: 512;

// How much of a body `close` reads and throws away to keep a connection, and for how long.
DRAIN_LIMIT :: 64 * 1024;
DRAIN_TIMEOUT :: 500;

// A response state with a read buffer: recycled ones come from the scheduler's pool. A call made
// outside a scheduler gets new ones every time, since its response outlives the scheduler.
take_source :: () -> *Body_Source {
    pool := client_pool();
    source: *Body_Source;

    if pool.recycle && pool.sources.count > 0 {
        source = pool.sources[pool.sources.count - 1];
        pool.sources.count -= 1;
    } else {
        source = New(Body_Source);
        source.pooled = pool.recycle;

        pool.sources_created += 1;
    }

    if pool.recycle && pool.buffers.count > 0 {
        source.buffer = .{ SOURCE_BUFFER_BYTES, pool.buffers[pool.buffers.count - 1] };
        pool.buffers.count -= 1;
    } else {
        source.buffer = NewArray(SOURCE_BUFFER_BYTES, u8, false);
    }

    return source;
}

// What `read_some` reads the connection into, and the most sources and buffers a scheduler keeps
// for reuse (each).
SOURCE_BUFFER_BYTES :: 16 * 1024;
SOURCES_KEPT :: 64;

// The read buffer is not needed once the body is in memory or the connection is gone: it goes back
// to the scheduler's pool (or is freed, outside a scheduler), so a response that is held but
// already complete costs only its state.
release_buffer :: (source: *Body_Source) {
    if !source.buffer.data return;

    // Nothing may still point into it.
    source.input = .{};
    source.pending = .{};

    if Io.in_scheduler() {
        pool := client_pool();

        if pool.recycle && pool.buffers.count < SOURCES_KEPT {
            array_add(*pool.buffers, source.buffer.data);
            source.buffer = .[];

            return;
        }
    }

    free(source.buffer.data);

    source.buffer = .[];
}

// Frees the memory of a source, or puts it back in the pool; the connection must be released first.
discard_source :: (source: *Body_Source) {
    parser_free(*source.parser);
    release_buffer(source);

    if source.host.count free(source.host.data);

    array_reset(*source.stored);
    array_reset(*source.sample);

    free_error(*source.deferred);

    // Back to the pool of the scheduler it is released on (any: it is plain memory), unless that
    // is full or there is none, which is the case for a response from a call outside a scheduler.
    if source.pooled && Io.in_scheduler() {
        pool := client_pool();

        if pool.recycle && pool.sources.count < SOURCES_KEPT {
            source.* = .{};
            source.pooled = true;

            array_add(*pool.sources, source);

            return;
        }
    }

    free(source);
}

// The connection is of no further use: close it.
drop_connection :: (source: *Body_Source) {
    if !source.held return;

    unregister_source(source);
    close(source.stream);

    source.held = false;
}

// A read failed: close the connection and remember that.
fail_source :: (source: *Body_Source, error: Client_Error) -> Client_Error {
    drop_connection(source);

    source.failed = true;

    return error;
}

// The body has been received to its end: the connection goes back to the pool when the framing
// allows another request on it.
connection_done :: (source: *Body_Source) {
    if !source.held return;

    unregister_source(source);

    source.held = false;

    if source.keep_alive && source.kind != .UNTIL_CLOSE && source.input.count == 0 {
        checkin(source.owner, source.host, source.port, source.tls, source.stream);
    } else {
        close(source.stream);
    }
}

read_wire :: (source: *Body_Source) -> int, Io.Io_Error {
    remaining := Io.NO_DEADLINE;

    if source.deadline_ms > 0 {
        remaining = source.deadline_ms - now_ms();

        if remaining <= 0 return 0, .TIMED_OUT;
    }

    count: s64;
    error: Io.Io_Error;

    Io.with_deadline(remaining, #code {
        count, error = Io.read(source.stream.reader, source.buffer);
    });

    return count, error;
}

// Makes `source.pending` hold body bytes, reading from the connection when it has none. Returns
// true when the body has ended instead (and the connection has been dealt with). On an error the
// connection is left for the caller to close.
fill_pending :: (source: *Body_Source) -> bool, Client_Error {
    while source.pending.count == 0 {
        if source.finished return true, .{};

        used, event := parse(*source.parser, source.input);

        source.input = advance(source.input, used);

        if event.kind == {
            case .BODY;
                source.pending = event.data;
                source.received += event.data.count;

            case .END;
                source.finished = true;

                connection_done(source);

                return true, .{};

            case .NEED_MORE;
                count, read_error := read_wire(source);

                if read_error != .NONE return false, io_failure(read_error, "reading the response body");

                if count == 0 {
                    end := parse_eof(*source.parser);

                    if end.kind != .END {
                        return false, client_error(.TRANSPORT, .END_OF_FILE, "the server closed the connection before the response body was complete");
                    }

                    source.finished = true;

                    connection_done(source);

                    return true, .{};
                }

                source.input = .{ count, source.buffer.data };

            case .ERROR;
                return false, client_error(.TRANSPORT, .INVALID, tprint("the response body is not valid HTTP: %", source.parser.error.message));

            case;
                return false, client_error(.TRANSPORT, .INVALID, "the response body is not valid HTTP: an unexpected message");
        }
    }

    return false, .{};
}

note_sample :: (source: *Body_Source, data: [] u8) {
    source.sample_total += data.count;

    room := SAMPLE_KEPT - source.sample.count;

    if room > 0 append_bytes(*source.sample, data.data, min(room, data.count));
}

// Appends the rest of the body to `into`.
read_all :: (source: *Body_Source, into: *[..] u8) -> Client_Error {
    if source.kind == .LENGTH && source.length - source.received + into.count > source.max_size {
        drop_connection(source);

        source.failed = true;

        return client_error(.TOO_LARGE, .NONE, tprint("the body is % bytes, more than the limit of % (max_response_size)", source.length, source.max_size));
    }

    while true {
        eof, error := fill_pending(source);

        if error.kind != .NONE return fail_source(source, error);

        if eof break;

        if into.count + source.pending.count > source.max_size {
            drop_connection(source);

            source.failed = true;

            return client_error(.TOO_LARGE, .NONE, tprint("the body is longer than the limit of % bytes (max_response_size)", source.max_size));
        }

        append_bytes(into, source.pending.data, source.pending.count);
        note_sample(source, source.pending);

        source.pending = .{};
    }

    return .{};
}

take_stored :: (source: *Body_Source) -> [..] u8 {
    data := source.stored;

    if source.stored_at > 0 {
        memcpy(data.data, data.data + source.stored_at, data.count - source.stored_at);

        data.count -= source.stored_at;
    }

    note_sample(source, .{ data.count, data.data });

    source.stored = .{};
    source.stored_at = 0;
    source.has_stored = false;

    return data;
}

// The source if the body can be read, else why not.
readable :: (response: *Client_Response) -> *Body_Source, Client_Error {
    source := response.source;

    if !source {
        return null, client_error(.BODY, .NONE, "the response has no body to read: it was closed, or the request failed");
    }

    if source.deferred.kind != .NONE {
        error := source.deferred;

        source.deferred = .{};

        return null, error;
    }

    if source.released return null, client_error(.BODY, .NONE, source.why);

    if source.failed return null, client_error(.BODY, .NONE, "an earlier read of this body failed");

    if source.done return null, client_error(.BODY, .NONE, "the body was already read: a body can be read once");

    return source, .{};
}

// The body held in memory without using it up, for the expectations: the same bytes can be
// shown in a failure and read again. The second value says why it is not available.
peek_body :: (response: *Client_Response) -> string, string {
    source := response.source;

    if !source return "", "the response has no body";

    if source.has_stored return string.{ source.stored.count - source.stored_at, source.stored.data + source.stored_at }, "";

    if source.deferred.kind != .NONE return "", error_text(source.deferred);

    if source.released return "", source.why;

    if source.failed return "", "an earlier read of this body failed";

    if source.done return "", "the body was already read";

    source.reading = true;

    error := read_all(source, *source.stored);

    source.reading = false;

    if error.kind != .NONE {
        text := error_text(error);

        free_error(*error);

        return "", text;
    }

    source.has_stored = true;

    return string.{ source.stored.count, source.stored.data }, "";
}

// Gives up the connection of an unread body: reads and drops a short rest so the connection can
// be pooled, else closes it.
release_source :: (source: *Body_Source) {
    if !source.held return;

    if source.finished || source.kind == .UNTIL_CLOSE || !source.keep_alive || source.failed || (source.kind == .LENGTH && source.length - source.received > DRAIN_LIMIT) {
        drop_connection(source);

        return;
    }

    total := 0;

    source.reading = true;

    Io.with_deadline(DRAIN_TIMEOUT, #code {
        while source.held {
            eof, error := fill_pending(source);

            if error.kind != .NONE {
                free_error(*error);

                break;
            }

            if eof break;

            total += source.pending.count;
            source.pending = .{};

            if total > DRAIN_LIMIT break;
        }
    });

    source.reading = false;

    // Whatever is left over is not worth waiting for.
    drop_connection(source);
}

// A connection held by a response that was neither read nor closed is part of the scheduler's
// list. There is a bounded number of those: the oldest is closed when a new one would pass it.
register_source :: (source: *Body_Source) {
    pool := client_pool();

    array_add(*pool.open, source);

    index := 0;

    while pool.open.count > client_settings.max_open_responses && index < pool.open.count - 1 {
        old := pool.open[index];

        if old.reading {
            index += 1;

            continue;
        }

        drop_connection(old);

        old.released = true;
        old.why = "the connection was closed because too many responses were open at once (client_settings.max_open_responses): read bodies to the end or close responses";
    }
}

unregister_source :: (source: *Body_Source) {
    pool := client_pool();

    for pool.open {
        if it == source {
            array_ordered_remove_by_index(*pool.open, it_index);

            return;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The connection pool

Pool_Entry :: struct {
    host: string;
    port: u16;
    tls: *Tls.Tls_Config; // null for plain http: a TLS connection is only reused under the same config
    owner: *void; // the transport's data: a connection is only reused by the transport that made it
    stream: Link;
    since: s64;
}

Client_Pool :: struct {
    idle: [..] Pool_Entry;
    opened: int;
    open: [..] *Body_Source; // responses holding a connection, oldest first

    // Response bodies' state and read buffers, kept for the next response.
    sources: [..] *Body_Source;
    buffers: [..] *u8;
    sources_created: int;
    recycle := true; // false in the scheduler of a call made outside one
}

pool_key: s32;

client_pool :: () -> *Client_Pool {
    pool := cast(*Client_Pool) Io.scheduler_local(*pool_key);

    if pool return pool;

    pool = New(Client_Pool);

    Io.set_scheduler_local(*pool_key, pool, drop_pool);

    return pool;
}

drop_pool :: (value: *void) {
    pool := cast(*Client_Pool) value;

    for pool.idle {
        close(it.stream, now = true);
        free(it.host.data);
    }

    // Responses that were never read or closed lose their connection with the scheduler.
    for pool.open {
        close(it.stream, now = true);

        it.held = false;
        it.released = true;
        it.why = "the scheduler ended before the body was read";
    }

    for pool.sources free(it);
    for pool.buffers free(it);

    array_reset(*pool.idle);
    array_reset(*pool.open);
    array_reset(*pool.sources);
    array_reset(*pool.buffers);
    free(pool);
}

// A connection to the URL's host: an idle one from the pool, or a new one.
checkout :: (client: *Client, url: Url) -> Link, reused: bool, Client_Error {
    pool := client_pool();
    now := now_ms();
    owner := client.transport.data;
    tls := tls_config_of(client, url);

    // Newest first; connections that waited too long are closed on the way.
    index := pool.idle.count - 1;

    while index >= 0 {
        entry := pool.idle[index];

        if now - entry.since > client_settings.idle_timeout {
            drop_entry(pool, index);
        } else if entry.owner == owner && entry.port == url.port && entry.tls == tls && entry.host == url.host {
            stream := entry.stream;

            free(entry.host.data);
            array_ordered_remove_by_index(*pool.idle, index);

            return stream, true, .{};
        }

        index -= 1;
    }

    stream: Link;
    error: Io.Io_Error;

    if client.transport.open {
        stream, error = client.transport.open(owner, url);
    } else {
        address: Io.Address;
        tcp: Io.Tcp_Stream;

        address, error = Io.resolve(url.host, url.port);

        if error == .NONE tcp, error = Io.connect(address);

        if error == .NONE stream = tcp_link(tcp);
    }

    if error != .NONE {
        return .{}, false, io_failure(error, tprint("connecting to %:%", url.host, url.port));
    }

    if url.secure {
        secured, failure := secure_link(client, stream, url);

        if failure.kind != .NONE {
            close(stream);

            return .{}, false, failure;
        }

        stream = secured;
    }

    pool.opened += 1;

    return stream, false, .{};
}

drop_entry :: (pool: *Client_Pool, index: int) {
    entry := pool.idle[index];

    // Out of the pool before closing: a close can wait (io_uring), and other fibers may use the
    // pool meanwhile.
    array_ordered_remove_by_index(*pool.idle, index);
    free(entry.host.data);
    close(entry.stream);
}

checkin :: (owner: *void, host: string, port: u16, tls: *Tls.Tls_Config, stream: Link) {
    pool := client_pool();

    if client_settings.max_idle_per_host <= 0 {
        close(stream);

        return;
    }

    entry: Pool_Entry;

    entry.host = copy_string(host);
    entry.port = port;
    entry.tls = tls;
    entry.owner = owner;
    entry.stream = stream;
    entry.since = now_ms();

    array_add(*pool.idle, entry);

    // Too many for this host: the oldest ones go. They leave the pool before any close, since a
    // close can wait and let other fibers check connections in meanwhile.
    count := 0;

    for pool.idle {
        if it.owner == owner && it.port == port && it.tls == tls && it.host == host count += 1;
    }

    excess: [..] Link;
    excess.allocator = temp;

    index := 0;

    while count > client_settings.max_idle_per_host && index < pool.idle.count {
        it := pool.idle[index];

        if it.owner == owner && it.port == port && it.tls == tls && it.host == host {
            array_add(*excess, it.stream);
            free(it.host.data);
            array_ordered_remove_by_index(*pool.idle, index);
            count -= 1;
        } else {
            index += 1;
        }
    }

    for excess close(it);
}

// ---------------------------------------------------------------------------------------------
// TLS

// The config an `https://` URL's connection is made with (null for plain http): the client's own,
// or the program's default.
tls_config_of :: (client: *Client, url: Url) -> *Tls.Tls_Config {
    if !url.secure return null;

    if client.tls return client.tls;

    return *default_tls;
}

// The default for clients with no `tls`: system roots, ALPN `http/1.1`. Built once by whichever
// scheduler thread needs it first; the others wait for it.
default_tls: Tls.Tls_Config;
default_tls_error: Tls.Tls_Error;
default_tls_state: s32; // 0: not built, 1: being built, 2: ready, 3: failed

default_tls_ready :: () -> Tls.Tls_Error {
    while true {
        state := atomic_load(*default_tls_state);

        if state == 2 return .{};
        if state == 3 return default_tls_error;

        if state == 0 {
            exchanged, _ := atomic_compare_exchange(*default_tls_state, 0, 1);

            if exchanged {
                default_tls.alpn = string.["http/1.1"];
                default_tls_error = Tls.tls_config_init(*default_tls);

                atomic_store(*default_tls_state, ifx default_tls_error.kind == .NONE then 2 else 3);

                continue;
            }
        }

        Io.sleep(1);
    }

    return .{};
}

// Starts TLS on a new connection: the handshake runs on the calling fiber, under the request's
// deadline. The link it returns replaces `link` (it closes the connection underneath); on a
// failure `link` is still the caller's to close.
secure_link :: (client: *Client, link: Link, url: Url) -> Link, Client_Error {
    config := tls_config_of(client, url);

    if config == *default_tls {
        error := default_tls_ready();

        if error.kind != .NONE return .{}, tls_failure(error, url);
    } else if !config.native {
        return .{}, client_error(.TLS, .NONE, "the client's tls config was not set up: call Tls.tls_config_init on it first");
    }

    stream, error := Tls.tls_connect(config, link.reader, link.writer, url.host);

    if error.kind != .NONE return .{}, tls_failure(error, url);

    secured: Link;

    secured.reader = Tls.reader_of(stream);
    secured.writer = Tls.writer_of(stream);
    secured.peer = link.peer;

    return secured, .{};
}

tls_failure :: (error: Tls.Tls_Error, url: Url) -> Client_Error {
    // A timeout or cancellation is the request's, like any other wait.
    if error.kind == .IO return io_failure(error.io, tprint("the TLS handshake with %:%", url.host, url.port));

    return client_error(.TLS, .NONE, tprint("TLS with %:%: %", url.host, url.port, Tls.error_text(error)));
}
