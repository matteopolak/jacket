# HTTP/2

The part of `Extensions/Http` that speaks HTTP/2 (RFC 9113) and HPACK (RFC 7541), for both the server and the client. Users never call it directly: a TLS connection that picks `h2` through ALPN is handed to it, and the usual `App` handlers or `Client` calls work unchanged. Turning it on and off, and the limits a user can set, are covered in [http-server.md](http-server.md) and [http-client.md](http-client.md). This page is for changing the implementation.

## Files

| File | What it holds |
| --- | --- |
| `h2_frame.jai` | Frame codec: the 9-byte header, each frame type's payload, `h2_put_*` writers. No IO. |
| `hpack.jai`, `hpack_huffman.jai` | Header compression: encoder and decoder, each with its own dynamic table, plus the Huffman code (the decoder table is built at compile time with `#run`). |
| `h2.jai` | The shared connection: `H2_Conn`, `H2_Stream`, the reader loop, the writer fiber, flow control, frame rules. |
| `h2_server.jai` | `serve_h2`: turns each request stream into a `Request_Frame` and runs `handle_request` on it. |
| `h2_client.jai` | One pooled `H2_Conn` per origin, `converse_h2` for one request, retry on refused streams. |

## How a connection runs

Three kinds of fibers share an `H2_Conn`, all on one `Io` scheduler, so there are no locks:

- **Reader** (`h2_read_loop`). Takes frames off the link and dispatches them by type (`h2_on_data`, `h2_on_headers`, ...). When a header block completes, it calls the side's hook: `h2_server_headers` or `h2_client_headers`.
- **Writer** (`h2_writer`). The only fiber that waits on the socket. Everyone else appends frames to `conn.out`. The writer swaps that buffer with `conn.spare` and writes the batch.
- **Request fibers.** On the server, a handler per stream. On the client, the callers.

Because only the writer blocks on writes, a peer that stops reading can't stall the reader. Senders wait on `conn.drained` only when more than `H2_QUEUE_LIMIT` (256 KiB) is queued.

Frames go into `conn.out` in the order the peer must see them. HPACK depends on that: a header block is encoded at the moment its frame is queued, so both dynamic tables stay in step. Never encode a block early and queue it later.

## Streams and lifetimes

`H2_Stream` and `H2_Conn` are reference counted (`refs`):

- The connection's stream table holds one reference, and whoever is using a stream (a handler, a body reader) adds one.
- `h2_stream_release` and `h2_conn_release` free on the last reference.
- `removed` marks a stream already taken out of the table.

A stream closes once both sides have sent `END_STREAM` or one has reset it (`h2_check_closed`). `h2_mark_dead` wakes every waiter and cancels handler fibers when the connection fails.

## Flow control

- **Receiving.** `h2_consumed` runs when the user takes body bytes. A stream-level `WINDOW_UPDATE` goes out once half the initial window has been taken. The connection level is returned in `H2_UPDATE_BATCH` (16 KiB) steps.
- **Sending.** `h2_send_data` splits by the peer's frame size and waits on the stream's `changed` condition when either window runs out.
- **Start-up windows.** The server's are set by `h2_window` and `h2_connection_window`. The client's are fixed: `H2_CLIENT_WINDOW` (256 KiB per stream) and `H2_CLIENT_CONNECTION_WINDOW` (1 MiB). The extra connection room is sent as a `WINDOW_UPDATE` right after the first SETTINGS (`h2_conn_start`).

## Errors

The RFC splits errors in two, and the code follows:

- A **connection error** returns a non-`NO_ERROR` `H2_Error` from a frame handler, which makes the reader call `h2_terminate` (GOAWAY, then close).
- A **stream error** calls `h2_reset_stream` (RST_STREAM) and keeps the connection.

Header blocks larger than `H2_BLOCK_FACTOR` × the header list limit are refused before they are decoded.

Graceful shutdown (`h2_shutdown`) sends GOAWAY with the last stream the peer opened and sets `draining`, so new streams are refused while open ones finish. The client marks streams the server never processed as `refused` (by REFUSED_STREAM, or above GOAWAY's last stream). The client's request loop retries those once.

## How to change it

- **New frame type or setting.** Add the codec in `h2_frame.jai`, then a case in `h2_dispatch`. Unknown frame types must stay ignored (RFC 9113 §4.1).
- **Server-only or client-only behaviour.** It goes in `h2_server.jai` or `h2_client.jai`, reached through the role hooks. Keep `h2.jai` free of role checks where you can.
- **Header rules.** `h2_connection_specific` lists the HTTP/1.1 fields that are malformed in HTTP/2. The server's request checks are in `h2_build_request`.
- Not supported on purpose: cleartext h2c, server push (the server ignores it and the client sends `ENABLE_PUSH=0`) and priorities (parsed, then ignored).

## Tests

Everything lives in `stdlib/Extensions/Http/tests/`. Run each one from that directory.

- `h2-frames.jai` and `hpack.jai` test the codecs, including the RFC 7541 examples.
- `h2-server.jai` drives the server from a hand-written peer over TLS, so each frame and error code is checked on the wire.
- `h2-client.jai` runs the client against this library's own server on loopback: one shared connection, flow control both ways, streamed responses, and falling back to HTTP/1.1.
- `h2-client-faults.jai` uses a hand-written server that misbehaves in ways ours never does: REFUSED_STREAM, a GOAWAY in the middle of a batch of requests.

None of them run on wasm, because the server and TLS aren't built there.

## Dependencies

- `Extensions/Tls` provides the link and ALPN.
- `Io` provides fibers, conditions, deadlines and wait groups.
- The HTTP/1.1 side's `Request_Frame`, `Body` and `Stream` are reused so that handlers can't tell which protocol carried a request.
