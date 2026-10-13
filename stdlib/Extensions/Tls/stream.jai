// A TLS connection over any `Io.Reader` / `Io.Writer` pair (a TCP stream's, a pipe's, an Http
// `Link`'s). The handshake is a plain blocking call on the calling fiber: it waits through the Io
// calls under it, so `Io.with_deadline` limits it and `Io.cancel` ends it. After it, the stream is
// read and written like any other, through `reader_of` and `writer_of`, or directly with `read`,
// `write_some` and `write`.
//
//     stream, error := tls_connect(*config, Io.reader_of(tcp), Io.writer_of(tcp), "example.com");
//     if error.kind != .NONE { ...; Io.close(tcp); return; }
//     defer close(stream);
//
//     write(stream, request);
//     count, io_error := read(stream, buffer);
//
// One stream belongs to one scheduler thread. One fiber may read while another writes (every call
// into the TLS library is short and not interrupted by a switch to another fiber of the stream);
// two fibers reading, or two writing, take turns call by call, so their data interleaves.

Tls_Stream :: struct {
    reader: Io.Reader;
    writer: Io.Writer;

    ssl: *void;

    // The running fiber's context, saved before every call into the library, so the callbacks the
    // library makes (which are `#c_call`) can run Jai code and park the fiber.
    fiber_context: #Context;

    // Calls into the library take turns. A fiber never waits for the network while it holds this:
    // the receive callback asks the library to retry and the fiber waits outside, and the send
    // callback only appends to `out`.
    lock: Io.Fiber_Mutex;

    // What the library has produced and the connection has not taken yet, in order. Appended to
    // under `lock` by the send callback; `flush` writes it out under `flush_lock`, so the bytes
    // of two flushes never interleave. The library is never parked in a send: that is what lets
    // both ends of a connection write a lot at once and still read each other.
    out: [..] u8;
    flush_lock: Io.Fiber_Mutex;

    // What the Io call under the running library call returned, when it failed.
    io_error: Io.Io_Error;

    // Set once the connection is unusable (an Io call failed in the middle of a record, a record
    // did not decrypt, the peer cut the connection short): what every later call returns.
    broken: Io.Io_Error;
    // The error behind `broken`, for `last_error`.
    failure: Tls_Error;

    // The underlying reader says when it is readable: the library is never made to wait for input
    // inside a callback (it returns "want read" and the fiber waits outside), so a fiber waiting
    // for data does not block a writer of the same stream.
    can_wait: bool;
    ready: bool;

    established: bool;
    peer_closed: bool;
    closing: bool;
    // Calls in progress; the stream is freed after `close` when the last one has left.
    users: s64;

    // The server name a client asked for (SNI), on a server stream.
    peer_name_count: u8;
    peer_name_bytes: [255] u8;
}

// ---------------------------------------------------------------------------------------------
// Connecting

// Runs the client handshake over `reader` and `writer` and checks the server's certificate against
// `server_name` (which is also sent as the SNI name). On success the stream owns the connection:
// `close` closes it. On an error nothing is closed, the connection is the caller's to close.
tls_connect :: (config: *Tls_Config, reader: Io.Reader, writer: Io.Writer, server_name: string) -> *Tls_Stream, Tls_Error {
    native := config.native;

    if !native return null, make_error(.CONFIG, "tls_config_init was not called");
    if !native.client return null, native.client_error;

    if config.verify && server_name.count == 0 {
        return null, make_error(.CONFIG, "tls_connect needs the server's name to check its certificate against");
    }

    stream, error := open_stream(native.client, reader, writer);

    if error.kind != .NONE return null, error;

    if server_name.count > 0 {
        status := mbedtls_ssl_set_hostname(stream.ssl, temp_c_string(server_name));

        if status != 0 {
            release_stream(stream);

            return null, from_library(.CONFIG, status);
        }
    }

    error = run_handshake(stream);

    if error.kind == .NONE && native.external_verify {
        error = verify_externally(stream, native, server_name);
    }

    if error.kind != .NONE {
        release_stream(stream);

        return null, error;
    }

    stream.established = true;

    return stream, .{};
}

// Runs the server handshake. The config needs a certificate and key. On success the stream owns
// the connection; on an error the caller closes it.
tls_accept :: (config: *Tls_Config, reader: Io.Reader, writer: Io.Writer) -> *Tls_Stream, Tls_Error {
    native := config.native;

    if !native return null, make_error(.CONFIG, "tls_config_init was not called");
    if !native.server return null, make_error(.CONFIG, "the config has no certificate and key to accept connections with");

    stream, error := open_stream(native.server, reader, writer);

    if error.kind != .NONE return null, error;

    error = run_handshake(stream);

    if error.kind != .NONE {
        release_stream(stream);

        return null, error;
    }

    stream.established = true;

    return stream, .{};
}

