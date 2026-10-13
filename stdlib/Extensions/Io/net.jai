// TCP for fibers: addresses, listeners and streams. Every procedure that can wait (`accept`,
// `connect`, `read`, `write`, `close`, `resolve`) blocks only the calling fiber and must run on a
// fiber, inside `io_run`. They return what happened and an `Io_Error`; `platform_code()` has the
// system's own number when the error is `.OTHER`.
//
//     listener, error := tcp_listen(":8080");
//     while true {
//         conn, error := accept(listener);
//         if error != .NONE break;
//         spawn(handle, conn);
//     }

Address_Family :: enum u8 {
    NONE;
    IPV4;
    IPV6;
}

// An IP address and port. `bytes` is in network order; an IPv4 address uses the first four.
Address :: struct {
    family: Address_Family;
    // In host order.
    port: u16;
    bytes: [16] u8;
    // IPv6 scope (interface index), 0 if none.
    scope: u32;
}

// A listening socket.
Tcp_Listener :: struct {
    handle: Handle;
}

// A connected socket, from `accept` or `connect`.
Tcp_Stream :: struct {
    handle: Handle;
}

address_v4 :: (a: u8, b: u8, c: u8, d: u8, port: u16) -> Address {
    address: Address;
    address.family = .IPV4;
    address.port = port;
    address.bytes[0] = a;
    address.bytes[1] = b;
    address.bytes[2] = c;
    address.bytes[3] = d;

    return address;
}

// Reads an address literal: `1.2.3.4:80`, `:8080` (any IPv4 address), `[::1]:80`, `1.2.3.4`,
// `::1`. It does not look names up; `resolve` does.
parse_address :: (text: string) -> Address, bool {
    address: Address;
    host := text;
    port_text: string;
    has_port := false;

    colons := 0;

    for text {
        if it == #char ":" colons += 1;
    }

    if text.count > 0 && text[0] == #char "[" {
        end := -1;

        for text {
            if it == #char "]" {
                end = it_index;
                break;
            }
        }

        if end < 0 return .{}, false;

        host = string.{ end - 1, text.data + 1 };

        rest := string.{ text.count - end - 1, text.data + end + 1 };
        if rest.count > 0 {
            if rest[0] != #char ":" return .{}, false;

            port_text = string.{ rest.count - 1, rest.data + 1 };
            has_port = true;
        }
    } else if colons == 1 {
        for text {
            if it == #char ":" {
                host = string.{ it_index, text.data };
                port_text = string.{ text.count - it_index - 1, text.data + it_index + 1 };
                has_port = true;
            }
        }
    }

    if has_port {
        port, ok := parse_decimal(port_text, 65535);
        if !ok return .{}, false;

        address.port = cast(u16) port;
    }

    if host.count == 0 {
        address.family = .IPV4;

        return address, true;
    }

    if parse_ipv4(host, *address.bytes) {
        address.family = .IPV4;

        return address, true;
    }

    if parse_ipv6(host, *address.bytes) {
        address.family = .IPV6;

        return address, true;
    }

    return .{}, false;
}

// `1.2.3.4:80` or `[::1]:80`, in temporary storage.
address_to_string :: (address: Address) -> string {
    if address.family == .IPV4 {
        return tprint("%.%.%.%:%", address.bytes[0], address.bytes[1], address.bytes[2], address.bytes[3], address.port);
    }

    if address.family != .IPV6 return "none";

    groups: [8] u16;
    for 0..7 groups[it] = join_bytes(address.bytes[it * 2], address.bytes[it * 2 + 1]);

    // The longest run of zero groups, at least two, becomes "::".
    best_start := -1;
    best_length := 0;
    start := 0;

    while start < 8 {
        if groups[start] != 0 {
            start += 1;

            continue;
        }

        end := start;
        while end < 8 && groups[end] == 0 end += 1;

        if end - start > best_length && end - start >= 2 {
            best_start = start;
            best_length = end - start;
        }

        start = end;
    }

    builder: String_Builder;

    init_string_builder(*builder);

    append(*builder, "[");

    index := 0;
    while index < 8 {
        if index == best_start {
            append(*builder, "::");
            index += best_length;

            continue;
        }

        if index > 0 && index != best_start + best_length append(*builder, ":");

        print_to_builder(*builder, "%", formatInt(groups[index], base = 16));
        index += 1;
    }

    print_to_builder(*builder, "]:%", address.port);

    return builder_to_string(*builder, do_reset = true);
}

// The system's error number behind the last failed IO call of the running fiber.
last_platform_code :: () -> s32 {
    fiber := current_fiber();

    return fiber.platform_code;
}

// ---------------------------------------------------------------------------------------------
// Listening and connecting

