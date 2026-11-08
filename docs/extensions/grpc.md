# gRPC

`Extensions/Grpc` provides TLS HTTP/2 client and server RPCs, identity and bounded gzip message
framing, metadata, canonical status and owned Protobuf message helpers. Unary,
server-streaming, client-streaming and bidirectional methods use one call lifecycle.
`Extensions/Protobuf/Schema` can generate typed adapters with `GRPC = true`.

## Calls and completion

Keep a channel at a stable address inside an active `Io.io_run` scheduler. Initialize
its TLS configuration with trusted roots and ALPN `h2`; configure
`channel.transport.base_url` and `channel.transport.tls`. The configuration and
channel must outlive all calls and `close_channel`. There is no HTTP/1.1 fallback,
plaintext transport, redirect, automatic retry or replay after REFUSED_STREAM/GOAWAY.

```jai,ignore
method := Grpc.Method.{ path = "/example.Echo/Unary" };
call, opened := Grpc.open_call(*channel, method);
defer Grpc.close(*call);
sent := Grpc.send(*call, request);
half := Grpc.close_send(*call);
response, kind := Grpc.receive(*call, Api.Response);
defer Proto.free(*response);
status := Grpc.finish(*call);
defer Grpc.free_status(*status);
```

Check each returned status. `receive` distinguishes MESSAGE, END and ERROR; a valid
message can precede a non-OK final result. `finish` provides authoritative RPC status
and discards unread messages individually. A decoded message, HTTP 200 and byte EOF
are insufficient. Returned statuses own their message, rich status details and
trailing metadata; repeated `finish` returns independent owned results. `close`
releases an unfinished call and is idempotent on its cleared handle.

`send` borrows its value until it returns. `receive` uses owned Protobuf decode;
release a MESSAGE with `Proto.free`. `send_bytes`/`receive_bytes` support a caller's
encoded message codec; received byte arrays are owned and freed with `free(data)`.
Framing accepts arbitrary HTTP/2 fragmentation and coalescing, rejects oversized
lengths before allocating payload storage, and never collects a whole stream.

Exactly one sender and one receiver may operate concurrently on a scheduler. Use
separate fibers for simultaneous large sends and receives, and join them before
closing the call. Multiple concurrent readers or writers are rejected. Applications
that send indefinitely without reading can deadlock against a peer doing the same.
Cross-thread use is unsupported.

## Compression and initial response metadata

Set `Call_Options.compression = .GZIP` for outgoing compressed messages. Clients
advertise `identity,gzip`; a server configured for gzip falls back to identity
when its peer does not advertise gzip. Each `send` or `send_bytes` accepts
`Send_Options.{compress = false}` to send an uncompressed envelope on the same
call. Generated context send methods expose the same option.
`received_compressed` reports the wire flag of the last received MESSAGE.

The encoded and decompressed bounds apply separately. Gzip is decoded as a complete
payload: all concatenated members and checksums must validate, trailing garbage
is rejected, and the aggregate decompressed limit prevents expansion beyond the
configured message bound. Compression temporarily owns bounded encoded and decoded
buffers; it does not collect a stream.

Server handlers may call `set_initial_metadata(call, entries)` before their first
send to replace application response metadata. Generated server contexts expose
the same operation. Entries are copied, so their source can be released immediately.
The first send or handler completion commits initial HEADERS. Invalid metadata
leaves the previous block intact; attempts after commitment return
FAILED_PRECONDITION without cancelling the call. This lets a handler select
metadata from its request while preserving one sender and independent receiver.

## Generated services

```jai,ignore
Api :: #import "Extensions/Protobuf/Schema"(
    GRPC = true, PATH = #run Proto.beside("echo.proto"));
response, status := Api.example.Echo.Unary_call(*channel, request);
```

Each service exposes method descriptors, `<Method>_Client`, `<Method>_Server`,
`<Method>_open`, a `Handlers` record and `register`. Unary methods additionally expose
`<Method>_call`. Client contexts provide typed send/receive, half-close, finish,
cancel, close and context access. Server contexts provide typed receive/send,
metadata, cancellation and deadline access. Wire paths preserve original package,
service and method names independently of escaped Jai identifiers. All wrappers
share the schema's message graph and the common runtime.

```jai,ignore
echo :: (call: *Api.example.Echo.Unary_Server, state: *void) -> Grpc.Status {
    request, kind := Api.example.Echo.Unary_Server.receive(call);
    if kind != .MESSAGE return .{ code = .INVALID_ARGUMENT };
    defer Proto.free(*request);
    return Api.example.Echo.Unary_Server.send(call, request);
}
handlers := Api.example.Echo.Handlers.{ state = *state, Unary = echo };
Api.example.Echo.register(*app, *handlers);
```

Each route borrows its handlers record and state. Keep both alive through registration
and every in-flight handler. Different services keep independent route state;
registration uses Http's existing router and middleware. Routes cannot be unregistered or
replaced while serving. Duplicate registrations record startup errors. A missing
typed handler responds UNIMPLEMENTED. `Handlers.options` configures server message,
metadata and deadline limits, and its metadata supplies initial response metadata.
Handler-returned status fields borrow application storage through `serve_call`'s
return; use literals, persistent state or the request fiber's temporary allocator.

## Deadlines and cancellation

`Call_Options.deadline_ms` is an absolute monotonic millisecond deadline; zero means
no RPC deadline. It covers connect/admission, writes, reads and final completion,
without restarting between messages. The remaining budget becomes `grpc-timeout`;
servers parse all six units and round sub-millisecond budgets up. Pass
`call_deadline(parent)` explicitly to child call options. `initial_metadata` borrows
call-owned metadata until close; client metadata becomes available after receiving
initial response headers.