// The ALPN protocol both sides chose ("" when none was).
alpn_protocol :: (stream: *Tls_Stream) -> string {
    return c_text(mbedtls_ssl_get_alpn_protocol(stream.ssl));
}

// The name a client asked for in its SNI extension, on a stream from `tls_accept` ("" if it sent
// none). It points into the stream.
server_name :: (stream: *Tls_Stream) -> string {
    return string.{ stream.peer_name_count, stream.peer_name_bytes.data };
}

// "TLSv1.3" or "TLSv1.2".
version :: (stream: *Tls_Stream) -> string {
    return c_text(mbedtls_ssl_get_version(stream.ssl));
}

// Why the last call that failed did (`.NONE` before any failure).
last_error :: (stream: *Tls_Stream) -> Tls_Error {
    return stream.failure;
}

// ---------------------------------------------------------------------------------------------
// Reading and writing

// Waits for data and reads what is there, up to `buffer.count` bytes. 0 is the end: the peer sent
// close_notify. A connection that ends without it is an error (`.CONNECTION_RESET`, and
// `last_error` says `.TRUNCATED`): the data may have been cut off.
read :: (stream: *Tls_Stream, buffer: [] u8) -> s64, Io.Io_Error {
    if buffer.count == 0 return 0, .NONE;
    if !enter(stream) return 0, .CLOSED;

    defer leave(stream);

    while true {
        if stream.broken != .NONE return 0, stream.broken;
        if stream.peer_closed return 0, .NONE;

        result, error := call_library(stream, .READ, buffer.data, cast(u64) buffer.count);

        if error != .NONE return 0, error;

        // An alert or a key update the read produced goes out, unless a write is already doing it.
        flush_error := flush(stream, false);

        if result > 0 return result, .NONE;
        if flush_error != .NONE && result != ERR_SSL_PEER_CLOSE_NOTIFY return 0, flush_error;

        if result == ERR_SSL_PEER_CLOSE_NOTIFY {
            stream.peer_closed = true;

            return 0, .NONE;
        }

        // The library reports a connection that ended without close_notify as a read of 0 bytes.
        if result == 0 return 0, fail(stream, ERR_SSL_CONN_EOF);

        if result == ERR_SSL_RECEIVED_NEW_SESSION_TICKET continue;

        if result == ERR_SSL_WANT_READ {
            error = wait_for_input(stream);

            if error != .NONE return 0, error;

            continue;
        }

        return 0, fail(stream, result);
    }

    return 0, .NONE;
}

// Waits until the library has accepted at least one byte of `bytes` (a record holds up to 8 KiB of
// them) and sends it; returns how many it took.
write_some :: (stream: *Tls_Stream, bytes: [] u8) -> s64, Io.Io_Error {
    if bytes.count == 0 return 0, .NONE;
    if !enter(stream) return 0, .CLOSED;

    defer leave(stream);

    while true {
        if stream.broken != .NONE return 0, stream.broken;

        result, error := call_library(stream, .WRITE, bytes.data, cast(u64) bytes.count);

        if error != .NONE return 0, error;

        // The write is done when its bytes are on the connection: this is where it waits when the
        // peer is slow, and why a writer cannot pile up unbounded output.
        error = flush(stream, true);

        if error != .NONE return 0, error;
        if result > 0 return result, .NONE;

        if result == ERR_SSL_WANT_READ {
            error = wait_for_input(stream);

            if error != .NONE return 0, error;

            continue;
        }

        if result == ERR_SSL_RECEIVED_NEW_SESSION_TICKET continue;

        return 0, fail(stream, result);
    }

    return 0, .NONE;
}

// Sends all of `bytes`. On an error it returns how many went out.
write :: (stream: *Tls_Stream, bytes: [] u8) -> s64, Io.Io_Error {
    return Io.write(writer_of(stream), bytes);
}

write :: (stream: *Tls_Stream, text: string) -> s64, Io.Io_Error {
    return Io.write(writer_of(stream), text);
}

// Waits until `read` would not have to wait for the network: decrypted bytes are waiting, or
// something has arrived (which may still be only part of a record).
wait_readable :: (stream: *Tls_Stream) -> Io.Io_Error {
    if !enter(stream) return .CLOSED;

    defer leave(stream);

    if stream.broken != .NONE return stream.broken;
    if stream.peer_closed return .NONE;
    if mbedtls_ssl_check_pending(stream.ssl) != 0 || mbedtls_ssl_get_bytes_avail(stream.ssl) > 0 return .NONE;
    if !stream.can_wait return .UNSUPPORTED;

    return Io.wait_readable(stream.reader);
}

