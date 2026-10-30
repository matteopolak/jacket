// test: skip_on(.WASM, reason = "needs the Http server and router, which are not built for wasm")
// Request bodies read by the handler (`*Body`: `read`, `read_all`, `read_json`), the per-route
// body limit, bodies a handler leaves unread, `created`, and streamed responses
// (`stream_response`, `file_response`).
#load "support/server_support.jai";
#import "Extensions/Json";
#import "File";

Point :: struct {
    x: int;
    y: int;
}

client: Client;

// ---- Handlers

// Reads in pieces smaller than the body, so the pieces of the reads and of the socket differ.
checksum :: (body: *Body) -> string, Http_Error {
    buffer: [7] u8;
    total := 0;
    size := 0;

    while true {
        count, error := read(body, buffer);

        if error.kind != .NONE return "", error;

        if count == 0 break;

        for 0..count - 1 total += buffer[it];

        size += count;
    }

    return tprint("% bytes sum %", size, total), .{};
}

streamed_json :: (body: *Body) -> string, Http_Error {
    point, error := read_json(body, Point);

    if error.kind != .NONE return "", error;

    return tprint("%,%", point.x, point.y), .{};
}

buffered_json :: (body: Json(Point)) -> string {
    return tprint("%;%", body.temporary.x, body.temporary.y);
}

refuses :: (body: *Body) -> Http_Error {
    return forbidden("no uploads");
}

ignores :: () -> string {
    return "ignored";
}

whole :: (body: *Body) -> string, Http_Error {
    bytes, error := read_all(body);

    if error.kind != .NONE return "", error;

    return tprint("%", bytes.count), .{};
}

declared :: (body: *Body) -> string {
    return tprint("%", body.content_length);
}

make_point :: (body: Json(Point)) -> Response {
    return created(body.temporary, tprint("/points/%", body.temporary.x));
}

ping :: () -> string {
    return "pong";
}

ticker :: (stream: *Stream, data: *void) {
    for 1..3 {
        if !send_chunk(stream, tprint("tick %\n", it)) return;
    }

    big: [100_000] u8;

    for *big it.* = #char "z";

    send_chunk(stream, big);
}

events :: () -> Response {
    return stream_response(ticker, null, "text/plain");
}

BIG_FILE :: "/tmp/jacket-http-stream-test-file.bin";
BIG_SIZE :: 200_000;

small_file :: () -> Response {
    return file_response(tprint("%/fixtures/stream-file.txt", #filepath));
}

big_file :: () -> Response {
    return file_response(BIG_FILE, "application/x-test");
}

missing_file :: () -> Response {
    return file_response("/nonexistent/for/sure.txt");
}

// ---- Tests

reading :: (base: string, server: *Server) {
    body: [..] u8;
    sum := 0;

    for 0..999 {
        value := cast(u8) (it % 251);

        array_add(*body, value);
        sum += value;
    }

    text := string.{ body.count, body.data };

    response := post_body(*client, tprint("%/sum", base), text, "application/octet-stream");

    check(response.status == 200 && body_of(response) == tprint("1000 bytes sum %", sum), "read in small pieces: % %", response.status, body_of(response));

    response = post_body(*client, tprint("%/sum", base), "", "text/plain");
    check(response.status == 200 && body_of(response) == "0 bytes sum 0", "read of an empty body: % %", response.status, body_of(response));

    response = post_body(*client, tprint("%/stream-json", base), "{\"x\":3,\"y\":4}");
    check(response.status == 200 && body_of(response) == "3,4", "read_json: % %", response.status, body_of(response));

    response = post_body(*client, tprint("%/stream-json", base), "{\"x\":3,\"y\":4}", "text/plain");
    check(response.status == 415, "read_json wants a JSON content type: %", response.status);

    response = post_body(*client, tprint("%/stream-json", base), "{\"x\":3");
    check(response.status == 400, "read_json of broken JSON: %", response.status);

    response = post_body(*client, tprint("%/stream-json", base), "{\"x\":\"a\",\"y\":4}");
    check(response.status == 422 && contains(body_of(response), "\"field\":\"x\""), "read_json of the wrong shape: % %", response.status, body_of(response));

    response = post_body(*client, tprint("%/declared", base), "12345", "text/plain");
    check(body_of(response) == "5", "content_length: %", body_of(response));

    // A chunked request body goes through the same reads.
    stream := raw_connect(server);

    send_text(stream, "POST /sum HTTP/1.1\r\nHost: t\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n4\r\ndefg\r\n0\r\n\r\n");

    responses := read_responses(stream, 1, 2000);

    check(responses.count == 1 && responses[0].body == tprint("7 bytes sum %", 97 + 98 + 99 + 100 + 101 + 102 + 103), "a chunked body read in pieces: %", responses.count);

    send_text(stream, "POST /declared HTTP/1.1\r\nHost: t\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n0\r\n\r\n");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].body == "-1", "content_length of a chunked body: %", responses.count);
    Io.close(stream);
}

