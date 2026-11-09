# Tls

TLS 1.2 and 1.3 for fibers on a vendored mbedTLS (LTS). It wraps any `Io.Reader`/`Io.Writer` pair (a TCP stream) in a client or server connection whose reader and writer are again an `Io.Reader`/`Io.Writer`, so the [HTTP client and server](http-client.md) run over it unchanged. Not on wasm. Shared rules: [README](README.md#conventions-of-io-tls-http-and-json).

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

- `tls_config_init` parses PEM and builds the mbedTLS configs once; the config is then shared read-only by all threads. `tls_connect` (SNI plus certificate name check) and `tls_accept` return `*Tls_Stream, Tls_Error`. `Tls_Error.kind` is `IO`, `CONFIG`, `HANDSHAKE`, `VERIFY`, `PROTOCOL`, `TRUNCATED` or `CLOSED`; `alert` holds a fatal alert the peer sent.
- No mbedTLS type or `WANT_READ` reaches the user. A handshake is an ordinary blocking call: `Io.with_deadline` bounds it, `Io.cancel` ends it. `close` sends close_notify and frees the stream; call it once. `read` returns 0 only after the peer's close_notify; a connection ending without one is `.CONNECTION_RESET` with `last_error(stream).kind == .TRUNCATED`, and Http decides what that means for a message.
- **The library never waits for the network.** The receive callback makes the fiber wait outside the library (`Io.wait_readable`); the send callback only appends ciphertext to the stream's `out` buffer, which `flush` writes under a separate `Fiber_Mutex` after each library call. No fiber holds the library lock while waiting on the peer, so both ends can write a megabyte into small socket buffers and still read each other (an earlier design parked inside the send callback and deadlocked on IOCP; `test_duplex_with_small_socket_buffers`). One fiber can read while another writes.
- Writes batch up to 64 KiB and reads look ahead up to 64 KiB, about 15x fewer system calls. The cipher is the limit (`benches/bench.jai`).
- **ALPN.** A client offering protocols against a server with a list and nothing in common fails the handshake. The HTTP server accepts none, `http/1.1` or `h2` (`h2` must be in its `alpn`).
- **Trust.** With `roots_pem` set, mbedTLS checks the chain and name against exactly those on every OS. Otherwise: on Unix, the first readable bundle among `$SSL_CERT_FILE` and the usual `/etc/ssl` and `/etc/pki` paths plus `$SSL_CERT_DIR`; on macOS, `Security.framework` judges the chain after the handshake; on Windows, CryptoAPI does (no revocation check). The OS stores aren't read into mbedTLS because they don't expose what the system trusts, so that check runs on the scheduler thread and may block briefly. `verify_proc` hooks your own chain check; `verify = false` skips every check. Names match against DNS/IP SANs, never the CN alone; `verify_name = false` still checks the chain. `crl_pem` supplies revocation lists (the server's certificate needs one from its issuer). `root_must_be_self_signed` makes the chain end in a self-signed root.
- **Peer certificate.** `peer_certificate_der(stream)` gives the leaf's DER; `peer_certificate_signature_hash(stream)` gives its signature hash name (used by SCRAM-PLUS in [Postgres](postgres.md)); `certificate_names` lists SANs and CN for callers that match names themselves.
- **Memory.** About 34 KiB per stream: a 16 KiB input buffer, an 8 KiB output buffer (`MBEDTLS_SSL_OUT_CONTENT_LEN`, which also bounds the largest handshake message, so a certificate chain over about 8 KiB can't be served), and the library and stream contexts.
- **Threads.** One mbedTLS context per connection, used only by its scheduler thread, and one shared immutable `Tls_Config`. mbedTLS is built with `MBEDTLS_THREADING_C` because TLS 1.3, PSA key slots and the RNG are global.
- Limits: TLS 1.2/1.3 only, no renegotiation, one certificate per config, no session resumption, no OCSP, no RSASSA-PSS-keyed certificates. `key_log_proc` emits NSS key log lines for decrypting captures.
- **Build.** `tools/native/native-libs.json` pins the tarball and builds one merged `mbedtls` library with `tools/native/mbedtls/jai_mbedtls_config.h` ([native-libs](../tools/native-libs.md)). `libmbedtls.a` must be in the native-libs directory (release archives ship it). Windows links `bcrypt`.

## How to change it

- Private keys: PEM or DER (PKCS#8, RSA, EC, encrypted PKCS#8 or OpenSSL-style encrypted PEM). A wrong or missing password is `.CONFIG` with `wrong_key_password` set.
- Explicit-parameter EC keys (LibreSSL writes them) are handled by a patch to mbedTLS's `pkparse.c` (`prepare_mbedtls_pkparse` in `tools/native/build_native_libs.py`). A bump of the mbedTLS pin makes the patch fail loudly if the code moved; recheck it then.
- Another mbedTLS function: declare it in `bindings.jai` (opaque `*void` objects). A `static inline` function or struct field goes through `tools/native/mbedtls/jai_mbedtls.c`.
- SNI selection, tickets, OCSP stapling, resumption: `build_server`/`build_client` in `config.jai`; `sni_callback` in `stream.jai` already sees the name first. ALPN-to-protocol choice for HTTP: `choose_protocol` in `Http/server.jai`.
- Library options (suites, record sizes, versions): edit the config header, then `PYTHONPATH=tools python3 tools/native/build_native_libs.py mbedtls`.
- Build-time knob: `MBEDTLS_SSL_OUT_CONTENT_LEN` (8192). Environment: `SSL_CERT_FILE`, `SSL_CERT_DIR`.
- Tests: `stdlib/Extensions/Tls/tests/` (`tls.jai` runs per backend natively and under `-interp`; `interop.jai` talks to `openssl s_client`/`s_server` and is skipped without OpenSSL 3 or on Windows). Test certificates are string constants in `tests/fixtures/certs.jai` regenerated with `fixtures/make_certs.sh`; they are valid 2020 to 2120 and for tests only.

Run benchmarks with `jacket bench --path stdlib/Extensions/Tls`. Cases live in `benches/`; setup precedes `reset_timer`, and inputs/results use `black_box`.
The TCP, TLS upload/download and handshake cases start their own loopback peer using the committed test certificates.
