// What handlers return and how it becomes a response, error responses and `on_error`, middleware,
// groups (nested, with parameters in the prefix) and large bodies.
#load "support/server_support.jai";
#import "Extensions/Json";

User :: struct {
    id: int;
    name: string;
}

Order :: struct {
    org: string;
    id: int;
}

// ---- Return mapping

as_struct :: () -> User {
    return .{ 1, "Ada" };
}

as_list :: () -> [] User {
    return User.[.{ 1, "Ada" }, .{ 2, "Grace" }];
}

as_string :: () -> string {
    return "plain words";
}

as_bytes :: () -> [] u8 {
    return u8.[1, 2, 3, 255];
}

as_response :: () -> Response {
    return with_header(json(User.{ 3, "Linus" }, 202), "X-Extra", "1");
}

as_nothing :: () {}

as_html :: () -> Response {
    return html("<p>hi</p>");
}

as_redirect :: () -> Response {
    return redirect("/elsewhere", 307);
}

as_created :: (body: Text) -> User {
    return .{ 9, body.value };
}

as_error_only :: (flag: Path("flag", bool)) -> Http_Error {
    if flag.value return unauthorized("flag set");
    return .{};
}

as_pair :: (flag: Path("flag", bool)) -> User, Http_Error {
    if flag.value return .{}, conflict("it %", "conflicts");
    return .{ 5, "Five" }, .{};
}

big :: () -> string {
    builder: String_Builder;
    for 0..1999 append(*builder, "0123456789012345678901234567890123456789\n");
    return builder_to_string(*builder,, temp);
}

bad_header :: () -> Response {
    return with_header(text("x"), "Bad Name", "v");
}

// ---- Errors

fail_with :: (code: Path("code", int)) -> Http_Error {
    if code.value == 400 return bad_request("bad: %", "reason");
    if code.value == 401 return with_header(unauthorized("who are you?"), "WWW-Authenticate", "Bearer");
    if code.value == 403 return forbidden("no");
    if code.value == 404 return not_found();
    if code.value == 409 return conflict("taken");
    if code.value == 410 return gone("it is gone");
    if code.value == 422 return unprocessable("odd");
    if code.value == 429 return with_header(too_many_requests("slow down"), "Retry-After", "30");
    if code.value == 500 return internal_error("oops");
    if code.value == 503 return unavailable("later");
    if code.value == 418 return http_error(418, "teapot");

    typed := http_error(code.value, "typed");
    typed.problem_type = "https://example.com/probs/out-of-credit";
    typed.title = "Out of credit";

    return typed;
}

// ---- Middleware

trace: [..] string;

log_calls :: (request: *Request, next: Next) -> Response {
    // Kept after the request, so copied out of temporary storage.
    array_add(*trace, copy_string(tprint("log in %", request.path)));
    response := next(request);

    array_add(*trace, copy_string(tprint("log out %", response.status)));

    return response;
}

add_header_mw :: (request: *Request, next: Next) -> Response {
    array_add(*trace, "header mw");
    response := next(request);
    add_header(*response, "X-Seen", "yes");

    return response;
}

require_token :: (request: *Request, next: Next) -> Response {
    array_add(*trace, "auth");
    token, found := header(request, "Authorization");

    if !found return error_response(request, unauthorized("a token is required"));
    if token != "Bearer secret" return error_response(request, forbidden("that token is not valid"));

    return next(request);
}

stamp :: (request: *Request, next: Next) -> Response {
    array_add(*trace, "stamp");
    response := next(request);
    add_header(*response, "X-Stamp", "v1");

    return response;
}

admin_home :: () -> string {
    return "admin home";
}

admin_user :: (id: Path("id", int)) -> string {
    return tprint("admin user %", id.value);
}

v1_ping :: () -> string {
    return "pong";
}

org_order :: (org: Path("org", string), id: Path("id", int)) -> string {
    return tprint("% %", org.value, id.value);
}

org_both :: (order: Params(Order)) -> Order {
    return order.value;
}

public_home :: () -> string {
    return "public";
}

app: App;
custom: App;

