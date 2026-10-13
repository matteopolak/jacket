// The client against the server and against hand-made servers: JSON calls and their errors,
// the connection pool (reuse, eviction, stale connections), redirects, timeouts, refused
// connections, https and URL errors, chunked and until-close responses.
#load "lib/server_support.jai";
#import "Extensions/Json";

User :: struct {
    id: int;
    name: string;
}

New_User :: struct {
    name: string;
}

users: [..] User;

get_user :: (id: Path("id", int)) -> User, Http_Error {
    for users if it.id == id.value return it, .{};

    return .{}, not_found("No user with id %", id.value);
}

create_user :: (body: Json(New_User)) -> User, Http_Error {
    if !body.name return .{}, bad_request("name is required");

    user := User.{ users.count + 1, copy_string(body.name) };

    array_add(*users, user);

    return user, .{};
}

rename_user :: (id: Path("id", int), body: Json(New_User)) -> User, Http_Error {
    for *users {
        if it.id == id.value {
            it.name = copy_string(body.name);

            return it.*, .{};
        }
    }

    return .{}, not_found("No user with id %", id.value);
}

remove_user :: (id: Path("id", int)) -> Http_Error {
    for users {
        if it.id == id.value {
            array_ordered_remove_by_index(*users, it_index);

            return .{};
        }
    }

    return not_found("No user with id %", id.value);
}

not_json :: () -> string {
    return "this is not JSON";
}

text_hello :: () -> string {
    return "hello text";
}

slow :: (ms: Path("ms", int)) -> string {
    Io.sleep(ms.value);
    return "late";
}

// What the server received, as text: method, target, body, and some headers.
inspect :: (request: *Request) -> string {
    auth, _ := header(request, "Authorization");
    ctype, _ := header(request, "Content-Type");
    custom, _ := header(request, "X-Custom");
    agent, _ := header(request, "User-Agent");

    return tprint("% % body=[%] auth=[%] ctype=[%] custom=[%] agent=[%]", request.method_text, request.target, body_text(request), auth, ctype, custom, agent);
}

moved :: (request: *Request) -> Response {
    return redirect("/inspect?moved=1", 301);
}

found :: (request: *Request) -> Response {
    return redirect("/inspect?found=1", 302);
}

see_other :: (request: *Request) -> Response {
    return redirect("/inspect?seen=1", 303);
}

temporary :: (request: *Request) -> Response {
    return redirect("/inspect?temporary=1", 307);
}

permanent :: (request: *Request) -> Response {
    return redirect("/inspect?permanent=1", 308);
}

relative :: (request: *Request) -> Response {
    return redirect("sibling", 302);
}

sibling :: () -> string {
    return "sibling reached";
}

loop :: () -> Response {
    return redirect("/loop", 302);
}

Other :: struct {
    base: string;
}

other_origin: string;

to_other :: () -> Response {
    return redirect(tprint("%/inspect", other_origin), 302);
}

app: App;
second: App;

pooling :: (base: string, server: *Server) {
    check(Io.backend_name().count > 0, "backend: %", Io.backend_name());

    before := connections_opened();

    for 1..5 {
        text, error := get_text(tprint("%/text", base));
        check(error.kind == .NONE && text == "hello text", "get_text: % %", text, error_text(error));
        free(text.data);
    }

    check(connections_opened() - before == 1, "five requests, one connection: % opened", connections_opened() - before);
    check(idle_connection_count() == 1, "and it is kept: % idle", idle_connection_count());

    // Concurrent requests need their own connections; afterwards at most max_idle_per_host stay.
    saved := client_settings.max_idle_per_host;

    client_settings.max_idle_per_host = 2;

    group: Io.Wait_Group;

    Io.add(*group, 6);

    for 1..6 Io.spawn(parallel_get, ParallelArgs.{ tprint("%/slow/50", base), *group });

    Io.wait(*group);

    check(idle_connection_count() == 2, "idle connections are capped at max_idle_per_host: %", idle_connection_count());

    client_settings.max_idle_per_host = saved;

    close_idle_connections();
    check(idle_connection_count() == 0, "close_idle_connections");

    // Idle connections are evicted after idle_timeout.
    opened := connections_opened();
    text, error := get_text(tprint("%/text", base));

    free(text.data);
    check(idle_connection_count() == 1, "one idle again");

    client_settings.idle_timeout = 60;
    Io.sleep(120);

    text, error = get_text(tprint("%/text", base));
    free(text.data);
    check(connections_opened() == opened + 2, "a connection that idled too long is not reused: % opened", connections_opened() - opened);

    client_settings.idle_timeout = 30_000;
}