// Listens on `address` (a literal such as `":8080"`, port 0 picks a free one; see
// `local_address`). `reuse_port` lets several listeners, one per thread, share the port; the
// kernel then spreads the connections between them.
tcp_listen :: (text: string, backlog := 1024, reuse_port := false) -> Tcp_Listener, Io_Error {
    address, ok := parse_address(text);
    if !ok return .{}, .INVALID;

    return tcp_listen(address, backlog, reuse_port);
}

tcp_listen :: (address: Address, backlog := 1024, reuse_port := false) -> Tcp_Listener, Io_Error {
    current_fiber();

    listener: Tcp_Listener;
    storage: Sock_Storage;

    length := to_sockaddr(address, *storage);

    if length == 0 return .{}, .INVALID;

    fd := sys_socket(family_number(address.family), sockets_nonblocking());
    if fd < 0 return .{}, error_from_result(fd);

    handle := Handle.{ fd, .SOCKET };

    result := sys_prepare_listener(fd, reuse_port);

    if result >= 0 result = sys_bind(fd, *storage, length);
    if result >= 0 result = sys_listen(fd, cast(s32) backlog);

    if result < 0 {
        error := error_from_result(result);
        discard_handle(handle);

        return .{}, error;
    }

    listener.handle = handle;

    return listener, .NONE;
}

// Waits for the next connection.
accept :: (listener: Tcp_Listener) -> Tcp_Stream, Io_Error {
    stream: Tcp_Stream;

    op := begin_op(.ACCEPT, listener.handle);
    error := await_op(op);
    fd := op.result;

    end_op(op);

    if error != .NONE return stream, error;

    stream.handle = .{ fd, .SOCKET };
    sys_set_option(fd, IPPROTO_TCP, TCP_NODELAY, 1);

    return stream, .NONE;
}

// Connects to `address`. The one with a name is `connect("host:port")`.
connect :: (address: Address) -> Tcp_Stream, Io_Error {
    current_fiber();

    stream: Tcp_Stream;
    storage: Sock_Storage;

    length := to_sockaddr(address, *storage);

    if length == 0 return stream, .INVALID;

    fd := sys_socket(family_number(address.family), sockets_nonblocking());
    if fd < 0 return stream, error_from_result(fd);

    handle := Handle.{ fd, .SOCKET };

    op := begin_op(.CONNECT, handle);
    op.address = storage;
    op.address_length = length;

    error := await_op(op);

    end_op(op);

    if error != .NONE {
        discard_handle(handle);

        return stream, error;
    }

    sys_set_option(fd, IPPROTO_TCP, TCP_NODELAY, 1);

    stream.handle = handle;

    return stream, .NONE;
}

// Connects to `"host:port"`, `"1.2.3.4:80"` or `"[::1]:80"`, looking `host` up if it is a name.
connect :: (target: string) -> Tcp_Stream, Io_Error {
    split := -1;

    for target {
        if it == #char ":" split = it_index;
    }

    if split < 0 return .{}, .INVALID;

    address, literal := parse_address(target);
    if literal return connect(address);

    port, ok := parse_decimal(string.{ target.count - split - 1, target.data + split + 1 }, 65535);
    if !ok return .{}, .INVALID;

    resolved, error := resolve(string.{ split, target.data }, cast(u16) port);
    if error != .NONE return .{}, error;

    return connect(resolved);
}

// The address this socket is bound to; with `tcp_listen(":0")`, which port it got.
local_address :: (listener: Tcp_Listener) -> Address, Io_Error {
    return socket_address(listener.handle, false);
}

local_address :: (stream: Tcp_Stream) -> Address, Io_Error {
    return socket_address(stream.handle, false);
}

peer_address :: (stream: Tcp_Stream) -> Address, Io_Error {
    return socket_address(stream.handle, true);
}

// ---------------------------------------------------------------------------------------------
// Reading and writing

// Waits for data and reads what is there, up to `buffer.count` bytes. 0 means the peer closed its
// end. An error comes with 0 bytes.
read :: (stream: Tcp_Stream, buffer: [] u8) -> s64, Io_Error {
    if buffer.count == 0 return 0, .NONE;

    op := begin_op(.RECV, stream.handle);
    op.buffer = buffer.data;
    op.length = buffer.count;

    error := await_op(op);
    count := op.result;

    end_op(op);

    if error != .NONE return 0, error;

    return count, .NONE;
}

// Reads until `buffer` is full. `.END_OF_FILE` if the peer closed first, with the bytes read.
read_full :: (stream: Tcp_Stream, buffer: [] u8) -> s64, Io_Error {
    return read_full(reader_of(stream), buffer);
}