responses :: (api: *Client) {
    // ---- Return mapping
    response := get_path(api, "/struct");

    check(response.status == 200 && body_of(response) == "{\"id\":1,\"name\":\"Ada\"}" && header_of(response, "Content-Type") == "application/json", "struct as JSON: % %", response.status, body_of(response));

    response = get_path(api, "/list");
    check(body_of(response) == "[{\"id\":1,\"name\":\"Ada\"},{\"id\":2,\"name\":\"Grace\"}]", "slice as JSON: %", body_of(response));

    response = get_path(api, "/string");
    check(body_of(response) == "plain words" && header_of(response, "Content-Type") == "text/plain; charset=utf-8", "string: % %", body_of(response), header_of(response, "Content-Type"));

    response = get_path(api, "/bytes");
    check(response.body.count == 4 && response.body[3] == 255 && header_of(response, "Content-Type") == "application/octet-stream", "bytes: % %", response.body.count, header_of(response, "Content-Type"));

    response = get_path(api, "/response");
    check(response.status == 202 && header_of(response, "X-Extra") == "1" && contains(body_of(response), "Linus"), "Response as is: % %", response.status, body_of(response));

    response = get_path(api, "/nothing");
    check(response.status == 204 && response.body.count == 0 && !has_header(response, "Content-Length"), "void is 204: % %", response.status, header_of(response, "Content-Length"));

    response = get_path(api, "/html");
    check(header_of(response, "Content-Type") == "text/html; charset=utf-8" && body_of(response) == "<p>hi</p>", "html: %", header_of(response, "Content-Type"));

    response = get_path(api, "/redirect");
    check(response.status == 307 && header_of(response, "Location") == "/elsewhere", "redirect: % %", response.status, header_of(response, "Location"));

    response = post_body(api, "/created", "Hopper", "text/plain");
    check(response.status == 201 && contains(body_of(response), "Hopper"), "POST value is 201: %", response.status);

    response = get_path(api, "/error-only/false");
    check(response.status == 204, "no error and nothing else is 204: %", response.status);

    response = get_path(api, "/error-only/true");
    check(response.status == 401, "error only: %", response.status);

    response = get_path(api, "/pair/false");
    check(response.status == 200 && contains(body_of(response), "Five"), "value, no error: % %", response.status, body_of(response));

    response = get_path(api, "/pair/true");
    check(response.status == 409 && contains(body_of(response), "it conflicts"), "value and error: % %", response.status, body_of(response));

    response = get_path(api, "/bad-header");
    check(response.status == 500 && contains(body_of(response), "invalid response"), "a response the writer refuses is a 500: % %", response.status, body_of(response));

    response = get_path(api, "/big");
    check(response.body.count == 2000 * 41, "a big body arrives whole: %", response.body.count);

    // ---- Errors
    expected :: int.[400, 401, 403, 404, 409, 410, 418, 422, 429, 500, 503];

    for code: expected {
        response = get_path(api, tprint("/fail/%", code));
        check(response.status == code, "error %: got %", code, response.status);
        check(header_of(response, "Content-Type") == "application/problem+json", "error % content type: %", code, header_of(response, "Content-Type"));
        check(contains(body_of(response), tprint("\"status\":%", code)) && contains(body_of(response), "\"type\":\"about:blank\""), "error % body: %", code, body_of(response));
    }

    response = get_path(api, "/fail/400");
    check(contains(body_of(response), "\"title\":\"Bad Request\"") && contains(body_of(response), "\"detail\":\"bad: reason\""), "the format is applied: %", body_of(response));

    response = get_path(api, "/fail/404");
    check(!contains(body_of(response), "detail"), "no detail when none was given: %", body_of(response));

    response = get_path(api, "/fail/401");
    check(header_of(response, "WWW-Authenticate") == "Bearer", "error headers: %", header_of(response, "WWW-Authenticate"));

    response = get_path(api, "/fail/429");
    check(header_of(response, "Retry-After") == "30", "Retry-After: %", header_of(response, "Retry-After"));

    response = get_path(api, "/fail/402");
    check(response.status == 402 && contains(body_of(response), "https://example.com/probs/out-of-credit") && contains(body_of(response), "Out of credit"), "problem type and title: %", body_of(response));

    // ---- Middleware and groups
    trace.count = 0;

    response = get_path(api, "/public");
    check(body_of(response) == "public" && header_of(response, "X-Seen") == "yes", "app middleware: % %", body_of(response), header_of(response, "X-Seen"));
    check(trace.count == 3 && trace[0] == "log in /public" && trace[1] == "header mw" && trace[2] == "log out 200", "app middleware order: %", trace);

    trace.count = 0;

    response = get_path(api, "/nowhere");
    check(response.status == 404 && header_of(response, "X-Seen") == "yes", "app middleware sees a 404: % %", response.status, header_of(response, "X-Seen"));

    response = get_path(api, "/admin");
    check(response.status == 401 && contains(body_of(response), "a token is required"), "group middleware answers itself: % %", response.status, body_of(response));
    check(header_of(response, "X-Seen") == "yes", "app middleware wraps the group's: %", header_of(response, "X-Seen"));

    response = get_path(api, "/admin", Header_Field.[.{ "Authorization", "Bearer wrong" }]);
    check(response.status == 403, "a bad token: %", response.status);

    trace.count = 0;

    token := Header_Field.[.{ "Authorization", "Bearer secret" }];

    response = get_path(api, "/admin", token);
    check(body_of(response) == "admin home" && trace.count == 4 && trace[2] == "auth", "a good token: % %", body_of(response), trace);

    response = get_path(api, "/admin/users/12", token);
    check(body_of(response) == "admin user 12", "a param under a group: %", body_of(response));

    trace.count = 0;

    response = get_path(api, "/admin/v1/ping", token);
    check(body_of(response) == "pong" && header_of(response, "X-Stamp") == "v1", "a nested group: % %", body_of(response), header_of(response, "X-Stamp"));
    check(trace.count == 5 && trace[2] == "auth" && trace[3] == "stamp", "nested middleware order: %", trace);

    response = get_path(api, "/admin/v1/ping");
    check(response.status == 401, "a nested group keeps the parent's middleware: %", response.status);

    response = get_path(api, "/public", token);
    check(!has_header(response, "X-Stamp"), "group middleware stays in its group");

    response = get_path(api, "/orgs/acme/orders/4");
    check(body_of(response) == "acme 4", "params in a group prefix: %", body_of(response));

    response = get_path(api, "/orgs/acme/both/4");
    check(contains(body_of(response), "\"org\":\"acme\"") && contains(body_of(response), "\"id\":4"), "Params covering prefix and route: %", body_of(response));
}

