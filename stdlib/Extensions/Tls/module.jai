// Tls: TLS 1.2 and 1.3 over Io readers and writers, on a vendored mbedTLS (tools/native/native-libs.json).
// A jaic extension, not official Jai: other Jai compilers have no such module, and it may change
// between jaic releases. See docs/extensions/tls.md.
//
// Fill in a `Tls_Config`, call `tls_config_init` once, then wrap an accepted or connected TCP stream with `tls_connect`
// (client) or `tls_accept` (server). A stream is also an `Io.Reader` and `Io.Writer` (`reader_of`, `writer_of`), so code
// written for those, like Http, runs over TLS unchanged. The handshake and every read and write are blocking calls on a
// fiber, inside `Io.io_run`: a deadline (`Io.with_deadline`) limits them and `Io.cancel` ends them, and
// one stream belongs to one scheduler thread.
//
// Every call reports failure in its result and never stops the program: the handshake calls give a `Tls_Error` whose
// `kind` says what went wrong (`error_text` makes a line of it), while `read` and `write` give an `Io.Io_Error` and
// `last_error(stream)` the TLS reason. A client verifies the server against the system's trusted roots unless
// `roots_pem` gives others; names are matched against the certificate's subject alternative names. Memory: a stream is
// allocated with `context.allocator` by `tls_connect` or `tls_accept` and released by `close`; the config's parsed
// certificates are released by `tls_config_free`. mbedTLS is built into an archive by tools/native/build_native_libs.py
// and linked into the executable, so nothing has to be installed. WebAssembly has no sockets and no fibers.
//
// The example runs a client and a server on loopback. The certificates and the key are throwaway ones made for
// tests, and the certificate is valid for `localhost`. Never use them for anything else.
//
// ```jai,skip_on(wasm)
// Io :: #import "Extensions/Io";
//
// ROOT :: #string END
// -----BEGIN CERTIFICATE-----
// MIIBXzCCAQWgAwIBAgIBATAKBggqhkjOPQQDAjAWMRQwEgYDVQQDDAtKYWkgVGVz
// dCBjYTAgFw0yMDAxMDEwMDAwMDBaGA8yMTIwMDEwMTAwMDAwMFowFjEUMBIGA1UE
// AwwLSmFpIFRlc3QgY2EwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQD1LxitDVO
// QKMsMsgSpdi5JldYih52cephbw9hiejFcmOH3zQJAkGrcEwyH4CzLFJKjhUKaT54
// 4UDSgerXHZBpo0IwQDAPBgNVHRMBAf8EBTADAQH/MA4GA1UdDwEB/wQEAwIBBjAd
// BgNVHQ4EFgQUunX0563ngeQ6w8JsMUiqadFmd6YwCgYIKoZIzj0EAwIDSAAwRQIh
// AI1Q4QwolbUZQxGjLzBltR4I6BPZLStohfpAiI20nwPRAiAjsRP061IzprBMR5JF
// WnaWEW4Nf8JpkkFzPPaEp947tQ==
// -----END CERTIFICATE-----
// END
//
// CERTIFICATE :: #string END
// -----BEGIN CERTIFICATE-----
// MIIBzDCCAXKgAwIBAgIBAzAKBggqhkjOPQQDAjAWMRQwEgYDVQQDDAtKYWkgVGVz
// dCBjYTAgFw0yMDAxMDEwMDAwMDBaGA8yMTIwMDEwMTAwMDAwMFowFjEUMBIGA1UE
// AwwLc2VydmVyLnRlc3QwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQC5kvwaJTy
// O8qTdjNn9OgWvQ+QmLQDQDdzaUggjqJiooWHhj4peLTZFKWJHzwgZK3TRCnK9PXw
// VYwzFQ5BLKq/o4GuMIGrMAkGA1UdEwQCMAAwDgYDVR0PAQH/BAQDAgWgMBMGA1Ud
// JQQMMAoGCCsGAQUFBwMBMDkGA1UdEQQyMDCCCWxvY2FsaG9zdIILc2VydmVyLnRl
// c3SHBH8AAAGHEAAAAAAAAAAAAAAAAAAAAAEwHQYDVR0OBBYEFIDVLb76eRrc3ond
// VlB/e95+HlPnMB8GA1UdIwQYMBaAFLp19Oet54HkOsPCbDFIqmnRZnemMAoGCCqG
// SM49BAMCA0gAMEUCIE3wJvrmLnn+L4DdTxS3SZyHLO5z/yT/dh7hyz1vGv5oAiEA
// v9a4lUdydpygfLwCjF7DCb8AD9qcpTQOAj8cFauUUCs=
// -----END CERTIFICATE-----
// END
//
// KEY :: #string END
// -----BEGIN EC PRIVATE KEY-----
// MHcCAQEEILAaHg0fuEMf2UFJprp01LKXf54EQN3Xb7zkKzdoIxK1oAoGCCqGSM49
// AwEHoUQDQgAEAuZL8GiU8jvKk3YzZ/ToFr0PkJi0A0A3c2lIII6iYqKFh4Y+KXi0
// 2RSliR88IGSt00QpyvT18FWMMxUOQSyqvw==
// -----END EC PRIVATE KEY-----
// END
//
// Echo :: struct {
//     listener: Io.Tcp_Listener;
//     config: *Tls_Config;
// }
//
// // The server: accept one connection, answer a read with the same bytes, then close.
// echo_once :: (job: *Echo) {
//     tcp, io_error := Io.accept(job.listener);
//     assert(io_error == .NONE);
//
//     stream, error := tls_accept(job.config, Io.reader_of(tcp), Io.writer_of(tcp));
//
//     if error.kind != .NONE {
//         Io.close(tcp);
//         return;
//     }
//
//     buffer: [64] u8;
//     count, read_error := read(stream, buffer);
//
//     if read_error == .NONE write(stream, string.{ count, buffer.data });
//
//     close(stream);
// }
//
// main :: () {
//     Io.io_run(() {
//         server: Tls_Config;
//         server.certificate_chain_pem = CERTIFICATE;
//         server.private_key_pem = KEY;
//         assert(tls_config_init(*server).kind == .NONE);
//         defer tls_config_free(*server);
//
//         // A client trusts the system's roots unless told otherwise; this one trusts only the test authority that signed the certificate.
//         client: Tls_Config;
//         client.roots_pem = ROOT;
//         assert(tls_config_init(*client).kind == .NONE);
//         defer tls_config_free(*client);
//
//         listener, listen_error := Io.tcp_listen("127.0.0.1:0");
//         assert(listen_error == .NONE);
//         defer Io.close(listener);
//
//         job := Echo.{ listener = listener, config = *server };
//         Io.spawn(echo_once, *job);
//
//         address, _ := Io.local_address(listener);
//         tcp, connect_error := Io.connect(address);
//         assert(connect_error == .NONE);
//
//         // The name is sent as SNI and checked against the certificate.
//         stream, error := tls_connect(*client, Io.reader_of(tcp), Io.writer_of(tcp), "localhost");
//
//         // On an error nothing is closed: the TCP connection is still the caller's.
//         assert(error.kind == .NONE, "%", error_text(error));
//         defer close(stream);
//
//         assert(version(stream) == "TLSv1.3");
//
//         write(stream, "ping");
//
//         buffer: [64] u8;
//         count, read_error := read(stream, buffer);
//
//         assert(read_error == .NONE && string.{ count, buffer.data } == "ping");
//     });
// }
// ```

#assert OS != .WASM "Extensions/Tls runs on fibers over Io, and WebAssembly has neither";

#load "error.jai";
#load "config.jai";
#load "stream.jai";
#load "names.jai";

#scope_module

#load "bindings.jai";

#if OS == .MACOS {
    #load "trust_macos.jai";
} else #if OS == .WINDOWS {
    #load "trust_windows.jai";
}

#import "Basic";
Io :: #import "Extensions/Io";
