# Tls (`Extensions/Tls`)

Shared rules: [README](README.md#conventions-of-io-tls-http-and-json).

## What it is

TLS 1.2 and 1.3 for fibers on a vendored mbedTLS 3.6.7 LTS. It wraps any `Io.Reader`/`Io.Writer` pair (a TCP stream) in a client or server connection whose reader and writer are again an `Io.Reader`/`Io.Writer`, so the [HTTP client and server](http-client.md) run over it unchanged. Not available on wasm.

```jai
Tls :: #import "Extensions/Tls";

config: Tls.Tls_Config;                        // a client that trusts the system's roots
config.alpn = string.["http/1.1"];
error := Tls.tls_config_init(*config);
if error.kind != .NONE { print("%\n", Tls.error_text(error)); return; }

tcp, io_error := Io.connect("example.com:443");
stream, tls_error := Tls.tls_connect(*config, Io.reader_of(tcp), Io.writer_of(tcp), "example.com");
if tls_error.kind != .NONE { Io.close(tcp); return; }       // on error the caller closes the connection
defer Tls.close(stream);

Tls.write(stream, "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n");
reply, _ := Io.read_all(Tls.reader_of(stream));
```

## How it works

- **API.** `Tls_Config` (`certificate_chain_pem`, `private_key_pem`, `roots_pem` (empty: system roots), `alpn`, `verify` (default true), `client_auth`, `min_version`/`max_version`, `verify_proc`/`verify_data`). `tls_config_init` parses PEM and builds the mbedTLS configs once; the config is then shared read-only by all threads; `tls_config_free` after the streams close. `tls_connect(config, reader, writer, server_name)` (SNI plus certificate name check) and `tls_accept` (`can_accept(*config)`) return `*Tls_Stream, Tls_Error`. `read`, `write_some`, `write`, `wait_readable`, `close`, `reader_of`, `writer_of`; `alpn_protocol`, `server_name`, `version`, `cipher_suite`, `last_error`. `Tls_Error.kind`: `IO`, `CONFIG`, `HANDSHAKE`, `VERIFY`, `PROTOCOL`, `TRUNCATED`, `CLOSED`.
- No mbedTLS type or `WANT_READ` reaches the user. A handshake is an ordinary blocking call: `Io.with_deadline` bounds it (`kind == .IO`, `io == .TIMED_OUT`), `Io.cancel` ends it. `close` sends close_notify (1 s deadline; not with `now`), closes the connection under it and frees the stream: call once. `read` returns 0 only after the peer's close_notify; a connection ending without it is `.CONNECTION_RESET` with `last_error(stream).kind == .TRUNCATED` (Http decides what that means for a message; `ended_without_close_notify(reader)` helps).
- **The library never waits for the network.** The receive callback makes the fiber wait outside the library (`Io.wait_readable`, then asks again); the send callback only appends ciphertext to the stream's `out` buffer. After each library call, with its lock released, `flush` writes `out` under a separate `Fiber_Mutex`, so flushes never interleave. A `write` returns when flushed (backpressure). Because no fiber holds the library lock while waiting for the peer, both ends can write a megabyte into small socket buffers and still read each other (the first design parked inside the send callback and deadlocked on IOCP; `test_duplex_with_small_socket_buffers`). One fiber can read while another writes. An `Io` failure while flushing breaks the stream (`broken` is returned by every later call); a timeout of the outside wait does not. Callbacks are `#c_call` and run on the fiber's saved context.
- Writes batch up to 64 KiB (`BATCH_BYTES`) and reads take up to 64 KiB ahead (`ahead`, none held when quiet): about 15x fewer system calls, around 10% more throughput; the cipher (mbedTLS's C ChaCha20-Poly1305 and one-block AES-GCM, roughly 120 MiB/s per stream here) is the limit. `bench/bench.jai` measures it.
- **ALPN:** a client that offers protocols and a server with a list, none in common, fail the handshake. The HTTP server accepts none or `http/1.1`.
- **Trust.** `roots_pem` given: mbedTLS checks the chain and name against exactly those, on every OS. Linux/Unix: first readable bundle among `$SSL_CERT_FILE`, `/etc/ssl/certs/ca-certificates.crt`, `/etc/pki/tls/certs/ca-bundle.crt`, and others, plus files in `$SSL_CERT_DIR`; parsed once; none found is `.CONFIG` ("no system root certificates were found"). macOS: `Security.framework` judges the chain after the handshake. Windows: CryptoAPI judges it after the handshake (no revocation check). The OS stores are deliberately not read into mbedTLS (they do not expose what the system trusts, e.g. per-user settings, on-demand AuthRoot), so the check runs on the scheduler thread and may block briefly. `trust_macos.jai` and `trust_windows.jai` are type-checked but the real system paths are exercised only by an `https://` request to a public site. `verify_proc` is the same hook for your own chain check (DER chain and name in; `""` accepts). `verify = false` skips every check. Names are matched against DNS/IP SANs, never the CN alone.
- **Memory.** About 34 KiB per TLS stream, held for the connection's life: 16 KiB input buffer (fixed), 8 KiB output buffer (`MBEDTLS_SSL_OUT_CONTENT_LEN`; it also bounds the largest handshake message, so a certificate chain over about 8 KiB cannot be served; 4 KiB was rejected for that), the 568-byte library context and our 1136-byte `Tls_Stream`.
- **Threads.** One mbedTLS context per connection, used only by its scheduler thread; one shared immutable `Tls_Config`. mbedTLS is built with `MBEDTLS_THREADING_C` (pthreads; a `CRITICAL_SECTION` shim on Windows) because TLS 1.3, PSA key slots and the RNG are global. Startup runs once per process.
- **Build.** `tools/native-libs.json` pins the release tarball and builds one merged `mbedtls` library from the `library/*.c` units (minus `net_sockets.c`, `psa_its_file.c`) plus `tools/native/mbedtls/jai_mbedtls.c`, with `tools/native/mbedtls/jai_mbedtls_config.h` as configuration ([native-libs](../tools/native-libs.md)). A program using Tls or Http links about 1 MB of it; `libmbedtls.a` must be in the native-libs directory (release archives ship it). Windows links `bcrypt`.
- Limits: TLS 1.2/1.3 only, no renegotiation, one certificate per config, no session resumption or ticket issuing, no OCSP/CRL, encrypted private keys are not read, two fibers reading the same stream take turns record by record.

## How to change it

- **Another mbedTLS function:** declare it in `bindings.jai` (opaque `*void` objects). A `static inline` function or struct field goes through `tools/native/mbedtls/jai_mbedtls.c`.
- **SNI certificate selection, tickets, OCSP stapling, resumption:** `build_server`/`build_client` in `config.jai`; `sni_callback` in `stream.jai` already sees the name first.
- **HTTP/2:** `choose_protocol` in `Http/server.jai`.
- **Library options** (suites, record sizes, versions): `jai_mbedtls_config.h`, then `python3 tools/build_native_libs.py mbedtls`.

## Configuration

`Tls_Config` fields above. Build time: `MBEDTLS_SSL_OUT_CONTENT_LEN` (8192). Environment: `SSL_CERT_FILE`, `SSL_CERT_DIR`.

## Dependencies and tests

[Io](io-engine.md) (`Reader`, `Writer`, `wait_readable`, deadlines, `Fiber_Mutex`), vendored mbedTLS, `Security.framework`/`CoreFoundation` on macOS, `crypt32`/`bcrypt` on Windows. Tests in `stdlib/Extensions/Tls/tests/`, run per backend (`tls.jai` sets `JAI_IO_BACKEND`) natively and under `-interp`: `tls.jai` (handshakes, versions, SNI, ALPN, large and awkward-size transfers, close_notify and truncation, deadlines and cancel, verification failures, client certificates, `verify_proc`, concurrency, in-memory pipe, config errors) and `interop.jai` (against `openssl s_client`/`s_server`; skipped without OpenSSL 3 or on Windows). Test certificates are string constants in `tests/data/certs.jai`, regenerated with `data/make_certs.sh` (`--jai-only`); they are valid 2020 to 2120 and **for tests only**. `tools/io_stress.py` includes `tls.jai` and `https.jai`.