custom_errors :: (api: *Client) {
    response := get_path(api, "/nothing");

    check(response.status == 404 && body_of(response) == "{\"error\":\"not found\",\"code\":404}" && header_of(response, "Content-Type") == "application/json", "on_error for 404: % %", response.status, body_of(response));

    response = fetch(api, .DELETE, "/users/1");
    check(response.status == 405 && header_of(response, "Allow") == "GET, HEAD, OPTIONS" && contains(body_of(response), "\"code\":405"), "on_error for 405 keeps Allow: % %", response.status, header_of(response, "Allow"));

    response = get_path(api, "/users/x");
    check(response.status == 400 && contains(body_of(response), "\"error\":\"invalid path parameter `id`"), "on_error for an extractor: % %", response.status, body_of(response));

    response = get_path(api, "/fail");
    check(response.status == 409 && body_of(response) == "{\"error\":\"taken\",\"code\":409}", "on_error for a handler's error: % %", response.status, body_of(response));
}

format_error :: (error: Http_Error, request: *Request) -> Response {
    message := ifx error.detail.count then error.detail else error.title;
    if error.status == 404 message = "not found";

    builder: String_Builder;

    append(*builder, "{\"error\":");
    append_json_string(*builder, message);
    print(*builder, ",\"code\":%}", error.status);

    return json_text(builder_to_string(*builder,, temp), error.status);
}

json_text :: (body: string, status: int) -> Response {
    return bytes_response(body, "application/json", status);
}

user_by_id :: (id: Path("id", int)) -> string {
    return "user";
}

always_conflict :: () -> Http_Error {
    return conflict("taken");
}

main :: () {
    if !supported() return;

    get(*app, "/struct", as_struct);
    get(*app, "/list", as_list);
    get(*app, "/string", as_string);
    get(*app, "/bytes", as_bytes);
    get(*app, "/response", as_response);
    get(*app, "/nothing", as_nothing);
    get(*app, "/html", as_html);
    get(*app, "/redirect", as_redirect);
    post(*app, "/created", as_created);
    get(*app, "/error-only/{flag}", as_error_only);
    get(*app, "/pair/{flag}", as_pair);
    get(*app, "/big", big);
    get(*app, "/bad-header", bad_header);
    get(*app, "/fail/{code}", fail_with);
    get(*app, "/public", public_home);

    use(*app, log_calls);
    use(*app, add_header_mw);

    admin := group(*app, "/admin", require_token);

    get(admin, "", admin_home);
    get(admin, "/users/{id}", admin_user);

    v1 := group(admin, "/v1", stamp);

    get(v1, "/ping", v1_ping);

    orgs := group(*app, "/orgs/{org}");

    get(orgs, "/orders/{id}", org_order);
    get(orgs, "/both/{id}", org_both);

    run_app(*app, responses);

    get(*custom, "/users/{id}", user_by_id);
    get(*custom, "/fail", always_conflict);
    custom.on_error = format_error;

    run_app(*custom, custom_errors);

    finish();
}
