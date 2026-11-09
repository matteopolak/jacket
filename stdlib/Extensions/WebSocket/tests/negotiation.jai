#import "Basic";
#import "Extensions/WebSocket";

main :: () {
    accept := ws_accept_key("dGhlIHNhbXBsZSBub25jZQ==");

    assert(accept == "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    free(accept);
    compression, error := ws_negotiate("permessage-deflate; server_no_context_takeover; client_max_window_bits=10; server_max_window_bits=12");

    assert(error.kind == .NONE && compression.enabled && compression.client_max_window_bits == 10 && compression.server_max_window_bits == 12);
    _, error = ws_negotiate("permessage-deflate; server_max_window_bits=7");
    assert(error.kind == .HANDSHAKE);
    _, error = ws_negotiate("permessage-deflate; client_no_context_takeover; client_no_context_takeover");
    assert(error.kind == .HANDSHAKE);
    _, error = ws_negotiate("permessage-deflate; client_max_window_bits", true);
    assert(error.kind == .HANDSHAKE);
    _, error = ws_negotiate("permessage-deflate;", true);
    assert(error.kind == .HANDSHAKE);
    _, error = ws_negotiate("permessage-deflate,", true);
    assert(error.kind == .HANDSHAKE);
}
