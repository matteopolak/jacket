// Independent grpcio-peer client; invoked by tools/grpc interop harness.
#import "Basic";
G :: #import "Extensions/Grpc";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";
#load "../../../stdlib/Extensions/Tls/tests/fixtures/certs.jai";

exercise :: (channel: *G.Channel, method: G.Method, inputs: [] string, outputs: [] string, expected := G.Status_Code.OK) {
    call, status := G.open_call(channel, method);

    assert(status.code == .OK);
    defer G.close(*call);

    for inputs {
        sent := G.send_bytes(*call, .{ it.count, it.data });
        assert(sent.code == .OK);
    }

    half := G.close_send(*call);

    assert(half.code == .OK);

    for outputs {
        bytes, kind := G.receive_bytes(*call);
        assert(kind == .MESSAGE && string.{ bytes.count, bytes.data } == it);
        if bytes.data free(bytes.data);
    }

    final := G.finish(*call);

    defer G.free_status(*final);
    assert(final.code == expected, "unexpected final status %", final.code);
    if expected == .INVALID_ARGUMENT assert(final.message == (ifx inputs[0] == "error" then "bad %\nrequest" else "after data"));

    if method.path == "/fixture.Echo/Unary" {
        assert(final.metadata.count == 3);
        assert(final.metadata[0].name == "x-repeat" && final.metadata[0].value == "one");
        assert(final.metadata[1].name == "x-repeat" && final.metadata[1].value == "two");
        assert(final.metadata[2].name == "data-bin" && final.metadata[2].value == "\x00\xff");
        initial := G.initial_metadata(*call);

        assert(initial.count == 1 && initial[0].name == "x-initial" && initial[0].value == "one");
    }
}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2, "supply https://host:port");

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
        exercise(*channel, .{ path = "/fixture.Echo/Unary" }, string.["hello"], string.["hello"]);
        exercise(*channel, .{ path = "/fixture.Echo/Unary" }, string.[""], string.[""]);
        exercise(*channel, .{ path = "/fixture.Echo/Unary" }, string.["error"], .[], .INVALID_ARGUMENT);
        exercise(*channel, .{ path = "/fixture.Echo/ServerStream", server_streaming = true }, string.["hi"], string.["hi", "", "hi"]);
        exercise(*channel, .{ path = "/fixture.Echo/ClientStream", client_streaming = true }, string.["a", "", "b"], string.["ab"]);
        exercise(*channel, .{ path = "/fixture.Echo/Bidi", client_streaming = true, server_streaming = true }, string.["a", "", "b"], string.["a", "", "b"]);
        exercise(*channel, .{ path = "/fixture.Echo/ServerStream", server_streaming = true }, string.["late-error"], string.["late-error"], .INVALID_ARGUMENT);
        exercise(*channel, .{ path = "/fixture.Echo/Bidi", client_streaming = true, server_streaming = true }, .[], .[]);
        exercise(*channel, .{ path = "/fixture.Echo/ClientStream", client_streaming = true }, .[], string.[""]);
        exercise(*channel, .{ path = "/fixture.Echo/Missing" }, string.[""], .[], .UNIMPLEMENTED);
    }

    Io.io_run(entry, args[1]);
    print("all four independent-server modes passed\n");
}

// Exercise actual wire cancellation/deadline against the independent peer.
exercise_scopes :: (channel: *G.Channel) {
    for deadline: bool.[true, false] {
        call, opened := G.open_call(channel, .{ path = "/fixture.Echo/Unary" }, .{ deadline_ms = ifx deadline then to_milliseconds(current_time_monotonic()) + 100 else 0 });

        assert(opened.code == .OK);
        defer G.close(*call);
        payload := ifx deadline then "deadline" else "cancel";

        assert(G.send_bytes(*call, .{ payload.count, payload.data }).code == .OK);
        assert(G.close_send(*call).code == .OK);

        if !deadline {
            assert(Io.sleep(20) == .NONE);
            G.cancel(*call);
        }

        status := G.finish(*call);

        defer G.free_status(*status);
        assert(status.code == ifx deadline then G.Status_Code.DEADLINE_EXCEEDED else G.Status_Code.CANCELLED);
    }

    assert(channel.active_calls == 0 && channel.transport.active_exchanges == 0);
}
