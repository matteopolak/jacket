// test: skip_on(.WASM, reason = "native RPC/HTTP scheduler")
#load "../../Http/tests/support/server_support.jai";
Tls :: #import "Extensions/Tls";
G :: #import "Extensions/Grpc";
#load "../../Tls/tests/fixtures/certs.jai";

STALL :: G.Method.{ path = "/fixture.Echo/Stall", client_streaming = true, server_streaming = true };
FAST :: G.Method.{ path = "/fixture.Echo/Fast" };

stalled :: (call: *G.Call, _data: *void) -> G.Status {
    while !G.cancelled(call) {
        Io.sleep(1);
    }

    return .{ code = .CANCELLED };
}

fast :: (call: *G.Call, _data: *void) -> G.Status {
    message, kind := G.receive_bytes(call);
    if kind != .MESSAGE return .{ code = .INTERNAL };
    defer if message.data free(message.data);
    return G.send_bytes(call, message);
}

handler :: (request: *Request) -> Response {
    if request.path == STALL.path return G.serve_call(request, STALL, stalled, null);
    return G.serve_call(request, FAST, fast, null);
}

Blocked :: struct {
    call: *G.Call;
    done: Io.Wait_Group;
    returned: bool;
}

receive_blocked :: (job: *Blocked) {
    defer Io.done(*job.done);
    _, kind := G.receive_bytes(job.call);
    assert(kind == .ERROR);
    job.returned = true;
}

send_blocked :: (job: *Blocked) {
    defer Io.done(*job.done);
    message := NewArray(600 * 1024, u8);

    defer free(message.data);
    status := G.send_bytes(job.call, message);

    assert(status.code == .CANCELLED);
    G.free_status(*status);
    job.returned = true;
}

warm_channel :: (channel: *G.Channel, options := G.Call_Options.{}) {
    call, opened := G.open_call(channel, FAST, options);

    assert(opened.code == .OK);
    defer G.close(*call);
    sent := G.send_bytes(*call, bytes_of("warm"));

    assert(sent.code == .OK);
    half := G.close_send(*call);

    assert(half.code == .OK);
    bytes, kind := G.receive_bytes(*call);

    assert(kind == .MESSAGE);
    if bytes.data free(bytes.data);

    final := G.finish(*call);

    assert(final.code == .OK);
    G.free_status(*final);
}

exercise :: (base: string, _server: *Server) {
    config: Tls.Tls_Config;
    config.roots_pem = CA_PEM;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    channel: G.Channel;
    channel.transport.base_url = base;
    channel.transport.tls = *config;
    defer G.close_channel(*channel);
    expired, status := G.open_call(*channel, STALL, .{ deadline_ms = to_milliseconds(current_time_monotonic()) - 1 });

    assert(!expired.state && status.code == .DEADLINE_EXCEEDED && channel.active_calls == 0);
    channel.max_active_calls = 0;
    refused, limit := G.open_call(*channel, FAST);

    assert(!refused.state && limit.code == .RESOURCE_EXHAUSTED && channel.active_calls == 0);
    channel.max_active_calls = 128;
    warm_channel(*channel);
    huge_budget, valid := G.parse_timeout("99999999H");

    assert(valid);
    warm_channel(*channel, .{ deadline_ms = to_milliseconds(current_time_monotonic()) + huge_budget });

    {
        call, opened := G.open_call(*channel, STALL, .{ deadline_ms = to_milliseconds(current_time_monotonic()) + 100 });

        assert(opened.code == .OK);
        _, kind := G.receive_bytes(*call);

        assert(kind == .ERROR);
        final := G.finish(*call);

        assert(final.code == .DEADLINE_EXCEEDED && final.cause == .DEADLINE);
        G.free_status(*final);
        G.close(*call);
        G.close(*call);
    }

    {
        call, opened := G.open_call(*channel, STALL);

        assert(opened.code == .OK);
        job: Blocked;
        job.call = *call;
        Io.add(*job.done, 1);
        Io.spawn(receive_blocked, *job);
        Io.yield();
        assert(!job.returned);
        // An unrelated call progresses while this receive is blocked.
        other, started := G.open_call(*channel, FAST);

        assert(started.code == .OK);
        sent := G.send_bytes(*other, bytes_of("progress"));

        assert(sent.code == .OK);
        half := G.close_send(*other);

        assert(half.code == .OK);
        message, kind := G.receive_bytes(*other);

        assert(kind == .MESSAGE && string.{ message.count, message.data } == "progress");
        if message.data free(message.data);

        final := G.finish(*other);

        assert(final.code == .OK);
        G.free_status(*final);
        G.close(*other);
        G.cancel(*call);
        G.cancel(*call);
        Io.wait(*job.done);
        assert(job.returned);
        canceled := G.finish(*call);

        assert(canceled.code == .CANCELLED);
        G.free_status(*canceled);
        G.close(*call);
    }

    {
        call, opened := G.open_call(*channel, STALL);

        assert(opened.code == .OK);
        job: Blocked;
        job.call = *call;
        Io.add(*job.done, 1);
        Io.spawn(send_blocked, *job);
        Io.yield();
        warm_channel(*channel); // unrelated stream progresses while the writer stalls
        assert(!job.returned);
        G.cancel(*call);
        Io.wait(*job.done);
        assert(job.returned);
        G.close(*call);
    }

    {
        call, opened := G.open_call(*channel, STALL, .{ deadline_ms = to_milliseconds(current_time_monotonic()) + 100 });

        assert(opened.code == .OK);
        message := NewArray(600 * 1024, u8);

        defer free(message.data);
        send_status := G.send_bytes(*call, message);

        assert(send_status.code == .DEADLINE_EXCEEDED);
        G.free_status(*send_status);
        G.close(*call);
    }

    assert(channel.active_calls == 0 && channel.transport.active_exchanges == 0);
}

app: App;

main :: () {
    if !supported() return;

    post(*app, STALL.path, handler);
    post(*app, FAST.path, handler);
    config: Tls.Tls_Config;
    config.certificate_chain_pem = SERVER_PEM;
    config.private_key_pem = SERVER_KEY;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    app.config.tls = *config;
    app.config.h2_window = 16384;
    app.config.body_timeout = 1;
    app.config.write_timeout = 0;
    run_server(*app, exercise);
    finish();
}
