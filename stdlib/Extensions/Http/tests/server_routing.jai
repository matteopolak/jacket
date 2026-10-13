// The router through a real server: static, {param} and {rest...} segments, priorities, trailing
// slashes and percent-decoding, 404 and 405 with Allow, automatic HEAD and OPTIONS, and routes
// that conflict.
#load "lib/server_support.jai";

name_of :: (name: Path("name", string)) -> string {
    return tprint("name=%", name.value);
}

static_new :: () -> string {
    return "static new";
}

by_id :: (id: Path("id", int)) -> string {
    return tprint("id=%", id.value);
}

user_post :: (user: Path("user", int), post: Path("post", string)) -> string {
    return tprint("user=% post=%", user.value, post.value);
}

file_rest :: (path: Rest("path")) -> string {
    return tprint("rest=%", path.value);
}

file_exact :: () -> string {
    return "exact file";
}

root :: () -> string {
    return "root";
}

list :: () -> string {
    return "list";
}

made :: () -> Response {
    return with_header(text("made", 201), "X-Made", "yes");
}

put_it :: () -> string {
    return "put";
}

head_special :: () -> Response {
    return with_header(text("body that HEAD does not send"), "X-Head", "own");
}

app: App;

routing :: (base: string, server: *Server) {
    // Static beats {param}, which beats {rest...}.
    response := get_path(base, "/");

    check(response.status == 200 && body_of(response) == "root", "root: % %", response.status, body_of(response));

    response = get_path(base, "/users");
    check(body_of(response) == "list", "users list: %", body_of(response));

    response = get_path(base, "/users/");
    check(body_of(response) == "list", "a trailing slash is ignored: %", body_of(response));

    response = get_path(base, "/users/new");
    check(body_of(response) == "static new", "static wins over {id}: %", body_of(response));

    response = get_path(base, "/users/42");
    check(body_of(response) == "id=42", "{id}: %", body_of(response));

    response = get_path(base, "/users/42/posts/hello");
    check(body_of(response) == "user=42 post=hello", "two params: %", body_of(response));

    response = get_path(base, "/users/abc");
    check(response.status == 400 && contains(body_of(response), "invalid path parameter `id`"), "{id} that is not an integer: % %", response.status, body_of(response));

    // A static segment that fails further down is backed out of.
    response = get_path(base, "/users/new/posts/x");
    check(response.status == 400, "backtracks from the static branch: % %", response.status, body_of(response));

    response = get_path(base, "/files/a/b/c.txt");
    check(body_of(response) == "rest=a/b/c.txt", "rest: %", body_of(response));

    response = get_path(base, "/files/one");
    check(body_of(response) == "rest=one", "rest with one segment: %", body_of(response));

    response = get_path(base, "/files/exact");
    check(body_of(response) == "exact file", "static beats rest: %", body_of(response));

    response = get_path(base, "/files");
    check(response.status == 404, "{rest...} needs a segment: %", response.status);

    // Percent-encoded parameters are decoded; %2F stays inside its segment.
    response = get_path(base, "/hello/a%20b");
    check(body_of(response) == "name=a b", "percent-decoding: %", body_of(response));

    response = get_path(base, "/hello/a%2Fb");
    check(body_of(response) == "name=a/b", "an encoded slash is one segment: %", body_of(response));

    response = get_path(base, "/hello/caf%C3%A9");
    check(body_of(response) == "name=café", "UTF-8: %", body_of(response));

    response = get_path(base, "/hello/x?ignored=1");
    check(body_of(response) == "name=x", "the query is not part of the path: %", body_of(response));

    // 404.
    response = get_path(base, "/nothing/here");
    check(response.status == 404, "404: %", response.status);
    check(header_of(response, "Content-Type") == "application/problem+json", "404 is problem+json: %", header_of(response, "Content-Type"));
    check(contains(body_of(response), "\"status\":404") && contains(body_of(response), "\"instance\":\"/nothing/here\""), "404 body: %", body_of(response));

    response = get_path(base, "/users//7");
    check(response.status == 404, "an empty segment matches no {param}: %", response.status);

    // 405 with Allow.
    response = fetch(base, .DELETE, "/users");
    check(response.status == 405, "405: %", response.status);
    check(header_of(response, "Allow") == "GET, HEAD, POST, OPTIONS", "Allow: %", header_of(response, "Allow"));

    response = fetch(base, .PUT, "/users/5");
    check(response.status == 200 && body_of(response) == "put", "PUT on /users/{id}: % %", response.status, body_of(response));

    response = fetch(base, .POST, "/users/5");
    check(response.status == 405 && header_of(response, "Allow") == "GET, HEAD, PUT, OPTIONS", "Allow on /users/{id}: % %", response.status, header_of(response, "Allow"));

    // POST that answers with its own status.
    response = fetch(base, .POST, "/users");
    check(response.status == 201 && header_of(response, "X-Made") == "yes", "Response as is: % %", response.status, header_of(response, "X-Made"));

    // HEAD is GET without the body; Content-Length still says how long it would be.
    response = fetch(base, .HEAD, "/users/new");
    check(response.status == 200 && response.body.count == 0, "HEAD has no body: % %", response.status, response.body.count);
    check(header_of(response, "Content-Length") == "10", "HEAD keeps Content-Length: %", header_of(response, "Content-Length"));

    response = fetch(base, .HEAD, "/head");
    check(header_of(response, "X-Head") == "own" && response.body.count == 0, "a HEAD route of its own: %", header_of(response, "X-Head"));

    response = fetch(base, .HEAD, "/nothing");
    check(response.status == 404 && response.body.count == 0, "HEAD of a missing route: %", response.status);

    // OPTIONS.
    response = fetch(base, .OPTIONS, "/users");
    check(response.status == 204 && header_of(response, "Allow") == "GET, HEAD, POST, OPTIONS", "OPTIONS: % %", response.status, header_of(response, "Allow"));

    response = fetch(base, .OPTIONS, "/users/9");
    check(header_of(response, "Allow") == "GET, HEAD, PUT, OPTIONS", "OPTIONS on a param route: %", header_of(response, "Allow"));

    response = fetch(base, .OPTIONS, "/missing");
    check(response.status == 404, "OPTIONS of a missing route: %", response.status);

    // Methods the server has no routes for.
    response = fetch(base, .TRACE, "/users");
    check(response.status == 501, "TRACE: %", response.status);

    // The Server and Date headers.
    response = get_path(base, "/");
    check(header_of(response, "Server") == "jaic", "Server header: %", header_of(response, "Server"));
    check(header_of(response, "Date").count == 29, "Date header: %", header_of(response, "Date"));
}

