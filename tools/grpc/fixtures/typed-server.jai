#import "Basic";
#import "String";
#import "File";
Http :: #import "Extensions/Http";
G :: #import "Extensions/Grpc";
P :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(GRPC = true, PATH = #run P.beside("echo.proto"));
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";
#load "../../../stdlib/Extensions/Tls/tests/fixtures/certs.jai";

// Returned fields borrow this request fiber's temporary storage through serve_call.
unary_status :: (invalid: bool) -> G.Status {
    status: G.Status;

    if invalid {
        status.code = .INVALID_ARGUMENT;
        status.message = "bad %\nrequest";
    }

    array_add(*status.metadata, .{ "x-repeat", "one" },, temp);
    array_add(*status.metadata, .{ "x-repeat", "two" },, temp);
    array_add(*status.metadata, .{ "data-bin", "\x00\xff" },, temp);

    return status;
}

check_request_metadata :: (entries: [] G.Metadata_Entry) {
    repeats := 0;
    binaries := 0;

    for entries {
        if it.name == "x-client" {
            assert(it.value == ifx repeats == 0 then "one" else "two");
            repeats += 1;
        }

        if it.name == "client-bin" {
            assert(it.value == "\x00\xff");
            binaries += 1;
        }
    }

    assert(repeats == 2 && binaries == 1);
}

echo :: (call: *$T, mode: int) -> G.Status {
    if mode == 2 {
        all: [..] u8;

        defer array_reset(*all);

        while true {
            value, kind := T.receive(call);
            if kind == .END break;
            if kind != .MESSAGE return .{ code = .INTERNAL };
            for value.data array_add(*all, it);

            P.free(*value);
        }

        return T.send(call, Api.fixture.Payload.{ data = all });
    }

    while true {
        value, kind := T.receive(call);
        if kind == .END break;
        if kind != .MESSAGE return .{ code = .INTERNAL };

        defer P.free(*value);
        request_text := string.{ value.data.count, value.data.data };
        if mode == 0 && request_text == "hello" check_request_metadata(T.metadata(call));
        if mode == 0 && request_text == "error" return unary_status(true);

        if mode == 0 && (request_text == "deadline" || request_text == "cancel") {
            changed: Io.Condition;
            error := Io.wait(*changed);

            assert(error == .TIMED_OUT || error == .CANCELED);
            print("incoming % terminated\n", request_text);

            return .{ code = ifx error == .TIMED_OUT then G.Status_Code.DEADLINE_EXCEEDED else G.Status_Code.CANCELLED };
        }

        status := T.send(call, value);
        if status.code != .OK return status;
        if mode == 0 return unary_status(false);

        if mode == 1 && request_text == "cancel" {
            changed: Io.Condition;
            assert(Io.wait(*changed) == .CANCELED);
            print("incoming cancel terminated\n");
            return .{ code = .CANCELLED };
        }

        if mode == 1 {
            status = T.send(call, Api.fixture.Payload.{});
            if status.code != .OK return status;
            status = T.send(call, value);
            if status.code != .OK return status;
        }
    }

    return .{};
}

Unary_handler :: (call: *Api.fixture.Echo.Unary_Server, _state: *void) -> G.Status {
    return echo(call, 0);
}

ServerStream_handler :: (call: *Api.fixture.Echo.ServerStream_Server, _state: *void) -> G.Status {
    return echo(call, 1);
}

ClientStream_handler :: (call: *Api.fixture.Echo.ClientStream_Server, _state: *void) -> G.Status {
    return echo(call, 2);
}

Bidi_handler :: (call: *Api.fixture.Echo.Bidi_Server, _state: *void) -> G.Status {
    return echo(call, 3);
}

app: Http.App;

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2, "supply port-file path");
    handlers := Api.fixture.Echo.Handlers.{ options = .{ metadata = G.Metadata_Entry.[.{ "x-initial", "one" }] }, Unary = Unary_handler, ServerStream = ServerStream_handler, ClientStream = ClientStream_handler, Bidi = Bidi_handler };

    Api.fixture.Echo.register(*app, *handlers);
    config: Tls.Tls_Config;
    config.certificate_chain_pem = SERVER_PEM;
    config.private_key_pem = SERVER_KEY;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    app.config.tls = *config;
    app.config.write_timeout = 0;
    app.config.address = "127.0.0.1:0";

    entry :: (path: string) {
        server, error := Http.start(*app);

        assert(error.kind == .NONE);
        defer Http.close_server(*server);
        url := Http.server_url(server);

        assert(write_entire_file(path, slice(url, 8, url.count - 8)));
        Http.wait(server);
    }

    Io.io_run(entry, args[1]);
}
