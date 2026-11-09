# WebSocket

`Extensions/WebSocket` implements RFC 6455 messages and RFC 7692 permessage-deflate. The codec is sans-IO and builds for wasm; native connections use the Http client's transport and TLS configuration and the Http server's routed upgrade callback.

```jai,skip_on(wasm)
#import "Extensions/WebSocket";
Http :: #import "Extensions/Http";
Io :: #import "Extensions/Io";

echo :: (connection: *Ws_Connection, data: *void) {
    while true {
        event := ws_read(connection);
        if event.kind != .MESSAGE return;
        if ws_write(connection, event.data, event.opcode).kind != .NONE return;
    }
}
route :: (request: *Http.Request) -> Http.Response {
    return ws_upgrade(request, echo);
}
main :: () {
    app: Http.App;
    defer Http.free_app(*app);
    Http.get(*app, "/echo", route);
    Http.serve(*app);
}
```

Clients call `ws_connect("wss://host/echo", config, *http_client)` inside `Io.io_run`. The optional Http client supplies its transport and TLS trust configuration. URLs must be absolute `ws://` or `wss://`; redirects are refused. Upgrade requests force HTTP/1.1 without changing global client settings. HTTP/2 WebSockets and extensions other than permessage-deflate are outside this API.

## How it works

`ws_parse(*codec, bytes)` returns consumed bytes and one `Ws_Event`. Feed the remainder after each event; read more after `.NEED_MORE`. Set `codec.role` to `.CLIENT` when receiving server frames (default `.SERVER`). Clients must mask every frame and servers must not mask. The codec rejects reserved bits/opcodes, noncanonical lengths, invalid continuations and malformed control frames before accepting their payloads. Text UTF-8 validation continues across fragments, allowing control frames between them. Close codes and UTF-8 reasons are checked too.

Events borrow codec-owned storage until the next parse. `ws_free(*codec)` frees all buffers and resets it. Failures are sticky values with a kind, static message and byte offset. For compressed content the offset locates the wire message, since an output byte has no unique wire offset. The codec does not retain arbitrary input: it retains one bounded message, a control payload and at most one negotiated history window.

`ws_frame` appends one frame to a caller-owned dynamic byte array. Its explicit masking key makes the portable codec independent of an operating-system random source. Native client writes obtain a fresh key from the OS secure generator for every frame. Native `ws_write` validates text and fragments messages at the configured frame limit.

`ws_upgrade` validates the HTTP/1.1 GET, version 13 and canonical 16-byte Base64 key, chooses the first configured subprotocol the client offered, and returns a 101 response. Route middleware can reject it like any response. After writing the response, Http invokes the callback with the active plain/TLS link and any bytes already read after the head. The server retains ownership of that link and closes it when the callback returns.

`ws_read` returns messages or close events, answers pings and consumes pongs. `ws_close` sends close, waits under a cleanup deadline for the peer, then releases buffers and any client-owned link. `ws_dispose` aborts without a close handshake and is safe to repeat. Server callbacks borrow their connection; returning automatically closes it. Client callers own their connection and should always defer `ws_dispose`. Do not copy an active `Ws_Connection`. One reader and one writer may run concurrently; writes are serialized across whole messages. Close must run without a concurrent reader.

## Compression

`ws_negotiate` parses a header capped at 8 KiB. Server/client no-context-takeover and window bits 8–15 are supported. A server can decline an unsupported or invalid offer while accepting the base protocol; a client rejects invalid or unsolicited extension responses. Windows are only included in the server response when offered.

Deflate's `deflate_sync`/`inflate_sync` operate on a message with the RFC 7692 four-byte flush tail omitted. Both directions retain only the negotiated sliding history, unless no-context-takeover is selected. Uncompressed messages and control frames do not update compression history. Inflate enforces both output size and negotiated match distance. For outgoing windows below 15, encoding uses stored blocks so it never emits a distance outside the smaller window. Output is bounded while decoding, before allocation or copying beyond the limit. Compressed input also counts against the message limit.

## Configuration

`Ws_Options` defaults to a 16 MiB frame and 64 MiB message. Set it on `Ws_Codec.options` or `Ws_Config.limits`. Both wire message storage and inflated output are bounded. Native writes require a positive frame limit. `Ws_Config.compression` defaults to true, and `protocols` is an ordered borrowed list (empty by default).

Timeouts are milliseconds: handshake/write 30000, read 0, close 5000. A nonpositive timeout disables that budget. Outer Io deadlines still apply; close cleanup uses `Io.with_grace`. Automatic control replies also obey the write budget. Protocol failures send an appropriate close when possible; transport failures release the connection. A raw EOF without a close frame is an error.

## How to change it

Framing, UTF-8 and limits live in `module.jai`; extension/key parsing in `handshake.jai`; HTTP integration, ownership and fiber I/O in `connection.jai`. Add malformed-wire regressions before changing state transitions. Preserve pending bytes and the active TLS link when changing Http handoff. Keep history bounded in both directions and test dependent messages across flush boundaries when modifying Deflate.

Dependencies are `Basic`, `String`, `Base64`, internal SHA-1, and `Extensions/Deflate`. Native code also uses Http, Io, Tls through Http, and the internal OS secure random helper. No external service is needed for unit tests.

## Verification

```sh
export PATH="$TOOLCHAIN/bin:$PATH" JAIC_STDLIB="$PWD/stdlib"
jacket test --path stdlib/Extensions/WebSocket
jacket test --path stdlib/Extensions/WebSocket --interp
jacket test --path stdlib/Extensions/WebSocket --os wasm --runner 'node tools/wasi_run.ts'
jacket bench --path stdlib/Extensions/WebSocket
```

The benchmark starts its own loopback echo server for ws/wss and plain/Deflate modes using the committed TLS certificates. Setup precedes `reset_timer`; message contents and replies pass through `black_box`. These cases do not run external Python or Autobahn conformance.

The external Python conformance driver uses a separate echo peer:

```sh
jaic stdlib/Extensions/WebSocket/tests/support/peer.jai -exe target/ws-peer
jaic stdlib/Extensions/WebSocket/tests/support/conformance.jai -exe target/ws-conformance
PYTHONPATH=tools target/ws-conformance python3.14 target/ws-peer target/ws-interop
```

The driver asserts the expected ten-result pass list over ws/wss, with and without compression, including no-context-takeover. It uses the pinned Python `websockets` wheel and reports that Autobahn was not run. Full Autobahn acceptance requires a separate Docker run.
