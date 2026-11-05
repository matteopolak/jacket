# gRPC feature and coverage comparison

This comparison separates observed interoperability from public API documentation.
It describes the repository's supported RPC profile, rather than certifying every
feature of another implementation. The [conformance procedure](grpc-conformance.md)
and [runtime contract](../extensions/grpc.md) define the corresponding local limits.

## Compared implementations

| Implementation | Pin | Evidence used here |
| --- | --- | --- |
| `Extensions/Grpc` | Enclosing repository revision | Owning regressions and independent peers; final-head acceptance still pending |
| gRPC C++ | [`f2f2334fb937ec78b3dc629ccc163b06913deb40`](https://github.com/grpc/grpc/tree/f2f2334fb937ec78b3dc629ccc163b06913deb40) | Unmodified canonical interop client, built Release; 21 core cases against each raw/generated Jai server |
| grpc-go | [`v1.85.0`, `c9ab42cdb0a2b3e7b9d5510ebbc999638ee41eb7`](https://github.com/grpc/grpc-go/tree/c9ab42cdb0a2b3e7b9d5510ebbc999638ee41eb7) | Official TestService server configured through its public API; 14 core procedures plus both soaks passed; actual peer-address adapter passed scaling |
| grpcio | `1.78.0`, Python `3.14.3`, Protobuf `6.33.5` | Current raw/generated four-mode peer matrix passed in both directions |

The public [Jai community catalogue](https://github.com/Jai-Community/Jai-Community-Library/wiki/References)
and GitHub repository-metadata searches for `grpc language:Jai` and
`grpc Jai in:name,description` did not identify a relevant Jai gRPC library on
2026-11-04. This is a bounded discovery result, not proof that none exists.
Its functionality and performance remain unknown; no forbidden Jai implementation
source was inspected. Add an identifiable alternative and its public documentation
before making any suitability or performance claim about it.

## Feature matrix

**Tested** means an actual named local peer case ran. **Documented** means the
upstream public API describes the capability, without a matching local result.
**Pending** and **unknown** are not passes. The C++ tested cells describe its
client role in this comparison; they do not establish its entire server profile.

| Capability | Extensions/Grpc | gRPC C++ client | grpc-go | grpcio |
| --- | --- | --- | --- | --- |
| Unary and three streaming modes | Tested in both directions | Tested against Jai server | Tested as independent server | Tested in both directions after compression changes |
| TLS verification and HTTP/2 | Tested server on macOS; other native acceptance pending; reverse client tested | Tested against synthetic CA and ALPN `h2` | Tested as independent server | Tested with synthetic CA |
| Protobuf generation | Shared opt-in message/service graph; owning tests | Generated canonical TestService client exercised | Official generated TestService peer | Bundled pinned protoc used in earlier matrix |
| Initial, trailing and binary metadata | Tested in both directions, including request-dependent initial values | Canonical custom-metadata case passed | Tested as independent server | Repeated/binary metadata matrix passed |
| Status and Unicode message preservation | Tested in both directions, including whitespace and non-BMP text | Canonical status cases passed | Tested as independent server | Application-error checks passed |
| Gzip and mixed per-message compression | All four compression cases passed in both directions; separate encoded/decoded bounds | All four canonical client compression cases passed; canonical server also tested | Go helper lacks canonical compression probes; C++ used | Not tested here |
| Cancellation and deadlines | Canonical cases in both directions plus owning lifecycle tests | Canonical cancellation/deadline cases passed | Tested as independent server | Incoming deadline/cancel checks passed |
| Connection scaling at two concurrent streams | Both directions tested | Tested with explicit two-connection service config | Actual peer-address wrapper passed scaling | Not tested here |
| RPC and channel soak | Both directions: 1,000 iterations per case, zero allowed failures | Both cases passed against each Jai server adapter | Both cases passed as independent server | Not run here |
| Ownership/resource policy | Caller-owned returned values; configured message, metadata, call and connection bounds | Implementation-specific; not measured here | Implementation-specific; not measured here | Implementation-specific; not measured here |
| Native/portable execution | Native RPC scheduler; portable codec; WebAssembly RPC unsupported | Native peer tested on macOS only here | Native peer tested on macOS here | Native peer tested on macOS |
| Automatic retry, balancing, cloud credentials, ORCA, caching GET | Outside current API or explicit profile decision pending | Not evaluated by this matrix | Not evaluated by this matrix | Not evaluated by this matrix |

Public API references: [C++](https://grpc.io/docs/languages/cpp/basics/),
[Go](https://grpc.io/docs/languages/go/basics/),
[Python](https://grpc.io/docs/languages/python/basics/). Documentation support does
not replace running the full applicable case inventory against the pinned build.
The cloud/proxy/ORCA profile decisions and RFC 7540 versus RFC 9113 differences
remain separate from these feature observations.

## Equivalent-task performance

Candidate-source measurements used an Apple M5 with 16 GiB RAM, verified
loopback TLS/HTTP/2 and one reused channel. The identical optimized Jai client
sent a 271,828-byte request and checked a 314,159-byte zero response against an
optimized native Jai server and the pinned optimized Go TestService server.
Each provider completed four interleaved repetitions with 1,010 calls each;
the first ten calls per repetition were retained as warmups. All 8,080 calls
passed the same status and payload checks; 4,000 calls per provider were scored.

| Server | Median run p50 (ms) | Median run p95 (ms) | Median run p99 (ms) | Median task/s | Task/s range |
| --- | ---: | ---: | ---: | ---: | ---: |
| Extensions/Grpc | 1.498 | 1.602 | 1.752 | 662 | 659–664 |
| grpc-go | 1.247 | 1.539 | 1.693 | 776 | 757–785 |

Go completed this particular task faster on this host. Across repetitions, p50
ranged from 1.494–1.505 ms for Jai and 1.239–1.249 ms for Go. The first call on
each new channel, excluded from warm scoring, ranged from 4.409–7.402 ms and
3.986–4.492 ms respectively. These measurements include the common client's
allocation, encoding, decoding, validation and call cleanup as well as both
providers' transport/TLS work. They establish no pre-feature speedup; there was
no earlier gRPC implementation to compare.

The raw receipt retains every warmup/scored duration, provider/compiler hashes,
executable byte sizes, build settings and per-repetition distributions. Durations
are represented in nanoseconds; observed values were multiples of one microsecond.
Executable size excludes shared libraries and debug sidecars, so it is not a
deployment-footprint comparison. Allocation counts and peak RSS were not measured.
The enclosing implementation source is still awaiting final-head review/CI;
these candidate observations are not a release acceptance claim.


A separate diagnostic run validated 20,000 calls per provider and collected
five-second native client/server stack samples. Probes with the same client
configuration negotiated TLS 1.3, ChaCha20-Poly1305 and ALPN `h2` for both servers.
In the Jai server's 3,974 main-thread observations, 2,191 ended in MbedTLS's
ChaCha20/Poly1305 routines, 1,332 in `kevent` and 103 in `memmove`. These are
sampled wall stacks including idle waits, not CPU-utilization percentages. Go's
traces used its ARM64 vector `xorKeyStreamVX` path; its different OS-thread
population prevents comparing percentages with the single Jai thread.

The traces identify existing TLS-record crypto as a major cost center for this
workload. They do not assign every part of the end-to-end difference to crypto.
The gRPC engine shares the repository's MbedTLS-backed transport; those provider
costs are included in the application task. The scored table uses unsampled runs.

## Refreshing the comparison

Rebuild both adapters and oracle tools from the recorded source pins. Run the same
complete inventories with verified TLS and preserve case IDs, failures, explicit
configuration and compiler/schema/tool hashes. Rerun the earlier grpcio matrix after
runtime or generator changes. Replace candidate observations with the exact final
source receipt before release acceptance. When benchmarking, keep payloads, API
work, TLS policy, build optimization and channel lifetime equivalent, and retain
unfavorable results and missing comparisons. Update this document together with
runtime profile or dependency changes.

The measured task harness is `tools/grpc/conformance/compare-unary.py`. It accepts
prebuilt optimized `--client`, `--jai-server` and `--go-server` executables,
`--scratch` and `--output` directories. Use the same compiler/native provider to
build the Jai client and server with `-O2`; build the pinned Go peer with its
default optimizations. The server fixture receives `GRPC_PROTO_ROOT`; the client
also receives `TYPED :: false`.

The harness alternates provider order across four repetitions. Each repetition
runs 1,010 calls on one channel, retains every timing and excludes the first ten
warmups symmetrically from the scored 1,000. It uses the same optimized Jai client
for both servers because canonical C++ soak output truncates submillisecond
latencies to zero. The score includes client allocation, codec, TLS/network,
response validation and call cleanup; per-call logging is outside the timer.
It is an equivalent application task comparison, not isolated server CPU time.
Both providers verify the synthetic CA and ALPN `h2`. Go uses `GOMAXPROCS=2`;
Jai uses its single scheduler thread. Report the actual provider/build/hardware
receipts with the data. Nearest recorded quantiles use `round((n-1)*p)`; sample
ranges across repetitions show variability. Allocation count and peak RSS remain
unavailable until measured directly.

Example after building the optimized providers and acquiring a quiet window:

```sh
PYTHONPATH=tools python tools/grpc/conformance/compare-unary.py \
  --client target/benchmark-client --jai-server target/benchmark-server \
  --go-server target/test-service-peer --scratch target/grpc-comparison \
  --output target/grpc-comparison-results
```
