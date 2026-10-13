// What a handler sees and returns: the request, the response, and the error a handler can answer
// with instead. Everything here is plain data plus small builders.

// A parsed request, as handlers and middleware see it. The strings (target, header names and
// values) stay valid until the response has been written; the body is a view of the buffered
// request body. Copy what has to outlive the request.
Request :: struct {
    method: Method;
    method_text: string; // the spelling on the wire, for methods the codec does not know
    target: string; // the request target as sent: `/users/7?x=1`
    path: string; // the target without its query, still percent-encoded
    query: string; // the text after `?`, still percent-encoded
    version: Version;
    headers: [] Header_Field;
    body: [] u8; // the whole body (a limit from the server config keeps it bounded)
    peer: Io.Address;

    // The values of the route's `{params}`, in template order, percent-decoded.
    params: [] Pair;

    // The app that is serving it, for state and configuration.
    app: *App;

    // Internal: middleware chain position and the decoded query.
    chain: [] Middleware;
    cursor: int;
    endpoint: Handler;
    query_pairs: [] Pair;
    query_parsed: bool;
}

// A handler after its extractors are built: `(request) -> Response`. Routes store one per route.
Handler :: #type (request: *Request) -> Response;

// The rest of the middleware chain and then the handler: `next(request)` runs it and returns its
// response, which the middleware may change or replace.
Next :: #type (request: *Request) -> Response;

// A request filter: log, check a token, add headers, or answer itself without calling `next`.
//
//     require_token :: (request: *Request, next: Next) -> Response {
//         token, found := header(request, "Authorization");
//         if !found return error_response(request, unauthorized("a token is required"));
//
//         return next(request);
//     }
Middleware :: #type (request: *Request, next: Next) -> Response;

// The value of the first header called `name` (any case), and whether there was one.
header :: (request: *Request, name: string) -> string, bool {
    return find_header(request.headers, name);
}

header :: (request: *Request, id: Header_Name) -> string, bool {
    return find_header(request.headers, id);
}

// The decoded pairs of the query string (`?a=1&b=two`), parsed on first use.
query_pairs :: (request: *Request) -> [] Pair {
    if !request.query_parsed {
        request.query_pairs = parse_query(request.query,, temp);
        request.query_parsed = true;
    }

    return request.query_pairs;
}

// The first query parameter called `name`, and whether it was given.
query_param :: (request: *Request, name: string) -> string, bool {
    for query_pairs(request) {
        if it.key == name return it.value, true;
    }

    return "", false;
}

// The value of a `{param}` of the matched route.
path_param :: (request: *Request, name: string) -> string, bool {
    for request.params {
        if it.key == name return it.value, true;
    }

    return "", false;
}

// `1.2.3.4:5678` or `[::1]:5678`.
peer_text :: (request: *Request) -> string {
    return Io.address_to_string(request.peer);
}

// The body as text (a view of the same bytes).
body_text :: (request: *Request) -> string {
    return string.{ request.body.count, request.body.data };
}

// ---------------------------------------------------------------------------------------------
// Responses

// What a handler returns when it wants full control. The headers and body must stay valid until
// the response is written: use string literals or the temporary allocator (`tprint`, `,, temp`),
// which the server resets after each response. The builders below do.
Response :: struct {
    status := 200;
    headers: [..] Header_Field;
    body: [] u8;
}

// Adds a header. The name and value must stay valid until the response is written.
add_header :: (response: *Response, name: string, value: string) {
    header: Header_Field;

    header.name = name;
    header.value = value;
    header.id = lookup_header_name(name);

    array_add(*response.headers, header,, temp);
}

// `response` with one more header: `with_header(json(user), "ETag", tag)`.
with_header :: (response: Response, name: string, value: string) -> Response {
    result := response;

    // The copy shares the header array; give it one of its own.
    result.headers = .{};

    for response.headers array_add(*result.headers, it,, temp);

    add_header(*result, name, value);

    return result;
}

// An empty response with only a status.
status_only :: (status: int) -> Response {
    response: Response;

    response.status = status;

    return response;
}

// `value` as JSON with `Content-Type: application/json`. The body is built in the temporary
// allocator.
json :: (value: $T, status := 200) -> Response {
    response: Response;

    text := to_json(value,, temp);

    response.status = status;
    response.body = .{ text.count, text.data };

    add_header(*response, "Content-Type", "application/json");

    return response;
}

// Plain text with `Content-Type: text/plain; charset=utf-8`.
text :: (body: string, status := 200) -> Response {
    return bytes_response(body, "text/plain; charset=utf-8", status);
}

// HTML with `Content-Type: text/html; charset=utf-8`.
html :: (body: string, status := 200) -> Response {
    return bytes_response(body, "text/html; charset=utf-8", status);
}

