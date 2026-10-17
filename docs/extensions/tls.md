# Tls (`Extensions/Tls`)

Shared rules of Io, Tls, Http and Json (pointers or values, `close(x, now)`, errors, who frees, milliseconds): [conventions](extensions-conventions.md).

## What it is

TLS 1.2 and 1.3 for fibers, on a vendored [mbedTLS](https://github.com/Mbed-TLS/mbedtls) 3.6.7 (LTS; Apache-2.0 OR GPL-2.0-or-later, the licence file ships with the libraries). `#import "Extensions/Tls"` wraps any `Io.Reader`/`Io.Writer` pair, such as a TCP stream, in a TLS client or server connection whose reader and writer are again an `Io.Reader` and `Io.Writer`, so the [HTTP client and server](http-client.md) run over it unchanged. A jaic extension, not available on wasm (a compile-time `#assert` says why).

```jai
Tls :: #import "Extensions/Tls";

main :: () {
    Io.io_run(() {
        config: Tls.Tls_Config;                                 // a client that trusts the system's roots
        config.alpn = string.["http/1.1"];

        error := Tls.tls_config_init(*config);
        if error.kind != .NONE { print("%\n", Tls.error_text(error)); return; }

        tcp, io_error := Io.connect("example.com:443");
        if io_error != .NONE return;

        stream, tls_error := Tls.tls_connect(*config, Io.reader_of(tcp), Io.writer_of(tcp), "example.com");
        if tls_error.kind != .NONE { print("%\n", Tls.error_text(tls_error)); Io.close(tcp); return; }
        defer Tls.close(stream);

        Tls.write(stream, "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n");

        reply, _ := Io.read_all(Tls.reader_of(stream));
        print("%\n", string.{ reply.count, reply.data });
    });
}
```

## How it works

**API.**

| Name | |
| --- | --- |
| `Tls_Config` | `certificate_chain_pem`, `private_key_pem` (a server's, or a client's own), `roots_pem` (trust anchors; empty means the system's), `alpn` (names, most preferred first), `verify` (client: check the server, default true), `client_auth` (server: require and check a client certificate against the roots), `min_version`/`max_version` (`.TLS_1_2`, `.TLS_1_3`), `verify_proc`/`verify_data` (your own chain check), and `native`, which `tls_config_init` fills in |
| `tls_config_init(*config) -> Tls_Error` | parses the PEM and builds the mbedTLS configurations, once; then the config is shared read-only by every connection on every thread |
| `tls_config_free(*config)` | after the streams made from it are closed |
| `tls_connect(*config, reader, writer, server_name) -> *Tls_Stream, Tls_Error` | the client handshake; `server_name` is sent as SNI and checked against the certificate |
| `tls_accept(*config, reader, writer) -> *Tls_Stream, Tls_Error` | the server handshake; the config needs a certificate and key (`can_accept(*config)` says) |
| `read`, `write_some`, `write`, `wait_readable`, `close(stream, now := false)` | direct use; `reader_of(stream)` and `writer_of(stream)` make the `Io.Reader`/`Io.Writer` (the reader has a `wait_proc` when the reader underneath has one) |
| `ended_without_close_notify(reader)` | whether `reader` is a TLS stream's reader whose connection ended without close_notify (`last_error(stream).kind == .TRUNCATED`); for protocols that frame their own messages, like HTTP |
| `alpn_protocol(stream)`, `server_name(stream)`, `version(stream)`, `cipher_suite(stream)`, `last_error(stream)` | what was negotiated; the SNI name a client sent; `"TLSv1.3"`; mbedTLS's name of the cipher suite (`"TLS1-3-CHACHA20-POLY1305-SHA256"`); why the last read or write failed |
| `Tls_Error` | `kind` (`IO`, `CONFIG`, `HANDSHAKE`, `VERIFY`, `PROTOCOL`, `TRUNCATED`, `CLOSED`), `io` (the `Io_Error` of a failed or timed-out connection), `code` (mbedTLS's own error number), `verify_flags`, and a sentence for people (`detail(*error)`); `error_text(error)` makes one line. Both return a copy in temporary storage, never a pointer into the error, so they stay valid when the error is copied |

No mbedTLS type or `WANT_READ`/`WANT_WRITE` reaches the user. A handshake is an ordinary blocking call on a fiber: `Io.with_deadline` limits it (`kind == .IO`, `io == .TIMED_OUT`), `Io.cancel` ends it (`.CANCELED`). `tls_connect` and `tls_accept` own the connection only on success; on an error the caller closes it. `close` sends close_notify (not with `now`, which waits for nothing), closes the connection under it and frees the stream, so call it once; a fiber still waiting on the stream is woken with `.CLOSED` and the memory goes when it has left. `read` returns 0 only after the peer's close_notify; a connection that ends without it is `.CONNECTION_RESET` with `last_error(stream).kind == .TRUNCATED` (the data may be cut off, which HTTP framing then notices too). The Tls layer keeps that strict on purpose; [Http](http-client.md) decides what it means for a message (a read-until-close body over TLS ends normally there, as in browsers and Python's `ssl`).

**The library never waits for the network.** mbedTLS calls our receive callback, which has the fiber wait *outside* the library when no data is known to be there (`Io.wait_readable`, then the library is asked again), and our send callback, which only appends the ciphertext to a buffer of the stream (`out`) and takes it at once. After every call into the library (handshake, read, write, close_notify), with the library's lock released, `flush` writes `out` to the connection under a separate `Fiber_Mutex`, so the bytes of two flushes never interleave and arrive in the order the library produced them. A `write` returns when its bytes are flushed, so a slow peer holds the writer (backpressure) and a writer cannot pile up output; a read that produced an alert or a key update flushes too, but leaves the work to a flush already running instead of waiting for it. Because no fiber holds the library's lock while it waits for the peer, a fiber that is waiting to read is never blocked behind a writer that is waiting for the socket: both ends can write a megabyte at once into sockets of a few KiB and still read each other (the first design parked the sender inside the send callback and deadlocked on IOCP; `test_duplex_with_small_socket_buffers` shrinks the socket buffers to show it). One fiber can read while another writes (HTTP/2 needs that). A source that cannot wait (an in-memory pipe) is read directly, parking inside the library after the pending output is flushed. A timeout of the wait outside leaves the stream usable; an `Io` failure while flushing breaks it (`broken` is the `Io_Error` every later call returns), and so does a failure while the stream is being closed (reported as `.CLOSED`). The callbacks are `#c_call` and run on the fiber's own context, saved before each call. `close` sends close_notify with a 1 second deadline and does not queue behind a flush in progress.

**ALPN.** `alpn` is a list for the library. A client that offers protocols and a server that has a list, with none in common, fail the handshake (`.HANDSHAKE`); a peer with no list uses none. The HTTP server accepts no ALPN or `http/1.1`; `h2` is where a future HTTP/2 path would branch (see `choose_protocol` in `Http/server.jai`).

**Trust.**

| | Where the client's trust comes from |
| --- | --- |
| `roots_pem` given | mbedTLS verifies the chain and name against exactly those, on every OS |
| Linux and other Unix | the first readable bundle of `$SSL_CERT_FILE`, `/etc/ssl/certs/ca-certificates.crt`, `/etc/pki/tls/certs/ca-bundle.crt`, `/etc/ssl/ca-bundle.pem`, `/etc/pki/tls/cacert.pem`, `/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem`, `/etc/ssl/cert.pem`, plus every file in the directories of `$SSL_CERT_DIR` (colon-separated). Parsed once per process, on first use, and shared. No bundle at all is a `.CONFIG` error from `tls_connect` ("no system root certificates were found") |
| macOS | `Security.framework` (`SecPolicyCreateSSL` with the host name, `SecTrustEvaluateWithError`, macOS 10.14 or later) judges the chain after the handshake; mbedTLS's own check is off |
| Windows | CryptoAPI (`CertGetCertificateChain`, then `CertVerifyCertificateChainPolicy` with the SSL policy and the server name) judges the chain after the handshake; mbedTLS's own check is off. Revocation is not checked |

The root stores of macOS and Windows are not read into mbedTLS on purpose: `SecTrustCopyAnchorCertificates` does not return what the system trusts, per-user and per-domain trust settings live elsewhere, and Windows fetches roots on demand (AuthRoot) that are not in `ROOT` until needed. Go makes the same choice. The cost is that the check runs on the scheduler's thread and may block it for a moment while the OS fetches an intermediate or a root. The macOS and Windows code (`trust_macos.jai`, `trust_windows.jai`, about 100 lines each) is type-checked for macOS and Windows (x64 and arm64) here and has not been run: the tests that use the test CA go through mbedTLS everywhere, and only an `https://` request to a public site exercises the system paths. `verify_proc` is the same mechanism with your own procedure: the chain in DER and the server name in, `""` (accept) or a reason out; it replaces the library's check, and the tests use it to cover the post-handshake path on Linux. A config with `verify = false` skips every check.

Host names are checked by mbedTLS against the certificate's DNS and IP subject alternative names (never the common name alone) on Linux and with `roots_pem`, and by the OS policy on macOS and Windows.

**Memory per connection.** Measured with `Tls_Stream` pairs on loopback in one process (400 connections, RSS growth divided by 800 streams, Linux x64, ECDSA P-256 certificate):

| `MBEDTLS_SSL_OUT_CONTENT_LEN` | per TLS stream | throughput of one stream, both ends on one thread |
| --- | --- | --- |
| 4096 | 29.9 KiB | 115 MiB/s |
| **8192 (shipped)** | **34.2 KiB** | 120 MiB/s |
| 16384 (the library's default) | 42.3 KiB | 128 MiB/s |

The input buffer is fixed at 16 KiB (peers send full records), the library's `ssl_context` is 568 bytes and our `Tls_Stream` 1136 bytes (it holds a copy of the fiber's `Context`), and the rest is allocator slack and the peer certificate the client keeps. The output buffer also bounds the largest handshake message mbedTLS can send, because it does not fragment them: a server certificate chain that does not fit in 8 KiB cannot be served (a chain of two ECDSA certificates is about 1 KiB, two RSA-4096 ones about 3 KiB). 4096 was rejected for that reason; the throughput cost of 8 KiB records against 16 KiB is about 6%. The header is `tools/native/mbedtls/jai_mbedtls_config.h`. An idle TLS connection therefore costs about 34 KiB more than a plain one, since the library's buffers live as long as the connection (it has no way to drop them while idle).

<a id="throughput"></a>
**Throughput.** `bench/bench.jai` measures one stream in each direction and handshakes per second, with plain TCP for reference (`jaic stdlib/Extensions/Tls/bench/bench.jai -O2 -exe /tmp/tls-bench`; arguments: MiB per transfer, rounds, handshakes). Both ends run on one thread, so a number is encryption and decryption of every byte plus the scheduler. Medians of 9 interleaved runs of 3 transfers of 128 MiB, 4-core Xeon VM, Linux x64, io_uring, the stock native libraries (the stream negotiates `TLS1-3-CHACHA20-POLY1305-SHA256`, the library's first choice):

| | client to server | server to client | handshakes/s |
| --- | --- | --- | --- |
| before | 112.5 MiB/s | 109.7 MiB/s | 189 |
| + several records in one send | 121.3 | 120.1 | 191 |
| + reading ahead (shipped) | 123.2 | 123.2 | 191 |

Two things cost operations on the connection rather than cipher time. Every 8 KiB record was its own send, and every record read cost a wait and a read (153,000 `io_uring_enter` calls for 256 MiB). A write now lets the library make records into `out` until about 64 KiB are there and sends them at once (`BATCH_BYTES`), and a read takes up to 64 KiB from the connection (16 KiB for the first, doubling while reads come back full) into a buffer that exists until the library has taken it (`ahead`; a quiet connection holds none). The same transfer needs about 10,000 calls. The gain is 10% here because the cipher is the limit: mbedTLS's ChaCha20-Poly1305 is C without vector instructions (about 300 MiB/s for one direction of one core, measured with `mbedtls_chachapoly_encrypt_and_tag`), and its AES-GCM does one 16-byte block at a time (about 260 MiB/s with AES-NI), so AES-GCM is no faster here. A faster GCM is being evaluated separately. AES-NI and PCLMULQDQ are in the build (`MBEDTLS_AESNI_C`, runtime detection, `-O2`); the record size no longer matters much either (it cost 6% before).

**Threads.** One mbedTLS context per connection, used only by the scheduler thread that made it; one `Tls_Config` shared by all threads and immutable after `tls_config_init`. mbedTLS is built with `MBEDTLS_THREADING_C` (pthreads; a small `CRITICAL_SECTION` shim in `tools/native/mbedtls/jai_mbedtls.c` on Windows) because TLS 1.3, PSA's key slots and the random generator are global. Startup (`psa_crypto_init` and the Windows mutex hooks) runs once per process under `pthread_once`/`InitOnceExecuteOnce`. The system roots are loaded the same way.

**Build.** `tools/native-libs.json` pins the release tarball (sha256 `a7e8bcbe...`) and builds one merged `mbedtls` library from its 107 `library/*.c` units (everything but `net_sockets.c`, which needs sockets of its own, and `psa_its_file.c`, persistent keys) plus the glue `tools/native/mbedtls/jai_mbedtls.c`, with `tools/native/mbedtls/jai_mbedtls_config.h` as the user configuration. See [native-libs](../tools/native-libs.md). A program that uses Tls, or Http (whose client and server call it), links about 1 MB of it into the executable; `libmbedtls.a` must exist in the native-libs directory (release archives ship it). On Windows `bcrypt` is linked for the random generator, on Linux nothing beyond libc.

## How to change it

- Another mbedTLS function: add a declaration to `bindings.jai` (opaque `*void` for its objects) and call it where it belongs; a function that is `static inline` in the headers, or a structure field, goes through a small function in `tools/native/mbedtls/jai_mbedtls.c`.
- Server certificate by SNI, session tickets on the server, OCSP stapling or client session resumption: `build_server` and `build_client` in `config.jai` set up the two mbedTLS configurations; `sni_callback` in `stream.jai` already sees the name before a certificate is chosen.
- HTTP/2: `choose_protocol` in `Http/server.jai` decides which loop a connection runs after the handshake.
- The library's options (cipher suites, record sizes, TLS versions, features) are in `jai_mbedtls_config.h`; its defaults are mbedTLS 3.6's. Rebuild with `python3 tools/build_native_libs.py mbedtls`.

## Configuration

`Tls_Config` as above. Build-time: `MBEDTLS_SSL_OUT_CONTENT_LEN` (8192). Environment: `SSL_CERT_FILE`, `SSL_CERT_DIR` (Linux and other Unix, system roots only).

## Dependencies

[Io](io-streams.md) (`Reader`, `Writer`, `wait_readable`, deadlines, `Fiber_Mutex`), the vendored mbedTLS, and on macOS `Security.framework` and `CoreFoundation`, on Windows `crypt32` and `bcrypt`.

## Limits

TLS 1.2 and 1.3 only (no DTLS, no TLS 1.0/1.1, no SSL 3), no renegotiation. One certificate per config. The client does not resume sessions (it reads and ignores the server's tickets), the server issues no tickets. No OCSP, no CRLs, no certificate transparency; the macOS and Windows verifiers do what the OS does, the Linux path checks the chain, validity dates, name and key usage only. Encrypted private keys are not read. Two fibers reading (or writing) the same stream take turns record by record. Windows chain building may fetch from the network and block the thread.

## Tests

`stdlib/Extensions/Tls/tests/`, run once per backend (`tls.jai` sets `JAI_IO_BACKEND` to each) natively and under `jaic -interp`; the certificates are string constants in `tests/data/certs.jai` (generated by `make_certs.sh`), so a program reads no file and cross builds work:

- `tls.jai` (with `lib/suite.jai`): handshakes with EC and RSA certificates under TLS 1.2, 1.3 and both, SNI seen by the server, ALPN with a match, none in common and none offered, a version mismatch, 1 MB in both directions at once (twice, per version), writes of every awkward size (around a block, a record and a batch, read back in other pieces), close_notify and the end of the stream, a connection dropped without it, a record cut in the middle, a handshake under a deadline against a server that never answers (`TIMED_OUT`) and one that is canceled, verification failures (unknown CA, wrong name, an expired certificate, a name the certificate is not valid for, the system roots), `verify = false`, client certificates (accepted, missing, from an untrusted CA), a custom `verify_proc` (accepting and rejecting, with the chain and name it was given), `wait_readable` (idle timeout, bytes the library holds, the peer's end), a read and a write on one stream at once, `close` under a waiting reader, 60 concurrent connections, a handshake over an in-memory pipe, and config errors.
- `interop.jai`: `openssl s_client` against our server and our client against `openssl s_server -www`, on TLS 1.2 and 1.3, EC and RSA certificates, ALPN, client certificates and each AES-GCM and ChaCha20-Poly1305 suite offered alone, on loopback. Skipped (with a message) when there is no `openssl` command, and on Windows.
- `data/make_certs.sh` (`--jai-only` rewrites just `certs.jai`) regenerates the test CA, a server certificate (EC), an RSA one, one for another name, an expired one, one from a CA nobody trusts and a client certificate, with the `openssl` command. They are valid from 2020 to 2120 (so a machine with a wrong clock still accepts them), and the keys are committed: **they are for the tests only**.
- `bench/bench.jai`: the throughput of one stream and handshakes per second (see [throughput](#throughput)); not run by the tests.
- `stdlib/Extensions/Http/tests/https.jai`: the server and client over TLS (see [http-server](http-server.md#https) and [http-client](http-client.md#https)).

`tools/io_stress.py` runs `tls.jai` and `https.jai` among its default programs; `interop.jai` starts openssl processes on ports and is left out (it needs OpenSSL 3: LibreSSL, macOS's `openssl`, makes it skip).
