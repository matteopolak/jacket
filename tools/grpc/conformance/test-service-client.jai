// Canonical client procedures from the pinned official interoperability catalogue.
#load "test-service-common.jai";

#if TYPED {
    Empty_Context :: Api.grpc.testing.TestService.EmptyCall_Client;
    Unary_Context :: Api.grpc.testing.TestService.UnaryCall_Client;
    Input_Context :: Api.grpc.testing.TestService.StreamingInputCall_Client;
    Output_Context :: Api.grpc.testing.TestService.StreamingOutputCall_Client;
    Bidi_Context :: Api.grpc.testing.TestService.FullDuplexCall_Client;
} else {
    Empty_Context :: G.Call;
    Unary_Context :: G.Call;
    Input_Context :: G.Call;
    Output_Context :: G.Call;
    Bidi_Context :: G.Call;
}

open_Empty :: (channel: *G.Channel, options := G.Call_Options.{}) -> Empty_Context, G.Status {
    #if TYPED return Api.grpc.testing.TestService.EmptyCall_open(channel, options);
    else return G.open_call(channel, Api.grpc.testing.TestService.EmptyCall, options);
}

open_Unary :: (channel: *G.Channel, options := G.Call_Options.{}) -> Unary_Context, G.Status {
    #if TYPED return Api.grpc.testing.TestService.UnaryCall_open(channel, options);
    else return G.open_call(channel, Api.grpc.testing.TestService.UnaryCall, options);
}

open_Input :: (channel: *G.Channel, options := G.Call_Options.{}) -> Input_Context, G.Status {
    #if TYPED return Api.grpc.testing.TestService.StreamingInputCall_open(channel, options);
    else return G.open_call(channel, Api.grpc.testing.TestService.StreamingInputCall, options);
}

open_Output :: (channel: *G.Channel, options := G.Call_Options.{}) -> Output_Context, G.Status {
    #if TYPED return Api.grpc.testing.TestService.StreamingOutputCall_open(channel, options);
    else return G.open_call(channel, Api.grpc.testing.TestService.StreamingOutputCall, options);
}

open_Bidi :: (channel: *G.Channel, options := G.Call_Options.{}) -> Bidi_Context, G.Status {
    #if TYPED return Api.grpc.testing.TestService.FullDuplexCall_open(channel, options);
    else return G.open_call(channel, Api.grpc.testing.TestService.FullDuplexCall, options);
}

half :: (call: *$T) -> G.Status {
    #if T == G.Call return G.close_send(call);
    else return T.close_send(call);
}

complete :: (call: *$T) -> G.Status {
    #if T == G.Call return G.finish(call);
    else return T.finish(call);
}

release :: (call: *$T) {
    #if T == G.Call G.close(call);
    else T.close(call);
}

abort :: (call: *$T) {
    #if T == G.Call G.cancel(call);
    else T.cancel(call);
}

ok :: (status: G.Status) {
    owned := status;
    defer G.free_status(*owned);
    assert(status.code == .OK, "RPC operation failed: % %", status.code, status.message);
}

expect_payload :: (payload: *Api.grpc.testing.Payload, size: s32) {
    assert(payload && payload.body.count == size);
    for payload.body assert(it == 0);
}

expect_end :: (call: *$T, $Message: Type) {
    value, kind := read_message(call, Message);
    if kind == .MESSAGE P.free(*value);
    assert(kind == .END);
    ok(complete(call));
}

empty_unary :: (channel: *G.Channel) {
    call, status := open_Empty(channel);

    ok(status);
    defer release(*call);
    ok(write_message(*call, Api.grpc.testing.Empty.{}));
    ok(half(*call));
    response, kind := read_message(*call, Api.grpc.testing.Empty);

    assert(kind == .MESSAGE);
    P.free(*response);
    expect_end(*call, Api.grpc.testing.Empty);
}

custom_entries :: G.Metadata_Entry.[.{ "x-grpc-test-echo-initial", "test_initial_metadata_value" }, .{ "x-grpc-test-echo-trailing-bin", "\xab\xab\xab" }];

check_initial :: (call: *$T) {
    found := false;
    for metadata(call) if it.name == custom_entries[0].name && it.value == custom_entries[0].value found = true;
    assert(found);
}

check_trailing :: (status: G.Status) {
    owned := status;

    defer G.free_status(*owned);
    assert(status.code == .OK);
    found := false;
    for status.metadata if it.name == custom_entries[1].name && it.value == custom_entries[1].value found = true;

    assert(found);
}