// Raw bytes with a content type of your choice.
bytes_response :: (body: string, content_type: string, status := 200) -> Response {
    response: Response;

    response.status = status;
    response.body = .{ body.count, body.data };

    add_header(*response, "Content-Type", content_type);

    return response;
}

bytes_response :: (body: [] u8, content_type: string, status := 200) -> Response {
    response: Response;

    response.status = status;
    response.body = body;

    add_header(*response, "Content-Type", content_type);

    return response;
}

// A redirect to `location`: 302 unless you pass 301, 303, 307 or 308.
redirect :: (location: string, status := 302) -> Response {
    response: Response;

    response.status = status;

    add_header(*response, "Location", location);

    return response;
}

// 204 with no body.
no_content :: () -> Response {
    return status_only(204);
}

// ---------------------------------------------------------------------------------------------
// Errors

Http_Error_Kind :: enum u8 {
    NONE; // no error: `.{}`
    STATUS; // an HTTP status: from a handler, or (client) a response that was not 2xx
    TRANSPORT; // client: connecting, sending or receiving failed (`transport` says how)
    TIMEOUT; // client: the request took longer than its timeout
    INVALID_URL; // client: the URL cannot be requested
    UNSUPPORTED; // client: not implemented yet (https)
    DECODE; // client: the response was not what was asked for (not JSON, wrong shape)
    REDIRECTS; // client: too many redirects
}

// What a handler returns to answer with a status instead of a value, and what the client returns
// for anything that went wrong. A handler's `.{}` means no error. The server sends it as an
// RFC 9457 `application/problem+json` document:
//
//     return .{}, not_found("No user with id %", id.value);
//     // 404 {"type":"about:blank","title":"Not Found","status":404,"detail":"No user with id 7"}
//
// `app.on_error` changes the format for all of them.
Http_Error :: struct {
    kind: Http_Error_Kind;
    status: int; // the HTTP status (0 when there was no response)
    title: string; // short and fixed per status; the reason phrase when empty
    detail: string; // what happened this time
    problem_type: string; // a URI naming the kind of problem; "about:blank" when empty
    location: string; // for bad input: `body`, `query`, `path`, `header` or `form`
    field: string; // for bad input: the parameter, header or JSON path that was wrong
    headers: [] Header_Field; // sent with the error (`Retry-After`, `WWW-Authenticate`); see with_header

    // Client side.
    body: string; // the response body, when the status was not 2xx
    transport: Io.Io_Error; // why a TRANSPORT error happened

    // Set when the strings above were allocated for this error (the client's errors): release
    // them with `free_error`.
    owned: bool;
}

// An error with a status and a detail text (used as it is). The named helpers below take a
// `tprint` format instead: `not_found("No user %", id)`.
http_error :: (status: int, detail := "") -> Http_Error {
    error: Http_Error;

    error.kind = .STATUS;
    error.status = status;
    error.title = status_reason(status);
    error.detail = detail;

    return error;
}

bad_request :: (format := "", args: ..Any) -> Http_Error {
    return http_error(400, tprint(format, ..args));
}

unauthorized :: (format := "", args: ..Any) -> Http_Error {
    return http_error(401, tprint(format, ..args));
}

forbidden :: (format := "", args: ..Any) -> Http_Error {
    return http_error(403, tprint(format, ..args));
}

not_found :: (format := "", args: ..Any) -> Http_Error {
    return http_error(404, tprint(format, ..args));
}

conflict :: (format := "", args: ..Any) -> Http_Error {
    return http_error(409, tprint(format, ..args));
}

gone :: (format := "", args: ..Any) -> Http_Error {
    return http_error(410, tprint(format, ..args));
}

unprocessable :: (format := "", args: ..Any) -> Http_Error {
    return http_error(422, tprint(format, ..args));
}

too_many_requests :: (format := "", args: ..Any) -> Http_Error {
    return http_error(429, tprint(format, ..args));
}

internal_error :: (format := "", args: ..Any) -> Http_Error {
    return http_error(500, tprint(format, ..args));
}

unavailable :: (format := "", args: ..Any) -> Http_Error {
    return http_error(503, tprint(format, ..args));
}

// `error` with one more header (`Retry-After`, `WWW-Authenticate`...).
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

    if error.kind == .TRANSPORT return tprint("%: %", error.detail, error.transport);

    return error.detail;
}

// The response for an error: what `app.on_error` returns, or the problem document. Middleware use
// it to answer without calling `next`.
error_response :: (request: *Request, error: Http_Error) -> Response {
    if request.app && request.app.on_error return request.app.on_error(error, request);

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
input_error :: (status: int, location: string, field: string, detail: string) -> Http_Error {
    error := http_error(status);

    error.detail = detail;
    error.location = location;
    error.field = field;

    return error;
}
