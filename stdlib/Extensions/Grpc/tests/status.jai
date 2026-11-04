#import "Basic";
G :: #import "Extensions/Grpc";
H :: #import "Extensions/Http";

main :: () {
    for code: 0..16 {
        fields := H.Header_Field.[.{ name = "grpc-status", value = tprint("%", code) }, .{ name = "grpc-message", value = "bad%20%25%0A" }];
        status := G.parse_status(fields, 200);
        defer G.free_status(*status);
        assert(cast(int) status.code == code && status.message == "bad %\n" && status.cause == .NONE);
    }

    assert(G.parse_status(.[], 200).code == .UNKNOWN);
    assert(G.parse_status(.[], 404).code == .UNIMPLEMENTED);
    assert(G.parse_status(.[], 503).code == .UNAVAILABLE);
    assert(G.parse_status(H.Header_Field.[.{ name = "grpc-status", value = "0" }], 503).code == .OK);
    assert(G.parse_status(H.Header_Field.[.{ name = "grpc-status", value = "17" }], 200).code == .UNKNOWN);

    for text: string.["1n", "1u", "1m", "1S", "1M", "1H", "99999999H"] {
        value, ok := G.parse_timeout(text);
        assert(ok && value > 0);
    }

    for text: string.["", "1", "1x", "123456789m", "-1m"] {
        _, ok := G.parse_timeout(text);
        assert(!ok);
    }

    leading_zero := G.parse_status(H.Header_Field.[.{ name = "grpc-status", value = "00" }], 200);

    defer G.free_status(*leading_zero);
    assert(leading_zero.code == .UNKNOWN && leading_zero.cause == .PROTOCOL);
    rich := G.parse_status(H.Header_Field.[.{ name = "grpc-status", value = "3" }, .{ name = "grpc-status-details-bin", value = "CAM=" }], 200);

    defer G.free_status(*rich);
    assert(rich.code == .INVALID_ARGUMENT && rich.status_details == "\x08\x03");
    conflicting := G.parse_status(H.Header_Field.[.{ name = "grpc-status", value = "13" }, .{ name = "grpc-status-details-bin", value = "CAM" }], 200);

    defer G.free_status(*conflicting);
    assert(conflicting.code == .INTERNAL && conflicting.cause == .PROTOCOL);
    mappings := G.Status_Code.[.INTERNAL, .INTERNAL, .INTERNAL, .INTERNAL, .INTERNAL, .UNKNOWN, .INTERNAL, .UNAVAILABLE, .CANCELLED, .INTERNAL, .INTERNAL, .RESOURCE_EXHAUSTED, .PERMISSION_DENIED];
    for mappings assert(G.http2_reset_status(cast(u32) it_index) == it);

    assert(G.http2_reset_status(0xffffffff) == .UNKNOWN);
    print("ok\n");
}