unary_request :: (channel: *G.Channel, compressed: bool, expect_compressed: bool, check_compressed: bool, response_compressed: bool, check_response: bool, metadata_test := false, expected := G.Status_Code.OK, message := "") {
    options: G.Call_Options;
    if compressed options.compression = .GZIP;
    if metadata_test options.metadata = custom_entries;

    call, status := open_Unary(channel, options);

    ok(status);
    defer release(*call);
    bytes := NewArray(271828, u8);

    defer free(bytes.data);
    payload := Api.grpc.testing.Payload.{ body = bytes };
    expected_compression := Api.grpc.testing.BoolValue.{ value = expect_compressed };
    response_compression := Api.grpc.testing.BoolValue.{ value = response_compressed };
    response_status := Api.grpc.testing.EchoStatus.{ code = cast(s32) expected, message = message };
    request := Api.grpc.testing.SimpleRequest.{ response_size = 314159, payload = *payload };

    if check_compressed request.expect_compressed = *expected_compression;
    if check_response request.response_compressed = *response_compression;
    if expected != .OK && !check_compressed request.response_status = *response_status;

    sent := write_message(*call, request, compressed);

    G.free_status(*sent);
    half_status := half(*call);

    G.free_status(*half_status);
    response, kind := read_message(*call, Api.grpc.testing.SimpleResponse);

    defer if kind == .MESSAGE P.free(*response);

    if expected == .OK {
        assert(kind == .MESSAGE);
        expect_payload(response.payload, 314159);
        if check_response assert(compressed_request(*call) == response_compressed);
        if metadata_test check_initial(*call);
    } else assert(kind != .MESSAGE);

    final := complete(*call);
    if metadata_test check_trailing(final);
    else {
        defer G.free_status(*final);
        assert(final.code == expected);
        if !check_compressed assert(final.message == message);
    }
}

input_stream :: (channel: *G.Channel, compression_test: bool, probe := false) {
    options: G.Call_Options;
    if compression_test && !probe options.compression = .GZIP;

    call, status := open_Input(channel, options);

    ok(status);
    defer release(*call);
    sizes: [] s32 = s32.[27182, 8, 1828, 45904];
    if compression_test sizes = s32.[27182, 45904];
    if probe sizes = s32.[27182];

    for sizes {
        bytes := NewArray(it, u8);

        defer free(bytes.data);
        payload := Api.grpc.testing.Payload.{ body = bytes };
        compression := Api.grpc.testing.BoolValue.{ value = it_index == 0 };
        request := Api.grpc.testing.StreamingInputCallRequest.{ payload = *payload };

        if compression_test request.expect_compressed = *compression;

        sent := write_message(*call, request, compression_test && !probe && it_index == 0);

        G.free_status(*sent);
    }

    half_status := half(*call);

    G.free_status(*half_status);
    response, kind := read_message(*call, Api.grpc.testing.StreamingInputCallResponse);

    defer if kind == .MESSAGE P.free(*response);

    if probe {
        final := complete(*call);
        defer G.free_status(*final);
        assert(kind != .MESSAGE && final.code == .INVALID_ARGUMENT);
    } else {
        assert(kind == .MESSAGE && response.aggregated_payload_size == (ifx compression_test then 73086 else 74922));
        expect_end(*call, Api.grpc.testing.StreamingInputCallResponse);
    }
}

receive_payload :: (call: *$T, expected: s32, compression: bool = false, check_compression := false) {
    response, kind := read_message(call, Api.grpc.testing.StreamingOutputCallResponse);

    assert(kind == .MESSAGE);
    defer P.free(*response);
    expect_payload(response.payload, expected);
    if check_compression assert(compressed_request(call) == compression);
}

server_streaming :: (channel: *G.Channel, compressed: bool) {
    call, status := open_Output(channel);

    ok(status);
    defer release(*call);
    sizes: [] s32 = s32.[31415, 9, 2653, 58979];
    if compressed sizes = s32.[31415, 92653];

    parameters: [..] Api.grpc.testing.ResponseParameters;

    defer array_reset(*parameters);
    flags: [4] Api.grpc.testing.BoolValue;

    for sizes {
        flags[it_index].value = it_index == 0;
        parameter := Api.grpc.testing.ResponseParameters.{ size = it };
        if compressed parameter.compressed = *flags[it_index];
        array_add(*parameters, parameter);
    }

    ok(write_message(*call, Api.grpc.testing.StreamingOutputCallRequest.{ response_parameters = parameters }));
    ok(half(*call));
    for sizes receive_payload(*call, it, it_index == 0, compressed);

    expect_end(*call, Api.grpc.testing.StreamingOutputCallResponse);
}