// Sends some of `data`: at least one byte unless `data` is empty. Returns how many.
write_some :: (stream: Tcp_Stream, data: [] u8) -> s64, Io_Error {
    if data.count == 0 return 0, .NONE;

    op := begin_op(.SEND, stream.handle);
    op.buffer = data.data;
    op.length = data.count;

    error := await_op(op);
    count := op.result;

    end_op(op);

    if error != .NONE return 0, error;

    return count, .NONE;
}

// Sends all of `data`, waiting as often as it takes. On an error it returns how many bytes went out.
write :: (stream: Tcp_Stream, data: [] u8) -> s64, Io_Error {
    return write(writer_of(stream), data);
}

write :: (stream: Tcp_Stream, text: string) -> s64, Io_Error {
    return write(stream, .{ text.count, text.data });
}

// Closes the sending side; the peer reads the end of the stream once it has what was sent.
shutdown_write :: (stream: Tcp_Stream) -> Io_Error {
    current_fiber();

    return error_from_result(sys_shutdown(stream.handle.fd, SHUT_WR));
}

// Closes the socket. Fibers still waiting on it return `.CLOSED`. Works for a canceled fiber too,
// so `defer close(conn)` always cleans up. With `now` it closes at once without waiting, for code
// that cannot block: a `Local_Drop`, or a fiber that must not give up its turn. Fibers still
// waiting on the socket are not woken, so use the plain call unless the scheduler is ending.
close :: (stream: Tcp_Stream, now := false) -> Io_Error {
    if now {
        close_without_waiting(stream.handle);

        return .NONE;
    }

    return close_handle(stream.handle);
}

close :: (listener: Tcp_Listener, now := false) -> Io_Error {
    if now {
        close_without_waiting(listener.handle);

        return .NONE;
    }

    return close_handle(listener.handle);
}

// ---------------------------------------------------------------------------------------------
// Names

// Looks `host` up (an address literal, or a name for the system's resolver, which
// runs on a helper thread so only this fiber waits) and gives its first address, IPv4 preferred.
resolve :: (host: string, port: u16) -> Address, Io_Error {
    current_fiber();

    address, literal := parse_address(host);
    if literal {
        address.port = port;

        return address, .NONE;
    }

    if host.count == 0 || host.count > 253 return .{}, .INVALID;

    found, error := resolve_with_helper(host, port);

    // A system without a hosts file still knows what localhost is.
    if error == .RESOLVE_FAILED && host == "localhost" return address_v4(127, 0, 0, 1, port), .NONE;

    return found, error;
}

#scope_module

// Whether this scheduler's backend wants non-blocking sockets.
sockets_nonblocking :: () -> bool {
    scheduler := current_scheduler();

    return scheduler.engine.backend != .IO_URING;
}

family_number :: (family: Address_Family) -> s32 {
    return ifx family == .IPV6 then AF_INET6 else AF_INET;
}

parse_decimal :: (text: string, limit: s64) -> s64, bool {
    if text.count == 0 || text.count > 10 return 0, false;

    value := 0;

    for text {
        digit := it;
        if digit < #char "0" || digit > #char "9" return 0, false;

        value = value * 10 + (digit - #char "0");
        if value > limit return 0, false;
    }

    return value, true;
}

parse_ipv4 :: (text: string, bytes: *[16] u8) -> bool {
    part := 0;
    digits := 0;
    value := 0;

    for text {
        c := it;

        if c == #char "." {
            if digits == 0 || part >= 3 return false;

            bytes.*[part] = cast(u8) value;
            part += 1;
            digits = 0;
            value = 0;
        } else if c >= #char "0" && c <= #char "9" {
            value = value * 10 + (c - #char "0");
            digits += 1;

            if digits > 3 || value > 255 return false;
        } else {
            return false;
        }
    }

    if digits == 0 || part != 3 return false;

    bytes.*[3] = cast(u8) value;

    return true;
}

join_bytes :: (high: u8, low: u8) -> u16 {
    value: u16 = high;
    value <<= 8;

    return value | low;
}

hex_value :: (c: u8) -> s64 {
    if c >= #char "0" && c <= #char "9" return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;

    return -1;
}