// Ends the stream: sends close_notify (unless `now`, which does not wait and sends nothing) and
// closes the connection under it, which wakes fibers still waiting on it with `.CLOSED`. The stream is
// freed once those fibers have left, so it must not be used after `close`; close it once.
close :: (stream: *Tls_Stream, now := false) -> Io.Io_Error {
    if stream.closing return .NONE;

    stream.closing = true;
    stream.users += 1;

    if !now && stream.established && stream.broken == .NONE {
        // The peer may be gone, or not reading: the close below does not depend on this, and a
        // peer that is not reading must not hold up whoever closes.
        Io.with_deadline(CLOSE_NOTIFY_MS, #code {
            _, error := call_library(stream, .CLOSE_NOTIFY, null, 0);

            if error == .NONE flush(stream, false);
        });
    }

    error := Io.close(stream.reader, now);

    stream.users -= 1;

    if stream.users == 0 release_stream(stream);

    return error;
}

// Whether `reader` is a TLS stream's reader (from `reader_of`) whose connection ended without the
// peer's close_notify (`last_error(stream).kind == .TRUNCATED`). For a protocol that frames its own
// messages, such as HTTP, where that end is no failure once the message is complete.
ended_without_close_notify :: (reader: Io.Reader) -> bool {
    if reader.read_proc != read_proc return false;

    stream := cast(*Tls_Stream) reader.data;

    return stream.failure.kind == .TRUNCATED;
}

reader_of :: (stream: *Tls_Stream) -> Io.Reader {
    reader: Io.Reader;

    reader.data = stream;
    reader.read_proc = read_proc;
    reader.close_proc = close_proc;

    if stream.can_wait reader.wait_proc = wait_proc;

    return reader;
}

writer_of :: (stream: *Tls_Stream) -> Io.Writer {
    writer: Io.Writer;

    writer.data = stream;
    writer.write_proc = write_proc;
    // Closing either half closes the stream, once: close just one of them.
    writer.close_proc = close_proc;

    return writer;
}

#scope_module

read_proc :: (data: *void, buffer: [] u8) -> s64, Io.Io_Error {
    return read(cast(*Tls_Stream) data, buffer);
}

write_proc :: (data: *void, bytes: [] u8) -> s64, Io.Io_Error {
    return write_some(cast(*Tls_Stream) data, bytes);
}

wait_proc :: (data: *void) -> Io.Io_Error {
    return wait_readable(cast(*Tls_Stream) data);
}

close_proc :: (data: *void, now: bool) -> Io.Io_Error {
    return close(cast(*Tls_Stream) data, now);
}

c_text :: (text: *u8) -> string {
    if !text return "";

    length := 0;

    while text[length] != 0 length += 1;

    return string.{ length, text };
}

// How long `close` waits to send close_notify.
CLOSE_NOTIFY_MS :: 1000;

Library_Call :: enum u8 {
    HANDSHAKE;
    READ;
    WRITE;
    CLOSE_NOTIFY;
}

open_stream :: (conf: *void, reader: Io.Reader, writer: Io.Writer) -> *Tls_Stream, Tls_Error {
    ssl := jai_tls_ssl_new();

    if !ssl return null, make_error(.CONFIG, "out of memory");

    status := mbedtls_ssl_setup(ssl, conf);

    if status != 0 {
        jai_tls_ssl_free(ssl);

        return null, from_library(.CONFIG, status);
    }

    stream := New(Tls_Stream);

    stream.reader = reader;
    stream.writer = writer;
    stream.ssl = ssl;
    stream.can_wait = reader.wait_proc != null;
    stream.out.allocator = context.allocator;

    mbedtls_ssl_set_bio(ssl, stream, bio_send, bio_receive, null);
    jai_tls_ssl_set_user(ssl, stream);

    return stream, .{};
}

release_stream :: (stream: *Tls_Stream) {
    jai_tls_ssl_free(stream.ssl);

    array_free(stream.out);

    free(stream);
}

// A call that uses the stream counts as a user, so `close` cannot free it under the call.
enter :: (stream: *Tls_Stream) -> bool {
    if stream.closing return false;

    stream.users += 1;

    return true;
}

leave :: (stream: *Tls_Stream) {
    stream.users -= 1;

    if stream.users == 0 && stream.closing release_stream(stream);
}