ParallelArgs :: struct {
    url: string;
    group: *Io.Wait_Group;
}

parallel_get :: (args: ParallelArgs) {
    defer Io.done(args.group);

    text, error := get_text(args.url);

    check(error.kind == .NONE && text == "late", "parallel request: %", error_text(error));

    free(text.data);
}

json_calls :: (base: string, server: *Server) {
    user, error := post_json(User, tprint("%/users", base), New_User.{ "Ada" });

    check(error.kind == .NONE && user.id == 1 && user.name == "Ada", "post_json: % %", user.name, error_text(error));
    free_json(*user);

    user, error = get_json(User, tprint("%/users/1", base));
    check(error.kind == .NONE && user.name == "Ada", "get_json: %", error_text(error));
    free_json(*user);

    user, error = put_json(User, tprint("%/users/1", base), New_User.{ "Ada Lovelace" });
    check(error.kind == .NONE && user.name == "Ada Lovelace", "put_json: %", error_text(error));
    free_json(*user);

    // A 404 carries the status, the problem's title and detail, and the body.
    user, error = get_json(User, tprint("%/users/99", base));
    check(error.kind == .STATUS && error.status == 404, "get_json of a missing user: %", error_text(error));
    check(error.title == "Not Found" && error.detail == "No user with id 99", "title and detail from the problem: % / %", error.title, error.detail);
    check(contains(error.body, "\"status\":404"), "the body is kept: %", error.body);
    free_error(*error);

    user, error = post_json(User, tprint("%/users", base), New_User.{ "" });
    check(error.kind == .STATUS && error.status == 400 && error.detail == "name is required", "a 400 from the handler: %", error_text(error));
    free_error(*error);

    // A reply that is not the JSON asked for.
    user, error = get_json(User, tprint("%/not-json", base));
    check(error.kind == .DECODE && error.status == 200, "not JSON: %", error_text(error));
    free_error(*error);

    user, error = get_json(User, tprint("%/text", base));
    check(error.kind == .DECODE, "text where JSON was expected: %", error_text(error));
    free_error(*error);

    error = post_json(tprint("%/users", base), New_User.{ "Grace" });
    check(error.kind == .NONE, "post_json without a response type: %", error_text(error));
    check(users.count == 2, "it did post: %", users.count);

    error = delete_request(tprint("%/users/2", base));
    check(error.kind == .NONE && users.count == 1, "delete_request: % %", error_text(error), users.count);

    error = delete_request(tprint("%/users/2", base));
    check(error.kind == .STATUS && error.status == 404, "deleting twice: %", error_text(error));
    free_error(*error);

    // send(): every part of a request.
    response, send_error := send(.{ method = .PUT, url = tprint("%/inspect?a=b", base), headers = .[.{ "X-Custom", "7" }, .{ "Authorization", "Bearer t" }], body = u8.[#char "h", #char "i"] });

    check(send_error.kind == .NONE && response.status == 200, "send: %", error_text(send_error));
    check(body_of(response) == "PUT /inspect?a=b body=[hi] auth=[Bearer t] ctype=[] custom=[7] agent=[jaic-http]", "what the server saw: %", body_of(response));
    free_response(*response);

    response, send_error = send(.{ url = tprint("%/users/404", base) });
    check(send_error.kind == .NONE && response.status == 404, "a 404 is a response, not an error: %", error_text(send_error));
    free_response(*response);

    response, send_error = send(.{ method = .HEAD, url = tprint("%/text", base) });
    check(send_error.kind == .NONE && response.status == 200 && response.body.count == 0, "HEAD through send: % %", response.status, response.body.count);
    check(header(*response, "content-length") == "10", "with the length");
    free_response(*response);

    response, send_error = send(.{ url = tprint("%/inspect", base), headers = .[.{ "User-Agent", "mine/1" }] });
    check(contains(body_of(response), "agent=[mine/1]"), "a User-Agent of your own: %", body_of(response));
    free_response(*response);

    response, send_error = send(.{ url = tprint("%/inspect", base), headers = .[.{ "Bad Header", "x" }] });
    check(send_error.kind != .NONE, "an invalid header is refused before sending");
    free_error(*send_error);
}

follow :: (base: string, path: string, method := Method.GET, body: [] u8 = .[]) -> Client_Response {
    response, error := send(.{ method = method, url = tprint("%%", base, path), body = body, headers = .[.{ "Authorization", "Bearer keep" }] });

    check(error.kind == .NONE, "% %: %", method_name(method), path, error_text(error));

    return response;
}

redirects :: (base: string, server: *Server) {
    other_origin = replace(base, "127.0.0.1", "localhost");
    response := follow(base, "/moved");

    check(response.status == 200 && contains(body_of(response), "GET /inspect?moved=1"), "301 on GET: %", body_of(response));
    check(contains(body_of(response), "auth=[Bearer keep]"), "credentials are kept on the same host: %", body_of(response));
    free_response(*response);

    response = follow(base, "/found", .POST, u8.[#char "x"]);
    check(contains(body_of(response), "GET /inspect?found=1 body=[]"), "302 turns a POST into a GET: %", body_of(response));
    free_response(*response);

    response = follow(base, "/see-other", .PUT, u8.[#char "x"]);
    check(contains(body_of(response), "GET /inspect?seen=1 body=[]"), "303 turns a PUT into a GET: %", body_of(response));
    free_response(*response);

    response = follow(base, "/temporary", .POST, u8.[#char "p", #char "q"]);
    check(contains(body_of(response), "POST /inspect?temporary=1 body=[pq]"), "307 repeats the request: %", body_of(response));
    free_response(*response);

    response = follow(base, "/permanent", .PUT, u8.[#char "z"]);
    check(contains(body_of(response), "PUT /inspect?permanent=1 body=[z]"), "308 repeats the request: %", body_of(response));
    free_response(*response);

    response = follow(base, "/relative");
    check(body_of(response) == "sibling reached", "a relative Location: %", body_of(response));
    free_response(*response);

    // Another host: the credentials stay home. "localhost" and "127.0.0.1" are different origins.
    response = follow(base, "/to-other");
    check(response.status == 200 && contains(body_of(response), "auth=[]"), "no Authorization to another host: %", body_of(response));
    free_response(*response);

    // Not following.
    error: Http_Error;
    response, error = send(.{ url = tprint("%/moved", base), follow_redirects = false });
    check(error.kind == .NONE && response.status == 301 && header(*response, "Location") == "/inspect?moved=1", "follow_redirects = false: %", response.status);
    free_response(*response);

    // Too many.
    response, error = send(.{ url = tprint("%/loop", base), max_redirects = 4 });
    check(error.kind == .REDIRECTS && contains(error.detail, "more than 4 redirects"), "a redirect loop: %", error_text(error));
    free_error(*error);
}

failures_of_transport :: (base: string, server: *Server) {
    // Timeout.
    started := current_time_monotonic();
    response, error := send(.{ url = tprint("%/slow/600", base), timeout = 150 });
    elapsed := to_milliseconds(current_time_monotonic() - started);

    check(error.kind == .TIMEOUT, "timeout: %", error_text(error));
    check(elapsed >= 100 && elapsed < 500, "after about 150 ms: % ms", elapsed);
    free_error(*error);

    // The client is usable afterwards.
    text, text_error := get_text(tprint("%/text", base));

    check(text_error.kind == .NONE, "after a timeout: %", error_text(text_error));
    free(text.data);

    // The connection that timed out is not reused by mistake: its late answer would be read as the next one.
    text, text_error = get_text(tprint("%/text", base));
    check(text == "hello text", "no stray answer from the timed out request: %", text);
    free(text.data);

    // Refused.
    refused_url := "";

    {
        listener, _ := Io.tcp_listen("127.0.0.1:0");
        address, _ := Io.local_address(listener);
        refused_url = tprint("http://%/", Io.address_to_string(address));
        Io.close(listener);
    }

    response, error = send(.{ url = refused_url, timeout = 2000 });
    check(error.kind == .TRANSPORT && error.transport == .CONNECTION_REFUSED, "a refused connection: % %", error_text(error), error.transport);
    free_error(*error);

    // URLs.
    response, error = send(.{ url = "https://example.com/" });
    check(error.kind == .UNSUPPORTED && contains(error.detail, "https is not supported yet"), "https: %", error_text(error));
    free_error(*error);

    response, error = send(.{ url = "ftp://example.com/" });
    check(error.kind == .INVALID_URL && contains(error.detail, "scheme `ftp`"), "other schemes: %", error_text(error));
    free_error(*error);

    response, error = send(.{ url = "example.com/path" });
    check(error.kind == .INVALID_URL && contains(error.detail, "not an absolute URL"), "no scheme: %", error_text(error));
    free_error(*error);

    response, error = send(.{ url = "http:///path" });
    check(error.kind == .INVALID_URL && contains(error.detail, "names no host"), "no host: %", error_text(error));
    free_error(*error);

    response, error = send(.{ url = "http://localhost:99999/" });
    check(error.kind == .INVALID_URL && contains(error.detail, "the port"), "bad port: %", error_text(error));
    free_error(*error);

    response, error = send(.{ url = "http://no-such-host.invalid/", timeout = 3000 });
    check(error.kind == .TRANSPORT || error.kind == .TIMEOUT, "an unknown host: %", error_text(error));
    free_error(*error);
}

// A stale pooled connection: the server closes it while idle, and the next request still works.
stale :: (base: string, server: *Server) {
    opened := connections_opened();

    text, error := get_text(tprint("%/text", base));

    check(error.kind == .NONE, "first: %", error_text(error));
    free(text.data);

    // The server's idle timeout (100 ms) closes it behind the client's back.
    Io.sleep(300);

    text, error = get_text(tprint("%/text", base));
    check(error.kind == .NONE && text == "hello text", "a request on a stale connection is retried: %", error_text(error));
    free(text.data);
    check(connections_opened() == opened + 2, "on a new connection: % opened", connections_opened() - opened);

    // A POST is not repeated behind the caller's back.
    Io.sleep(300);

    _, post_error := post_json(User, tprint("%/users", base), New_User.{ "Once" });

    check(post_error.kind == .TRANSPORT, "a POST on a stale connection fails instead of repeating: %", error_text(post_error));
    free_error(*post_error);
}

// Servers made by hand, for responses our own server does not give.
Script :: struct {
    listener: Io.Tcp_Listener;
    replies: [..] string;
    count: int;
}

scripted_server :: (script: *Script) {
    while true {
        stream, error := Io.accept(script.listener);

        if error != .NONE break;

        Io.spawn(scripted_connection, ScriptedArgs.{ script, stream });
    }
}

ScriptedArgs :: struct {
    script: *Script;
    stream: Io.Tcp_Stream;
}

scripted_connection :: (args: ScriptedArgs) {
    defer Io.close(args.stream);

    buffer: [2048] u8;

    // Read the request head, answer with the next scripted reply.
    seen: [..] u8;

    while true {
        count, error := Io.read(args.stream, buffer);

        if error != .NONE || count == 0 return;

        for 0..count - 1 array_add(*seen, buffer[it]);

        text := string.{ seen.count, seen.data };

        if find_index_from_left(text, "\r\n\r\n") >= 0 break;
    }

    reply := args.script.replies[args.script.count % args.script.replies.count];

    args.script.count += 1;

    Io.write(args.stream, reply);
}

scripted :: (base: string, server: *Server) {
    script: Script;

    listener, error := Io.tcp_listen("127.0.0.1:0");

    check(error == .NONE, "scripted listener");

    script.listener = listener;
    address, _ := Io.local_address(listener);
    url := tprint("http://%/x", Io.address_to_string(address));

    array_add(*script.replies, "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nX-Kind: chunked\r\n\r\n5\r\nhello\r\n6;a=b\r\n world\r\n0\r\nTrailer-X: 1\r\n\r\n");
    array_add(*script.replies, "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nuntil the end");
    array_add(*script.replies, "HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 201 Created\r\nContent-Length: 4\r\n\r\ndone");
    array_add(*script.replies, "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nabc");
    array_add(*script.replies, "garbage that is not http\r\n\r\n");
    array_add(*script.replies, "HTTP/1.1 200 OK\r\nContent-Length: 3\r\nContent-Length: 4\r\n\r\nabcd");

    Io.spawn(scripted_server, *script);

    response, send_error := send(.{ url = url });

    check(send_error.kind == .NONE && body_of(response) == "hello world" && header(*response, "X-Kind") == "chunked", "a chunked response: % %", body_of(response), error_text(send_error));
    free_response(*response);

    response, send_error = send(.{ url = url });
    check(send_error.kind == .NONE && body_of(response) == "until the end", "a response that ends with the connection: % %", body_of(response), error_text(send_error));
    free_response(*response);

    response, send_error = send(.{ url = url });
    check(send_error.kind == .NONE && response.status == 201 && body_of(response) == "done", "a 100 Continue before the answer: % %", response.status, error_text(send_error));
    free_response(*response);

    response, send_error = send(.{ url = url });
    check(send_error.kind == .TRANSPORT, "a body shorter than its Content-Length: %", error_text(send_error));
    free_error(*send_error);

    response, send_error = send(.{ url = url });
    check(send_error.kind == .TRANSPORT && contains(send_error.detail, "not valid HTTP"), "garbage: %", error_text(send_error));
    free_error(*send_error);

    response, send_error = send(.{ url = url });
    check(send_error.kind == .TRANSPORT, "two different Content-Lengths: %", error_text(send_error));
    free_error(*send_error);

    Io.close(script.listener);
}

main :: () {
    if !supported() return;

    other_origin = "";

    get(*app, "/text", text_hello);
    get(*app, "/not-json", not_json);
    get(*app, "/slow/{ms}", slow);
    get(*app, "/users/{id}", get_user);
    put(*app, "/users/{id}", rename_user);
    delete(*app, "/users/{id}", remove_user);
    post(*app, "/users", create_user);
    get(*app, "/inspect", inspect);
    put(*app, "/inspect", inspect);
    post(*app, "/inspect", inspect);
    get(*app, "/moved", moved);
    get(*app, "/found", found);
    post(*app, "/found", found);
    put(*app, "/see-other", see_other);
    get(*app, "/see-other", see_other);
    post(*app, "/temporary", temporary);
    put(*app, "/permanent", permanent);
    get(*app, "/relative", relative);
    get(*app, "/sibling", sibling);
    get(*app, "/loop", loop);
    get(*app, "/to-other", to_other);

    run_server(*app, pooling);
    run_server(*app, json_calls);
    run_server(*app, redirects);
    run_server(*app, failures_of_transport);
    run_server(*app, scripted);

    quick: App;

    quick.config.idle_timeout = 100;
    get(*quick, "/text", text_hello);
    post(*quick, "/users", create_user);

    run_server(*quick, stale);

    finish();
}
