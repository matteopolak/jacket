// The test client (test_client.jai) and the expectations (expect.jai): an app driven in memory
// through its real connection loop, outside and inside a scheduler, over a socket too, and the
// wording of what a failed expectation prints.
#import "Basic";
#import "String";
#import "Extensions/Http";
Io :: #import "Extensions/Io";

User :: struct {
    id: int;
    name: string;
}

New_User :: struct {
    name: string;
}

World :: struct {
    hits: int;
    other: *Client; // a client of the backend app, for the handler that calls another service
}

world: World;

big_buffer: [..] u8;

hello :: () -> string {
    return "hello";
}

bump :: (state: *State(World)) -> int {
    state.hits += 1;

    return state.hits;
}

find_user :: (id: Path("id", int)) -> User, Http_Error {
    if id.value == 1 return User.{ 1, "Ada" }, .{};

    return .{}, not_found("No user with id %", id.value);
}

add_user :: (body: Json(New_User)) -> User, Http_Error {
    if !body.name return .{}, unprocessable("name must not be empty");

    return User.{ 2, copy_string(body.name) }, .{};
}

rest_for :: (ms: Path("ms", int)) -> string {
    Io.sleep(ms.value);

    return "rested";
}

produce :: (channel: *Io.Channel(int)) {
    Io.sleep(2);
    Io.send(channel, 42);
}

through_channel :: () -> string {
    channel := Io.make_channel(int, 1);

    Io.spawn(produce, channel);

    value, _ := Io.recv(channel);

    Io.free_channel(channel);

    return tprint("got %", value);
}

through_backend :: (state: *State(World)) -> string, Http_Error {
    response, error := get(state.other, "/hello");

    defer close(*response);

    if error.kind != .NONE return "", internal_error("the backend said: %", error_text(error));

    status_error := error_for_status(*response);

    if status_error.kind != .NONE return "", internal_error("the backend said: %", error_text(status_error));

    text, read_error := read_text(*response);

    if read_error.kind != .NONE return "", internal_error("the backend said: %", error_text(read_error));

    return tprint("backend: %", text), .{};
}

length_of :: (body: Bytes) -> int {
    return body.value.count;
}

big :: () -> [] u8 {
    return big_buffer;
}

binary :: () -> [] u8 {
    return u8.[1, 2, 3, 255];
}

long_text :: () -> string {
    return copy_string(tprint("%", repeat("x", 1000)),, temp);
}

repeat :: (text: string, times: int) -> string {
    builder: String_Builder;

    for 1..times append(*builder, text);

    return builder_to_string(*builder,, temp);
}

taken :: () -> Http_Error {
    return conflict("already exists");
}

go_home :: () -> Response {
    return redirect("/hello");
}

who :: (request: *Request) -> string {
    return peer_text(request);
}

token :: (token: Header("x-token", string)) -> string {
    return token.value;
}

created_empty :: () -> Response {
    return status_only(201);
}

make_app :: (app: *App) {
    app.state = *world;

    get(app, "/hello", hello);
    get(app, "/bump", bump);
    get(app, "/users/{id}", find_user);
    post(app, "/users", add_user);
    get(app, "/rest/{ms}", rest_for);
    get(app, "/channel", through_channel);
    get(app, "/backend", through_backend);
    post(app, "/length", length_of);
    get(app, "/big", big);
    get(app, "/binary", binary);
    get(app, "/long", long_text);
    get(app, "/taken", taken);
    get(app, "/home", go_home);
    get(app, "/who", who);
    get(app, "/token", token);
    get(app, "/empty", created_empty);
}

app: App;
backend: App;
empty_backend: App;

supported :: () -> bool {
    #if OS == .WINDOWS {
        print("skipped: Extensions/Io has no IO on Windows yet\n");

        return false;
    }

    return true;
}

