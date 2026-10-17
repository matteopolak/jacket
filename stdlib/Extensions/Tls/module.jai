// Tls: TLS 1.2 and 1.3 over Io readers and writers, on a vendored mbedTLS (tools/native-libs.json).
// A jaic extension, not official Jai: other Jai compilers have no such module, and it may change
// between jaic releases. See docs/extensions/tls.md.
//
//     Tls :: #import "Extensions/Tls";
//
//     config: Tls.Tls_Config;                       // a client that trusts the system's roots
//     error := Tls.tls_config_init(*config);
//
//     tcp, io_error := Io.connect("example.com:443");
//     stream, error := Tls.tls_connect(*config, Io.reader_of(tcp), Io.writer_of(tcp), "example.com");
//
//     Tls.write(stream, "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n");
//     count, io_error := Tls.read(stream, buffer);
//     Tls.close(stream);
//
// A stream is also an `Io.Reader` and `Io.Writer` (`reader_of`, `writer_of`), so code written for
// those, like Http, runs over TLS unchanged. The handshake and every read and write are blocking
// calls on a fiber, inside `Io.io_run`. Servers use `tls_accept` on an accepted connection.
//
// mbedTLS needs the C library only; it is built into an archive by tools/build_native_libs.py and
// linked into executables, so nothing has to be installed. WebAssembly has no sockets and no fibers.

#assert OS != .WASM "Extensions/Tls runs on fibers over Io, and WebAssembly has neither";

#load "error.jai";
#load "config.jai";
#load "stream.jai";

#scope_module

#load "bindings.jai";

#if OS == .MACOS {
    #load "trust_macos.jai";
} else #if OS == .WINDOWS {
    #load "trust_windows.jai";
}

#import "Basic";
Io :: #import "Extensions/Io";
