// The operating system calls the engine and the network and file procedures make, and the numbers
// that differ between systems. Everything here returns a result of 0 or more, or the negative
// errno, the way the io_uring kernel interface does, so the backends can treat a failed call and a
// failed operation alike. The Windows versions live in win32.jai; on Windows the `E_*` numbers are
// the system's own (Win32 and Winsock error codes), so `last_platform_code()` stays meaningful.
//
// Sockets are created non-blocking and close-on-exec, and never raise SIGPIPE.

// Internal result for "this system has no implementation". Not an errno.
E_UNSUPPORTED :: 0x7fff_0001;

#if OS == .MACOS {
    E_PERM :: 1;
    E_NOENT :: 2;
    E_INTR :: 4;
    E_BADF :: 9;
    E_NOMEM :: 12;
    E_ACCES :: 13;
    E_EXIST :: 17;
    E_NOTDIR :: 20;
    E_ISDIR :: 21;
    E_INVAL :: 22;
    E_MFILE :: 24;
    E_NOSPC :: 28;
    E_PIPE :: 32;
    E_AGAIN :: 35;
    E_INPROGRESS :: 36;
    E_NOTSOCK :: 38;
    E_ADDRINUSE :: 48;
    E_ADDRNOTAVAIL :: 49;
    E_NETUNREACH :: 51;
    E_CONNABORTED :: 53;
    E_CONNRESET :: 54;
    E_NOTCONN :: 57;
    E_TIMEDOUT :: 60;
    E_CONNREFUSED :: 61;
    E_HOSTUNREACH :: 65;
    E_NOSYS :: 78;
    E_CANCELED :: 89;

    AF_INET6 :: 30;
    SOL_SOCKET :: 0xffff;
    SO_REUSEADDR :: 4;
    SO_REUSEPORT :: 0x200;
    SO_ERROR :: 0x1007;
    SO_NOSIGPIPE :: 0x1022;
    O_NONBLOCK :: 0x4;
    O_CREAT :: 0x200;
    O_TRUNC :: 0x400;
    O_EXCL :: 0x800;
    O_APPEND :: 0x8;
    O_CLOEXEC :: 0x1000000;
    SC_PROCESSORS_ONLINE :: 58;
} else #if OS == .WINDOWS {
    // Win32 and Winsock error numbers.
    E_PERM :: 1314;
    E_NOENT :: 2;
    E_INTR :: 10004;
    E_BADF :: 6;
    E_AGAIN :: 10035;
    E_NOMEM :: 8;
    E_ACCES :: 5;
    E_EXIST :: 80;
    E_NOTDIR :: 267;
    E_ISDIR :: 336;
    E_INVAL :: 87;
    E_MFILE :: 4;
    E_NOSPC :: 112;
    E_PIPE :: 109;
    E_NOSYS :: 120;
    E_NOTSOCK :: 10038;
    E_ADDRINUSE :: 10048;
    E_ADDRNOTAVAIL :: 10049;
    E_NETUNREACH :: 10051;
    E_CONNABORTED :: 10053;
    E_CONNRESET :: 10054;
    E_NOTCONN :: 10057;
    E_TIMEDOUT :: 10060;
    E_CONNREFUSED :: 10061;
    E_HOSTUNREACH :: 10065;
    E_INPROGRESS :: 997;
    E_CANCELED :: 995;

    AF_INET6 :: 23;
    SOL_SOCKET :: 0xffff;
    SO_REUSEADDR :: 4;
    // Windows has no SO_REUSEPORT; `sys_prepare_listener` does the equivalent.
    SO_REUSEPORT :: 0;
    SO_ERROR :: 0x1007;
    // Abstract open flags, translated by `sys_open`.
    O_NONBLOCK :: 0x800;
    O_CREAT :: 0x40;
    O_TRUNC :: 0x200;
    O_EXCL :: 0x80;
    O_APPEND :: 0x400;
    O_CLOEXEC :: 0x80000;
    SC_PROCESSORS_ONLINE :: 0;
} else {
    E_PERM :: 1;
    E_NOENT :: 2;
    E_INTR :: 4;
    E_BADF :: 9;
    E_AGAIN :: 11;
    E_NOMEM :: 12;
    E_ACCES :: 13;
    E_EXIST :: 17;
    E_NOTDIR :: 20;
    E_ISDIR :: 21;
    E_INVAL :: 22;
    E_MFILE :: 24;
    E_NOSPC :: 28;
    E_PIPE :: 32;
    E_NOSYS :: 38;
    E_NOTSOCK :: 88;
    E_ADDRINUSE :: 98;
    E_ADDRNOTAVAIL :: 99;
    E_NETUNREACH :: 101;
    E_CONNABORTED :: 103;
    E_CONNRESET :: 104;
    E_NOTCONN :: 107;
    E_TIMEDOUT :: 110;
    E_CONNREFUSED :: 111;
    E_HOSTUNREACH :: 113;
    E_INPROGRESS :: 115;
    E_CANCELED :: 125;

    AF_INET6 :: 10;
    SOL_SOCKET :: 1;
    SO_REUSEADDR :: 2;
    SO_REUSEPORT :: 15;
    SO_ERROR :: 4;
    O_NONBLOCK :: 0x800;
    O_CREAT :: 0x40;
    O_TRUNC :: 0x200;
    O_EXCL :: 0x80;
    O_APPEND :: 0x400;
    O_CLOEXEC :: 0x80000;
    SC_PROCESSORS_ONLINE :: 84;
}