conflicts :: () {
    clash: App;

    get(*clash, "/a/{x}", by_id_x);
    get(*clash, "/a/{y}", by_id_y);
    get(*clash, "/a/b", static_new);
    post(*clash, "/a/{z}", by_id_z);
    get(*clash, "/f/{path...}", file_rest);
    get(*clash, "/f/{other...}", file_rest_other);

    check(clash.setup_errors.count == 2, "two conflicts are reported, not %: %", clash.setup_errors.count, clash.setup_errors);

    if clash.setup_errors.count == 2 {
        message := clash.setup_errors[0];

        check(contains(message, "GET /a/{y}") && contains(message, "GET /a/{x}"), "the message names both templates: %", message);
        check(contains(clash.setup_errors[1], "/f/{other...}"), "the second conflict: %", clash.setup_errors[1]);
    }

    server, error := start(*clash);

    check(server == null && error == .INVALID, "a conflicting app does not start: %", error);
}

by_id_x :: (x: Path("x", int)) -> string {
    return "x";
}

by_id_y :: (y: Path("y", int)) -> string {
    return "y";
}

by_id_z :: (z: Path("z", int)) -> string {
    return "z";
}

file_rest_other :: (other: Rest("other")) -> string {
    return "other";
}

main :: () {
    if !supported() return;

    get(*app, "/", root);
    get(*app, "/users", list);
    post(*app, "/users", made);
    get(*app, "/users/new", static_new);
    get(*app, "/users/{id}", by_id);
    put(*app, "/users/{id}", put_it);
    get(*app, "/users/{user}/posts/{post}", user_post);
    get(*app, "/hello/{name}", name_of);
    get(*app, "/files/{path...}", file_rest);
    get(*app, "/files/exact", file_exact);
    get(*app, "/head", head_special);
    head(*app, "/head", head_special);

    run_server(*app, routing);

    Io.io_run(conflicts);

    finish();
}
