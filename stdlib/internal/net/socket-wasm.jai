// The Socket module on WASI preview 1. The record layouts and constants are Linux's, as in the
// runtime's file calls. WASI can only use a socket the host pre-opened: `accept`, `recv`, `send`,
// `shutdown` and `close_and_reset` work on one. Everything that would create or set up a socket
// (`socket`, `bind`, `connect`, `listen`, `setsockopt`, ...) cannot exist there, so each returns -1
// with the error `ENOSYS`, as a call the system does not have would.

sa_family_t :: u16;
in_port_t :: u16;
in_addr_t :: u32;
socklen_t :: u32;

// A socket handle: the file descriptor. Compare a handle against `INVALID_SOCKET`, never against a number.
Socket :: s32;
INVALID_SOCKET: Socket : -1;
SOCKET_ERROR :: -1;

// The error `get_last_socket_error()` reports when a non-blocking call has nothing to do yet.
SOCKET_WOULDBLOCK :: cast(Wasi.OS_Error_Code) 11;
// The error for a non-blocking `connect` under way. WASI cannot connect, so nothing reports it.
SOCKET_WOULDBLOCK_CONNECT :: cast(Wasi.OS_Error_Code) 115;
// The error every call that cannot exist on WASI reports.
SOCKET_UNSUPPORTED :: cast(Wasi.OS_Error_Code) Wasi.ENOSYS;

AF_UNIX :: 1;
AF_INET :: 2;
AF_INET6 :: 10;

INADDR_ANY :: cast(in_addr_t) 0x00000000;
INADDR_LOOPBACK :: cast(in_addr_t) 0x7f000001;

SOL_SOCKET :: 1;
SO_REUSEADDR :: 2;
SO_ERROR :: 4;
SO_KEEPALIVE :: 9;
TCP_KEEPIDLE :: 4;
TCP_KEEPINTVL :: 5;
TCP_KEEPCNT :: 6;

SOCK :: enum u32 {
    STREAM :: 1;
    DGRAM :: 2;
    NONBLOCK :: 2048;
    CLOEXEC :: 524288;

    SOCK_STREAM :: STREAM;
    SOCK_DGRAM :: DGRAM;
}

IPPROTO :: enum u32 {
    IP :: 0;
    TCP :: 6;
    UDP :: 17;
    IPV6 :: 41;
}

// Only `PEEK` and `WAITALL` are passed on by `recv`; `send` takes no flags.
MSG :: enum_flags s32 {
    PEEK :: 0x2;
    WAITALL :: 0x100;
}

// The directions `shutdown` can close: `RD` stops receiving, `WR` stops sending and `RDWR` closes both.
SHUT :: enum u32 {
    RD :: 0;
    WR :: 1;
    RDWR :: 2;
    SHUT_RD :: RD;
    SHUT_WR :: WR;
    SHUT_RDWR :: RDWR;
}

// The generic socket address header: a family and 14 bytes of family-specific data.
sockaddr :: struct {
    sa_family: sa_family_t;
    sa_data: [14] u8;
}

// An IPv4 address: `s_addr` holds the four octets in network byte order, the first octet at the lowest address.
in_addr :: struct {
    s_addr: in_addr_t;
}

in6_addr :: struct {
    s6_addr: [16] u8;
}

sockaddr_in :: struct {
    sin_family: sa_family_t;
    sin_port: in_port_t;
    sin_addr: in_addr;
    sin_zero: [8] u8;
}

sockaddr_in6 :: struct {
    sin6_family: sa_family_t;
    sin6_port: in_port_t;
    sin6_flowinfo: u32;
    sin6_addr: in6_addr;
    sin6_scope_id: u32;
}

timeval :: struct {
    tv_sec: s64;
    tv_usec: s64;
}

fd_set :: struct {
    fds_bits: [16] s64;
}

htons :: (value: u16) -> u16 {
    return host_port(value);
}

ntohs :: (value: u16) -> u16 {
    return host_port(value);
}

htonl :: (value: u32) -> u32 {
    return (value >> 24) | ((value >> 8) & 0xff00) | ((value << 8) & 0xff0000) | (value << 24);
}

ntohl :: (value: u32) -> u32 {
    return htonl(value);
}

