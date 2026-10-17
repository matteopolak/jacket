Socket :: SOCKET;
INVALID_SOCKET :: cast(Socket) 0xffffffffffffffff;
SOCKET_ERROR :: -1;
SOCKET_WOULDBLOCK :: WSAEWOULDBLOCK;
SOCKET_WOULDBLOCK_CONNECT :: SOCKET_WOULDBLOCK;

IOW :: Control.IOW;

// Official Winsock2 FIONBIO encoding; the character literal form is incomplete.
FIONBIO :: #run IOW(102, 126, u32);

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

get_last_socket_error :: inline () -> W.OS_Error_Code {
    return WSAGetLastError();
}

set_blocking :: (s: Socket, blocking: bool) -> bool {
    nonblocking: u32 = 1;
    if blocking nonblocking = 0;

    if ioctlsocket(s, FIONBIO, *nonblocking) != 0 {
        report_error();
        return false;
    }

    return true;
}

set_keepalive :: (s: Socket, keepalive: bool, idle_seconds_before_keepalive: s32 = 0, keepalive_interval_seconds: s32 = 0, num_keepalive_attempts: s32 = 0) -> bool {
    enabled: s32;
    if keepalive enabled = 1;

    if setsockopt(s, SOL_SOCKET, SO_KEEPALIVE, cast(*u8) *enabled, 4) != 0 {
        report_error();
        return false;
    }

    if !keepalive return true;

    // Reviewed Windows compatibility: these values are native milliseconds,
    // and num_keepalive_attempts is ignored on this platform.
    options := tcp_keepalive.{ 1, cast(u32) idle_seconds_before_keepalive, cast(u32) keepalive_interval_seconds };
    returned: u32;

    if WSAIoctl(s, SIO_KEEPALIVE_VALS, *options, size_of(tcp_keepalive), null, 0, *returned, null, null) != 0 {
        report_error();
        return false;
    }

    return true;
}

close_and_reset :: (s: *Socket, loc := #caller_location) {
    if s.* != INVALID_SOCKET {
        status := closesocket(s.*);
        if status != 0 report_error();
        s.* = INVALID_SOCKET;
    }
}

FD_SET :: (fd: Socket, set: *fd_set) {
    Control.add_descriptor(fd, set);
}

FD_CLR :: (fd: Socket, set: *fd_set) {
    Control.remove_descriptor(fd, set);
}

#scope_file
#import "Basic";
W :: #import "Windows";

Control :: #import "internal/net/winsock-control";
