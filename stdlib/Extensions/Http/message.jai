// What a handler sees and returns: the request, the response, and the error a handler can answer
// with instead. Everything here is plain data plus small builders.

// A parsed request, as handlers and middleware see it. The strings (target, header names and
// values) stay valid until the response has been written: copy what has to outlive the request.
// The body is not here: take a `*Body` parameter to read it (see `Body`), or `Json(T)`, `Form(T)`.
Request :: struct {
    method: Method;
    method_text: string; // the spelling on the wire, for methods the codec does not know
    target: string; // the request target as sent: `/users/7?x=1`
    path: string; // the target without its query, still percent-encoded
    query: string; // the text after `?`, still percent-encoded
    version: Version;
    headers: [] Header_Field;
    peer: Io.Address;
}

// A handler after its extractors are built: `(request) -> Response`. Routes store one per route.
Handler :: #type (request: *Request) -> Response;

// The rest of the middleware chain and then the handler: `next(request)` runs it and returns its
// response, which the middleware may change or replace.
Next :: #type (request: *Request) -> Response;

// A request filter: log, check a token, add headers, or answer itself without calling `next`.
//
// ```jai
// require_token :: (request: *Request, next: Next) -> Response {
//     token, found := header(request, "Authorization");
//     if !found return error_response(request, unauthorized("a token is required"));
//
//     return next(request);
// }
// ```
Middleware :: #type (request: *Request, next: Next) -> Response;

// The value of the first header called `name` (any case), and whether there was one.
header :: (request: *Request, name: string) -> string, bool {
    return find_header(request.headers, name);
}

// The first query parameter called `name`, percent-decoded, and whether it was given.
query_param :: (request: *Request, name: string) -> string, bool {
    for query_pairs(request) {
        if it.key == name return it.value, true;
    }

    return "", false;
}

// The state of the app that serves `request` (`app.state`), or null when it was never set. For
// middleware; handlers take a `*State(T)` instead.
app_state :: (request: *Request, $T: Type) -> *T {
    return cast(*T) frame_of(request).app.state;
}

// `1.2.3.4:5678` or `[::1]:5678`.
peer_text :: (request: *Request) -> string {
    return Io.address_to_string(request.peer);
}

// Responses

// What a handler returns when it wants full control. The headers and body must stay valid until
// the response is written: use string literals or the temporary allocator (`tprint`, `,, temp`),
// which the server resets after each response. The builders below do.
Response :: struct {
    status := 200;
    headers: [..] Header_Field;
    body: [] u8;

    // Set by `stream_response` and `file_response`: the body is produced while it is sent.
    source: *Response_Source;
}

// `response` with one more header: `with_header(json(user), "ETag", tag)`. The name and value
// must stay valid until the response is written.
//
// - name: the header name.
// - value: the header value.
with_header :: (response: Response, name: string, value: string) -> Response {
    result := response;

    // The copy shares the header array; give it one of its own.
    result.headers = .{};

    for response.headers array_add(*result.headers, it,, temp);

    add_header(*result, name, value);

    return result;
}

// An empty response with only a status (`status_only(204)`).
status_only :: (status: int) -> Response {
    response: Response;

    response.status = status;

    return response;
}

// `value` as JSON with `Content-Type: application/json`. The body is built in the temporary
// allocator.
//
// - value: any value the Json module can encode.
// - status: the HTTP status code.
json :: (value: $T, status := 200) -> Response {
    response: Response;

    text := to_json(value,, temp);

    response.status = status;
    response.body = .{ text.count, text.data };

    add_header(*response, "Content-Type", "application/json");

    return response;
}

// `value` as JSON with status 201 and, when `location` is given, a `Location` header: the answer
// to a request that made something.
//
// ```jai,ignore
// return created(user, tprint("/users/%", user.id));
// ```
created :: (value: $T, location := "") -> Response {
    response := json(value, 201);

    if location.count add_header(*response, "Location", location);

    return response;
}

// Plain text with `Content-Type: text/plain; charset=utf-8`.
//
// - body: sent as it is.
// - status: the HTTP status code.
text :: (body: string, status := 200) -> Response {
    return bytes_response(body, "text/plain; charset=utf-8", status);
}

// HTML with `Content-Type: text/html; charset=utf-8`.
//
// - body: sent as it is. Nothing is escaped.
// - status: the HTTP status code.
html :: (body: string, status := 200) -> Response {
    return bytes_response(body, "text/html; charset=utf-8", status);
}