bidi :: (channel: *G.Channel, cancel_first := false, metadata_test := false, status_test := false) {
    options: G.Call_Options;
    if metadata_test options.metadata = custom_entries;

    call, status := open_Bidi(channel, options);

    ok(status);
    defer release(*call);

    if status_test {
        expected := Api.grpc.testing.EchoStatus.{ code = 2, message = "test status message" };

        ok(write_message(*call, Api.grpc.testing.StreamingOutputCallRequest.{ response_status = *expected }));
        half_status := half(*call);

        G.free_status(*half_status);
        final := complete(*call);

        defer G.free_status(*final);
        assert(final.code == .UNKNOWN && final.message == expected.message);

        return;
    }

    input_sizes: [] s32 = s32.[27182, 8, 1828, 45904];
    output_sizes: [] s32 = s32.[31415, 9, 2653, 58979];

    if metadata_test {
        input_sizes = s32.[271828];
        output_sizes = s32.[314159];
    }

    for input_sizes {
        bytes := NewArray(it, u8);

        defer free(bytes.data);
        payload := Api.grpc.testing.Payload.{ body = bytes };
        parameters: [..] Api.grpc.testing.ResponseParameters;

        defer array_reset(*parameters);
        array_add(*parameters, .{ size = output_sizes[it_index] });
        request := Api.grpc.testing.StreamingOutputCallRequest.{ response_parameters = parameters, payload = *payload };

        ok(write_message(*call, request));
        receive_payload(*call, output_sizes[it_index]);
        if metadata_test check_initial(*call);

        if cancel_first {
            abort(*call);
            final := complete(*call);

            defer G.free_status(*final);
            assert(final.code == .CANCELLED);

            return;
        }
    }

    ok(half(*call));
    if metadata_test check_trailing(complete(*call));
    else expect_end(*call, Api.grpc.testing.StreamingOutputCallResponse);
}

empty_stream :: (channel: *G.Channel) {
    call, status := open_Bidi(channel);

    ok(status);
    defer release(*call);
    ok(half(*call));
    expect_end(*call, Api.grpc.testing.StreamingOutputCallResponse);
}

unimplemented_result :: (call: *$T) {
    defer release(call);
    sent := write_message(call, Api.grpc.testing.Empty.{});

    G.free_status(*sent);
    half_status := half(call);

    G.free_status(*half_status);
    final := complete(call);

    defer G.free_status(*final);
    assert(final.code == .UNIMPLEMENTED);
}

unimplemented :: (channel: *G.Channel, service: bool) {
    #if TYPED {
        if service {
            call, status := Api.grpc.testing.UnimplementedService.UnimplementedCall_open(channel);
            ok(status);
            unimplemented_result(*call);
        } else {
            call, status := Api.grpc.testing.TestService.UnimplementedCall_open(channel);
            ok(status);
            unimplemented_result(*call);
        }
    } else {
        method := ifx service then Api.grpc.testing.UnimplementedService.UnimplementedCall else Api.grpc.testing.TestService.UnimplementedCall;
        call, status := G.open_call(channel, method);
        ok(status);
        unimplemented_result(*call);
    }
}

cancel_after_begin :: (channel: *G.Channel) {
    call, status := open_Input(channel);

    ok(status);
    defer release(*call);
    abort(*call);
    final := complete(*call);

    defer G.free_status(*final);
    assert(final.code == .CANCELLED);
}

deadline :: (channel: *G.Channel) {
    empty_unary(channel); // establish the channel before measuring the call's one-millisecond budget
    call, status := open_Bidi(channel, .{ deadline_ms = to_milliseconds(current_time_monotonic()) + 1 });

    defer G.free_status(*status);
    if status.code == .DEADLINE_EXCEEDED return;

    assert(status.code == .OK);
    defer release(*call);
    bytes := NewArray(27182, u8);

    defer free(bytes.data);
    payload := Api.grpc.testing.Payload.{ body = bytes };
    sent := write_message(*call, Api.grpc.testing.StreamingOutputCallRequest.{ payload = *payload });

    G.free_status(*sent);
    final := complete(*call);

    defer G.free_status(*final);
    assert(final.code == .DEADLINE_EXCEEDED);
}

