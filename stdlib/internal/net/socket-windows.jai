// A socket handle: Winsock's unsigned pointer-sized `SOCKET`. Unix uses an `s32` file descriptor. Compare a handle against
// `INVALID_SOCKET`, never against a number.
Socket :: SOCKET;
INVALID_SOCKET :: cast(Socket) 0xffffffffffffffff;
SOCKET_ERROR :: -1;
// The error `get_last_socket_error()` reports when a non-blocking call has nothing to do yet (`WSAEWOULDBLOCK`).
SOCKET_WOULDBLOCK :: WSAEWOULDBLOCK;
// The error `get_last_socket_error()` reports when a non-blocking `connect` has started and is still in progress. Winsock
// uses `WSAEWOULDBLOCK` for this, so it equals `SOCKET_WOULDBLOCK` here, unlike the Unix targets.
SOCKET_WOULDBLOCK_CONNECT :: SOCKET_WOULDBLOCK;

// Builds a Winsock control code for `ioctlsocket` from the group character `x`, the command number `y` and the type of the
// argument `T` (its size is encoded in the code). `FIONBIO` is `IOW(102, 126, u32)`.
IOW :: Control.IOW;

// Official Winsock2 FIONBIO encoding; the character literal form is incomplete.
FIONBIO :: #run IOW(102, 126, u32);

// Starts Winsock 2.2 and returns whether it is ready.
//
// Winsock counts the calls, so calling it again is harmless. A failure is logged through `context.logger` and returns
// false. Call it once before the first socket call; on Unix it only returns true.
socket_init :: () -> bool {
    data: WSADATA;
    status := WSAStartup(WINSOCK_VERSION, *data);

    if status != 0 {
        text := sprint("Winsock initialization error %\n", status);
        defer free(text);
        context.logger(text, context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false;
    }

    return true;
}

// The error code of the last failed socket call on this thread: `WSAGetLastError` on Windows, `errno` on Unix.
//
// Read it straight after the call that failed. Compare it with `SOCKET_WOULDBLOCK`, `SOCKET_WOULDBLOCK_CONNECT` or the
// `WSAE...` constants.
get_last_socket_error :: inline () -> W.OS_Error_Code {
    return WSAGetLastError();
}

// Switches `s` between blocking and non-blocking mode and returns whether it worked.
//
// In non-blocking mode `recv`, `send`, `accept` and `connect` return an error instead of waiting, and
// `get_last_socket_error()` is `SOCKET_WOULDBLOCK`. Reads and writes can then move fewer bytes than asked for. A failure is
// logged through `context.logger`. New sockets start in blocking mode.
set_blocking :: (s: Socket, blocking: bool) -> bool {
    nonblocking: u32 = 1;
    if blocking nonblocking = 0;

    if ioctlsocket(s, FIONBIO, *nonblocking) != 0 {
        report_error();
        return false;
    }

    return true;
}

// Turns TCP keep-alive probes on `s` on or off and returns whether it worked.
//
// With `keepalive` false the probes are switched off and the timings are ignored. When it is true, each timing that is
// above zero replaces the system default: seconds of idleness before the first probe, seconds between probes, and the
// number of unanswered probes before the connection is dropped. Zero keeps the system's value. A failure is logged through
// `context.logger` and returns false; options already set stay set.
//
// Winsock counts the two timings in milliseconds; they are converted from seconds here. The probe count is applied only where
// the system has the option (Windows 10 and later) and is otherwise left at the system's own.
set_keepalive :: (s: Socket, keepalive: bool, idle_seconds_before_keepalive: s32 = 0, keepalive_interval_seconds: s32 = 0, num_keepalive_attempts: s32 = 0) -> bool {
    enabled: s32;
    if keepalive enabled = 1;

    if setsockopt(s, SOL_SOCKET, SO_KEEPALIVE, cast(*u8) *enabled, 4) != 0 {
        report_error();
        return false;
    }

    if !keepalive return true;

    // Two hours before the first probe and one second between them are what the system uses when none is set.
    options := tcp_keepalive.{ 1, Control.keepalive_milliseconds(idle_seconds_before_keepalive, 7200000), Control.keepalive_milliseconds(keepalive_interval_seconds, 1000) };
    returned: u32;

    if WSAIoctl(s, SIO_KEEPALIVE_VALS, *options, size_of(tcp_keepalive), null, 0, *returned, null, null) != 0 {
        report_error();
        return false;
    }

    if num_keepalive_attempts > 0 setsockopt(s, cast(s32) IPPROTO.TCP, TCP_KEEPCNT, cast(*u8) *num_keepalive_attempts, 4);

    return true;
}

// Closes the socket `s` points to and sets it to `INVALID_SOCKET`.
//
// A handle that is already `INVALID_SOCKET` is left alone, so it is safe to call twice. A failing close is logged through
// `context.logger` but the handle is reset anyway.
close_and_reset :: (s: *Socket, loc := #caller_location) {
    if s.* != INVALID_SOCKET {
        status := closesocket(s.*);
        if status != 0 report_error();
        s.* = INVALID_SOCKET;
    }
} @FreesArgument

// Adds `fd` to `set`. A socket already in the set is not added twice, and a full set (`fd_array.count` sockets, 64 by
// default) ignores further sockets without an error.
FD_SET :: (fd: Socket, set: *fd_set) {
    Control.add_descriptor(fd, set);
}

// Removes `fd` from `set`, shifting the later sockets down. A socket that is not in the set is ignored.
FD_CLR :: (fd: Socket, set: *fd_set) {
    Control.remove_descriptor(fd, set);
}

#scope_module
// The result of a `bind` that was refused before reaching the system: `-1`, with the socket error set to invalid argument.
refuse_address :: () -> s32 {
    WSASetLastError(WSAEINVAL);

    return -1;
}

#scope_file
#import "Basic";
W :: #import "internal/bindings/windows-api";

Control :: #import "internal/net/winsock-control";