// Raw bytes (a `[] u8` or a `string`) with a content type of your choice. The bytes must stay
// valid until the response is written.
//
// - body: a `[] u8` or a `string`, sent as it is.
// - content_type: the `Content-Type` header value.
// - status: the HTTP status code.
bytes_response :: (body: $T, content_type: string, status := 200) -> Response {
    response: Response;

    response.status = status;

    #if T == string {
        response.body = .{ body.count, body.data };
    } else {
        response.body = body;
    }

    add_header(*response, "Content-Type", content_type);

    return response;
}

// A redirect to `location`: 302 unless you pass 301, 303, 307 or 308.
//
// - location: the URL to go to, as an absolute URL or a path.
// - status: 301, 302, 303, 307 or 308. Use 303 after a POST, 307 or 308 to keep the method.
redirect :: (location: string, status := 302) -> Response {
    response: Response;

    response.status = status;

    add_header(*response, "Location", location);

    return response;
}

// Errors

// Whether an `Http_Error` is set: `.NONE` (a handler's `.{}`) or `.STATUS`, an HTTP status to answer with.
Http_Error_Kind :: enum u8 {
    NONE; // no error: `.{}`
    STATUS; // an HTTP status, from a handler
}

// What a handler returns to answer with a status instead of a value. A handler's `.{}` means
// no error. The server sends it as an RFC 9457 `application/problem+json` document:
//
// ```jai,ignore
// return .{}, not_found("No user with id %", id.value);
// // 404 {"type":"about:blank","title":"Not Found","status":404,"detail":"No user with id 7"}
// ```
//
// `app.on_error` changes the format for all of them.
Http_Error :: struct {
    kind: Http_Error_Kind;
    status: int; // the HTTP status
    title: string; // short and fixed per status; the reason phrase when empty
    detail: string; // what happened this time
    problem_type: string; // a URI naming the kind of problem; "about:blank" when empty
    location: string; // for bad input: `body`, `query`, `path`, `header` or `form`
    field: string; // for bad input: the parameter, header or JSON path that was wrong
    headers: [] Header_Field; // sent with the error (`Retry-After`, `WWW-Authenticate`); see with_header
}

// An error with a status and a detail text (used as it is). The named helpers below take a
// `tprint` format instead: `not_found("No user %", id)`.
//
// - status: the HTTP status code.
// - detail: text for the client, used as it is.
http_error :: (status: int, detail := "") -> Http_Error {
    error: Http_Error;

    error.kind = .STATUS;
    error.status = status;
    error.title = status_reason(status);
    error.detail = detail;

    return error;
}

// An `Http_Error` with status 400, for when the request is malformed or its content is invalid.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
bad_request :: (format := "", args: ..Any) -> Http_Error {
    return http_error(400, tprint(format, ..args));
}

// An `Http_Error` with status 401, for when the caller has not authenticated. Add `WWW-Authenticate` with `with_header`.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
//
// ```jai,skip_on(wasm)
// error := with_header(unauthorized("a token is required"), "WWW-Authenticate", "Bearer");
//
// assert(error.status == 401);
// assert(error.headers.count == 1);
// ```
unauthorized :: (format := "", args: ..Any) -> Http_Error {
    return http_error(401, tprint(format, ..args));
}

// An `Http_Error` with status 403, for when the caller is known but not allowed.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
forbidden :: (format := "", args: ..Any) -> Http_Error {
    return http_error(403, tprint(format, ..args));
}

// An `Http_Error` with status 404, for when there is nothing at the requested resource.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
//
// ```jai,skip_on(wasm)
// error := not_found("No user %", 7);
//
// assert(error.status == 404);
// assert(error.title == "Not Found");
// assert(error.detail == "No user 7");
// ```
not_found :: (format := "", args: ..Any) -> Http_Error {
    return http_error(404, tprint(format, ..args));
}

// An `Http_Error` with status 409, for when the request clashes with the current state, such as a duplicate.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
conflict :: (format := "", args: ..Any) -> Http_Error {
    return http_error(409, tprint(format, ..args));
}

// An `Http_Error` with status 410, for when the resource existed and has been removed for good.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
gone :: (format := "", args: ..Any) -> Http_Error {
    return http_error(410, tprint(format, ..args));
}

// An `Http_Error` with status 422, for when the request is well-formed but its content cannot be processed.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
unprocessable :: (format := "", args: ..Any) -> Http_Error {
    return http_error(422, tprint(format, ..args));
}

// An `Http_Error` with status 429, for when the caller is over a rate limit; add `Retry-After` with `with_header`.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
too_many_requests :: (format := "", args: ..Any) -> Http_Error {
    return http_error(429, tprint(format, ..args));
}

// An `Http_Error` with status 500, for when something failed that is not the caller's fault.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
internal_error :: (format := "", args: ..Any) -> Http_Error {
    return http_error(500, tprint(format, ..args));
}

