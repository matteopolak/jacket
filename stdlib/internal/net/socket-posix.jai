// A socket handle: the file descriptor (`s32`) on Unix. Windows uses Winsock's unsigned pointer-sized `SOCKET`. Compare a
// handle against `INVALID_SOCKET`, never against a number.
Socket :: s32;
INVALID_SOCKET: Socket : -1;
SOCKET_ERROR :: -1;
// The error `get_last_socket_error()` reports when a non-blocking call has nothing to do yet (`EWOULDBLOCK`). On Linux,
// macOS and Android `EAGAIN` has the same value for the sockets here.
SOCKET_WOULDBLOCK :: cast(P.OS_Error_Code) P.EWOULDBLOCK;
// The error `get_last_socket_error()` reports when a non-blocking `connect` has started and is still in progress
// (`EINPROGRESS` on Unix). Wait for the socket to become writable, then read `SO_ERROR` to learn the outcome.
SOCKET_WOULDBLOCK_CONNECT :: cast(P.OS_Error_Code) P.EINPROGRESS;

// Prepares the socket layer and returns whether it is ready.
//
// On Unix there is nothing to prepare and the result is always true. On Windows it starts Winsock 2.2, and a failure
// is logged through `context.logger` and returns false. Call it once before the first socket call in portable code; calling
// it again is harmless.
//
// ```jai,skip_on(wasm)
// assert(socket_init());
// ```
socket_init :: inline () -> bool {
    return true;
}

// The error code of the last failed socket call on this thread: `errno` on Unix, `WSAGetLastError` on Windows.
//
// Read it straight after the call that failed, before any other call can replace it. Compare it with `SOCKET_WOULDBLOCK`,
// `SOCKET_WOULDBLOCK_CONNECT` or the platform's own error constants.
//
// ```jai,skip_on(wasm)
// assert(socket_init());
//
// // A port that was bound and then released has nobody listening on it.
// probe := socket(AF_INET, .SOCK_STREAM, .TCP);
// assert(bind(probe, htonl(INADDR_LOOPBACK), 0) == 0);
//
// closed: sockaddr_in;
// length: socklen_t = size_of(sockaddr_in);
// assert(getsockname(probe, cast(*sockaddr) *closed, *length) == 0);
// close_and_reset(*probe);
//
// client := socket(AF_INET, .SOCK_STREAM, .TCP);
// defer close_and_reset(*client);
//
// assert(connect(client, cast(*sockaddr) *closed, length) != 0);
// assert(get_last_socket_error() != 0);
// ```
get_last_socket_error :: inline () -> P.OS_Error_Code {
    return P.errno();
}