parse_ipv6 :: (text: string, bytes: *[16] u8) -> bool {
    values: [8] u16;
    count := 0;
    compress_at := -1;
    index := 0;
    length := text.count;

    if length < 2 return false;

    if text[0] == #char ":" {
        if text[1] != #char ":" return false;

        compress_at = 0;
        index = 2;
    }

    while index < length {
        start := index;
        dotted := false;

        while index < length && text[index] != #char ":" {
            if text[index] == #char "." dotted = true;

            index += 1;
        }

        token := string.{ index - start, text.data + start };
        if token.count == 0 return false;

        if dotted {
            if index != length || count + 2 > 8 return false;

            v4: [16] u8;
            if !parse_ipv4(token, *v4) return false;

            values[count] = join_bytes(v4[0], v4[1]);
            values[count + 1] = join_bytes(v4[2], v4[3]);
            count += 2;

            break;
        }

        if token.count > 4 || count >= 8 return false;

        value := 0;

        for token {
            digit := hex_value(it);
            if digit < 0 return false;

            value = value * 16 + digit;
        }

        values[count] = cast(u16) value;
        count += 1;

        if index < length {
            index += 1;

            if index < length && text[index] == #char ":" {
                if compress_at >= 0 return false;

                compress_at = count;
                index += 1;
            } else if index == length {
                return false;
            }
        }
    }

    if compress_at < 0 {
        if count != 8 return false;
    } else {
        if count > 7 return false;

        // Shift the groups after the "::" to the end.
        tail := count - compress_at;
        moved := tail - 1;

        while moved >= 0 {
            values[8 - tail + moved] = values[compress_at + moved];
            moved -= 1;
        }

        for 0..8 - count - 1 values[compress_at + it] = 0;
    }

    for 0..7 {
        bytes.*[it * 2] = cast(u8) (values[it] >> 8);
        bytes.*[it * 2 + 1] = cast(u8) (values[it] & 0xff);
    }

    return true;
}

// The kernel's form of `address`, written into `storage`. Returns its length, 0 for no address.
to_sockaddr :: (address: Address, storage: *Sock_Storage) -> u32 {
    storage.* = .{};
    bytes := cast(*u8) storage;

    if address.family == .IPV4 {
        #if OS == .MACOS {
            bytes[0] = 16;
            bytes[1] = AF_INET;
        } else {
            bytes[0] = AF_INET;
        }

        bytes[2] = cast(u8) (address.port >> 8);
        bytes[3] = cast(u8) (address.port & 0xff);
        for 0..3 bytes[4 + it] = address.bytes[it];

        return 16;
    }

    if address.family == .IPV6 {
        #if OS == .MACOS {
            bytes[0] = 28;
            bytes[1] = AF_INET6;
        } else {
            bytes[0] = AF_INET6;
        }

        bytes[2] = cast(u8) (address.port >> 8);
        bytes[3] = cast(u8) (address.port & 0xff);
        for 0..15 bytes[8 + it] = address.bytes[it];

        scope := address.scope;
        for 0..3 bytes[24 + it] = cast(u8) (scope >> (8 * cast(u32) it));

        return 28;
    }

    return 0;
}

from_sockaddr :: (storage: *Sock_Storage) -> Address {
    bytes := cast(*u8) storage;
    address: Address;

    #if OS == .MACOS family := bytes[1];
    else family := bytes[0];

    if family == AF_INET {
        address.family = .IPV4;
        for 0..3 address.bytes[it] = bytes[4 + it];
    } else if family == AF_INET6 {
        address.family = .IPV6;
        for 0..15 address.bytes[it] = bytes[8 + it];
        for 0..3 address.scope |= (cast(u32) bytes[24 + it]) << (8 * cast(u32) it);
    } else {
        return address;
    }

    address.port = join_bytes(bytes[2], bytes[3]);

    return address;
}

socket_address :: (handle: Handle, peer: bool) -> Address, Io_Error {
    current_fiber();

    storage: Sock_Storage;

    result := sys_socket_name(handle.fd, *storage, peer);

    if result < 0 return .{}, error_from_result(result);

    return from_sockaddr(*storage), .NONE;
}

// Closes `handle` through the engine, for the running fiber, and cannot be refused because the
// fiber was canceled.
close_handle :: (handle: Handle) -> Io_Error {
    if handle.fd < 0 return .NONE;

    scheduler := current_scheduler();

    op := begin_op(.CLOSE, handle);
    op.fiber = scheduler.current;

    error: Io_Error;

    if !engine_submit(scheduler, op) {
        fiber := scheduler.current;
        fiber.op = op;
        park(scheduler, fiber);
        fiber.op = null;
    }

    if op.result < 0 error = error_from_result(op.result);

    end_op(op);

    return error;
}

// Closes `handle` at once, on the calling thread's scheduler, without parking the fiber (it may
// run outside any fiber, in a `Local_Drop`).
close_without_waiting :: (handle: Handle) {
    scheduler := context.io_scheduler;
    engine := scheduler.engine;

    if engine.backend == .EPOLL || engine.backend == .KQUEUE poll_close(scheduler, engine, handle.fd);

    sys_close_handle(handle);
}

// Closes a handle that never reached the engine, or whose operations all finished.
discard_handle :: (handle: Handle) {
    scheduler := current_scheduler();
    engine := scheduler.engine;

    if engine.backend == .EPOLL || engine.backend == .KQUEUE poll_close(scheduler, engine, handle.fd);

    sys_close_handle(handle);
}
