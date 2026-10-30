// What went wrong, in one value for everything the module does: a bad certificate in a config, a
// handshake that failed, a peer that is not who it says it is, a connection that broke.
//
// `kind` of a `Tls_Error` says which; `.NONE` is success. Branch on the kind, then show `error_text(error)` to a person
// (the line lives in temporary storage) or read `detail(*error)`, `error.io`, `error.alert` and `error.verify_flags`
// for the specifics. Calls that return `Io.Io_Error` instead (`read`, `write`) leave the TLS reason in
// `last_error(stream)`.
//
// ```jai
// config: Tls_Config;
// config.certificate_chain_pem = "this is not a certificate";
// config.private_key_pem = "this is not a key";
//
// error := tls_config_init(*config);
// assert(error.kind == .CONFIG);
// assert(error_text(error).count > 0);
// ```
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

// The sentence for people ("the certificate has expired") is kept inside the error in `detail_bytes`;
// read it with `detail(*error)` or print the whole error with `error_text`. `code` is mbedTLS's own
// negative error number (0 when the error did not come from it).
Tls_Error :: struct {
    kind: Tls_Error_Kind;
    // For IO, and the closest `Io_Error` for the other kinds that end a read or write.
    io: Io.Io_Error;
    code: s32;
    // For VERIFY with mbedTLS's own verification: its result bits (`X509_BADCERT_*`), else 0.
    verify_flags: u32;
    // For a fatal alert the peer sent: its description number (RFC 8446, section 6; `alert_name`
    // spells it), else 0.
    alert: u8;
    // For CONFIG: the private key was not read because the password does not decrypt it (or was
    // needed and not given).
    wrong_key_password: bool;
    detail_count: u8;
    detail_bytes: [191] u8;
}

// The sentence behind an error ("the certificate has expired"), or "". It is a copy in temporary
// storage, so it stays valid when the error is copied or goes out of scope, until the temporary
// storage is reset; copy it with `copy_string` to keep it longer.
detail :: (error: *Tls_Error) -> string {
    if error.detail_count == 0 return "";

    return copy_temporary_string(string.{ error.detail_count, error.detail_bytes.data });
}

// The name of an alert description ("unknown_ca"), or "" for a number the module does not know.
alert_name :: (alert: u8) -> string {
    if alert == {
        case 0; return "close_notify";
        case 10; return "unexpected_message";
        case 20; return "bad_record_mac";
        case 21; return "decryption_failed";
        case 22; return "record_overflow";
        case 30; return "decompression_failure";
        case 40; return "handshake_failure";
        case 41; return "no_certificate";
        case 42; return "bad_certificate";
        case 43; return "unsupported_certificate";
        case 44; return "certificate_revoked";
        case 45; return "certificate_expired";
        case 46; return "certificate_unknown";
        case 47; return "illegal_parameter";
        case 48; return "unknown_ca";
        case 49; return "access_denied";
        case 50; return "decode_error";
        case 51; return "decrypt_error";
        case 60; return "export_restriction";
        case 70; return "protocol_version";
        case 71; return "insufficient_security";
        case 80; return "internal_error";
        case 86; return "inappropriate_fallback";
        case 90; return "user_canceled";
        case 100; return "no_renegotiation";
        case 109; return "missing_extension";
        case 110; return "unsupported_extension";
        case 111; return "certificate_unobtainable";
        case 112; return "unrecognized_name";
        case 113; return "bad_certificate_status_response";
        case 114; return "bad_certificate_hash_value";
        case 115; return "unknown_psk_identity";
        case 116; return "certificate_required";
        case 120; return "no_application_protocol";
    }

    return "";
}

// One line for logs and messages: "certificate verification failed: the certificate has expired".
// Like `detail`, the text is in temporary storage, never inside the error.
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

// `from_library` for a failure of a connection's call: a fatal alert from the peer says which.
from_stream :: (stream: *Tls_Stream, kind: Tls_Error_Kind, code: s32) -> Tls_Error {
    error := from_library(kind, code);

    if code == ERR_SSL_FATAL_ALERT_MESSAGE {
        alert := jai_tls_ssl_alert(stream.ssl);

        if alert > 0 {
            error.alert = cast(u8) alert;

            name := alert_name(error.alert);

            set_detail(*error, ifx name.count then tprint("the peer sent the fatal alert % (%)", name, alert) else tprint("the peer sent the fatal alert %", alert));
        }
    }

    return error;
}

io_error :: (io: Io.Io_Error) -> Tls_Error {
    return make_error(.IO, "", 0, io);
}
