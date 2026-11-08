# gRPC interoperability and conformance

What checks `Extensions/Grpc` against independent implementations, and how to run and extend them. The [runtime contract](../extensions/grpc.md) says what the module supports. A built fixture is never a passed case; unexecuted or unsupported cases are listed as such.

## Interop matrix

`tools/grpc/interop.py` runs Jai and grpcio as TLS peers in both directions, with handwritten payloads and generated typed services. Each direction runs unary, server-streaming, client-streaming and bidirectional methods, empty payloads, application errors, initial/repeated/binary metadata, incoming deadlines and cancellation followed by a successful sibling call.

Use the `python_grpc` interpreter from `.github/pins.json` (CI exports it as `GRPC_PEER_PYTHON`; the harness rejects another patch release). Packages are pinned in `tools/grpc/requirements.txt`; `grpc_tools.protoc` is the generator, so no system protoc is used. To change the interpreter pin, edit the manifest and run `PYTHONPATH=tools python3 tools/ci/pins.py sync`.

```sh
export JAIC_NATIVE_LIBS="$(PYTHONPATH=tools python3 -c 'from toolkit.repo import cargo_target_dir; from repo_output import host_platform; print(cargo_target_dir(resolve=True) / "http-native" / host_platform())')"
PYTHONPATH=tools python3 tools/native/http_native.py --out "$JAIC_NATIVE_LIBS"
python3.14 -m venv target/grpc-peer
target/grpc-peer/bin/python -m pip install --only-binary=:all: -r tools/grpc/requirements.txt
PYTHONPATH=tools python3 tools/grpc/interop.py --jaic target/debug/jaic \
  --peer-python target/grpc-peer/bin/python --output target/grpc-interop
```

The native profile builds the hash-pinned MbedTLS archive and C adapter that `Extensions/Tls` uses (needs a C compiler and network for the source cache). The harness needs loopback listeners, trusts the repository's test CA and verifies the server identity, and requires ALPN `h2`. `--timeout` bounds each peer operation and readiness wait (30 s); readiness uses a port file. A teardown timeout is a failure, and failed cases are not retried.

The output folder keeps compile and peer logs, exact invocations, the generated Python codecs and `result.json` (package versions, OS/architecture, compiler and schema hashes, completed directions; no machine paths or hostnames). A failed run writes only a failed outcome and the exception class; raw diagnostics stay in local logs. Privacy tests: `PYTHONPATH=tools python3 -m unittest discover -s tools/grpc`.

The CI `grpc-interop` job is required for changes to Http, Grpc, Protobuf, Tls, Io or Deflate. It uploads `result.json` and, on failure, an allowlist of native JSON/stderr and peer/compiler logs. The Linux x64 and Windows x64 legs are independent; other Windows architectures have no peer gate. The Windows leg first runs `tools/grpc/windows_acceptance.py` (below).

The matrix covers what it executes. Scheduler, flow control, malformed input and cleanup need their focused module tests.

## Protocol conformance

Full acceptance also needs the canonical TestService cases, negative HTTP/2 peers, framing tests, HPACK stories and the Protobuf conformance runner. Fixtures use the synthetic test certificate and key, which must stay out of published binaries; independent clients must verify its CA. Use `JAIC_NATIVE_LIBS` for the native provider and an absolute `GRPC_PROTO_ROOT` for the pinned schema tree.

**HTTP/2 server.** `conformance/h2-server.jai` serves TLS with ALPN `h2` and drains request bodies through a fixed buffer. Arguments are a ready file (receives the port) and a stop file (creating it shuts the server down).

```sh
JAIC_STDLIB="$PWD/stdlib" jaic tools/grpc/conformance/h2-server.jai -exe target/h2-server
./target/h2-server target/h2.port target/h2.stop
```

