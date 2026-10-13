// The client against the server and against hand-made servers: the body readers and their
// errors, how a response holds and releases its connection, the pool (reuse, eviction, stale
// connections), redirects, timeouts, refused connections, https URLs and URL errors, chunked and
// until-close responses.
#load "lib/server_support.jai";

api: Client;

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

big_bytes: [..] u8;

// 200 000 bytes of the letter a.
big_body :: () -> [] u8 {
    return big_bytes;
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

// `GET url` and its whole body, the way a script would write it.
fetch_text :: (url: string) -> string, Client_Error {
    response, error := get(*api, url);

    defer close(*response);

    if error.kind != .NONE return "", error;

    return read_text(*response);
}

pooling :: (base: string, server: *Server) {
    check(Io.backend_name().count > 0, "backend: %", Io.backend_name());

    before := connections_opened();

    for 1..5 {
        text, error := fetch_text(tprint("%/text", base));
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
    text, error := fetch_text(tprint("%/text", base));

    free(text.data);
    check(idle_connection_count() == 1, "one idle again");

    client_settings.idle_timeout = 60;
    Io.sleep(120);

    text, error = fetch_text(tprint("%/text", base));
    free(text.data);
    check(connections_opened() == opened + 2, "a connection that idled too long is not reused: % opened", connections_opened() - opened);

    client_settings.idle_timeout = 30_000;
}

// The state and read buffer of a response are recycled per scheduler: sequential requests reuse
// one, a complete body does not hold its buffer, and the pools are capped.
response_memory :: (base: string, server: *Server) {
    _, _, created := source_pool_counts();

    for 1..30 {
        text, error := fetch_text(tprint("%/text", base));

        check(error.kind == .NONE && text == "hello text", "sequential get: % %", text, error_text(error));
        free(text.data);
    }

    _, _, now_created := source_pool_counts();

    check(now_created - created <= 1, "30 sequential requests allocated % response states", now_created - created);

    kept, buffers, _ := source_pool_counts();

    check(kept >= 1 && buffers >= 1, "and left them for the next one: % states, % buffers", kept, buffers);

    // Responses that are complete when the head arrives hold no buffer, so many of them need one.
    held: [..] Client_Response;

    for 1..10 {
        response, error := get(*api, tprint("%/text", base));

        check(error.kind == .NONE, "held response: %", error_text(error));
        array_add(*held, response);
    }

    _, _, after_held := source_pool_counts();

    check(after_held - now_created <= 10, "ten open responses needed % states", after_held - now_created);

    for *held {
        text, error := read_text(it);

        check(error.kind == .NONE && text == "hello text", "a held response still reads: %", text);
        free(text.data);
        close(it);
    }

    array_reset(*held);

    // A body that has not arrived keeps its buffer until the response is closed, then both go back.
    big, error := get(*api, tprint("%/big", base));

    check(error.kind == .NONE && big.status == 200, "big response");

    buffers_before := 0;

    _, buffers_before, _ = source_pool_counts();
    close(*big);

    _, buffers_after, _ := source_pool_counts();

    check(buffers_after == buffers_before + 1, "closing an unread big response returned its buffer: % to %", buffers_before, buffers_after);

    // The pools are capped (64 each), also when more responses than that are open at once.
    saved := client_settings.max_open_responses;

    client_settings.max_open_responses = 1000;

    for 1..100 {
        response, failure := get(*api, tprint("%/big", base));

        if failure.kind == .NONE array_add(*held, response);
    }

    for *held close(it);

    array_reset(*held);

    client_settings.max_open_responses = saved;

    kept, buffers, _ = source_pool_counts();

    check(kept > 0 && kept <= 64 && buffers > 0 && buffers <= 64, "the pools are capped: % states, % buffers", kept, buffers);
    close_idle_connections();
}

ParallelArgs :: struct {
    url: string;
    group: *Io.Wait_Group;
}

parallel_get :: (args: ParallelArgs) {
    defer Io.done(args.group);

    text, error := fetch_text(args.url);

    check(error.kind == .NONE && text == "late", "parallel request: %", error_text(error));

    free(text.data);
}

json_calls :: (base: string, server: *Server) {
    response, error := post(*api, tprint("%/users", base), New_User.{ "Ada" });

    check(error.kind == .NONE && response.status == 201, "post: %", error_text(error));

    user, decode_error := read_json(*response, User);

    check(decode_error.kind == .NONE && user.id == 1 && user.name == "Ada", "post then read_json: % %", user.name, error_text(decode_error));
    free_json(*user);
    close(*response);

    response, error = get(*api, tprint("%/users/1", base));
    user, decode_error = read_json(*response, User);
    check(decode_error.kind == .NONE && user.name == "Ada", "get then read_json: %", error_text(decode_error));
    free_json(*user);
    close(*response);

    response, error = put(*api, tprint("%/users/1", base), New_User.{ "Ada Lovelace" });
    user, decode_error = read_json(*response, User);
    check(decode_error.kind == .NONE && user.name == "Ada Lovelace", "put then read_json: %", error_text(decode_error));
    free_json(*user);
    close(*response);

    // A 404 is a response; error_for_status turns it into an error with the problem's title and
    // detail and the body.
    response, error = get(*api, tprint("%/users/99", base));
    check(error.kind == .NONE && response.status == 404, "a missing user is a response: % %", response.status, error_text(error));

    status_error := error_for_status(*response);

    check(status_error.kind == .STATUS && status_error.status == 404, "error_for_status: %", error_text(status_error));
    check(status_error.title == "Not Found" && status_error.detail == "No user with id 99", "title and detail from the problem: % / %", status_error.title, status_error.detail);
    check(contains(status_error.body, "\"status\":404"), "the body is kept: %", status_error.body);
    free_error(*status_error);
    close(*response);

    response, error = get(*api, tprint("%/users/1", base));
    status_error = error_for_status(*response);
    check(status_error.kind == .NONE, "a 2xx is no error: %", error_text(status_error));

    // ...and it left the body alone.
    text, text_error := read_text(*response);

    check(text_error.kind == .NONE && contains(text, "Ada"), "the body of a 2xx after error_for_status: %", text);
    free(text.data);
    close(*response);

    response, error = post(*api, tprint("%/users", base), New_User.{ "" });
    status_error = error_for_status(*response);
    check(status_error.kind == .STATUS && status_error.status == 400 && status_error.detail == "name is required", "a 400 from the handler: %", error_text(status_error));
    free_error(*status_error);
    close(*response);

    // A reply that is not the JSON asked for.
    response, error = get(*api, tprint("%/not-json", base));
    user, decode_error = read_json(*response, User);
    check(decode_error.kind == .DECODE && decode_error.status == 200, "not JSON: %", error_text(decode_error));
    check(decode_error.json.kind != .NONE, "the JSON error is attached");
    free_error(*decode_error);
    close(*response);

    response, error = get(*api, tprint("%/text", base));
    user, decode_error = read_json(*response, User);
    check(decode_error.kind == .DECODE, "text where JSON was expected: %", error_text(decode_error));
    free_error(*decode_error);
    close(*response);

    response, error = post(*api, tprint("%/users", base), New_User.{ "Grace" });
    check(error.kind == .NONE && response.status == 201, "a post whose body is not wanted: %", error_text(error));
    close(*response);
    check(users.count == 2, "it did post: %", users.count);

    response, error = delete(*api, tprint("%/users/2", base));
    check(error.kind == .NONE && response.status == 204 && users.count == 1, "delete: % %", response.status, users.count);
    close(*response);

    response, error = delete(*api, tprint("%/users/2", base));
    check(error.kind == .NONE && response.status == 404, "deleting twice: %", response.status);
    close(*response);

    // send: every part of a request.
    response, error = send(*api, .{ method = .PUT, url = tprint("%/inspect?a=b", base), headers = .[.{ "X-Custom", "7" }, .{ "Authorization", "Bearer t" }], body = u8.[#char "h", #char "i"] });

    check(error.kind == .NONE && response.status == 200, "send: %", error_text(error));

    text, text_error = read_text(*response);
    check(text == "PUT /inspect?a=b body=[hi] auth=[Bearer t] ctype=[] custom=[7] agent=[jaic-http]", "what the server saw: %", text);
    free(text.data);
    close(*response);

    response, error = send(*api, .{ url = tprint("%/users/404", base) });
    check(error.kind == .NONE && response.status == 404, "a 404 is a response, not an error: %", error_text(error));
    close(*response);

    response, error = send(*api, .{ method = .HEAD, url = tprint("%/text", base) });
    text, text_error = read_text(*response);
    check(error.kind == .NONE && response.status == 200 && text.count == 0, "HEAD through send: % %", response.status, text.count);
    check(header(*response, "content-length") == "10", "with the length");
    close(*response);

    response, error = send(*api, .{ url = tprint("%/inspect", base), headers = .[.{ "User-Agent", "mine/1" }] });
    text, text_error = read_text(*response);
    check(contains(text, "agent=[mine/1]"), "a User-Agent of your own: %", text);
    free(text.data);
    close(*response);

    response, error = send(*api, .{ url = tprint("%/inspect", base), headers = .[.{ "Bad Header", "x" }] });
    check(error.kind != .NONE, "an invalid header is refused before sending");
    check(response.status == 0 && response.source == null, "the failure response has no status and no body");
    _, read_error := read_text(*response);

    check(read_error.kind == .BODY, "and reading it says so: %", error_text(read_error));
    free_error(*error);
    free_error(*read_error);
    close(*response);
}

// ---- The body readers, over hand-made responses of each framing

BODY_TEXT :: "hello world";

framing_replies :: (kind: int) -> string {
    if kind == 0 return "HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\nhello world";
    if kind == 1 return "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n6;a=b\r\n world\r\n0\r\nTrailer-X: 1\r\n\r\n";

    return "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nhello world";
}

FRAMING_NAMES :: string.["Content-Length", "chunked", "read until close"];

// One response per request, in order, and what the client did with the connection.
Framed :: struct {
    listener: Io.Tcp_Listener;
    replies: [..] string;
    count: int;
    url: string;
}

framed_server :: (framed: *Framed) -> *Framed {
    listener, error := Io.tcp_listen("127.0.0.1:0");

    check(error == .NONE, "framed listener");

    framed.listener = listener;

    address, _ := Io.local_address(listener);

    framed.url = copy_string(tprint("http://%/x", Io.address_to_string(address)));

    Io.spawn(framed_accept, framed);

    return framed;
}

framed_accept :: (framed: *Framed) {
    while true {
        stream, error := Io.accept(framed.listener);

        if error != .NONE break;

        Io.spawn(framed_connection, FramedArgs.{ framed, stream });
    }
}

FramedArgs :: struct {
    framed: *Framed;
    stream: Io.Tcp_Stream;
}

// Answers every request that arrives on the connection with the next reply (a reply that says
// `Connection: close` ends it).
framed_connection :: (args: FramedArgs) {
    defer Io.close(args.stream);

    buffer: [2048] u8;
    seen: [..] u8;

    while true {
        count, error := Io.read(args.stream, buffer);

        if error != .NONE || count == 0 return;

        for 0..count - 1 array_add(*seen, buffer[it]);

        while find_index_from_left(string.{ seen.count, seen.data }, "\r\n\r\n") >= 0 {
            text := string.{ seen.count, seen.data };
            end := find_index_from_left(text, "\r\n\r\n") + 4;

            for 0..seen.count - end - 1 seen[it] = seen[end + it];

            seen.count -= end;

            reply := args.framed.replies[args.framed.count % args.framed.replies.count];

            args.framed.count += 1;

            Io.write(args.stream, reply);

            if contains(reply, "Connection: close") return;
        }
    }
}

body_readers :: (base: string, server: *Server) {
    for kind: 0..2 {
        framed: Framed;

        framed_server(*framed);

        array_add(*framed.replies, framing_replies(kind));

        name := FRAMING_NAMES[kind];

        // read_text
        response, error := get(*api, framed.url);
        text, read_error := read_text(*response);

        check(error.kind == .NONE && read_error.kind == .NONE && text == BODY_TEXT, "% read_text: % %", name, text, error_text(read_error));
        free(text.data);
        close(*response);

        // read_bytes
        response, error = get(*api, framed.url);
        bytes, bytes_error := read_bytes(*response);

        check(bytes_error.kind == .NONE && string.{ bytes.count, bytes.data } == BODY_TEXT, "% read_bytes: %", name, error_text(bytes_error));
        free(bytes.data);
        close(*response);

        // read_json
        framed.replies[0] = replace(replace(framing_replies(kind), "hello world", "\"quoted\""), "Content-Length: 11", "Content-Length: 8");

        if kind == 1 framed.replies[0] = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\n\"qu\r\n5\r\noted\"\r\n0\r\n\r\n";

        response, error = get(*api, framed.url);
        quoted, json_error := read_json(*response, string);

        check(json_error.kind == .NONE && quoted == "quoted", "% read_json: % %", name, quoted, error_text(json_error));
        free_json(*quoted);
        close(*response);

        framed.replies[0] = framing_replies(kind);

        // read_some in pieces of 4 bytes
        response, error = get(*api, framed.url);

        collected: [..] u8;
        piece: [4] u8;
        steps := 0;

        while true {
            count, some_error := read_some(*response, piece);

            if some_error.kind != .NONE {
                check(false, "% read_some: %", name, error_text(some_error));

                break;
            }

            if count == 0 break;

            check(count <= 4, "% a piece fits the buffer: %", name, count);

            for 0..count - 1 array_add(*collected, piece[it]);

            steps += 1;
        }

        check(string.{ collected.count, collected.data } == BODY_TEXT && steps >= 3, "% read_some in pieces: % in % steps", name, string.{ collected.count, collected.data }, steps);

        count, again_error := read_some(*response, piece);

        check(count == 0 && again_error.kind == .NONE, "% the end stays the end: % %", name, count, error_text(again_error));

        _, after_error := read_text(*response);

        check(after_error.kind == .BODY, "% read_text after read_some to the end: %", name, error_text(after_error));
        free_error(*after_error);
        close(*response);

        // The body can be read once.
        response, error = get(*api, framed.url);
        text, read_error = read_text(*response);
        free(text.data);

        _, second_error := read_text(*response);

        check(second_error.kind == .BODY && contains(second_error.detail, "already read"), "% a second read_text is an error: %", name, error_text(second_error));
        free_error(*second_error);

        _, second_bytes := read_bytes(*response);

        check(second_bytes.kind == .BODY, "% a second read_bytes: %", name, error_text(second_bytes));
        free_error(*second_bytes);

        _, second_json := read_json(*response, string);

        check(second_json.kind == .BODY, "% a second read_json: %", name, error_text(second_json));
        free_error(*second_json);

        _, second_some := read_some(*response, piece);

        check(second_some.kind == .BODY, "% read_some after a whole read: %", name, error_text(second_some));
        free_error(*second_some);

        // close is safe after the end, and again.
        close(*response);
        close(*response);
        check(response.status == 0 && response.source == null, "% close leaves an empty response", name);

        _, closed_error := read_text(*response);

        check(closed_error.kind == .BODY, "% reading a closed response: %", name, error_text(closed_error));
        free_error(*closed_error);

        // The connection went back to the pool when the body ended, but not for read until close.
        Io.close(framed.listener);
    }

    // The pool: a Content-Length or chunked body read to the end keeps the connection.
    close_idle_connections();

    for kind: 0..2 {
        framed: Framed;

        framed_server(*framed);

        array_add(*framed.replies, framing_replies(kind));

        before := connections_opened();

        for 1..3 {
            text, error := fetch_text(framed.url);
            check(error.kind == .NONE && text == BODY_TEXT, "% three requests: %", FRAMING_NAMES[kind], error_text(error));
            free(text.data);
        }

        opened := connections_opened() - before;

        if kind == 2 {
            check(opened == 3, "read until close: a connection each: %", opened);
            check(idle_connection_count() == 0, "read until close: nothing kept: %", idle_connection_count());
        } else {
            check(opened == 1, "% reading to the end keeps the connection: % opened", FRAMING_NAMES[kind], opened);
            check(idle_connection_count() == 1, "% and it is idle: %", FRAMING_NAMES[kind], idle_connection_count());
        }

        close_idle_connections();
        Io.close(framed.listener);
    }
}

// A body larger than max_response_size: the whole-body readers refuse it, read_some streams it.
large_bodies :: (base: string, server: *Server) {
    // /big answers with 200 000 bytes.
    response, error := send(*api, .{ url = tprint("%/big", base), max_response_size = 50_000 });

    check(error.kind == .NONE && response.status == 200, "/big: %", error_text(error));

    _, too_large := read_bytes(*response);

    check(too_large.kind == .TOO_LARGE, "read_bytes over the limit: %", error_text(too_large));
    free_error(*too_large);

    _, after := read_text(*response);

    check(after.kind == .BODY, "the connection is gone after that: %", error_text(after));
    free_error(*after);
    close(*response);

    // read_json refuses it the same way, before decoding anything.
    response, error = send(*api, .{ url = tprint("%/big", base), max_response_size = 50_000 });

    _, json_too_large := read_json(*response, User);

    check(json_too_large.kind == .TOO_LARGE, "read_json over the limit: %", error_text(json_too_large));
    free_error(*json_too_large);
    close(*response);

    // Streamed, the same body is fine, and the limit does not count.
    response, error = send(*api, .{ url = tprint("%/big", base), max_response_size = 50_000 });

    buffer: [4096] u8;
    total := 0;
    sum := 0;
    before := connections_opened();

    while true {
        count, some_error := read_some(*response, buffer);

        if some_error.kind != .NONE {
            check(false, "streaming: %", error_text(some_error));

            break;
        }

        if count == 0 break;

        for 0..count - 1 sum += buffer[it];

        total += count;
    }

    check(total == 200_000 && sum == 200_000 * 97, "streamed all of a body over the limit: % % ", total, sum);
    close(*response);

    // The connection survived the streamed body.
    text := fetch_text(tprint("%/text", base));

    check(text == "hello text" && connections_opened() == before, "the streamed connection was reused: % % ", text, connections_opened() - before);
    free(text.data);
}

// A response nobody read: its connection is not lost, not shared and not wedged.
unread_responses :: (base: string, server: *Server) {
    close_idle_connections();

    opened := connections_opened();

    // 1. Unread, then a new request on the same client: both work.
    first, error := get(*api, tprint("%/big", base));

    check(error.kind == .NONE && first.status == 200, "the first response");
    check(open_response_count() == 1, "it holds its connection: %", open_response_count());

    text, text_error := fetch_text(tprint("%/text", base));

    check(text == "hello text", "a request while a response is unread: % %", text, error_text(text_error));
    free(text.data);

    check(connections_opened() == opened + 2, "on a connection of its own: % opened", connections_opened() - opened);
    check(idle_connection_count() == 1, "only the finished one went back: %", idle_connection_count());

    // The unread one can still be read afterwards, completely.
    big, big_error := read_bytes(*first);

    check(big_error.kind == .NONE && big.count == 200_000, "the first response is intact: % %", big.count, error_text(big_error));
    free(big.data);
    check(open_response_count() == 0 && idle_connection_count() == 2, "reading released its connection: % open, % idle", open_response_count(), idle_connection_count());
    close(*first);

    // 2. close() on an unread short body drains it and keeps the connection.
    close_idle_connections();
    opened = connections_opened();

    short := get(*api, tprint("%/text", base));

    close(*short);
    close(*short);
    check(open_response_count() == 0 && idle_connection_count() == 1, "close of a short unread body keeps the connection: % open, % idle", open_response_count(), idle_connection_count());

    text, text_error = fetch_text(tprint("%/text", base));
    check(text == "hello text" && connections_opened() == opened + 1, "and the next request reuses it: % opened", connections_opened() - opened);
    free(text.data);

    // 3. close() on an unread long body closes the connection instead.
    close_idle_connections();
    opened = connections_opened();

    long := get(*api, tprint("%/big", base));

    close(*long);
    check(open_response_count() == 0 && idle_connection_count() == 0, "close of a long unread body closes the connection: % open, % idle", open_response_count(), idle_connection_count());

    text, text_error = fetch_text(tprint("%/text", base));
    check(text == "hello text", "the client is fine after that: %", text);
    free(text.data);

    // 4. A response that was only partly streamed is released the same way.
    partial := get(*api, tprint("%/big", base));

    piece: [100] u8;
    count, some_error := read_some(*partial, piece);

    check(count > 0 && some_error.kind == .NONE, "a first piece");
    close(*partial);
    check(open_response_count() == 0, "partly streamed, then closed: %", open_response_count());

    // 5. A response with no body never holds a connection.
    close_idle_connections();

    empty := head(*api, tprint("%/text", base));

    check(open_response_count() == 0 && idle_connection_count() == 1, "a HEAD response has nothing to wait for: % open, % idle", open_response_count(), idle_connection_count());
    close(*empty);

    // 5b. Neither does a small answer that is dropped unread: it came with its head and is
    // kept in memory, so the connection was free at once and the body is still readable.
    close_idle_connections();
    opened = connections_opened();

    small := get(*api, tprint("%/text", base));

    check(open_response_count() == 0 && idle_connection_count() == 1, "a small unread response holds nothing: % open, % idle", open_response_count(), idle_connection_count());

    text, text_error = fetch_text(tprint("%/text", base));
    check(text == "hello text" && connections_opened() == opened + 1, "the next request reuses the connection: % opened", connections_opened() - opened);
    free(text.data);

    text, text_error = read_text(*small);
    check(text == "hello text", "the dropped response can still be read: %", text);
    free(text.data);
    close(*small);

    // 6. Abandoned responses are bounded: past the limit the oldest is closed.
    close_idle_connections();

    saved := client_settings.max_open_responses;

    client_settings.max_open_responses = 3;

    abandoned: [5] Client_Response;

    for 0..4 abandoned[it], error = get(*api, tprint("%/big", base));

    check(open_response_count() == 3, "no more than the limit are held: %", open_response_count());

    _, evicted := read_text(*abandoned[0]);

    check(evicted.kind == .BODY && contains(evicted.detail, "max_open_responses"), "the oldest was closed, and says why: %", error_text(evicted));
    free_error(*evicted);

    newest, newest_error := read_bytes(*abandoned[4]);

    check(newest_error.kind == .NONE && newest.count == 200_000, "the newest is fine: %", error_text(newest_error));
    free(newest.data);

    for 0..4 close(*abandoned[it]);

    check(open_response_count() == 0, "all closed: %", open_response_count());

    client_settings.max_open_responses = saved;
    close_idle_connections();
}

follow :: (base: string, path: string, method := Method.GET, body: [] u8 = .[]) -> Reply {
    response, error := send(*api, .{ method = method, url = tprint("%%", base, path), body = body, headers = .[.{ "Authorization", "Bearer keep" }] });

    check(error.kind == .NONE, "% %: %", method_name(method), path, error_text(error));

    return into_reply(*response);
}

redirects :: (base: string, server: *Server) {
    other_origin = replace(base, "127.0.0.1", "localhost");
    response := follow(base, "/moved");

    check(response.status == 200 && contains(body_of(response), "GET /inspect?moved=1"), "301 on GET: %", body_of(response));
    check(contains(body_of(response), "auth=[Bearer keep]"), "credentials are kept on the same host: %", body_of(response));

    response = follow(base, "/found", .POST, u8.[#char "x"]);
    check(contains(body_of(response), "GET /inspect?found=1 body=[]"), "302 turns a POST into a GET: %", body_of(response));

    response = follow(base, "/see-other", .PUT, u8.[#char "x"]);
    check(contains(body_of(response), "GET /inspect?seen=1 body=[]"), "303 turns a PUT into a GET: %", body_of(response));

    response = follow(base, "/temporary", .POST, u8.[#char "p", #char "q"]);
    check(contains(body_of(response), "POST /inspect?temporary=1 body=[pq]"), "307 repeats the request: %", body_of(response));

    response = follow(base, "/permanent", .PUT, u8.[#char "z"]);
    check(contains(body_of(response), "PUT /inspect?permanent=1 body=[z]"), "308 repeats the request: %", body_of(response));

    response = follow(base, "/relative");
    check(body_of(response) == "sibling reached", "a relative Location: %", body_of(response));

    // Another host: the credentials stay home. "localhost" and "127.0.0.1" are different origins.
    response = follow(base, "/to-other");
    check(response.status == 200 && contains(body_of(response), "auth=[]"), "no Authorization to another host: %", body_of(response));

    // Not following.
    raw, error := send(*api, .{ url = tprint("%/moved", base), max_redirects = 0 });

    check(error.kind == .NONE && raw.status == 301 && header(*raw, "Location") == "/inspect?moved=1", "max_redirects = 0: %", raw.status);
    close(*raw);

    // Too many.
    raw, error = send(*api, .{ url = tprint("%/loop", base), max_redirects = 4 });
    check(error.kind == .REDIRECTS && contains(error.detail, "more than 4 redirects"), "a redirect loop: %", error_text(error));
    free_error(*error);

    // The answers in between are released, and one connection served the whole chain.
    close_idle_connections();

    opened := connections_opened();

    raw, error = send(*api, .{ url = tprint("%/moved", base) });
    check(error.kind == .NONE && raw.status == 200, "a followed redirect: %", error_text(error));
    check(open_response_count() == 0, "a small answer is in memory, its connection is back: %", open_response_count());

    text := read_text(*raw);

    check(contains(text, "GET /inspect?moved=1"), "its body: %", text);
    free(text.data);
    close(*raw);
    check(open_response_count() == 0 && connections_opened() == opened + 1 && idle_connection_count() == 1, "one connection for the chain: % open, % opened, % idle", open_response_count(), connections_opened() - opened, idle_connection_count());

    // A chain that fails holds nothing.
    close_idle_connections();

    raw, error = send(*api, .{ url = tprint("%/loop", base), max_redirects = 4 });
    free_error(*error);
    check(open_response_count() == 0, "a failed redirect chain holds nothing: %", open_response_count());
}

failures_of_transport :: (base: string, server: *Server) {
    // Timeout.
    started := current_time_monotonic();
    response, error := send(*api, .{ url = tprint("%/slow/600", base), timeout = 150 });
    elapsed := to_milliseconds(current_time_monotonic() - started);

    close(*response);

    check(error.kind == .TIMEOUT, "timeout: %", error_text(error));
    check(elapsed >= 100 && elapsed < 500, "after about 150 ms: % ms", elapsed);
    free_error(*error);

    // The client is usable afterwards.
    text, text_error := fetch_text(tprint("%/text", base));

    check(text_error.kind == .NONE, "after a timeout: %", error_text(text_error));
    free(text.data);

    // The connection that timed out is not reused by mistake: its late answer would be read as the next one.
    text, text_error = fetch_text(tprint("%/text", base));
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

    response, error = send(*api, .{ url = refused_url, timeout = 2000 });
    check(error.kind == .TRANSPORT && error.transport == .CONNECTION_REFUSED, "a refused connection: % %", error_text(error), error.transport);
    free_error(*error);

    // URLs.
    // https is a scheme like http: a closed port refuses the connection before any TLS starts.
    response, error = send(*api, .{ url = tprint("https%", slice(refused_url, 4, refused_url.count - 4)), timeout = 2000 });
    check(error.kind == .TRANSPORT && error.transport == .CONNECTION_REFUSED, "https to a closed port: % %", error_text(error), error.transport);
    free_error(*error);

    response, error = send(*api, .{ url = "ftp://example.com/" });
    check(error.kind == .INVALID_URL && contains(error.detail, "scheme `ftp`") && contains(error.detail, "http or https"), "other schemes: %", error_text(error));
    free_error(*error);

    response, error = send(*api, .{ url = "example.com/path" });
    check(error.kind == .INVALID_URL && contains(error.detail, "not an absolute URL"), "no scheme: %", error_text(error));
    free_error(*error);

    response, error = send(*api, .{ url = "http:///path" });
    check(error.kind == .INVALID_URL && contains(error.detail, "names no host"), "no host: %", error_text(error));
    free_error(*error);

    response, error = send(*api, .{ url = "http://localhost:99999/" });
    check(error.kind == .INVALID_URL && contains(error.detail, "the port"), "bad port: %", error_text(error));
    free_error(*error);

    response, error = send(*api, .{ url = "http://no-such-host.invalid/", timeout = 3000 });
    check(error.kind == .TRANSPORT || error.kind == .TIMEOUT, "an unknown host: %", error_text(error));
    free_error(*error);
}

// A stale pooled connection: the server closes it while idle, and the next request still works.
stale :: (base: string, server: *Server) {
    opened := connections_opened();

    text, error := fetch_text(tprint("%/text", base));

    check(error.kind == .NONE, "first: %", error_text(error));
    free(text.data);

    // The server's idle timeout (100 ms) closes it behind the client's back.
    Io.sleep(300);

    text, error = fetch_text(tprint("%/text", base));
    check(error.kind == .NONE && text == "hello text", "a request on a stale connection is retried: %", error_text(error));
    free(text.data);
    check(connections_opened() == opened + 2, "on a new connection: % opened", connections_opened() - opened);

    // A POST is not repeated behind the caller's back.
    Io.sleep(300);

    post_response, post_error := post(*api, tprint("%/users", base), New_User.{ "Once" });

    close(*post_response);
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
    array_add(*script.replies, "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n8\r\n[1,2,3,4\r\n8\r\n,5,6,7,8\r\n1\r\n]\r\n0\r\n\r\n");

    Io.spawn(scripted_server, *script);

    response, send_error := send(*api, .{ url = url });
    text, read_error := read_text(*response);

    check(send_error.kind == .NONE && text == "hello world" && header(*response, "X-Kind") == "chunked", "a chunked response: % %", text, error_text(send_error));
    close(*response);

    response, send_error = send(*api, .{ url = url });
    text, read_error = read_text(*response);
    check(send_error.kind == .NONE && text == "until the end", "a response that ends with the connection: % %", text, error_text(send_error));
    close(*response);

    response, send_error = send(*api, .{ url = url });
    text, read_error = read_text(*response);
    check(send_error.kind == .NONE && response.status == 201 && text == "done", "a 100 Continue before the answer: % %", response.status, error_text(send_error));
    close(*response);

    // A body shorter than its Content-Length: the head is a response, reading the body fails.
    response, send_error = send(*api, .{ url = url });
    check(send_error.kind == .NONE && response.status == 200, "the head of a short body: %", error_text(send_error));

    text, read_error = read_text(*response);
    check(read_error.kind == .TRANSPORT, "a body shorter than its Content-Length: %", error_text(read_error));
    free_error(*read_error);

    _, again_error := read_text(*response);

    check(again_error.kind == .BODY, "and the body stays failed: %", error_text(again_error));
    free_error(*again_error);
    close(*response);
    check(open_response_count() == 0, "a failed read holds nothing: %", open_response_count());

    response, send_error = send(*api, .{ url = url });
    check(send_error.kind == .TRANSPORT && contains(send_error.detail, "not valid HTTP"), "garbage: %", error_text(send_error));
    check(response.status == 0 && contains(response.reason, "not valid HTTP"), "the failure response says why: %", response.reason);
    free_error(*send_error);
    close(*response);

    response, send_error = send(*api, .{ url = url });
    check(send_error.kind == .TRANSPORT, "two different Content-Lengths: %", error_text(send_error));
    free_error(*send_error);
    close(*response);

    // A chunked body has no length to check first: read_json stops once it passes the limit.
    response, send_error = send(*api, .{ url = url, max_response_size = 10 });
    check(send_error.kind == .NONE, "a chunked JSON body: %", error_text(send_error));

    _, limit_error := read_json(*response, [] int);

    check(limit_error.kind == .TOO_LARGE, "a chunked body over the limit: %", error_text(limit_error));
    free_error(*limit_error);
    close(*response);

    Io.close(script.listener);
}

main :: () {
    if !supported() return;

    other_origin = "";

    for 1..200_000 array_add(*big_bytes, #char "a");

    get(*app, "/text", text_hello);
    get(*app, "/big", big_body);
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
    run_server(*app, response_memory);
    run_server(*app, json_calls);
    run_server(*app, body_readers);
    run_server(*app, large_bodies);
    run_server(*app, unread_responses);
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
