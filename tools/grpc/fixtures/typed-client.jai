#import "Basic";
G :: #import "Extensions/Grpc";
P :: #import "Extensions/Protobuf";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";
Api :: #import "Extensions/Protobuf/Schema"(GRPC = true, PATH = #run P.beside("echo.proto"));
#load "../../../stdlib/Extensions/Tls/tests/fixtures/certs.jai";

exercise :: ($Client: Type, client: *Client, inputs: [] string, outputs: [] string, expected := G.Status_Code.OK) {
    defer Client.close(client);

    for inputs {
        status := Client.send(client, Api.fixture.Payload.{ data = .{ it.count, it.data } });
        assert(status.code == .OK);
    }

    half := Client.close_send(client);

    assert(half.code == .OK);

    for outputs {
        value, kind := Client.receive(client);
        defer P.free(*value);
        assert(kind == .MESSAGE && string.{ value.data.count, value.data.data } == it);
    }

    final := Client.finish(client);

    defer G.free_status(*final);
    assert(final.code == expected);
    if expected == .INVALID_ARGUMENT assert(final.message == (ifx inputs[0] == "error" then "bad %\nrequest" else "after data"));

    #if Client == Api.fixture.Echo.Unary_Client {
        assert(final.metadata.count == 3);
        assert(final.metadata[0].name == "x-repeat" && final.metadata[0].value == "one");
        assert(final.metadata[1].name == "x-repeat" && final.metadata[1].value == "two");
        assert(final.metadata[2].name == "data-bin" && final.metadata[2].value == "\x00\xff");
        initial := Client.metadata(client);

        assert(initial.count == 1 && initial[0].name == "x-initial" && initial[0].value == "one");
    }
}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2);

    entry :: (base: string) {
        config: Tls.Tls_Config;
        config.roots_pem = CA_PEM;
        config.alpn = string.["h2"];
        assert(Tls.tls_config_init(*config).kind == .NONE);
        defer Tls.tls_config_free(*config);
        channel: G.Channel;
        channel.transport.base_url = base;
        channel.transport.tls = *config;
        defer G.close_channel(*channel);
        exercise_scopes(*channel);

        {
            client, opened := Api.fixture.Echo.Unary_open(*channel);
            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.Unary_Client, *client, string.["hello"], string.["hello"]);
        }

        {
            client, opened := Api.fixture.Echo.Unary_open(*channel);
            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.Unary_Client, *client, string.[""], string.[""]);
        }

        {
            client, opened := Api.fixture.Echo.Unary_open(*channel);
            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.Unary_Client, *client, string.["error"], .[], .INVALID_ARGUMENT);
        }

        {
            client, opened := Api.fixture.Echo.ServerStream_open(*channel);
            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.ServerStream_Client, *client, string.["hi"], string.["hi", "", "hi"]);
        }

        {
            client, opened := Api.fixture.Echo.ClientStream_open(*channel);
            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.ClientStream_Client, *client, string.["a", "", "b"], string.["ab"]);
        }

        {
            client, opened := Api.fixture.Echo.Bidi_open(*channel);
            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.Bidi_Client, *client, string.["a", "", "b"], string.["a", "", "b"]);
        }

        {
            client, opened := Api.fixture.Echo.ServerStream_open(*channel);
            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.ServerStream_Client, *client, string.["late-error"], string.["late-error"], .INVALID_ARGUMENT);
        }

        {
            client, opened := Api.fixture.Echo.Bidi_open(*channel);
            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.Bidi_Client, *client, .[], .[]);
        }

        {
            client, opened := Api.fixture.Echo.ClientStream_open(*channel);
            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.ClientStream_Client, *client, .[], string.[""]);
        }

        {
            large := alloc_string(300 * 1024 + 3);

            defer free(large);
            for *large it.* = #char "x";

            client, opened := Api.fixture.Echo.ServerStream_open(*channel);

            assert(opened.code == .OK);
            exercise(Api.fixture.Echo.ServerStream_Client, *client, string.[large], string.[large, "", large]);
        }
    }

    Io.io_run(entry, args[1]);
    print("generated clients: all four independent-server modes passed\n");
}

// Exercise actual wire cancellation/deadline against the independent peer.
exercise_scopes :: (channel: *G.Channel) {
    C :: Api.fixture.Echo.Unary_Client;

    for deadline: bool.[true, false] {
        client, opened := Api.fixture.Echo.Unary_open(channel, .{ deadline_ms = ifx deadline then to_milliseconds(current_time_monotonic()) + 100 else 0 });

        assert(opened.code == .OK);
        defer C.close(*client);
        payload := ifx deadline then "deadline" else "cancel";

        assert(C.send(*client, Api.fixture.Payload.{ data = .{ payload.count, payload.data } }).code == .OK);
        assert(C.close_send(*client).code == .OK);

        if !deadline {
            assert(Io.sleep(20) == .NONE);
            C.cancel(*client);
        }

        status := C.finish(*client);

        defer G.free_status(*status);
        assert(status.code == ifx deadline then G.Status_Code.DEADLINE_EXCEEDED else G.Status_Code.CANCELLED);
    }

    assert(channel.active_calls == 0 && channel.transport.active_exchanges == 0);
}