limits :: (base: string, server: *Server) {
    // The server's limit is 1000 bytes (see main); /tiny allows 10 and /huge any size.
    thousand: [..] u8;

    for 1..1000 array_add(*thousand, #char "a");

    text := string.{ thousand.count, thousand.data };

    response := post_body(*client, tprint("%/sum", base), text, "text/plain");

    check(response.status == 200, "a body at the server's limit: %", response.status);

    response = post_body(*client, tprint("%/tiny", base), "{\"x\":1,\"y\":2}");
    check(response.status == 200 && body_of(response) == "1;2", "a body under the route's limit: % %", response.status, body_of(response));

    response = post_body(*client, tprint("%/tiny", base), "{\"x\":1,\"y\":2,\"extra\":null}");
    check(response.status == 413, "Json over the route's limit: %", response.status);

    thousand.count = 0;

    for 1..5000 array_add(*thousand, #char "b");

    text = string.{ thousand.count, thousand.data };

    response = post_body(*client, tprint("%/sum", base), text, "text/plain");
    check(response.status == 413 && contains(body_of(response), "limit of 1000 bytes"), "over the server's limit: % %", response.status, body_of(response));

    response = post_body(*client, tprint("%/huge", base), text, "text/plain");
    check(response.status == 200 && body_of(response) == "5000 bytes sum 490000", "a route without a limit: % %", response.status, body_of(response));

    // A chunked body is counted as it arrives, and the connection closes after the 413.
    stream := raw_connect(server);

    send_text(stream, "POST /sum HTTP/1.1\r\nHost: t\r\nTransfer-Encoding: chunked\r\n\r\n");

    chunk := tprint("190\r\n%\r\n", string.{ 400, thousand.data });

    for 1..3 send_text(stream, chunk);

    reply := read_until_closed(stream, 2000);

    check(contains(reply, "413"), "a chunked body over the limit: %", reply);
    Io.close(stream);
}

unread :: (base: string, server: *Server) {
    // A handler that never reads: the small body is dropped and the connection carries on.
    stream := raw_connect(server);

    send_text(stream, "POST /ignores HTTP/1.1\r\nHost: t\r\nContent-Length: 5\r\n\r\nhelloGET /ping HTTP/1.1\r\nHost: t\r\n\r\n");

    responses := read_responses(stream, 2, 2000);

    check(responses.count == 2 && responses[0].body == "ignored" && responses[1].body == "pong", "an unread body is skipped: %", responses.count);
    Io.close(stream);

    // Expect: 100-continue and a handler that refuses without reading: no 100, no body wanted.
    stream = raw_connect(server);

    send_text(stream, "POST /refuses HTTP/1.1\r\nHost: t\r\nContent-Length: 5\r\nExpect: 100-continue\r\n\r\n");

    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 403, "refused before the body is asked for: % %", responses.count, ifx responses.count then responses[0].status else 0);
    check(closes_within(stream, 1000), "and the connection closes, since the body may follow");
    Io.close(stream);

    // Expect: 100-continue and a handler that reads: 100 comes when it asks.
    stream = raw_connect(server);

    send_text(stream, "POST /whole HTTP/1.1\r\nHost: t\r\nContent-Length: 5\r\nExpect: 100-continue\r\n\r\n");

    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 100, "100 Continue when the handler reads: %", responses.count);

    send_text(stream, "hello");
    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].status == 200 && responses[0].body == "5", "then the answer: %", responses.count);
    Io.close(stream);

    // The same with the body already sent along with the head: 100 still comes, then the answer.
    stream = raw_connect(server);

    send_text(stream, "POST /whole HTTP/1.1\r\nHost: t\r\nContent-Length: 5\r\nExpect: 100-continue\r\n\r\nhello");

    responses = read_responses(stream, 2, 2000);
    check(responses.count == 2 && responses[0].status == 100 && responses[1].status == 200 && responses[1].body == "5", "100 Continue even when the body came with the head: %", responses.count);
    Io.close(stream);

    // A body too large to be worth skipping: the response goes out and the connection closes.
    stream = raw_connect(server);

    send_text(stream, "POST /ignores HTTP/1.1\r\nHost: t\r\nContent-Length: 900000\r\n\r\n");

    responses = read_responses(stream, 1, 2000);
    check(responses.count == 1 && responses[0].body == "ignored", "an unread large body still gets its answer: %", responses.count);
    check(closes_within(stream, 1000), "and the connection closes");
    Io.close(stream);
}

