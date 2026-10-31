// test: only(.WASM)
// The parts of Socket that exist under WASI: addresses, byte order, text parsing and descriptor sets work; the calls that
// would make a socket fail with -1 and `ENOSYS` instead of failing to compile.
#import "Basic";
S :: #import "Socket";

ENOSYS :: 38;

unsupported :: (result: s64, what: string) {
    assert(result == -1, "% should fail", what);
    assert(cast(s32) S.get_last_socket_error() == ENOSYS, "% should report ENOSYS, got %", what, cast(s32) S.get_last_socket_error());
}

main :: () {
    assert(S.socket_init());

    unsupported(S.socket(S.AF_INET, .SOCK_STREAM, .TCP), "socket");
    unsupported(S.socket(S.AF_INET6, .SOCK_DGRAM, .UDP), "datagram socket");
    unsupported(S.bind(S.INVALID_SOCKET, "127.0.0.1", 0), "bind");
    unsupported(S.listen(S.INVALID_SOCKET, 4), "listen");

    address: S.sockaddr_in;
    length: S.socklen_t = size_of(S.sockaddr_in);

    unsupported(S.connect(S.INVALID_SOCKET, cast(*S.sockaddr) *address, length), "connect");
    unsupported(S.getsockname(S.INVALID_SOCKET, cast(*S.sockaddr) *address, *length), "getsockname");
    unsupported(S.setsockopt(S.INVALID_SOCKET, S.SOL_SOCKET, S.SO_REUSEADDR, *length, 4), "setsockopt");
    assert(!S.set_blocking(S.INVALID_SOCKET, false));
    assert(cast(s32) S.get_last_socket_error() == ENOSYS);

    // An address that cannot be parsed is refused with `EINVAL` before any system call.
    assert(S.bind(S.INVALID_SOCKET, "not an address", 80) == -1);
    assert(cast(s32) S.get_last_socket_error() == 22);

    // Closing a handle that is already closed does nothing.
    closed := S.INVALID_SOCKET;

    S.close_and_reset(*closed);
    assert(closed == S.INVALID_SOCKET);

    // Byte order and text.
    assert(S.htons(0x1234) == 0x3412 && S.ntohs(0x3412) == 0x1234);
    assert(S.htonl(0x7f000001) == 0x0100007f && S.ntohl(0x0100007f) == 0x7f000001);

    assert(S.inet_pton(S.AF_INET, "127.0.0.1", *address.sin_addr) == 1);
    assert(S.htonl(S.INADDR_LOOPBACK) == address.sin_addr.s_addr);
    assert(S.inet_pton(S.AF_INET, "1.2.3", *address.sin_addr) == 0);
    assert(S.inet_pton(S.AF_INET, "256.0.0.1", *address.sin_addr) == 0);
    assert(S.inet_pton(S.AF_INET, "1.2.3.4.5", *address.sin_addr) == 0);
    assert(S.inet_pton(S.AF_INET, "", *address.sin_addr) == 0);

    bytes: [16] u8;

    assert(S.inet_pton(S.AF_INET6, "::1", bytes.data) == 1);
    for 0..14 assert(bytes[it] == 0);

    assert(bytes[15] == 1);
    assert(S.inet_pton(S.AF_INET6, "2001:db8::ff00:42:8329", bytes.data) == 1);
    assert(bytes[0] == 0x20 && bytes[1] == 0x01 && bytes[2] == 0x0d && bytes[3] == 0xb8 && bytes[4] == 0 && bytes[10] == 0xff && bytes[15] == 0x29);
    assert(S.inet_pton(S.AF_INET6, "::ffff:1.2.3.4", bytes.data) == 1);
    assert(bytes[10] == 0xff && bytes[11] == 0xff && bytes[12] == 1 && bytes[15] == 4);
    assert(S.inet_pton(S.AF_INET6, "1:2:3:4:5:6:7:8", bytes.data) == 1 && bytes[15] == 8);
    assert(S.inet_pton(S.AF_INET6, "1:2:3:4:5:6:7", bytes.data) == 0);
    assert(S.inet_pton(S.AF_INET6, "1::2::3", bytes.data) == 0);
    assert(S.inet_pton(S.AF_INET6, "12345::", bytes.data) == 0);
    assert(S.inet_pton(S.AF_INET6, ":1", bytes.data) == 0);
    assert(S.inet_pton(S.AF_INET6, "1:", bytes.data) == 0);

    assert(S.inet_pton(S.AF_INET, "127.0.0.1", *address.sin_addr) == 1);
    address.sin_family = S.AF_INET;
    address.sin_port = S.htons(8080);
    text := S.to_string(address);

    defer free(text);
    assert(text == "127.0.0.1:8080", "got %", text);

    v6: S.sockaddr_in6;

    v6.sin6_family = S.AF_INET6;
    v6.sin6_port = S.htons(443);
    assert(S.inet_pton(S.AF_INET6, "::1", *v6.sin6_addr) == 1);
    text6 := S.to_string(v6);

    defer free(text6);
    assert(text6 == "[::1]:443", "got %", text6);

    // Descriptor sets.
    set: S.fd_set;

    S.FD_SET(7, *set);
    assert(S.FD_ISSET(7, *set) == 1 && S.FD_ISSET(8, *set) == 0);
    S.FD_CLR(7, *set);
    assert(S.FD_ISSET(7, *set) == 0);

    print("ok\n");
}