// Switches `s` between blocking and non-blocking mode and returns whether it worked.
//
// In non-blocking mode `recv`, `send`, `accept` and `connect` return -1 (or `INVALID_SOCKET`) instead of waiting, and
// `get_last_socket_error()` is `SOCKET_WOULDBLOCK` (or `SOCKET_WOULDBLOCK_CONNECT` for a connect under way). Reads and
// writes can then move fewer bytes than asked for. A failure is logged through `context.logger`, and the mode is left as it
// was. New sockets start in blocking mode.
//
// ```jai,skip_on(wasm)
// assert(socket_init());
//
// s := socket(AF_INET, .SOCK_STREAM, .TCP);
// defer close_and_reset(*s);
//
// assert(set_blocking(s, false));
// assert(set_blocking(s, true));
// ```
set_blocking :: (s: Socket, blocking: bool) -> bool {
    flags := P.fcntl(s, P.F_GETFL);
    if flags < 0 {
        report_error();
        return false;
    }

    if blocking flags &= ~P.O_NONBLOCK;
    else flags |= P.O_NONBLOCK;

    if P.fcntl(s, P.F_SETFL, flags) != 0 {
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
// ```jai,skip_on(wasm)
// assert(socket_init());
//
// s := socket(AF_INET, .SOCK_STREAM, .TCP);
// defer close_and_reset(*s);
//
// assert(set_keepalive(s, true));
// assert(set_keepalive(s, true, 30, 5, 4));
// assert(set_keepalive(s, false));
// ```
set_keepalive :: (s: Socket, keepalive: bool, idle_seconds_before_keepalive: s32 = 0, keepalive_interval_seconds: s32 = 0, num_keepalive_attempts: s32 = 0) -> bool {
    enabled: s32;
    if keepalive enabled = 1;

    if setsockopt(s, SOL_SOCKET, SO_KEEPALIVE, *enabled, 4) != 0 {
        report_error();
        return false;
    }

    if !keepalive return true;

    if idle_seconds_before_keepalive > 0 {
        #if OS == .MACOS {
            option :: TCP_KEEPALIVE;
        } else {
            option :: TCP_KEEPIDLE;
        }

        if setsockopt(s, cast(s32) IPPROTO.TCP, option, *idle_seconds_before_keepalive, 4) != 0 {
            report_error();
            return false;
        }
    }

    if keepalive_interval_seconds > 0 && setsockopt(s, cast(s32) IPPROTO.TCP, TCP_KEEPINTVL, *keepalive_interval_seconds, 4) != 0 {
        report_error();
        return false;
    }

    if num_keepalive_attempts > 0 && setsockopt(s, cast(s32) IPPROTO.TCP, TCP_KEEPCNT, *num_keepalive_attempts, 4) != 0 {
        report_error();
        return false;
    }

    return true;
}

// Closes the socket `s` points to and sets it to `INVALID_SOCKET`.
//
// A handle that is already `INVALID_SOCKET` is left alone, so it is safe to call twice and to put in a `defer` after the
// handle may have been closed elsewhere. A failing close is logged through `context.logger` but the handle is reset
// anyway. Data still queued for sending is delivered in the background where the system allows it; call `shutdown` first to
// signal the end of a stream to the peer in order.
//
// ```jai,skip_on(wasm)
// assert(socket_init());
//
// s := socket(AF_INET, .SOCK_STREAM, .TCP);
// assert(s != INVALID_SOCKET);
//
// close_and_reset(*s);
// assert(s == INVALID_SOCKET);
//
// close_and_reset(*s);
// ```
close_and_reset :: (s: *Socket, loc := #caller_location) {
    if s.* != INVALID_SOCKET {
        if P.close(s.*) != 0 report_error();
        s.* = INVALID_SOCKET;
    }
} @FreesArgument

FD_BITS_ELEMENT_TYPE :: type_of(fd_set.fds_bits[0]);

BITS_PER_FD_SET_MASK :: size_of(FD_BITS_ELEMENT_TYPE) * 8;

// Whether `fd` can be an index into `set`: the set is not null and `fd` lies in `0 ..< size_of(fd_set) * 8`.
check_fd_set :: (fd: s32, set: *fd_set) -> bool {
    return set != null && fd >= 0 && fd < size_of(fd_set) * 8;
}

// Returns 1 when descriptor `fd` is in `set` and 0 when it is not. A null set, a negative descriptor and one past the
// set's capacity (usually 1024) count as not in the set. After `select` returns, use it to ask which descriptors are ready.
//
// ```jai,skip_on(wasm)
// set: fd_set;
//
// FD_SET(7, *set);
// assert(FD_ISSET(7, *set) == 1);
// assert(FD_ISSET(8, *set) == 0);
// assert(FD_ISSET(100000, *set) == 0);
// ```
FD_ISSET :: (fd: s32, set: *fd_set) -> s32 {
    if !check_fd_set(fd, set) return 0;
    bits := cast(*u8) set;
    if (bits[fd / 8] & ((cast(u8) 1) << (fd % 8))) != 0 return 1;
    return 0;
}

// Adds descriptor `fd` to `set`. A descriptor at or beyond the set's capacity (usually 1024 bits) is ignored without
// an error, so a program with many descriptors should poll with a different mechanism. A negative descriptor or a null
// set fails an assertion.
//
// ```jai,skip_on(wasm)
// set: fd_set;
//
// FD_SET(7, *set);
// FD_SET(34, *set);
//
// assert(FD_ISSET(7, *set) == 1 && FD_ISSET(34, *set) == 1);
// ```
FD_SET :: (fd: s32, set: *fd_set) {
    if fd >= size_of(fd_set) * 8 return;
    assert(check_fd_set(fd, set));
    (cast(*u8) set)[fd / 8] |= (cast(u8) 1) << (fd % 8);
}

// Removes descriptor `fd` from `set`. Clearing a descriptor that is not in the set, or one at or beyond the set's capacity,
// does nothing. A negative descriptor or a null set fails an assertion.
//
// ```jai,skip_on(wasm)
// set: fd_set;
//
// FD_SET(7, *set);
// FD_CLR(7, *set);
// FD_CLR(7, *set);
//
// assert(FD_ISSET(7, *set) == 0);
// ```
FD_CLR :: (fd: s32, set: *fd_set) {
    if fd >= size_of(fd_set) * 8 return;
    assert(check_fd_set(fd, set));
    (cast(*u8) set)[fd / 8] &= ~((cast(u8) 1) << (fd % 8));
}

#scope_file
#import "Basic";
P :: #import "internal/bindings/posix-api";