// What a failed expectation printed, without the line that names the file.
after_location :: (text: string) -> string {
    newline := find_index_from_left(text, #char "\n");

    if newline < 0 return text;

    return slice(text, newline + 1, text.count - newline - 1);
}

// ---------------------------------------------------------------------------------------------

basics :: () {
    world.hits = 0;

    api := test_client(*app);

    response := get(api, "/hello");

    expect_status(response, 200);
    expect_text(response, "hello");
    expect_header(response, "content-type", "text/plain");
    expect_header(response, "Content-Type", "text/plain; charset=utf-8");
    expect_header(response, "Server", "jaic");
    expect_header(response, "Date");
    expect_no_header(response, "X-Nothing");
    expect_equal(response.method, Method.GET);
    expect_equal(response.url, "/hello");

    user := expect_json(get(api, "/users/1"), User);

    expect_equal(user.name, "Ada");

    created := expect_json(post(api, "/users", New_User.{ "Grace" }), User, status = 201);

    expect_equal(created.name, "Grace");

    expect_problem(get(api, "/users/7"), 404, "No user with id 7");
    expect_problem(get(api, "/users/x"), 400, "invalid path parameter");
    expect_problem(post(api, "/users", New_User.{ "" }), 422, "name must not be empty");
    expect_problem(get(api, "/missing"), 404);
    expect_problem(delete(api, "/hello"), 405);
    expect_problem(get(api, "/taken"), 409, "already exists");

    expect_redirect(get(api, "/home"), "/hello", 302);
    expect_status(get(api, "/empty"), 201);

    // A redirect is a response to check, or followed when the request asks for it.
    followed := send(api, .{ url = "/home", max_redirects = 5 });

    expect_status(followed, 200);
    expect_text(followed, "hello");

    // HEAD and OPTIONS come from the router.
    head_response := head(api, "/hello");

    expect_status(head_response, 200);
    expect_header(head_response, "Content-Length", "5");

    head_body, head_error := read_text(*head_response);

    expect_equal(head_error.kind, Client_Error_Kind.NONE);
    expect_equal(head_body.count, 0);

    expect_status(options(api, "/hello"), 204);

    // Any host reaches the app, with the Host header as written; a base URL is optional.
    expect_text(get(api, "http://example.test/hello"), "hello");

    // The body readers are the real client's.
    user_response := get(api, "/users/1");
    again, error := read_json(*user_response, User);

    expect_equal(error.kind, Client_Error_Kind.NONE);
    expect_equal(again.id, 1);

    missing := get(api, "/users/9");
    status_error := error_for_status(*missing);

    expect_equal(status_error.kind, Client_Error_Kind.STATUS);
    expect_equal(status_error.status, 404);
    expect_equal(status_error.detail, "No user with id 9");

    hello_response := get(api, "/hello");
    text, text_error := read_text(*hello_response);

    expect_equal(text_error.kind, Client_Error_Kind.NONE);
    expect_equal(text, "hello");

    // A body is read once.
    _, twice := read_text(*hello_response);

    expect_equal(twice.kind, Client_Error_Kind.BODY);

    expect_equal(delete(api, "/hello").status, 405);
}

// A failing check shows the body, also when an earlier check or the test itself already looked
// at it, and the test can still read it afterwards.
bodies_in_failures :: () {
    api := test_client(*app);

    begin_failure_capture();

    response := get(api, "/users/1");

    expect_status(response, 404);
    expect_header(response, "X-Nope");

    text, count := end_failure_capture();

    expect_equal(count, 2);
    expect(find_index_from_left(text, "\"name\":\"Ada\"") >= 0, "the body is shown");

    // Looking did not use the body up.
    user, error := read_json(*response, User);

    expect_equal(error.kind, Client_Error_Kind.NONE);
    expect_equal(user.name, "Ada");

    // A body the test read itself is shown from the part that was kept.
    begin_failure_capture();

    read_response := get(api, "/users/1");
    _, _ := read_text(*read_response);

    expect_status(read_response, 404);

    text, count = end_failure_capture();

    expect_equal(count, 1);
    expect(find_index_from_left(text, "\"name\":\"Ada\"") >= 0, "the start of a body that was read is shown");
}

relative_urls :: () {
    api := test_client(*app);

    // Without a base URL the memory transport answers for `http://localhost`.
    expect_text(get(api, "hello"), "hello");
    expect_text(get(api, "/hello?x=1"), "hello");

    plain: Client;

    response, error := send(*plain, .{ url = "/hello" });

    expect_equal(error.kind, Client_Error_Kind.INVALID_URL);
    expect_equal(response.status, 0);
    expect(find_index_from_left(response.reason, "no base_url") >= 0, "the failure says why");
}

state_is_the_tests :: () {
    world.hits = 0;

    api := test_client(*app);

    get(api, "/bump");
    get(api, "/bump");

    expect_equal(world.hits, 2, "the handler ran twice and the test sees its state");

    count := expect_json(get(api, "/bump"), int);

    expect_equal(count, 3);
    expect_equal(world.hits, 3);
}

through_the_whole_stack :: () {
    api := test_client(*app);

    // A request body over the 64 KiB the server reads in place, and a response of the same size.
    body: [..] u8;

    for 1..300_000 array_add(*body, cast(u8) (it & 0xff));

    posted := send(api, .{ method = .POST, url = "/length", body = body });

    expect_text(posted, "300000");

    large := get(api, "/big");

    expect_status(large, 200);
    large_body, _ := read_bytes(*large);

    expect_equal(large_body.count, 300_000);
    expect_equal(large_body[299_999], cast(u8) (300_000 & 0xff));

    // The server's limits answer for themselves.
    app.config.max_body_size = 1000;

    expect_problem(send(api, .{ method = .POST, url = "/length", body = body }), 413);

    app.config.max_body_size = 1024 * 1024;

    // The client may ask for an interim response first.
    expected := send(api, .{ method = .POST, url = "/length", headers = .[.{ "Expect", "100-continue" }], body = bytes_of("12345") });

    expect_text(expected, "5");

    // A request the codec refuses is answered by the codec.
    smuggled := send(api, .{ url = "/hello", headers = .[.{ "X-Bad", "a\r\nb" }] });

    expect_equal(smuggled.status, 0);
}

keeps_connections_alive :: () {
    Io.io_run(keep_alive_inside);
}

keep_alive_inside :: () {
    api := test_client(*app);

    before := Io.in_scheduler();

    expect(before, "this runs on a fiber");

    for 1..5 expect_status(get(api, "/hello"), 200);

    expect_equal(http_statistics().connections_opened, 1, "five requests, one connection");
    expect_equal(http_statistics().idle_connections, 1);

    // Connection: close ends it; the next request opens another.
    closing := get(api, "/hello", .[.{ "Connection", "close" }]);

    expect_header(closing, "Connection", "close");
    expect_equal(http_statistics().idle_connections, 0);

    expect_status(get(api, "/hello"), 200);
    expect_equal(http_statistics().connections_opened, 2);

    // The server closes after its request limit, and the client opens a new connection.
    app.config.max_requests_per_connection = 2;

    for 1..5 expect_status(get(api, "/hello"), 200);

    app.config.max_requests_per_connection = 10_000;

    expect(http_statistics().connections_opened >= 4, tprint("a limit of 2 requests per connection opened % connections", http_statistics().connections_opened));

    // Another client has connections of its own.
    other := test_client(*app);

    opened := http_statistics().connections_opened;

    expect_status(get(other, "/hello"), 200);
    expect_equal(http_statistics().connections_opened, opened + 1);
}

Several :: struct {
    api: *Client;
    group: Io.Wait_Group;
}

fetch_bump :: (several: *Several) {
    expect_status(get(several.api, "/bump"), 200);

    Io.done(*several.group);
}

concurrent_fibers :: () {
    Io.io_run(concurrently);
}

concurrently :: () {
    world.hits = 0;

    several := New(Several);

    several.api = test_client(*app);

    Io.add(*several.group, 20);

    for 1..20 Io.spawn(fetch_bump, several);

    Io.wait(*several.group);

    expect_equal(world.hits, 20, "twenty requests in flight at once");
    expect(http_statistics().connections_opened > 1, "concurrent requests use separate connections");
}

handlers_may_use_io :: () {
    api := test_client(*app);

    // Sleeping, channels between fibers and a second app, all without a scheduler in the test.
    expect_text(get(api, "/rest/20"), "rested");
    expect_text(get(api, "/channel"), "got 42");

    world.other = test_client(*backend);

    expect_text(get(api, "/backend"), "backend: hello");

    gone_backend := test_client(*empty_backend);

    world.other = gone_backend;

    expect_problem(get(api, "/backend"), 500, "the backend said");
}

timeouts :: () {
    api := test_client(*app);

    response, error := send(api, .{ url = "/rest/400", timeout = 50 });

    expect_equal(error.kind, Client_Error_Kind.TIMED_OUT);
    expect_equal(response.status, 0);

    api.timeout = 50;

    _, again := get(api, "/rest/400");

    expect_equal(again.kind, Client_Error_Kind.TIMED_OUT);

    // A slow handler within the limit is fine.
    api.timeout = 5000;

    expect_text(get(api, "/rest/10"), "rested");
}

default_headers :: () {
    api := test_client(*app);

    expect_problem(get(api, "/token"), 400, "x-token");

    set_header(api, "X-Token", "abc");

    expect_text(get(api, "/token"), "abc");

    set_header(api, "x-token", "def");

    expect_text(get(api, "/token"), "def");
    expect_text(get(api, "/token", .[.{ "X-Token", "mine" }]), "mine");

    remove_header(api, "X-Token");

    expect_problem(get(api, "/token"), 400);
}

over_a_socket :: () {
    api := test_client(*app, .{ over_tcp = true });

    expect_text(get(api, "/hello"), "hello");
    expect_json(get(api, "/users/1"), User);
    expect_problem(get(api, "/users/2"), 404);

    who_am_i := get(api, "/who");

    expect_body_contains(who_am_i, "127.0.0.1:");

    // The same client keeps its connection when it runs inside a scheduler.
    Io.io_run(socket_inside);
}

socket_inside :: () {
    api := test_client(*app, .{ over_tcp = true });

    for 1..3 expect_status(get(api, "/hello"), 200);

    expect_equal(http_statistics().connections_opened, 1);
}

conflicting_routes :: () {
    clashing: App;

    get(*clashing, "/a/{x}", hello);
    get(*clashing, "/a/{y}", hello);

    api := test_client(*clashing);

    response, error := send(api, .{ url = "/a/1" });

    expect_equal(error.kind, Client_Error_Kind.TRANSPORT);
    expect_equal(response.status, 0);
}

failure_text :: () {
    api := test_client(*app);

    begin_failure_capture();

    expect_status(get(api, "/users/9"), 200);

    text, count := end_failure_capture();

    expect_equal(count, 1);

    expect_equal(after_location(text), "    expected status 200 OK, got 404 Not Found\n      request   GET /users/9\n      response  404 Not Found\n                Content-Type: application/problem+json\n                {\"type\":\"about:blank\",\"title\":\"Not Found\",\"status\":404,\"detail\":\"No user with id 9\",\"instance\":\"/users/9\"}\n");
    expect(find_index_from_left(text, "tests/testing.jai") >= 0 || find_index_from_left(text, "testing.jai") >= 0, "the location names this file");
}

failure_messages :: () {
    api := test_client(*app);

    // A request with a body shows it.
    begin_failure_capture();

    expect_problem(post(api, "/users", New_User.{ "Edsger" }), 400);

    text, count := end_failure_capture();

    expect_equal(count, 1);
    expect_equal(after_location(text), "    expected a problem with status 400 Bad Request, got 201 Created\n      request   POST /users  {\"name\":\"Edsger\"}\n      response  201 Created\n                Content-Type: application/json\n                {\"id\":2,\"name\":\"Edsger\"}\n");

    // A header that differs shows the one that came.
    begin_failure_capture();

    expect_header(get(api, "/hello"), "Content-Type", "application/json");
    expect_header(get(api, "/hello"), "X-Missing", "1");
    expect_header(get(api, "/hello"), "X-Missing");
    expect_no_header(get(api, "/hello"), "Server");

    text, count = end_failure_capture();

    expect_equal(count, 4);
    expect(find_index_from_left(text, "expected header `Content-Type: application/json`, got `Content-Type: text/plain; charset=utf-8`") >= 0, "a differing header");
    expect(find_index_from_left(text, "expected header `X-Missing: 1`, but the response has no `X-Missing` header") >= 0, "a missing header");
    expect(find_index_from_left(text, "expected a `X-Missing` header, but the response has none") >= 0, "a missing header without a value");
    expect(find_index_from_left(text, "expected no `Server` header, got `Server: jaic`") >= 0, "an unwanted header");

    // JSON that is not what was asked for.
    begin_failure_capture();

    expect_json(get(api, "/hello"), User);
    expect_json(get(api, "/users/1"), int);
    expect_json(get(api, "/users/5"), User);
    expect_json(get(api, "/users/1"), User, status = 201);

    text, count = end_failure_capture();

    expect_equal(count, 4);
    expect(find_index_from_left(text, "expected a JSON body, but the Content-Type is `text/plain; charset=utf-8`") >= 0, "not JSON");
    expect(find_index_from_left(text, "the body is not the JSON expected (a s64)") >= 0, "wrong shape");
    expect(find_index_from_left(text, "expected a 2xx status, got 404 Not Found") >= 0, "an error status");
    expect(find_index_from_left(text, "expected status 201 Created, got 200 OK") >= 0, "another status");

    // Problems.
    begin_failure_capture();

    expect_problem(get(api, "/hello"), 200);
    expect_problem(get(api, "/users/5"), 404, "something else");
    expect_problem(get(api, "/users/5"), 400);

    text, count = end_failure_capture();

    expect_equal(count, 3);
    expect(find_index_from_left(text, "expected an application/problem+json body, but the Content-Type is `text/plain; charset=utf-8`") >= 0, "not a problem");
    expect(find_index_from_left(text, "expected the problem detail to contain \"something else\", got \"No user with id 5\"") >= 0, "other detail");
    expect(find_index_from_left(text, "expected a problem with status 400 Bad Request, got 404 Not Found") >= 0, "other status");

    // Bodies: binary, long, and text.
    begin_failure_capture();

    expect_text(get(api, "/binary"), "x");
    expect_text(get(api, "/long"), "x");
    expect_body_contains(get(api, "/hello"), "bye");
    expect_redirect(get(api, "/hello"), "/x");
    expect_redirect(get(api, "/home"), "/elsewhere");

    text, count = end_failure_capture();

    expect_equal(count, 5);
    expect(find_index_from_left(text, "<4 bytes of binary data>") >= 0, "binary body");
    expect(find_index_from_left(text, "... (600 more bytes)") >= 0, "long body is cut");
    expect(find_index_from_left(text, "expected the body to contain \"bye\"") >= 0, "contains");
    expect(find_index_from_left(text, "expected a 3xx redirect, got 200 OK") >= 0, "not a redirect");
    expect(find_index_from_left(text, "expected a redirect to `/elsewhere`, got `/hello`") >= 0, "another location");

    // Plain values.
    begin_failure_capture();

    expect(1 + 1 == 3, "arithmetic");
    expect(false);
    expect_equal(2, 3, "the count");
    expect_equal("a", "b");

    text, count = end_failure_capture();

    expect_equal(count, 4);
    expect(find_index_from_left(text, "expected: arithmetic") >= 0, "expect with a reason");
    expect(find_index_from_left(text, "the condition is false") >= 0, "expect without");
    expect(find_index_from_left(text, "the count: expected 3, got 2") >= 0, "numbers");
    expect(find_index_from_left(text, "expected \"b\", got \"a\"") >= 0, "strings");

    // No response at all names the request and why.
    begin_failure_capture();

    slow := send(api, .{ url = "/rest/300", timeout = 20 });

    expect_status(slow, 200);

    text, count = end_failure_capture();

    expect_equal(count, 1);
    expect(find_index_from_left(text, "request   GET /rest/300") >= 0, "the request");
    expect(find_index_from_left(text, "response  none: timed out while reading the response") >= 0, "the reason");

    // What holds counts as a check and prints nothing.
    begin_failure_capture();

    expect_status(get(api, "/hello"), 200);
    expect_json(get(api, "/users/1"), User);

    text, count = end_failure_capture();

    expect_equal(count, 0);
    expect_equal(text, "");
}

main :: () {
    if !supported() return;

    make_app(*app);

    get(*backend, "/hello", hello);

    get(*empty_backend, "/other", hello);

    for 1..300_000 array_add(*big_buffer, cast(u8) (it & 0xff));

    run_tests(.[
        .{ "requests, responses and every expectation", basics },
        .{ "relative URLs and the base URL", relative_urls },
        .{ "handlers share state with the test", state_is_the_tests },
        .{ "large bodies, limits and interim responses", through_the_whole_stack },
        .{ "connections are kept alive and counted", keeps_connections_alive },
        .{ "fibers in flight together", concurrent_fibers },
        .{ "handlers may sleep, use channels and call other apps", handlers_may_use_io },
        .{ "timeouts", timeouts },
        .{ "default headers", default_headers },
        .{ "over a real socket", over_a_socket },
        .{ "an app with conflicting routes", conflicting_routes },
        .{ "the text of a failed status check", failure_text },
        .{ "bodies in failures", bodies_in_failures },
        .{ "the text of every other failed check", failure_messages },
    ]);
}