scaling :: (channel: *G.Channel) {
    channel.transport.max_connections = 2;
    calls: [3] Bidi_Context;

    defer for *calls release(it);
    peers: [3] string;

    for i: 0..2 {
        status: G.Status;
        calls[i], status = open_Bidi(channel, .{ deadline_ms = to_milliseconds(current_time_monotonic()) + 5000 });
        ok(status);
        fill := Api.grpc.testing.BoolValue.{ value = true };
        parameters: [..] Api.grpc.testing.ResponseParameters;

        defer array_reset(*parameters);
        array_add(*parameters, .{ fill_peer_socket_address = *fill });
        ok(write_message(*calls[i], Api.grpc.testing.StreamingOutputCallRequest.{ response_parameters = parameters }));
        response, kind := read_message(*calls[i], Api.grpc.testing.StreamingOutputCallResponse);

        assert(kind == .MESSAGE && response.peer_socket_address.count > 0);
        peers[i] = copy_string(response.peer_socket_address);
        P.free(*response);
    }

    defer for peers free(it);
    assert(peers[0] == peers[1] && peers[0] != peers[2]);

    for *calls {
        ok(half(it));
        expect_end(it, Api.grpc.testing.StreamingOutputCallResponse);
    }
}

// Negative HTTP/2 catalogue procedures retain the same canonical UnaryCall payload.
negative_reset :: (channel: *G.Channel) {
    call, opened := open_Unary(channel);

    defer G.free_status(*opened);
    if opened.code != .OK return;

    defer release(*call);
    bytes := NewArray(271828, u8);

    defer free(bytes.data);
    payload := Api.grpc.testing.Payload.{ body = bytes };
    sent := write_message(*call, Api.grpc.testing.SimpleRequest.{ payload = *payload, response_size = 314159 });

    defer G.free_status(*sent);
    closed := half(*call);

    defer G.free_status(*closed);
    response, kind := read_message(*call, Api.grpc.testing.SimpleResponse);
    if kind == .MESSAGE P.free(*response);

    final := complete(*call);

    defer G.free_status(*final);
    assert(final.code != .OK);
}

Parallel_Unary :: struct {
    channel: *G.Channel;
    done: Io.Wait_Group;
}

parallel_unary :: (job: *Parallel_Unary) {
    defer Io.done(*job.done);
    unary_request(job.channel, false, false, false, false, false);
}

negative_max_streams :: (channel: *G.Channel) {
    // One connection ensures the peer's one-stream setting tests admission.
    channel.transport.max_connections = 1;
    unary_request(channel, false, false, false, false, false);
    job := Parallel_Unary.{ channel = channel };

    Io.add(*job.done, 10);
    for 0..9 Io.spawn(parallel_unary, *job);

    assert(Io.wait(*job.done) == .NONE);
}

run_case :: (channel: *G.Channel, name: string) {
    if name == "rst_after_header" || name == "rst_during_data" || name == "rst_after_data" negative_reset(channel);
    else if name == "goaway" {
        unary_request(channel, false, false, false, false, false);
        assert(Io.sleep(1000) == .NONE);
        unary_request(channel, false, false, false, false, false);
    } else if name == "max_streams" negative_max_streams(channel);
    else if name == "ping" || name == "data_frame_padding" || name == "no_df_padding_sanity_test" unary_request(channel, false, false, false, false, false);
    else if name == "empty_unary" empty_unary(channel);
    else if name == "large_unary" unary_request(channel, false, false, false, false, false);
    else if name == "client_compressed_unary" {
        unary_request(channel, false, true, true, false, false, expected = .INVALID_ARGUMENT);
        unary_request(channel, true, true, true, false, false);
        unary_request(channel, false, false, true, false, false);
    } else if name == "server_compressed_unary" {
        unary_request(channel, false, false, false, true, true);
        unary_request(channel, false, false, false, false, true);
    } else if name == "client_streaming" input_stream(channel, false);
    else if name == "client_compressed_streaming" {
        input_stream(channel, true, true);
        input_stream(channel, true);
    } else if name == "server_streaming" server_streaming(channel, false);
    else if name == "server_compressed_streaming" server_streaming(channel, true);
    else if name == "ping_pong" bidi(channel);
    else if name == "empty_stream" empty_stream(channel);
    else if name == "custom_metadata" {
        unary_request(channel, false, false, false, false, false, true);
        bidi(channel, metadata_test = true);
    } else if name == "status_code_and_message" {
        unary_request(channel, false, false, false, false, false, expected = .UNKNOWN, message = "test status message");
        bidi(channel, status_test = true);
    } else if name == "special_status_message" unary_request(channel, false, false, false, false, false, expected = .UNKNOWN, message = "\t\ntest with whitespace\r\nand Unicode BMP ☺ and non-BMP 😈\t\n");
    else if name == "unimplemented_method" unimplemented(channel, false);
    else if name == "unimplemented_service" unimplemented(channel, true);
    else if name == "cancel_after_begin" cancel_after_begin(channel);
    else if name == "cancel_after_first_response" bidi(channel, true);
    else if name == "timeout_on_sleeping_server" deadline(channel);
    else if name == "max_concurrent_streams_connection_scaling" scaling(channel);
    else assert(false, "unknown canonical case %", name);
}

