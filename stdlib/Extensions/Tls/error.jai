// What went wrong, in one value for everything the module does: a bad certificate in a config, a
// handshake that failed, a peer that is not who it says it is, a connection that broke.

Tls_Error_Kind :: enum u8 {
    NONE; // no error: `.{}`
    IO; // the connection under TLS failed or timed out: `io` says how (`.TIMED_OUT`, `.CANCELED`, ...)
    CONFIG; // the config cannot be used: a certificate or key that does not parse, no system roots, ...
    HANDSHAKE; // the handshake failed: the peers share no protocol version, cipher or ALPN protocol, or one sent a fatal alert
    VERIFY; // the peer's certificate was not accepted (`detail` says why)
    PROTOCOL; // a record that does not decrypt or parse, or an alert from the peer, on an open connection
    TRUNCATED; // the connection ended without the peer's close_notify: the data may be incomplete
    CLOSED; // the stream was closed
}

// `detail` is a sentence for people ("the certificate has expired"); `code` is mbedTLS's own negative
// error number (0 when the error did not come from it). Use `error_text` to print one.
Tls_Error :: struct {
    kind: Tls_Error_Kind;
    // For IO, and the closest `Io_Error` for the other kinds that end a read or write.
    io: Io.Io_Error;
    code: s32;
    // For VERIFY with mbedTLS's own verification: its result bits (`X509_BADCERT_*`), else 0.
    verify_flags: u32;
    detail_count: u8;
    detail_bytes: [191] u8;
}

// The detail as a string (it points into `error`, so keep the error where it is).
detail :: (error: *Tls_Error) -> string {
    return string.{ error.detail_count, error.detail_bytes.data };
}

// One line for logs and messages: "certificate verification failed: the certificate has expired".
error_text :: (error: Tls_Error) -> string {
    if error.kind == .NONE return "no error";

    text := detail(*error);

    what := ifx error.kind == .IO then "i/o error"
        else ifx error.kind == .CONFIG then "tls configuration"
        else ifx error.kind == .HANDSHAKE then "tls handshake failed"
        else ifx error.kind == .VERIFY then "certificate verification failed"
        else ifx error.kind == .PROTOCOL then "tls protocol error"
        else ifx error.kind == .TRUNCATED then "tls connection truncated"
        else "tls stream closed";

    if error.kind == .IO {
        if text.count return tprint("%: % (%)", what, error.io, text);

        return tprint("%: %", what, error.io);
    }

    if text.count return tprint("%: %", what, text);

    return what;
}

#scope_module

make_error :: (kind: Tls_Error_Kind, text: string, code := 0, io := Io.Io_Error.NONE) -> Tls_Error {
    error: Tls_Error;

    error.kind = kind;
    error.code = cast(s32) code;
    error.io = io;

    set_detail(*error, text);

    return error;
}

set_detail :: (error: *Tls_Error, text: string) {
    count := min(text.count, error.detail_bytes.count);

    if count > 0 memcpy(error.detail_bytes.data, text.data, count);

    error.detail_count = cast(u8) count;
}

// mbedTLS's description of one of its error numbers ("SSL - The peer notified us that the
// connection is going to be closed").
library_message :: (code: s32) -> string {
    buffer: [160] u8;

    mbedtls_strerror(code, buffer.data, buffer.count);

    length := 0;

    while length < buffer.count && buffer[length] != 0 length += 1;

    return copy_temporary_string(string.{ length, buffer.data });
}

from_library :: (kind: Tls_Error_Kind, code: s32) -> Tls_Error {
    return make_error(kind, library_message(code), code);
}

io_error :: (io: Io.Io_Error) -> Tls_Error {
    return make_error(.IO, "", 0, io);
}