// An `Http_Error` with status 503, for when the service cannot answer now; add `Retry-After` with `with_header`.
// `format` and `args` make the detail text as `tprint` does, in the temporary allocator.
unavailable :: (format := "", args: ..Any) -> Http_Error {
    return http_error(503, tprint(format, ..args));
}

// `error` with one more header (`Retry-After`, `WWW-Authenticate`...).
//
// - name: the header name.
// - value: the header value.
with_header :: (error: Http_Error, name: string, value: string) -> Http_Error {
    result := error;
    list: [..] Header_Field;

    for error.headers array_add(*list, it,, temp);

    header: Header_Field;

    header.name = name;
    header.value = value;
    header.id = lookup_header_name(name);

    array_add(*list, header,, temp);

    result.headers = list;

    return result;
}

// A one-line description of any error, for logs and messages.
error_text :: (error: Http_Error) -> string {
    if error.kind == .NONE return "no error";

    if error.kind == .STATUS {
        if error.detail.count return tprint("% %: %", error.status, error.title, error.detail);

        return tprint("% %", error.status, error.title);
    }

    return error.detail;
}

// The response for an error: what `app.on_error` returns, or the problem document. Middleware use
// it to answer without calling `next`.
error_response :: (request: *Request, error: Http_Error) -> Response {
    app := frame_of(request).app;

    if app && app.on_error return app.on_error(error, request);

    return problem_response(request, error);
}

// The default format, RFC 9457: `application/problem+json` with `type`, `title`, `status`,
// `detail`, `instance` (the request path) and, for bad input, `location` and `field`.
problem_response :: (request: *Request, error: Http_Error) -> Response {
    builder: String_Builder;

    append(*builder, "{\"type\":");
    append_json_string(*builder, ifx error.problem_type.count then error.problem_type else "about:blank");

    title := error.title;
    if title.count == 0 title = status_reason(error.status);

    append(*builder, ",\"title\":");
    append_json_string(*builder, title);

    print(*builder, ",\"status\":%", error.status);

    if error.detail.count {
        append(*builder, ",\"detail\":");
        append_json_string(*builder, error.detail);
    }

    if request && request.path.count {
        append(*builder, ",\"instance\":");
        append_json_string(*builder, request.path);
    }

    if error.location.count {
        append(*builder, ",\"location\":");
        append_json_string(*builder, error.location);
    }

    if error.field.count {
        append(*builder, ",\"field\":");
        append_json_string(*builder, error.field);
    }

    append(*builder, "}");

    body := builder_to_string(*builder,, temp);

    response: Response;

    response.status = error.status;
    response.body = .{ body.count, body.data };

    add_header(*response, "Content-Type", "application/problem+json");

    for error.headers add_header(*response, it.name, it.value);

    return response;
}

// The error for bad input found by an extractor: names where the input came from and which part.
//
// - status: usually 400 or 422.
// - location: where the bad input came from, such as `"query"` or `"body"`.
// - field: which part of it was bad.
// - detail: what is wrong with it.
input_error :: (status: int, location: string, field: string, detail: string) -> Http_Error {
    error := http_error(status);

    error.detail = detail;
    error.location = location;
    error.field = field;

    return error;
}

#scope_module

// The request as the server keeps it: the public `Request` is its first member, so a `*Request`
// handed to a handler or middleware is also a `*Request_Frame`. Only the server makes requests.
Request_Frame :: struct {
    request: Request;

    // The values of the route's `{params}`, in template order, percent-decoded.
    params: [] Pair;

    // The app that is serving it, for state and configuration.
    app: *App;

    // Middleware chain position and the decoded query.
    chain: [] Middleware;
    cursor: int;
    endpoint: Handler;
    query_pairs: [] Pair;
    query_parsed: bool;

    body: Body_Frame;
}

frame_of :: (request: *Request) -> *Request_Frame {
    return cast(*Request_Frame) request;
}

// The decoded pairs of the query string (`?a=1&b=two`), parsed on first use.
query_pairs :: (request: *Request) -> [] Pair {
    frame := frame_of(request);

    if !frame.query_parsed {
        frame.query_pairs = parse_query(request.query,, temp);
        frame.query_parsed = true;
    }

    return frame.query_pairs;
}

// Adds a header to a response being built. The name and value must stay valid until the response
// is written.
add_header :: (response: *Response, name: string, value: string) {
    header: Header_Field;

    header.name = name;
    header.value = value;
    header.id = lookup_header_name(name);

    array_add(*response.headers, header,, temp);
}

#scope_export