// Reads the text `source` as an address of the family `family` (`AF_INET` or `AF_INET6`) into `destination`
// (4 or 16 bytes). Returns 1 on success, 0 when the text is not an address, -1 for another family.
inet_pton :: (family: s32, source: *u8, destination: *void) -> s32 {
    text := to_string(source);

    if family == AF_INET return ifx parse_ipv4(text, destination) then cast(s32) 1 else 0;
    if family == AF_INET6 return ifx parse_ipv6(text, destination) then cast(s32) 1 else 0;

    Wasi.set_errno(cast(Wasi.OS_Error_Code) Wasi.EAFNOSUPPORT);

    return -1;
}

// The calls WASI has, for a socket the host pre-opened (Extensions/Wasi_Runtime answers them).
accept :: (fd: s32, address: *sockaddr, length: *socklen_t) -> s32 #foreign libc;
recv :: (fd: s32, buffer: *void, count: u64, flags: MSG) -> s64 #foreign libc;
send :: (fd: s32, buffer: *void, count: u64, flags: MSG) -> s64 #foreign libc;
shutdown :: (fd: s32, how: SHUT) -> s32 #foreign libc;

// The calls WASI does not have.
socket :: (domain: s32, kind: SOCK, protocol: IPPROTO) -> s32 {
    return unsupported();
}

bind :: (fd: s32, address: *sockaddr, length: socklen_t) -> s32 {
    return unsupported();
}

listen :: (fd: s32, backlog: s32) -> s32 {
    return unsupported();
}

connect :: (fd: s32, address: *sockaddr, length: socklen_t) -> s32 {
    return unsupported();
}

getsockname :: (fd: s32, address: *sockaddr, length: *socklen_t) -> s32 {
    return unsupported();
}

getpeername :: (fd: s32, address: *sockaddr, length: *socklen_t) -> s32 {
    return unsupported();
}

setsockopt :: (fd: s32, level: s32, name: s32, value: *void, length: socklen_t) -> s32 {
    return unsupported();
}

getsockopt :: (fd: s32, level: s32, name: s32, value: *void, length: *socklen_t) -> s32 {
    return unsupported();
}

sendto :: (fd: s32, buffer: *void, count: u64, flags: MSG, address: *sockaddr, length: socklen_t) -> s64 {
    return cast(s64) unsupported();
}

recvfrom :: (fd: s32, buffer: *void, count: u64, flags: MSG, address: *sockaddr, length: *socklen_t) -> s64 {
    return cast(s64) unsupported();
}

select :: (count: s32, readers: *fd_set, writers: *fd_set, failures: *fd_set, timeout: *timeval) -> s32 {
    return unsupported();
}

// Prepares the socket layer and returns whether it is ready. There is nothing to prepare.
socket_init :: inline () -> bool {
    return true;
}

// The error code of the last failed socket call: `errno`.
get_last_socket_error :: inline () -> Wasi.OS_Error_Code {
    return Wasi.errno();
}

// Switches `s` between blocking and non-blocking mode. WASI cannot change it, so this is `false`, with the error `ENOSYS`
// (and the failure logged, as everywhere else).
set_blocking :: (s: Socket, blocking: bool) -> bool {
    unsupported();
    report_error();

    return false;
}

// Turns TCP keep-alive probes on `s` on or off. WASI has no socket options, so this is `false`, with the error `ENOSYS`
// (and the failure logged).
set_keepalive :: (s: Socket, keepalive: bool, idle_seconds_before_keepalive: s32 = 0, keepalive_interval_seconds: s32 = 0, num_keepalive_attempts: s32 = 0) -> bool {
    unsupported();
    report_error();

    return false;
}

// Closes the socket `s` points to and sets it to `INVALID_SOCKET`. A handle that is already `INVALID_SOCKET` is left alone.
close_and_reset :: (s: *Socket, loc := #caller_location) {
    if s.* != INVALID_SOCKET {
        if Wasi.close(s.*) != 0 report_error();
        s.* = INVALID_SOCKET;
    }
} @FreesArgument

FD_BITS_ELEMENT_TYPE :: type_of(fd_set.fds_bits[0]);

BITS_PER_FD_SET_MASK :: size_of(FD_BITS_ELEMENT_TYPE) * 8;

check_fd_set :: (fd: s32, set: *fd_set) -> bool {
    return set != null && fd >= 0 && fd < size_of(fd_set) * 8;
}

// Whether descriptor `fd` is in `set`; 0 for a null set and for a descriptor outside the set.
FD_ISSET :: (fd: s32, set: *fd_set) -> s32 {
    if !check_fd_set(fd, set) return 0;
    bits := cast(*u8) set;
    if (bits[fd / 8] & ((cast(u8) 1) << (fd % 8))) != 0 return 1;
    return 0;
}

