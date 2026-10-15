// Shared by the server and client tests: an in-process server on a free port, the client to
// drive it, raw TCP for the things the client will not do (pipelining, half requests), and
// checks that count failures instead of stopping at the first.
#import "Basic";
#import "String";
#import "Extensions/Http";
Io :: #import "Extensions/Io";

elapsed_milliseconds :: (start: Apollo_Time) -> s64 {
    before, before_ok := to_nanoseconds(start);
    after, after_ok := to_nanoseconds(current_time_monotonic());
    assert(before_ok && after_ok, "monotonic time must fit nanoseconds");

    return (after - before) / 1_000_000;
}

failures := 0;

check :: (ok: bool, format: string, args: ..Any) {
    if ok return;

    failures += 1;

    print("FAIL: ");
    print(format, ..args);
    print("\n");
}

finish :: () {
    if failures {
        print("% checks failed\n", failures);
        exit(1);
    }

    print("ok\n");
}

// Runs `body` on a scheduler with `app` listening on a free local port. The server is stopped
// and drained afterwards.
Server_Test :: #type (base: string, server: *Server);

run_server :: (app: *App, body: Server_Test) {
    entry :: (data: *Run_Data) {
        data.app.config.address = "127.0.0.1:0";
        data.app.config.handle_signals = false;

        server, error := start(data.app);

        if error != .NONE {
            print("FAIL: cannot listen: %\n", error);
            exit(1);
        }

        data.body(server_url(server), server);

        stop(server);
        wait(server);
    }

    data := Run_Data.{ app, body };

    Io.io_run(entry, *data);
}

// Runs `body` with a test client for `app`: no socket and no scheduler to set up.
App_Test :: #type (api: *Client);

run_app :: (app: *App, body: App_Test) {
    body(test_client(app));
}

Run_Data :: struct {
    app: *App;
    body: Server_Test;
}

// Whether this platform can run the tests (Windows has no IO engine yet).
supported :: () -> bool {
    #if OS == .WINDOWS {
        print("skipped: Extensions/Io has no IO on Windows yet\n");

        return false;
    }

    return true;
}

// A response with its body read, for checks that look at both. Everything is in temporary
// storage.
Reply :: struct {
    status: int;
    reason: string;
    headers: [] Header_Field;
    body: string;
}

// Reads the body of `response` and closes it.
into_reply :: (response: *Client_Response) -> Reply {
    reply: Reply;

    reply.status = response.status;
    reply.reason = copy_string(response.reason,, temp);

    list: [..] Header_Field;

    for response.headers {
        copy := it;

        copy.name = copy_string(it.name,, temp);
        copy.value = copy_string(it.value,, temp);

        array_add(*list, copy,, temp);
    }

    reply.headers = list;

    if response.source {
        text, error := read_text(response);

        check(error.kind == .NONE, "reading the body: %", error_text(error));

        reply.body = copy_string(text,, temp);

        free(text.data);
    }

    close(response);

    return reply;
}

// A request through `api`, with the answer's body read; a transport failure fails the check and
// returns an empty reply.
fetch :: (api: *Client, method: Method, path: string, body := "", headers: [] Header_Field = .[]) -> Reply {
    request: Client_Request;

    request.method = method;
    request.url = path;
    request.headers = headers;
    request.body = .{ body.count, body.data };
    request.timeout = 5000;
    request.max_redirects = 0;

    response, error := send(api, request);

    check(error.kind == .NONE, "% % failed: %", method_name(method), path, error_text(error));

    return into_reply(*response);
}

get_path :: (api: *Client, path: string, headers: [] Header_Field = .[]) -> Reply {
    return fetch(api, .GET, path, "", headers);
}

post_body :: (api: *Client, path: string, body: string, content_type := "application/json") -> Reply {
    headers := Header_Field.[.{ "Content-Type", content_type }];

    return fetch(api, .POST, path, body, headers);
}

// The reply body as text.
body_of :: (reply: Reply) -> string {
    return reply.body;
}