`cancel` records CANCELLED and wakes blocked transport operations. Local expiry
records DEADLINE_EXCEEDED. The first locally observed terminal result wins;
client/server observations need not agree during races. CPU work must yield and
poll `cancelled`; the runtime does not preempt application code.

Transport timeouts remain separate from RPC deadlines; a call deadline does not
override the shared transport's physical write timeout. The client channel's
`transport.write_timeout` defaults to zero. For a server supporting unlimited or
long-stalled writes, explicitly set `app.config.write_timeout = 0` before starting;
Http's default physical writer timeout otherwise remains 30 seconds. Active HTTP/2
calls do not inherit the ordinary request-body or idle timeout. Choose a finite
physical write timeout deliberately when deployment policy requires it.

## Metadata, status and limits

Metadata is an ordered list, retaining repeated values. Application names are
lowercase letters/digits plus `_-.`; `grpc-*` names are reserved. Values of `-bin`
entries are raw bytes; wire encoding is unpadded Base64, and decoding accepts padded,
unpadded and comma-joined binary values. Status messages use UTF-8 byte percent
encoding. `status_details` preserves `google.rpc.Status` bytes and validates an
embedded code against the explicit status. HTTP mapping is used only when final
`grpc-status` is missing. Reset/GOAWAY details retain the HTTP/2 code and the peer's
explicit `not_processed` claim, without authorizing replay.

| Setting | Default | Effect |
| --- | --- | --- |
| `Call_Options.max_message` | 4 MiB | Per uncompressed Protobuf message |
| `Call_Options.max_wire_message` | 4 MiB | Per encoded envelope payload, checked before allocation |
| `Call_Options.compression` | IDENTITY | Outgoing algorithm; GZIP permits per-message identity overrides |
| `Call_Options.max_metadata` | 64 KiB | Each initial or terminal metadata block |
| `Channel.max_active_calls` | 128 | Admission fails RESOURCE_EXHAUSTED at the limit |
| `channel.transport.max_connections` | 8 | Bounded connection scope; must be positive |
| `channel.transport.write_timeout` | 0 | Physical write budget, milliseconds |
| `app.config.h2_window`, `h2_connection_window`, `h2_max_streams` | Http defaults | Inbound flow control and active server streams |

One partial message and a fixed input buffer belong to each call. Decoded messages
transfer to the application; there is no background decoded-message queue. Typed
sending temporarily owns the encoded message and its envelope. HTTP/2 backpressure
counts queued and writer-owned bytes, with one frame/header block able to cross the
queue threshold. Application-held messages and metadata are outside these limits.

Identity and gzip compression use the Protobuf codec. Other compression algorithms are rejected. Reflection, health
services, interceptors, automatic credentials, balancing, retries/hedging,
transcoding and gRPC-Web are outside this API. Portable framing/status/metadata
codecs work without the native call API. Native RPC transport uses the existing
Io/Tls backends, including Windows IOCP sockets and Windows certificate trust.
The Windows acceptance job must execute the targeted Io/Tls, HTTP/2 and RPC tests
and independent peers; source availability or cross-target type-checks alone do
not establish runtime support. Generated transport rejects WebAssembly use explicitly. Message-only Protobuf generation remains independent
of Http/Tls when GRPC is false.

## Changing and validating it

Keep envelope logic in `wire.jai`, metadata/status rules in their codec files, call
ownership/deadlines in `call.jai`, and routing in `server.jai`. Http owns the opaque
exchange and stream engine; gRPC must not reach into private H2 types. Extend schema
syntax, resolution and emission together, preserving original wire names and one
shared message graph. Generated code must delegate transport semantics to the
runtime rather than copy framing or cancellation logic.

Run the module tests through `jacket test` and the [independent peer matrix](../tools/grpc-interop.md).
Framing/metadata fuzz properties live in `tools/fuzz/properties/Grpc.jai`. The module
is automatically discovered by Jacket and is also listed in
`tests/compiler/optimized-modules.txt` for the optimized sanitizer sweep. Dependencies
are Basic, String, Base64, Deflate, Protobuf and Http codecs; native transport additionally
uses Http, Tls, Io and the pinned MbedTLS native archive (plus platform trust
libraries where needed). Independent grpcio/protoc packages
are test dependencies only.

Incoming `grpc-timeout` also scopes handler synchronization waits, including
conditions, channels and wait groups. A peer reset cancels the request fiber.
Shielded cooperative cleanup may outlive shutdown grace; server close joins it
before releasing handler or TLS state. Client channel close likewise shields its
final TLS join from caller cancellation and deadlines.

When the peer completes a response before accepting the whole upload, a blocked
send returns FAILED_PRECONDITION and marks only the send direction closed. Continue
to `finish` for the authoritative response status; the stopped upload does not
replace it with a connection failure. This also permits a reader and sender to
observe terminal progress in either scheduler order.

Generated unary `<Method>_call` helpers keep the same receive/finish path after
a send-direction stop. Early successful responses retain their actual payload;
remote errors retain their code, message and trailing metadata. True local terminal
failures return the same recorded result without waiting for another response.

Connection admission includes in-progress connections and peer-ended connections
until their writer and TLS teardown finishes. A second call to the same connecting
origin waits for and shares that attempt; another resource scope is rejected at the
configured channel connection limit.

See the [interop and comparison guide](../tools/grpc-interop.md) for pinned peers and the
benchmark method.
