// Sockets: the four calls WASI preview 1 has for a socket the host pre-opened (`sock_accept`,
// `sock_recv`, `sock_send`, `sock_shutdown`), under their C names. A program cannot create,
// bind or connect a socket here, so the Socket module reports those as unsupported itself.

#scope_module

wasi_sock_accept :: (fd: s32, flags: u32, accepted: u32) -> s32 #foreign wasi "sock_accept";
wasi_sock_recv :: (fd: s32, data: u32, data_count: u32, flags: u32, received: u32, received_flags: u32) -> s32 #foreign wasi "sock_recv";
wasi_sock_send :: (fd: s32, data: u32, data_count: u32, flags: u32, sent: u32) -> s32 #foreign wasi "sock_send";
wasi_sock_shutdown :: (fd: s32, how: u32) -> s32 #foreign wasi "sock_shutdown";

// The peer's address is not reported: WASI has no call for it, so the address buffer is left alone
// and its length set to 0.
#program_export "accept"
wasi_accept :: (fd: s32, address: *void, address_bytes: *u32) -> s32 #c_call {
    accepted: s32;
    code := wasi_sock_accept(fd, 0, offset32(*accepted));

    if code != 0 return failed(code);
    if address_bytes address_bytes.* = 0;

    return accepted;
}

// Linux `MSG_PEEK` and `MSG_WAITALL` become WASI's `recv_peek` and `recv_waitall`; other flags
// are not available.
#program_export "recv"
wasi_recv :: (fd: s32, buffer: *void, count: u64, flags: s32) -> s64 #c_call {
    if (flags & ~0x102) != 0 {
        error_number = EINVAL;
        return -1;
    }

    wasi_flags: u32;
    if flags & 0x2 wasi_flags |= 1;
    if flags & 0x100 wasi_flags |= 2;

    span := Span32.{ offset32(buffer), cast,no_check(u32) count };
    received: u32;
    received_flags: u32;
    code := wasi_sock_recv(fd, offset32(*span), 1, wasi_flags, offset32(*received), offset32(*received_flags));

    if code != 0 return failed(code);

    return received;
}

#program_export "send"
wasi_send :: (fd: s32, buffer: *void, count: u64, flags: s32) -> s64 #c_call {
    if flags != 0 {
        error_number = EINVAL;
        return -1;
    }

    span := Span32.{ offset32(buffer), cast,no_check(u32) count };
    sent: u32;
    code := wasi_sock_send(fd, offset32(*span), 1, 0, offset32(*sent));

    if code != 0 return failed(code);

    return sent;
}

// C's `SHUT_RD`, `SHUT_WR` and `SHUT_RDWR` (0, 1, 2) as WASI's bit set (1 for reading, 2 for writing).
#program_export "shutdown"
wasi_shutdown :: (fd: s32, how: s32) -> s32 #c_call {
    if how < 0 || how > 2 {
        error_number = EINVAL;
        return -1;
    }

    code := wasi_sock_shutdown(fd, cast(u32) (how + 1));

    if code != 0 return failed(code);

    return 0;
}