contains :: (text: string, needle: string) -> bool {
    return find_index_from_left(text, needle) >= 0;
}

header_of :: (reply: Reply, name: string) -> string {
    value, _ := find_header(reply.headers, name);

    return value;
}

has_header :: (reply: Reply, name: string) -> bool {
    _, found := find_header(reply.headers, name);

    return found;
}

// ---- Raw TCP

// Opens a connection to the server's address.
raw_connect :: (server: *Server) -> Io.Tcp_Stream {
    stream, error := Io.connect(server.address);

    check(error == .NONE, "raw connect failed: %", error);

    return stream;
}

// Reads until the server closes the connection or `milliseconds` pass; returns what came.
read_until_closed :: (stream: Io.Tcp_Stream, milliseconds: s64) -> string {
    builder: String_Builder;
    buffer: [4096] u8;

    Io.with_deadline(milliseconds, #code {
        while true {
            count, error := Io.read(stream, buffer);

            if error != .NONE || count == 0 break;

            append(*builder, string.{ count, buffer.data });
        }
    });

    return builder_to_string(*builder,, temp);
}

// One response read from a raw connection.
Raw_Response :: struct {
    status: int;
    headers: [..] Header_Field;
    body: string;
    closed: bool; // the connection ended with it
}

// Reads `count` responses from `stream` with the response codec, as a pipelining client would.
read_responses :: (stream: Io.Tcp_Stream, count: int, milliseconds: s64, method := Method.GET) -> [..] Raw_Response {
    results: [..] Raw_Response;

    Io.with_deadline(milliseconds, #code {
        results = collect_responses(stream, count, method);
    });

    return results;
}

collect_responses :: (stream: Io.Tcp_Stream, count: int, method: Method) -> [..] Raw_Response {
    results: [..] Raw_Response;

    parser: Parser;

    parser_init(*parser, .RESPONSE);
    parser.response_to = method;

    defer parser_free(*parser);

    buffer: [4096] u8;
    current: Raw_Response;
    body: [..] u8;

    while results.count < count {
        read, error := Io.read(stream, buffer);

        if error != .NONE || read == 0 {
            if results.count < count {
                check(false, "the connection ended after % of % responses (%)", results.count, count, error);
            }

            break;
        }

        data: [] u8 = .{ read, buffer.data };

        while true {
            used, event := parse(*parser, data);

            data = advance(data, used);

            if event.kind == .NEED_MORE break;

            if event.kind == .HEAD {
                current = .{};
                current.status = parser.head.status;
                body.count = 0;

                for parser.head.headers {
                    copy := it;

                    copy.name = copy_string(it.name,, temp);
                    copy.value = copy_string(it.value,, temp);

                    array_add(*current.headers, copy,, temp);
                }
            } else if event.kind == .BODY {
                for event.data array_add(*body, it);
            } else if event.kind == .END {
                current.body = copy_string(string.{ body.count, body.data },, temp);

                array_add(*results, current,, temp);

                if results.count == count break;
            } else if event.kind == .ERROR {
                check(false, "the server sent something that is not HTTP: %", parser.error.message);

                return results;
            }
        }
    }

    return results;
}

raw_header :: (response: Raw_Response, name: string) -> string {
    value, _ := find_header(response.headers, name);

    return value;
}

send_text :: (stream: Io.Tcp_Stream, text: string) {
    _, error := Io.write(stream, text);

    check(error == .NONE, "raw write failed: %", error);
}

// Waits until the connection reports an end (the server closed it), up to `milliseconds`.
closes_within :: (stream: Io.Tcp_Stream, milliseconds: s64) -> bool {
    buffer: [256] u8;
    closed := false;

    Io.with_deadline(milliseconds, #code {
        while true {
            count, error := Io.read(stream, buffer);

            if error == .TIMED_OUT break;

            if error != .NONE || count == 0 {
                closed = true;

                break;
            }
        }
    });

    return closed;
}