// One call into the library, with the stream locked and the fiber's context saved for the
// callbacks. Returns the library's result, or the error of waiting for the lock.
call_library :: (stream: *Tls_Stream, call: Library_Call, buffer: *u8, length: u64) -> s32, Io.Io_Error {
    // A close does not queue behind a fiber that is in the middle of a send: closing the
    // connection is what ends that send.
    if call == .CLOSE_NOTIFY {
        if !Io.try_lock(*stream.lock) return 0, .TIMED_OUT;
    } else {
        error := Io.lock(*stream.lock);

        if error != .NONE return 0, error;
    }

    defer Io.unlock(*stream.lock);

    // Closing ends everything but the close itself.
    if stream.closing && call != .CLOSE_NOTIFY return 0, .CLOSED;

    stream.fiber_context = context;
    stream.fiber_context.stack_trace = null;
    stream.io_error = .NONE;

    result: s32;

    if call == {
        case .HANDSHAKE;
            result = mbedtls_ssl_handshake(stream.ssl);
        case .READ;
            result = mbedtls_ssl_read(stream.ssl, buffer, length);
        case .WRITE;
            result = mbedtls_ssl_write(stream.ssl, buffer, length);
        case .CLOSE_NOTIFY;
            result = mbedtls_ssl_close_notify(stream.ssl);
    }

    return result, .NONE;
}

// The library wants input and none is there: wait for the connection to become readable.
wait_for_input :: (stream: *Tls_Stream) -> Io.Io_Error {
    // Without a way to wait the receive callback reads directly and never asks to wait.
    if !stream.can_wait return .NONE;

    error := Io.wait_readable(stream.reader);

    if error == .NONE stream.ready = true;

    return error;
}

// A library call failed for good: remember why and break the stream. Returns what the caller of
// `read` or `write_some` sees.
fail :: (stream: *Tls_Stream, result: s32) -> Io.Io_Error {
    if stream.io_error != .NONE {
        stream.failure = io_error(stream.io_error);
        stream.broken = stream.io_error;
    } else if result == ERR_SSL_CONN_EOF {
        stream.failure = make_error(.TRUNCATED, "the connection ended without close_notify", result, .CONNECTION_RESET);
        stream.broken = .CONNECTION_RESET;
    } else {
        stream.failure = from_library(.PROTOCOL, result);
        stream.failure.io = .CONNECTION_ABORTED;
        stream.broken = .CONNECTION_ABORTED;
    }

    return stream.broken;
}

// The library's way of reading and writing the connection. These run inside its calls, on the
// fiber that made the call (`stream.fiber_context`). The send callback takes the bytes at once and
// `flush` writes them after the call; the receive callback may park the fiber only for a source
// that cannot say when it is readable.
bio_send :: (context_data: *void, buffer: *u8, length: u64) -> s32 #c_call {
    stream := cast(*Tls_Stream) context_data;

    push_context stream.fiber_context {
        old := stream.out.count;

        array_reserve(*stream.out, old + cast(s64) length);
        memcpy(stream.out.data + old, buffer, cast(s64) length);

        stream.out.count = old + cast(s64) length;
    }

    return cast(s32) length;
}

bio_receive :: (context_data: *void, buffer: *u8, length: u64) -> s32 #c_call {
    stream := cast(*Tls_Stream) context_data;
    result: s32;

    push_context stream.fiber_context {
        if stream.can_wait && !stream.ready {
            // Nothing is known to be there: have the fiber wait outside the library.
            result = ERR_SSL_WANT_READ;
        } else {
            stream.ready = false;

            // A source that cannot say when it is readable is read here, parking the fiber: what
            // the library has produced so far must reach the peer first, or both would wait.
            if !stream.can_wait flush(stream, true);

            count, error := Io.read(stream.reader, .{ cast(s64) length, buffer });

            if error != .NONE && stream.can_wait && (error == .TIMED_OUT || error == .CANCELED || error == .CLOSED) {
                // The deadline passed or the fiber was canceled between the wait and this read.
                // The library has lost nothing, so let it ask again: the wait outside reports the
                // same error, and the stream stays usable.
                result = ERR_SSL_WANT_READ;
            } else if error != .NONE {
                stream.io_error = error;
                result = ERR_NET_RECV_FAILED;
            } else {
                result = cast(s32) count;
            }
        }
    }

    return result;
}

// Records the name a client sent (the library tells the server before it picks a certificate) and
// accepts every name.
sni_callback :: (data: *void, ssl: *void, name: *u8, length: u64) -> s32 #c_call {
    stream := cast(*Tls_Stream) jai_tls_ssl_user(ssl);

    count := cast(s64) length;

    if count > stream.peer_name_bytes.count count = stream.peer_name_bytes.count;

    for 0..count - 1 stream.peer_name_bytes[it] = name[it];

    stream.peer_name_count = cast(u8) count;

    return 0;
}

