# Protocol conformance

The gRPC interoperability smoke matrix is one acceptance layer. Full protocol
acceptance also needs canonical gRPC TestService cases, negative HTTP/2 peers,
HTTP/2 framing tests, stateful HPACK stories and the current Protobuf conformance
runner. A successful fixture build does not establish any of those results.

## HTTP/2 fixture

`tools/grpc/conformance/h2-server.jai` starts a TLS server with ALPN `h2`. It drains
request bodies through a fixed buffer before replying, so the application neither
collects arbitrary request bodies nor prematurely resets a streaming request.
Pass a ready-file path and a stop-file path. The ready file receives the bound
loopback port; creating the stop file initiates shutdown and joins the server.

```sh
JAIC_STDLIB="$PWD/stdlib" jaic tools/grpc/conformance/h2-server.jai -exe target/h2-server
./target/h2-server target/h2.port target/h2.stop
```

Use the repository's pinned HTTP native SDK profile through `JAIC_NATIVE_LIBS`.
The fixture uses the synthetic TLS test certificate and key, which must stay out of
published binaries and artifacts. Independent clients must verify its test CA;
disabling certificate verification changes the acceptance contract.

## Stateful HPACK fixture

`tools/grpc/conformance/hpack.jai` exercises the owning encoder and decoder. Its
argument is a JSON array of story-file paths from
[http2jp/hpack-test-case](https://github.com/http2jp/hpack-test-case), whose MIT
license accompanies the corpus. Each story gets a fresh compression context;
cases within the story share it. Decoder results are compared with the supplied
ordered header fields. Each output line records the story, case index, decoder
outcome, current advertised table limit and our encoded block as hexadecimal.
An independent decoder must also check every emitted block against the original
headers; decoding our own output alone does not establish interoperability.

```sh
JAIC_STDLIB="$PWD/stdlib" jaic tools/grpc/conformance/hpack.jai -exe target/hpack-corpus
./target/hpack-corpus target/hpack-stories.json > target/hpack-records.jsonl
```

Both fixtures load the actual HTTP codec and transport. Extend them at those
owning seams when behavior changes, keeping suite data and oracle implementations
independent. Preserve pinned revisions, corpus hashes, exact case identities and
failures. Retain raw diagnostics locally; public reports must exclude personal
machine paths and secrets.

## Standards and coverage

h2spec 2.6.0 targets RFC 7540 and RFC 7541. The transport targets RFC 9113 and RFC
7541. Run all strict cases before classifying any difference: RFC 9113 changed
closed-stream and deprecated priority handling. A reviewed standards difference
belongs in an explicit case ledger and must not be relabeled as a passing original
suite case. The initial strict run executed 147 cases with 143 passes and four
failures. The four failures remain failures in that original ledger; review
classifies them as RFC-version differences, not supported-profile passes.
`Http::h2-current-profile` separately checks RFC 9113 closed-DATA connection
credit, closed-HEADERS HPACK synchronization, and deprecated PRIORITY processing
while retaining malformed-length and stream-zero errors.

The full pinned HPACK corpus completed all 478 stories: 47,142 supplied wire
blocks decoded correctly, and an independent decoder verified all 50,526 emitted
blocks. A dynamic-name eviction lifetime defect found by this corpus has a focused
ASan regression. The 21 core canonical C++ client cases passed against both handwritten and
generated Jai servers, including compression, metadata, errors, scaling and
1,000-iteration RPC/channel soaks. The soak acceptance profile allows zero failures,
1,000 milliseconds per call and 1,200 seconds overall. These are candidate-source
results. The reverse raw/generated clients also passed all 21 core procedures:
14 against the configured official Go TestService, four compression procedures
against the canonical C++ server, connection scaling against the Go service with
an adapter that fills the actual observed peer address, and both 1,000-iteration
soaks against Go. The adapter preserves upstream streaming handlers and supplies
no fabricated connection identity. All eight negative HTTP/2 cases then passed against the patched independent
peer in both client modes, with both client and peer exits checked (16 case
executions). Final-head gates remain in progress. Automatic cloud credential
providers, GET/caching proxies and ORCA balancing metrics are outside the existing
explicit-metadata/TLS API profile. Their nine catalogue IDs remain unexecuted.
No unexecuted or unsupported case counts as passed. The full Protobuf ledger is
6,699 cases: 5,786 binary/JSON passes, 909 unsupported text cases and four
unsupported future-edition cases. The declared profile is binary/JSON, proto2,
proto3 and editions 2023/2024.

`hpack-decoder.go` supplies that independent check through `golang.org/x/net/http2/hpack`.
Build it in the pinned grpc-go oracle module, preserving that module's exact
`go.mod` and `go.sum`, then pass the emitted JSON-lines file. It verifies ordered
fields for every case, follows each advertised table limit and returns nonzero
on any owning-decoder or independent-decoder mismatch. The Go oracle is a test
dependency; application builds do not require Go.

## Negative HTTP/2 peer compatibility

The pinned gRPC negative peer predates Python 3 and the current `h2` API.
`tools/grpc/conformance/negative-peer.py` writes a separate compatibility copy,
retaining original/adapted SHA-256 identities and a complete unified diff. The
original source tree is never modified. Python syntax, byte buffers, integer
slicing and renamed `h2` APIs are updated without changing assertions or payloads.
The old Protobuf bindings use the pure Python Protobuf backend.

A separate correction clears a successfully parsed request and waits for a
complete five-byte envelope prefix. Without it, a valid empty END_STREAM DATA
frame parses the retained request twice and restarts the response from offset
zero. A standalone regression retains the original failure and corrected success,
including all fragmented prefix lengths. This is a patched compatibility fixture,
not an unchanged upstream execution.

The wrapper terminates verified test TLS with ALPN `h2`, then forwards unchanged
HTTP/2 bytes to the independent cleartext peer. Twisted callback failures make the
peer exit nonzero. Each case must record both client and peer exits; a successful
client alone cannot satisfy a server assertion. Supply the original pinned
`test/http2_test` directory and a distinct writable compatibility directory:

```sh
PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION=python python tools/grpc/conformance/negative-peer.py \
  upstream/test/http2_test target/negative-copy goaway \
  stdlib/Extensions/Tls/tests/fixtures/server.pem \
  stdlib/Extensions/Tls/tests/fixtures/server.key target/peer.port target/peer.stop
```

Use Python with `h2==4.3.0`, `Twisted==25.5.0` and the recorded Protobuf runtime.
The ready file supplies the TLS port; create the stop file after the client exits.
Compile `test-service-client.jai` with `GRPC_PROTO_ROOT` naming the pinned schema
tree and `TYPED` selecting raw/generated wrappers. Its case argument accepts all
eight negative catalogue names. Both modes must run the complete inventory.

Run `negative-peer-regression.py COPY` with the same Python environment to check
request consumption and all five incomplete-prefix lengths. Keep the original
peer failure and compatibility-copy success alongside the protocol receipts.
The padded-DATA case exposed a separate runtime defect: padding was charged to
the stream window but never credited when discarded. `Http/tests/h2-padding.jai`
checks that accounting directly. The one-stream concurrent-call case similarly
has a deterministic admission/cancellation regression in
`Http/tests/h2-channel-lifecycle.jai`.

The complete contained runner verifies the upstream checkout revision, executes
all eight cases in raw and generated modes and returns nonzero for any client,
peer, setup or timeout failure. It writes a 16-entry ledger and separate logs:

```sh
PYTHONPATH=tools python tools/grpc/conformance/run-negative.py \
  --upstream target/grpc-upstream --python target/peer-venv/bin/python \
  --raw-client target/test-service-raw-client \
  --typed-client target/test-service-typed-client \
  --scratch target/negative --output target/negative-results
```

Build both clients with the same compiler, native SDK and schema root:

```sh
jaic tools/grpc/conformance/test-service-client.jai \
  -add 'GRPC_PROTO_ROOT :: "target/grpc-upstream"; TYPED :: false;' \
  -exe target/test-service-raw-client
jaic tools/grpc/conformance/test-service-client.jai \
  -add 'GRPC_PROTO_ROOT :: "target/grpc-upstream"; TYPED :: true;' \
  -exe target/test-service-typed-client
```

`GRPC_PROTO_ROOT` must be an absolute path if the compiler runs from another
working directory. Keep `JAIC_STDLIB` and `JAIC_NATIVE_LIBS` set to the owning
checkout and native provider. Acquire the shared build/test admission gate when
running locally alongside other work; timed comparisons require an exclusive
quiet window.

The reverse soak fixture retains each elapsed duration and status for a second
aggregation pass. Channel-soak timing includes creating/connecting the new channel
and excludes its teardown, as the catalogue requires. All eight configured soak
executions (both directions, both adapters, two cases) passed 1,000 iterations
with zero failures after this timing correction. Client peer-address logging is
explicitly unavailable in this fixture; no socket identity is fabricated.

Gzip framing also has a fuzz property covering arbitrary fragmentation, encoded
and decoded bounds, exact round trips and corrupted gzip member lengths.

## Canonical case inventory

The pinned catalogue has 30 named interoperability cases. Current candidate
receipts cover the 21 supported core cases below in both directions and both adapters:
84 case executions, all passing. The nine cases outside the existing API profile
remain unexecuted; they are not counted as passes or silently removed. Negative HTTP/2
coverage is a separate 8-case client inventory, executed in both adapters (16
passes). The RFC/Protobuf ledgers remain separate from these denominators.

| Core cases | Independent client for Jai servers | Independent server for Jai clients |
| --- | --- | --- |
| `empty_unary`, `large_unary` | Canonical C++ | Configured official Go TestService |
| `client_compressed_unary`, `server_compressed_unary` | Canonical C++ | Canonical C++ |
| `client_streaming`, `server_streaming` | Canonical C++ | Configured official Go TestService |
| `client_compressed_streaming`, `server_compressed_streaming` | Canonical C++ | Canonical C++ |
| `ping_pong`, `empty_stream` | Canonical C++ | Configured official Go TestService |
| `custom_metadata` | Canonical C++ | Configured official Go TestService |
| `status_code_and_message`, `special_status_message` | Canonical C++ | Configured official Go TestService |
| `unimplemented_method`, `unimplemented_service` | Canonical C++ | Configured official Go TestService |
| `cancel_after_begin`, `cancel_after_first_response` | Canonical C++ | Configured official Go TestService |
| `timeout_on_sleeping_server` | Canonical C++ | Configured official Go TestService |
| `max_concurrent_streams_connection_scaling` | Canonical C++ with explicit service config | Go TestService with actual peer-address adapter |
| `rpc_soak`, `channel_soak` | Canonical C++, explicit nonzero profile | Configured official Go TestService, same acceptance profile |

The independent Go helper does not implement the canonical compression probes or
peer-address extension. Routing those cases to capable peers preserves the full
case inventory; those missing helper behaviors are not Jai runtime failures.
Scaling advertises two concurrent streams and allows two connections. The C++
client requires both settings below:

```sh
export GRPC_EXPERIMENTAL_MAX_CONCURRENT_STREAMS_CONNECTION_SCALING=true
interop_client --server_host=localhost --server_port="$port" \
  --use_tls=true --use_test_ca=false \
  --test_case=max_concurrent_streams_connection_scaling \
  --service_config_json='{"connectionScaling":{"maxConnectionsPerSubchannel":2}}'
```

Set `GRPC_DEFAULT_SSL_ROOTS_FILE_PATH` to the synthetic test CA. For each soak,
pass `--soak_iterations=1000 --soak_max_failures=0`,
`--soak_per_iteration_max_acceptable_latency_ms=1000` and
`--soak_overall_timeout_seconds=1200`; never use the C++ tool's zero latency and
overall-timeout defaults. The Jai fixture takes the corresponding explicit
`1000 1000 1200` arguments after its URL, case name and CA path. Each direction
uses one soak worker and no RPC deadline.

| Outside the existing supported profile, unexecuted | Capability absent from the declared API |
| --- | --- |
| `compute_engine_creds`, `jwt_token_creds`, `oauth2_auth_token`, `per_rpc_creds`, `google_default_credentials`, `compute_engine_channel_credentials` | Automatic cloud identity/credential providers; this API uses explicit metadata/TLS |
| `cacheable_unary` | GET/caching-proxy extension; supported RPC routes use POST |
| `orca_per_rpc`, `orca_oob` | ORCA/custom-balancing metrics API; no balancing API is declared |

These rows remain in the 30-case catalogue denominator. Adding their facilities
would extend the API and require implementation and independent execution; the
core result does not claim the entire upstream catalogue passed. The four strict
h2spec version differences retain their original failed identities:
`http2/5.1/8`, `http2/5.1/11`, `http2/5.3.1/1`, `http2/5.3.1/2`.

The compact negative-suite result exports only the exception class for a harness
error. Full diagnostic text stays in local peer logs. This preserves failure
counts and teardown evidence while keeping private paths and context out of the
shareable ledger; the failure-privacy regression exercises all 16 result rows.

## Canonical service fixtures

`test-service-common.jai` owns the shared schema import and adapter-neutral
message helpers. `test-service-server.jai` chooses handwritten or generated
registration at runtime; the two client builds select wrappers with `TYPED`.
Keep the official `.proto` tree pinned and use its original wire names. Extend
case procedures in the client/server fixtures and retain the catalogue assertions
when changing the runtime or generator.

```sh
jaic tools/grpc/conformance/test-service-server.jai \
  -add 'GRPC_PROTO_ROOT :: "/absolute/path/to/pinned/grpc";' \
  -exe target/test-service-server
./target/test-service-server raw target/server.port target/server.stop
```

Use `typed` in place of `raw` for generated registration. The ready file contains
the TLS port; creating the stop file closes the server. These fixtures use the
repository's synthetic certificate and key and the selected native TLS provider.

`test-service-peer.go` must be built from the pinned grpc-go module checkout,
using its unchanged `go.mod` and `go.sum`. Pass the repository fixture as an
absolute path to `go build -mod=readonly`. Its invocation is:

```sh
test-service-peer stdlib/Extensions/Tls/tests/fixtures/server.pem \
  stdlib/Extensions/Tls/tests/fixtures/server.key target/go.port target/go.stop
```

The optional final argument `observed-peer` enables only the actual transport
peer-address extension. Other requests remain handled by the official imported
TestService implementation. Preserve both source/tool hashes and the adapter
mode in receipts. The peer programs are development dependencies; applications
need only the owning Jai modules and native TLS provider.

## Verified Windows prerequisites

`tools/grpc/windows_acceptance.py` runs real Io/Tls module roots with exact
prerequisite IDs, followed by the Http/Grpc module roots. It accepts only complete
JSON reports in which every required ID actually passed. Missing, skipped,
duplicate, list-only or unfinished prerequisite results fail even if the runner
exits zero. Local raw JSON and stderr retain diagnosis; the compact result
contains case IDs/statuses and the runner hash, without private failure fields.

```sh
PYTHONPATH=tools python tools/grpc/windows_acceptance.py \
  --jacket target/debug/jacket.exe --output target/grpc-windows
```

Each test program has an explicit 120-second external runner watchdog. The
closed-port Http checks use no request deadline because Windows can report native
refusal after more than two seconds; they still require exactly transport /
connection-refused. The separate 150 ms request-timeout checks are unchanged.
The file-response test gets an isolated `$TEMP` path from jacket; its small
fixture has a repository LF attribute because content and length are exact bytes.
Change required IDs and the consumer regression together when renaming tests.
The workflow calls this driver and archives its compact `result.json`. Native
Windows acceptance remains required; the driver itself is not evidence of a pass.