AF_INET :: 2;
SOCK_STREAM :: 1;
IPPROTO_TCP :: 6;
TCP_NODELAY :: 1;
F_GETFL :: 3;
F_SETFL :: 4;
F_SETFD :: 2;
FD_CLOEXEC :: 1;
O_RDONLY :: 0;
O_WRONLY :: 1;
O_RDWR :: 2;
SHUT_RD :: 0;
SHUT_WR :: 1;

// A socket address of any family, as the kernel fills it in.
Sock_Storage :: struct {
    words: [16] u64;
}

// The most any call takes in one go, so that a 32-bit count never overflows.
MAX_TRANSFER :: 0x4000_0000;

#if OS != .WINDOWS {
    // `libc` is declared in stacks.jai.
    c_socket :: (family: s32, kind: s32, protocol: s32) -> s32 #foreign libc "socket";
    c_bind :: (fd: s32, address: *void, length: u32) -> s32 #foreign libc "bind";
    c_listen :: (fd: s32, backlog: s32) -> s32 #foreign libc "listen";
    c_connect :: (fd: s32, address: *void, length: u32) -> s32 #foreign libc "connect";
    c_accept :: (fd: s32, address: *void, length: *u32) -> s32 #foreign libc "accept";
    c_getsockname :: (fd: s32, address: *void, length: *u32) -> s32 #foreign libc "getsockname";
    c_getpeername :: (fd: s32, address: *void, length: *u32) -> s32 #foreign libc "getpeername";
    c_setsockopt :: (fd: s32, level: s32, name: s32, value: *void, length: u32) -> s32 #foreign libc "setsockopt";
    c_getsockopt :: (fd: s32, level: s32, name: s32, value: *void, length: *u32) -> s32 #foreign libc "getsockopt";
    c_shutdown :: (fd: s32, how: s32) -> s32 #foreign libc "shutdown";
    c_send :: (fd: s32, buffer: *void, length: u64, flags: s32) -> s64 #foreign libc "send";
    c_recv :: (fd: s32, buffer: *void, length: u64, flags: s32) -> s64 #foreign libc "recv";
    c_read :: (fd: s32, buffer: *void, length: u64) -> s64 #foreign libc "read";
    c_write :: (fd: s32, buffer: *void, length: u64) -> s64 #foreign libc "write";
    c_pread :: (fd: s32, buffer: *void, length: u64, offset: s64) -> s64 #foreign libc "pread";
    c_pwrite :: (fd: s32, buffer: *void, length: u64, offset: s64) -> s64 #foreign libc "pwrite";
    // open and fcntl are variadic in C, which matters on Apple arm64 (variadic arguments go on the stack).
    c_open :: (path: *u8, flags: s32, args: ..Any) -> s32 #foreign libc "open";
    c_close :: (fd: s32) -> s32 #foreign libc "close";
    c_fcntl :: (fd: s32, command: s32, args: ..Any) -> s32 #foreign libc "fcntl";
    c_pipe :: (fds: *s32) -> s32 #foreign libc "pipe";
    c_sysconf :: (name: s32) -> s64 #foreign libc "sysconf";
    c_lseek :: (fd: s32, offset: s64, whence: s32) -> s64 #foreign libc "lseek";

    #if OS == .MACOS {
        c_errno :: () -> *s32 #foreign libc "__error";
    } else {
        c_errno :: () -> *s32 #foreign libc "__errno_location";
        c_accept4 :: (fd: s32, address: *void, length: *u32, flags: s32) -> s32 #foreign libc "accept4";
    }

    errno_now :: () -> s32 {
        return c_errno().*;
    }

    // -errno when a call returned -1, else `result`.
    checked :: (result: s64) -> s64 {
        if result < 0 return -cast(s64) errno_now();

        return result;
    }

    // Makes `fd` non-blocking and close-on-exec.
    sys_nonblocking :: (fd: s32) -> s64 {
        flags := c_fcntl(fd, F_GETFL);
        if flags < 0 return -cast(s64) errno_now();

        if c_fcntl(fd, F_SETFL, cast(s32) (flags | O_NONBLOCK)) < 0 return -cast(s64) errno_now();
        if c_fcntl(fd, F_SETFD, cast(s32) FD_CLOEXEC) < 0 return -cast(s64) errno_now();

        return 0;
    }

    // Makes `fd` a socket the engine can use: close-on-exec, no SIGPIPE, and non-blocking when the
    // backend wants readiness (epoll, kqueue). io_uring does its own waiting and wants sockets
    // that block.
    prepare_socket :: (fd: s32, nonblocking: bool) -> s64 {
        if nonblocking {
            result := sys_nonblocking(fd);
            if result < 0 return result;
        } else {
            if c_fcntl(fd, F_SETFD, cast(s32) FD_CLOEXEC) < 0 return -cast(s64) errno_now();
        }

        #if OS == .MACOS {
            one: s32 = 1;
            c_setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, *one, 4);
        }

        return 0;
    }

    sys_socket :: (family: s32, nonblocking: bool) -> s64 {
        fd := c_socket(family, SOCK_STREAM, 0);
        if fd < 0 return -cast(s64) errno_now();

        result := prepare_socket(fd, nonblocking);
        if result < 0 {
            c_close(fd);

            return result;
        }

        return fd;
    }

    sys_set_option :: (fd: s64, level: s32, name: s32, value: s32) -> s64 {
        copy := value;

        return checked(c_setsockopt(cast(s32) fd, level, name, *copy, 4));
    }

    sys_bind :: (fd: s64, address: *Sock_Storage, length: u32) -> s64 {
        return checked(c_bind(cast(s32) fd, address, length));
    }

    sys_listen :: (fd: s64, backlog: s32) -> s64 {
        return checked(c_listen(cast(s32) fd, backlog));
    }

    sys_socket_name :: (fd: s64, address: *Sock_Storage, peer: bool) -> s64 {
        length: u32 = size_of(Sock_Storage);

        if peer return checked(c_getpeername(cast(s32) fd, address, *length));

        return checked(c_getsockname(cast(s32) fd, address, *length));
    }

    sys_shutdown :: (fd: s64, how: s32) -> s64 {
        return checked(c_shutdown(cast(s32) fd, how));
    }

    // The pending error of a socket (SO_ERROR), as a positive errno or 0.
    sys_socket_error :: (fd: s64) -> s64 {
        value: s32;
        length: u32 = 4;

        result := c_getsockopt(cast(s32) fd, SOL_SOCKET, SO_ERROR, *value, *length);

        if result < 0 return cast(s64) errno_now();

        return value;
    }

    sys_open :: (path: *u8, flags: s32, mode: u32) -> s64 {
        while true {
            fd := c_open(path, flags | O_CLOEXEC, cast(s32) mode);
            if fd >= 0 return fd;
            if errno_now() != E_INTR return -cast(s64) errno_now();
        }

        return 0;
    }

    sys_close :: (fd: s64) -> s64 {
        return checked(c_close(cast(s32) fd));
    }

    // A pipe whose read end is non-blocking when `nonblocking` (the backend waits for readiness);
    // the write end always blocks, which a one-byte wake-up never does for long.
    sys_pipe :: (fds: *[2] s32, nonblocking: bool) -> s64 {
        if c_pipe(fds.data) < 0 return -cast(s64) errno_now();

        result: s64;
        if nonblocking result = sys_nonblocking(fds.*[0]);
        else result = checked(c_fcntl(fds.*[0], F_SETFD, cast(s32) FD_CLOEXEC));

        if result >= 0 result = checked(c_fcntl(fds.*[1], F_SETFD, cast(s32) FD_CLOEXEC));

        if result < 0 {
            c_close(fds.*[0]);
            c_close(fds.*[1]);

            return result;
        }

        return 0;
    }

    // Closes a handle of any kind (sockets and files share one call on POSIX).
    sys_close_handle :: (handle: Handle) -> s64 {
        return sys_close(handle.fd);
    }

    // The options a listening socket gets before `bind`: the address may be reused after a restart,
    // and with `reuse_port` several listeners (one per thread) may share it.
    sys_prepare_listener :: (fd: s64, reuse_port: bool) -> s64 {
        result := sys_set_option(fd, SOL_SOCKET, SO_REUSEADDR, 1);

        if result >= 0 && reuse_port result = sys_set_option(fd, SOL_SOCKET, SO_REUSEPORT, 1);

        return result;
    }

    // The value of an environment variable, or "" if it is unset. `buffer` is scratch space that
    // the Windows version fills.
    env_text :: (name: *u8, buffer: *[256] u8) -> string {
        raw := c_getenv(name);
        if !raw return "";

        length := 0;
        while raw[length] length += 1;

        return to_string(raw, length);
    }

    sys_processors :: () -> s64 {
        count := c_sysconf(SC_PROCESSORS_ONLINE);
        if count < 1 return 1;

        return count;
    }

    // One attempt at each operation, never blocking. `E_AGAIN` means "not now".

    sys_accept :: (fd: s64, address: *Sock_Storage, length: *u32) -> s64 {
        while true {
            length.* = size_of(Sock_Storage);

            #if OS == .MACOS {
                accepted := c_accept(cast(s32) fd, address, length);
            } else {
                accepted := c_accept4(cast(s32) fd, address, length, O_NONBLOCK | O_CLOEXEC);
            }

            if accepted >= 0 {
                #if OS == .MACOS {
                    result := prepare_socket(accepted, true);
                    if result < 0 {
                        c_close(accepted);

                        return result;
                    }
                }

                return accepted;
            }

            if errno_now() != E_INTR return -cast(s64) errno_now();
        }

        return 0;
    }

    sys_connect :: (fd: s64, address: *Sock_Storage, length: u32) -> s64 {
        while true {
            if c_connect(cast(s32) fd, address, length) == 0 return 0;
            if errno_now() != E_INTR return -cast(s64) errno_now();
        }

        return 0;
    }

    sys_recv :: (fd: s64, buffer: *void, length: s64) -> s64 {
        while true {
            result := c_recv(cast(s32) fd, buffer, cast(u64) min(length, MAX_TRANSFER), 0);
            if result >= 0 return result;
            if errno_now() != E_INTR return -cast(s64) errno_now();
        }

        return 0;
    }

    sys_send :: (fd: s64, buffer: *void, length: s64) -> s64 {
        while true {
            #if OS == .MACOS flags: s32 = 0;
            else flags: s32 = 0x4000; // MSG_NOSIGNAL
            result := c_send(cast(s32) fd, buffer, cast(u64) min(length, MAX_TRANSFER), flags);
            if result >= 0 return result;
            if errno_now() != E_INTR return -cast(s64) errno_now();
        }

        return 0;
    }

    // Reads at `offset`, or at the current position when it is negative.
    sys_read :: (fd: s64, buffer: *void, length: s64, offset: s64) -> s64 {
        while true {
            count := cast(u64) min(length, MAX_TRANSFER);
            result: s64;

            if offset >= 0 result = c_pread(cast(s32) fd, buffer, count, offset);
            else result = c_read(cast(s32) fd, buffer, count);

            if result >= 0 return result;
            if errno_now() != E_INTR return -cast(s64) errno_now();
        }

        return 0;
    }

    sys_write :: (fd: s64, buffer: *void, length: s64, offset: s64) -> s64 {
        while true {
            count := cast(u64) min(length, MAX_TRANSFER);
            result: s64;

            if offset >= 0 result = c_pwrite(cast(s32) fd, buffer, count, offset);
            else result = c_write(cast(s32) fd, buffer, count);

            if result >= 0 return result;
            if errno_now() != E_INTR return -cast(s64) errno_now();
        }

        return 0;
    }

    sys_file_size :: (fd: s64) -> s64 {
        end := c_lseek(cast(s32) fd, 0, 2);
        if end < 0 return -cast(s64) errno_now();

        return end;
    }
}