created_response :: (base: string, server: *Server) {
    response := post_body(*client, tprint("%/points", base), "{\"x\":7,\"y\":1}");

    check(response.status == 201 && header_of(response, "Location") == "/points/7" && contains(body_of(response), "\"x\":7"), "created(value, location): % %", response.status, header_of(response, "Location"));
}

streaming :: (base: string, server: *Server) {
    response := get_path(*client, tprint("%/events", base));

    check(response.status == 200 && starts_with(body_of(response), "tick 1\ntick 2\ntick 3\nzzzz") && body_of(response).count == 21 + 100_000, "a streamed response: % %", response.status, body_of(response).count);
    check(header_of(response, "Content-Type") == "text/plain" && !has_header(response, "Content-Length"), "streams have no length: %", header_of(response, "Content-Length"));

    response = get_path(*client, tprint("%/ping", base));
    check(body_of(response) == "pong", "the connection is usable after a stream: %", body_of(response));

    // HTTP/1.0 has no chunked encoding: the stream ends with the connection.
    stream := raw_connect(server);

    send_text(stream, "GET /events HTTP/1.0\r\n\r\n");

    text := read_until_closed(stream, 3000);

    check(contains(text, "tick 3\n") && !contains(text, "chunked"), "a stream to an HTTP/1.0 client: %", text.count);
    Io.close(stream);

    // HEAD gets the head only.
    stream = raw_connect(server);

    send_text(stream, "HEAD /events HTTP/1.1\r\nHost: t\r\n\r\nGET /ping HTTP/1.1\r\nHost: t\r\n\r\n");

    responses := read_responses(stream, 1, 2000, .HEAD);

    check(responses.count == 1 && responses[0].status == 200 && responses[0].body == "", "HEAD of a stream: %", responses.count);
    Io.close(stream);
}

files :: (base: string, server: *Server) {
    response := get_path(*client, tprint("%/file", base));

    check(response.status == 200 && body_of(response) == "a file served in pieces\n", "file_response: % %", response.status, body_of(response));
    check(header_of(response, "Content-Type") == "text/plain; charset=utf-8" && header_of(response, "Content-Length") == "24", "type from the extension, length from the file: % %", header_of(response, "Content-Type"), header_of(response, "Content-Length"));

    response = get_path(*client, tprint("%/big-file", base));

    ok := response.status == 200 && body_of(response).count == BIG_SIZE && header_of(response, "Content-Type") == "application/x-test";

    for 0..BIG_SIZE - 1 {
        if body_of(response)[it] != cast(u8) (it % 253) {
            ok = false;

            break;
        }
    }

    check(ok, "a file bigger than one piece, byte for byte: % %", response.status, body_of(response).count);

    response = get_path(*client, tprint("%/missing-file", base));
    check(response.status == 404 && contains(body_of(response), "does not exist"), "a missing file is a 404: % %", response.status, body_of(response));

    stream := raw_connect(server);

    send_text(stream, "HEAD /file HTTP/1.1\r\nHost: t\r\n\r\n");

    responses := read_responses(stream, 1, 2000, .HEAD);

    check(responses.count == 1 && responses[0].status == 200 && raw_header(responses[0], "Content-Length") == "24", "HEAD of a file: %", responses.count);
    Io.close(stream);
}

run_with :: (config: Server_Config, test: Server_Test) {
    local: App;

    local.config = config;

    post(*local, "/sum", checksum);
    post(*local, "/huge", checksum, max_body = -1);
    post(*local, "/stream-json", streamed_json);
    post(*local, "/tiny", buffered_json, max_body = 20);
    post(*local, "/refuses", refuses);
    post(*local, "/ignores", ignores);
    post(*local, "/whole", whole);
    post(*local, "/declared", declared);
    post(*local, "/points", make_point);
    get(*local, "/ping", ping);
    get(*local, "/events", events);
    get(*local, "/file", small_file);
    get(*local, "/big-file", big_file);
    get(*local, "/missing-file", missing_file);

    run_server(*local, test);
    free_app(*local);
}

main :: () {
    if !supported() return;

    contents: [..] u8;

    for 0..BIG_SIZE - 1 array_add(*contents, cast(u8) (it % 253));

    check(write_entire_file(BIG_FILE, string.{ contents.count, contents.data }), "writing the test file");

    plain: Server_Config;

    run_with(plain, reading);
    run_with(plain, unread);
    run_with(plain, created_response);
    run_with(plain, streaming);
    run_with(plain, files);

    small: Server_Config;

    small.max_body_size = 1000;

    run_with(small, limits);

    finish();
}