positive_integer :: (text: string) -> s64 {
    value: s64;

    assert(text.count > 0);

    for text {
        assert(it >= #char "0" && it <= #char "9");
        assert(value <= (0x7fffffffffffffff - 9) / 10);
        value = value * 10 + cast(s64) (it - #char "0");
    }

    assert(value > 0);

    return value;
}

soak_rpc :: (channel: *G.Channel) -> bool, G.Status_Code {
    call, opened := open_Unary(channel);

    defer G.free_status(*opened);
    if opened.code != .OK return false, opened.code;

    defer release(*call);
    bytes := NewArray(271828, u8);

    defer free(bytes.data);
    payload := Api.grpc.testing.Payload.{ body = bytes };
    request := Api.grpc.testing.SimpleRequest.{ payload = *payload, response_size = 314159 };
    sent := write_message(*call, request);

    defer G.free_status(*sent);
    half_status := half(*call);

    defer G.free_status(*half_status);
    response, kind := read_message(*call, Api.grpc.testing.SimpleResponse);

    defer if kind == .MESSAGE P.free(*response);
    valid := kind == .MESSAGE && response.payload && response.payload.body.count == 314159;
    if valid for response.payload.body if it != 0 valid = false;

    final := complete(*call);

    defer G.free_status(*final);

    return valid && final.code == .OK, final.code;
}

Control :: struct {
    base: string;
    name: string;
    config: *Tls.Tls_Config;
    iterations: s64;
    acceptable_ms: s64;
    overall_seconds: s64;
}

Soak_Result :: struct {
    elapsed_ns: s64;
    valid: bool;
    status: G.Status_Code;
}

soak :: (channel: *G.Channel, control: *Control) {
    start := to_nanoseconds(current_time_monotonic());
    results: [..] Soak_Result;

    defer array_reset(*results);

    for i: 0..control.iterations - 1 {
        begin := to_nanoseconds(current_time_monotonic());
        if (begin - start) / 1000000000 >= control.overall_seconds break;

        valid: bool;
        code: G.Status_Code;
        elapsed: s64;

        if control.name == "channel_soak" {
            fresh: G.Channel;
            fresh.transport.base_url = control.base;
            fresh.transport.tls = control.config;
            valid, code = soak_rpc(*fresh);
            // The catalogue includes channel creation, but excludes teardown.
            elapsed = to_nanoseconds(current_time_monotonic()) - begin;
            G.close_channel(*fresh);
        } else {
            valid, code = soak_rpc(channel);
            elapsed = to_nanoseconds(current_time_monotonic()) - begin;
        }

        array_add(*results, Soak_Result.{ elapsed, valid, code });
    }

    // Retain every result, then aggregate without aborting at a failed RPC.
    failures: s64;

    for results {
        valid := it.valid && it.elapsed_ns / 1000000 <= control.acceptable_ms;
        if !valid failures += 1;
        print("thread_id: 0 soak iteration: % elapsed_ms: % peer: unavailable server_uri: % % elapsed_ns: % status: %\n", it_index, it.elapsed_ns / 1000000, control.base, ifx valid then "succeeded" else "failed", it.elapsed_ns, it.status);
    }

    print("soak completed: % requested: % failures: %\n", results.count, control.iterations, failures);
    assert(results.count == control.iterations && failures == 0);
}

entry :: (control: *Control) {
    channel: G.Channel;
    channel.transport.base_url = control.base;
    channel.transport.tls = control.config;
    defer G.close_channel(*channel);
    if control.name == "rpc_soak" || control.name == "channel_soak" soak(*channel, control);
    else run_case(*channel, control.name);
}

main :: () {
    args := get_command_line_arguments();

    defer free(args.data);
    assert(args.count == 4 || args.count == 7, "supply base URL, case, CA file and optional soak iterations/acceptable-ms/overall-seconds");
    roots, read := read_entire_file(args[3]);

    assert(read);
    defer free(roots);
    config: Tls.Tls_Config;
    config.roots_pem = roots;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    control := Control.{ base = args[1], name = args[2], config = *config };
    if control.name == "rpc_soak" || control.name == "channel_soak" {
        assert(args.count == 7, "soaks require explicit nonzero budgets");
        control.iterations = positive_integer(args[4]);
        control.acceptable_ms = positive_integer(args[5]);
        control.overall_seconds = positive_integer(args[6]);
    }

    Io.io_run(entry, *control);
    print("canonical case % passed\n", args[2]);
}