// Adds descriptor `fd` to `set`; one past the set's capacity is ignored.
FD_SET :: (fd: s32, set: *fd_set) {
    if fd >= size_of(fd_set) * 8 return;
    assert(check_fd_set(fd, set));
    (cast(*u8) set)[fd / 8] |= (cast(u8) 1) << (fd % 8);
}

// Removes descriptor `fd` from `set`.
FD_CLR :: (fd: s32, set: *fd_set) {
    if fd >= size_of(fd_set) * 8 return;
    assert(check_fd_set(fd, set));
    (cast(*u8) set)[fd / 8] &= ~((cast(u8) 1) << (fd % 8));
}

#scope_module
// The result of a `bind` that was refused before reaching the system: `-1`, with the socket error set to invalid argument.
refuse_address :: () -> s32 {
    Wasi.set_errno(cast(Wasi.OS_Error_Code) Wasi.EINVAL);

    return -1;
}

#scope_file
Wasi :: #import "internal/files/wasi-libc";
#import "Basic";

libc :: #system_library "libc"; @native(external)

unsupported :: () -> s32 {
    Wasi.set_errno(SOCKET_UNSUPPORTED);

    return -1;
}

parse_ipv4 :: (text: string, destination: *void) -> bool {
    bytes := cast(*u8) destination;
    part := 0;
    digits := 0;
    value := 0;

    if text.count == 0 return false;

    for 0..text.count - 1 {
        c := text[it];

        if c >= 48 && c <= 57 {
            value = value * 10 + (cast(s64) c - 48);
            digits += 1;
            if digits > 3 || value > 255 return false;
        } else if c == 46 {
            if digits == 0 || part >= 3 return false;

            bytes[part] = cast(u8) value;
            part += 1;
            digits = 0;
            value = 0;
        } else {
            return false;
        }
    }

    if digits == 0 || part != 3 return false;

    bytes[3] = cast(u8) value;

    return true;
}

hex_digit :: (c: u8) -> s64 {
    if c >= 48 && c <= 57 return cast(s64) c - 48;
    if c >= 97 && c <= 102 return cast(s64) c - 87;
    if c >= 65 && c <= 70 return cast(s64) c - 55;
    return -1;
}

// Groups of up to four hex digits separated by colons, with at most one `::` standing for zero groups, and an optional
// dotted IPv4 tail in place of the last two groups.
parse_ipv6 :: (text: string, destination: *void) -> bool {
    groups: [8] u16;
    count := 0;
    gap := -1;
    at := 0;

    if text.count == 0 return false;

    if text.count >= 2 && text[0] == 58 && text[1] == 58 {
        gap = 0;
        at = 2;
    } else if text[0] == 58 {
        return false;
    }

    while at < text.count {
        start := at;
        value := 0;

        while at < text.count && hex_digit(text[at]) >= 0 {
            value = value * 16 + hex_digit(text[at]);
            at += 1;
        }

        digits := at - start;

        if at < text.count && text[at] == 46 {
            if count > 6 return false;

            tail: [4] u8;
            if !parse_ipv4(string.{ text.count - start, text.data + start }, tail.data) return false;

            groups[count] = ((cast(u16) tail[0]) << 8) | tail[1];
            groups[count + 1] = ((cast(u16) tail[2]) << 8) | tail[3];
            count += 2;
            break;
        }

        if digits == 0 || digits > 4 || count >= 8 return false;

        groups[count] = cast(u16) value;
        count += 1;

        if at == text.count break;
        if text[at] != 58 return false;

        at += 1;

        if at < text.count && text[at] == 58 {
            if gap >= 0 return false;
            gap = count;
            at += 1;
        } else if at == text.count {
            return false;
        }
    }

    bytes := cast(*u8) destination;
    result: [8] u16;

    if gap < 0 {
        if count != 8 return false;
        result = groups;
    } else {
        if count >= 8 return false;
        if gap > 0 for 0..gap - 1 result[it] = groups[it];
        if count > gap for 0..count - gap - 1 result[8 - (count - gap) + it] = groups[gap + it];
    }

    for 0..7 {
        bytes[it * 2] = cast(u8) (result[it] >> 8);
        bytes[it * 2 + 1] = cast(u8) (result[it] & 255);
    }

    return true;
}