**HPACK.** `conformance/hpack.jai` takes a JSON array of story files from [http2jp/hpack-test-case](https://github.com/http2jp/hpack-test-case) (MIT). Each story gets a fresh context. Output lines hold the story, case index, decoder outcome, advertised table limit and our encoded block. `hpack-decoder.go` decodes every emitted block with `golang.org/x/net/http2/hpack`, built inside the pinned grpc-go module with its unchanged `go.mod`; decoding our own output alone proves nothing. All 478 stories pass: 47,142 supplied blocks and 50,526 emitted blocks.

```sh
JAIC_STDLIB="$PWD/stdlib" jaic tools/grpc/conformance/hpack.jai -exe target/hpack-corpus
./target/hpack-corpus target/hpack-stories.json > target/hpack-records.jsonl
```

**h2spec 2.6.0** targets RFC 7540; the transport targets RFC 9113. Of 147 strict cases, 143 pass. The four failures (`http2/5.1/8`, `http2/5.1/11`, `http2/5.3.1/1`, `http2/5.3.1/2`) are RFC version differences and stay failures in the ledger. `Http::h2-current-profile` checks the RFC 9113 behaviour separately: closed-stream DATA credit, HPACK sync after closed HEADERS, deprecated PRIORITY.

**Canonical cases.** The pinned catalogue has 30 cases. The 21 supported core cases pass in both directions and both adapters (84 runs): the canonical C++ client against Jai servers, and Jai clients against the official Go TestService (14 cases, plus scaling and both soaks through the adapter below) or the C++ server (four compression cases). Go lacks the compression probes and the peer-address extension, so those route to C++ and to an adapter that fills the real peer address (`conformance/test-service-peer.go`, built in the pinned grpc-go checkout with `go build -mod=readonly`; the optional `observed-peer` argument enables only that extension). Soaks run 1,000 iterations with zero allowed failures, 1,000 ms per call and 1,200 s overall; never use the C++ tool's zero defaults. Connection scaling needs `GRPC_EXPERIMENTAL_MAX_CONCURRENT_STREAMS_CONNECTION_SCALING=true` and `--service_config_json='{"connectionScaling":{"maxConnectionsPerSubchannel":2}}'`. Set `GRPC_DEFAULT_SSL_ROOTS_FILE_PATH` to the test CA.

Nine cases are outside the supported API and are not run: cloud credential providers (`compute_engine_creds`, `jwt_token_creds`, `oauth2_auth_token`, `per_rpc_creds`, `google_default_credentials`, `compute_engine_channel_credentials`), `cacheable_unary` (GET/caching proxies) and ORCA (`orca_per_rpc`, `orca_oob`). They stay in the denominator.

The Protobuf conformance ledger is 6,699 cases: 5,786 binary/JSON passes, 909 unsupported text cases and four unsupported future-edition cases.

**Test-service fixtures.** `test-service-common.jai` holds the schema import and shared helpers; `test-service-server.jai` takes `raw` or `typed`; the client is built twice with `TYPED :: false` and `TYPED :: true`.

```sh
jaic tools/grpc/conformance/test-service-server.jai -add 'GRPC_PROTO_ROOT :: "/abs/grpc";' -exe target/test-service-server
./target/test-service-server raw target/server.port target/server.stop
jaic tools/grpc/conformance/test-service-client.jai -add 'GRPC_PROTO_ROOT :: "/abs/grpc"; TYPED :: false;' -exe target/test-service-raw-client
```

**Negative HTTP/2 peer.** The pinned gRPC negative peer predates Python 3 and current `h2`. `conformance/negative-peer.py` writes a patched copy (original and adapted SHA-256 and a unified diff are kept; the original tree is untouched). It also fixes a defect where a valid empty END_STREAM DATA frame parsed the retained request twice; `negative-peer-regression.py COPY` checks that fix and all five incomplete-prefix lengths. It is a patched fixture, not an unchanged upstream run. It needs `h2==4.3.0`, `Twisted==25.5.0` and the recorded Protobuf runtime. A wrapper terminates test TLS and forwards the HTTP/2 bytes to the cleartext peer. `conformance/run-negative.py` verifies the upstream revision and runs all eight cases in raw and typed mode (16 runs), failing on any client, peer, setup or timeout failure; each case must record both client and peer exits.

```sh
PYTHONPATH=tools python tools/grpc/conformance/run-negative.py \
  --upstream target/grpc-upstream --python target/peer-venv/bin/python \
  --raw-client target/test-service-raw-client --typed-client target/test-service-typed-client \
  --scratch target/negative --output target/negative-results
```

Its result exports only the exception class for a harness error. The padded-DATA and one-stream concurrent-call cases have direct regressions in `Http/tests/h2-padding.jai` and `Http/tests/h2-channel-lifecycle.jai`. Gzip framing also has a fuzz property for fragmentation, bounds and corrupted member lengths.

**Windows gate.** `tools/grpc/windows_acceptance.py` runs the real Io/Tls module roots with exact prerequisite IDs, then the Http/Grpc roots, and accepts only complete reports in which every required ID passed. Missing, skipped, duplicate, list-only or unfinished results fail even if the runner exits zero. Each program has a 120 s external watchdog. Rename tests together with the required IDs.

```sh
PYTHONPATH=tools python tools/grpc/windows_acceptance.py --jacket target/debug/jacket.exe --output target/grpc-windows
```

## Comparison with other implementations

| Implementation | Pin | Used for |
| --- | --- | --- |
| gRPC C++ | [`f2f2334f`](https://github.com/grpc/grpc/tree/f2f2334fb937ec78b3dc629ccc163b06913deb40) | Unmodified canonical interop client (Release build) against Jai servers; also the compression server |
| grpc-go | [`v1.85.0`](https://github.com/grpc/grpc-go/tree/c9ab42cdb0a2b3e7b9d5510ebbc999638ee41eb7) | Official TestService server for Jai clients |
| grpcio | 1.78.0, Python 3.14.3, Protobuf 6.33.5 | The interop matrix above |

No Jai gRPC library was found in the [community catalogue](https://github.com/Jai-Community/Jai-Community-Library/wiki/References) or GitHub searches on 2026-11-04. That is a bounded search, not proof, and nothing was compared.

Unary RPC with a 271,828-byte request and 314,159-byte response over one reused TLS/HTTP/2 channel, measured with the same optimized Jai client against an optimized Jai server and the Go TestService server (Apple M5, four interleaved repetitions of 1,010 calls, first ten discarded): median p50 1.50 ms for Jai and 1.24 ms for Go. Both include TLS, so it is an application-task comparison, not server CPU time; Jai's MbedTLS ChaCha20-Poly1305 dominated its sampled stacks. There is no earlier Jai gRPC implementation to compare against.

To rerun, build the Jai client and server with `-O2` and the pinned Go peer, wait for a quiet machine, and run `tools/grpc/conformance/compare-unary.py`. The server fixture takes `GRPC_PROTO_ROOT`; the client also takes `TYPED :: false`. The harness alternates provider order, keeps every timing and reports ranges.

```sh
PYTHONPATH=tools python tools/grpc/conformance/compare-unary.py \
  --client target/benchmark-client --jai-server target/benchmark-server \
  --go-server target/test-service-peer --scratch target/grpc-comparison \
  --output target/grpc-comparison-results
```

## Changing it

- Contract changes touch `tools/grpc/fixtures/echo.proto` and both the raw and generated fixtures; keep the Python codec generated by the pinned protoc.
- Add failure cases through the peer's public handler and client API, never by copying another implementation's internals.
- Keep suite data and oracles independent of our code, and pin revisions, corpus hashes and case IDs. Public reports must exclude machine paths and secrets.
- Compiler, schema or transport edits invalidate earlier evidence: rerun the matrix. Update the comparison table with runtime profile or dependency changes.
- Go and Python peers are test dependencies only; applications need the Jai modules and the native TLS provider.
