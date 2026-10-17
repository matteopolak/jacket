Socket :: s32;
INVALID_SOCKET: Socket : -1;
SOCKET_ERROR :: -1;
SOCKET_WOULDBLOCK :: cast(P.OS_Error_Code) P.EWOULDBLOCK;
SOCKET_WOULDBLOCK_CONNECT :: cast(P.OS_Error_Code) P.EINPROGRESS;

socket_init :: inline () -> bool {
    return true;
}

get_last_socket_error :: inline () -> P.OS_Error_Code {
    return P.errno();
}

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

close_and_reset :: (s: *Socket, loc := #caller_location) {
    if s.* != INVALID_SOCKET {
        if P.close(s.*) != 0 report_error();
        s.* = INVALID_SOCKET;
    }
}

FD_BITS_ELEMENT_TYPE :: type_of(fd_set.fds_bits[0]);
BITS_PER_FD_SET_MASK :: size_of(FD_BITS_ELEMENT_TYPE) * 8;

check_fd_set :: (fd: s32, set: *fd_set) -> bool {
    return set != null && fd >= 0 && fd < size_of(fd_set) * 8;
}

FD_ISSET :: (fd: s32, set: *fd_set) -> s32 {
    if !check_fd_set(fd, set) return 0;
    bits := cast(*u8) set;
    if (bits[fd / 8] & ((cast(u8) 1) << (fd % 8))) != 0 return 1;
    return 0;
}

FD_SET :: (fd: s32, set: *fd_set) {
    if fd >= size_of(fd_set) * 8 return;
    assert(check_fd_set(fd, set));
    (cast(*u8) set)[fd / 8] |= (cast(u8) 1) << (fd % 8);
}

FD_CLR :: (fd: s32, set: *fd_set) {
    if fd >= size_of(fd_set) * 8 return;
    assert(check_fd_set(fd, set));
    (cast(*u8) set)[fd / 8] &= ~((cast(u8) 1) << (fd % 8));
}

#scope_file
#import "Basic";
P :: #import "POSIX";