// Writes what the library produced to the connection, in order. Only one flush writes at a time
// (the others wait their turn, or, with `wait` off, leave their bytes to the one that is writing,
// which does not stop before `out` is empty). A call that must know its bytes are out, like a
// write, waits; a read that produced an alert or a key update does not hold itself up for it.
// Deadlines and `cancel` apply as to any write. A failed write breaks the stream.
flush :: (stream: *Tls_Stream, wait: bool) -> Io.Io_Error {
    if stream.out.count == 0 return .NONE;

    if wait {
        error := Io.lock(*stream.flush_lock);

        if error != .NONE return error;
    } else if !Io.try_lock(*stream.flush_lock) {
        return .NONE;
    }

    defer Io.unlock(*stream.flush_lock);

    failure := Io.Io_Error.NONE;

    while stream.out.count > 0 && failure == .NONE {
        // Take everything there is: the library may add more while this waits, into a new buffer,
        // so the memory being written never moves.
        chunk := stream.out;

        stream.out = .{};
        stream.out.allocator = chunk.allocator;

        sent := 0;

        while sent < chunk.count {
            count, error := Io.write_some(stream.writer, .{ chunk.count - sent, chunk.data + sent });

            if error != .NONE {
                failure = error;

                break;
            }

            if count <= 0 {
                failure = .NO_SPACE;

                break;
            }

            sent += count;
        }

        array_free(chunk);
    }

    if failure != .NONE {
        array_reset(*stream.out);

        // A write that fails because the stream is being closed under it says so.
        if stream.closing failure = .CLOSED;

        if stream.broken == .NONE {
            stream.failure = io_error(failure);
            stream.broken = failure;
        }

        return failure;
    }

    return .NONE;
}

// ---------------------------------------------------------------------------------------------
// The handshake

run_handshake :: (stream: *Tls_Stream) -> Tls_Error {
    while true {
        result, error := call_library(stream, .HANDSHAKE, null, 0);

        if error != .NONE return io_error(error);

        // The handshake messages (and the alert of a failed one) go out before waiting for the peer.
        flush_error := flush(stream, true);

        if flush_error != .NONE && result != 0 return io_error(flush_error);
        if result == 0 return ifx flush_error != .NONE then io_error(flush_error) else Tls_Error.{};

        if result == ERR_SSL_WANT_READ {
            error = wait_for_input(stream);

            if error != .NONE return io_error(error);

            continue;
        }

        return handshake_failure(stream, result);
    }

    return .{};
}

handshake_failure :: (stream: *Tls_Stream, result: s32) -> Tls_Error {
    if stream.io_error != .NONE return io_error(stream.io_error);

    if result == ERR_X509_CERT_VERIFY_FAILED {
        return verify_failure(mbedtls_ssl_get_verify_result(stream.ssl));
    }

    if result == ERR_SSL_CONN_EOF {
        return make_error(.IO, "the peer closed the connection during the handshake", result, .CONNECTION_RESET);
    }

    return from_library(.HANDSHAKE, result);
}

// The reasons mbedTLS found, in words.
verify_failure :: (flags: u32) -> Tls_Error {
    buffer: [160] u8;

    count := mbedtls_x509_crt_verify_info(buffer.data, buffer.count, temp_c_string(""), flags);
    text := string.{ max(count, 0), buffer.data };

    // One line per reason, each ending in a newline.
    for 0..text.count - 1 {
        if buffer[it] == #char "\n" buffer[it] = #char ";";
    }

    if text.count > 0 && buffer[text.count - 1] == #char ";" text.count -= 1;

    error := make_error(.VERIFY, text, ERR_X509_CERT_VERIFY_FAILED);

    error.verify_flags = flags;

    return error;
}

// The client's second look at the server's certificate chain, by the verifier the config names
// (the operating system's, or the user's).
verify_externally :: (stream: *Tls_Stream, native: *Tls_Native, server_name: string) -> Tls_Error {
    chain: [..][] u8;

    defer array_free(chain);

    certificate := mbedtls_ssl_get_peer_cert(stream.ssl);

    while certificate {
        length: u64;
        der := jai_tls_crt_der(certificate, *length);

        array_add(*chain, .{ cast(s64) length, der });

        certificate = jai_tls_crt_next(certificate);
    }

    if chain.count == 0 return make_error(.VERIFY, "the server sent no certificate");

    reason := native.external_verify(chain, server_name, native.external_data);

    if reason.count > 0 return make_error(.VERIFY, reason);

    return .{};
}
