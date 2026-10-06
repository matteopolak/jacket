// Socket over loopback with OS-assigned ports: TCP listen/connect/accept, a 300 KiB echo pumped
// through non-blocking sockets with select, partial and peeked reads, end of stream, a refused
// connection, a send to a closed peer, UDP datagrams, IPv6 when the host has ::1, and the pure
// helpers (byte order, address formatting, fd_set and control-message macros).
//
// WASM targets have neither sockets nor libc's byte-order functions, so this does not run there.
#import "Basic";

#if OS == .WASM {
    main :: () {
        print("ok\n");
    }
} else {
    #import "Socket";
    #if OS != .WINDOWS #import "POSIX";

    main :: () {
        assert(socket_init(), "socket_init");
        check_byte_order();
        check_ipv4_formatting();
        check_ipv6_formatting();
        check_fd_set();
        #if OS != .WINDOWS check_control_message_macros();
        #if OS == .WINDOWS assert(IOW(#char "f", 126, u32) == FIONBIO, "IOW('f', 126, u32) is FIONBIO");
        check_tcp();
        check_refused_connection();
        check_closed_peer();
        check_udp();
        check_ipv6_loopback();
        check_bad_arguments();
        #if OS != .WINDOWS check_descriptor_passing();
        print("ok\n");
    }

    check_byte_order :: () {
        short := htons(0x1234);
        bytes := cast(*u8) *short;
        assert(bytes[0] == 0x12 && bytes[1] == 0x34, "htons stores big-endian: % %", bytes[0], bytes[1]);
        assert(ntohs(short) == 0x1234);
        long := htonl(0x0a0b0c0d);
        long_bytes := cast(*u8) *long;
        assert(long_bytes[0] == 0x0a && long_bytes[1] == 0x0b && long_bytes[2] == 0x0c && long_bytes[3] == 0x0d, "htonl");
        assert(ntohl(long) == 0x0a0b0c0d);
        assert(ntohs(htons(0)) == 0 && ntohs(htons(0xffff)) == 0xffff);
        assert(ntohl(htonl(0xffff_ffff)) == 0xffff_ffff && ntohl(htonl(1)) == 1);
        for value: cast(u16) 0..cast(u16) 300 assert(ntohs(htons(value * 211)) == value * 211);
    }

    ipv4 :: (a: u8, b: u8, c: u8, d: u8, port: u16) -> sockaddr_in {
        address: sockaddr_in;
        address.sin_family = AF_INET;
        address.sin_port = htons(port);
        bits := htonl(((cast(u32) a) << 24) | ((cast(u32) b) << 16) | ((cast(u32) c) << 8) | d);
        #if OS == .WINDOWS address.sin_addr.S_un.S_addr = bits;
        else address.sin_addr.s_addr = bits;
        return address;
    }

    check_ipv4_formatting :: () {
        assert(to_string(ipv4(127, 0, 0, 1, 8080)) == "127.0.0.1:8080", "got %", to_string(ipv4(127, 0, 0, 1, 8080)));
        assert(to_string(ipv4(0, 0, 0, 0, 0)) == "0.0.0.0:0");
        assert(to_string(ipv4(255, 255, 255, 255, 65535)) == "255.255.255.255:65535");
        assert(to_string(ipv4(10, 200, 3, 45, 7)) == "10.200.3.45:7");
        assert(ipv4_bits(ipv4(127, 0, 0, 1, 0)) == htonl(0x7f000001));
    }

    ipv6 :: (words: [8] u16, port: u16) -> sockaddr_in6 {
        address: sockaddr_in6;
        address.sin6_family = AF_INET6;
        address.sin6_port = htons(port);
        bytes: [16] u8;
        for words {
            bytes[it_index * 2] = cast(u8) (it >> 8);
            bytes[it_index * 2 + 1] = cast(u8) it;
        }
        memcpy(*address.sin6_addr, bytes.data, 16);
        return address;
    }

    check_ipv6_formatting :: () {
        Case :: struct {
            words: [8] u16;
            port: u16;
            text: string;
        }
        cases :: Case.[
            .{ .[0, 0, 0, 0, 0, 0, 0, 1], 443, "[::1]:443" },
            .{ .[0, 0, 0, 0, 0, 0, 0, 0], 0, "[::]:0" },
            .{ .[0x2001, 0xdb8, 0, 0, 0, 0, 0, 1], 80, "[2001:db8::1]:80" },
            .{ .[0xfe80, 0, 0, 0, 0x1234, 0, 0, 0], 1, "[fe80::1234:0:0:0]:1" },
            // A single zero word is not shortened.
            .{ .[1, 0, 2, 3, 4, 5, 6, 7], 2, "[1:0:2:3:4:5:6:7]:2" },
            // Of two equally long runs of zeros, the first is shortened.
            .{ .[1, 0, 0, 2, 0, 0, 3, 4], 3, "[1::2:0:0:3:4]:3" },
            // The longer run wins even when it comes second.
            .{ .[1, 0, 0, 2, 0, 0, 0, 4], 4, "[1:0:0:2::4]:4" },
            .{ .[0, 0, 0, 0, 0, 0xffff, 0x7f00, 1], 5, "[::ffff:7f00:1]:5" },
            .{ .[0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff], 65535, "[ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff]:65535" },
            .{ .[1, 2, 3, 4, 5, 6, 7, 0], 6, "[1:2:3:4:5:6:7:0]:6" },
            .{ .[1, 0, 0, 0, 0, 0, 0, 0], 7, "[1::]:7" },
        ];
        for cases {
            text := to_string(ipv6(it.words, it.port));
            assert(text == it.text, "case %: % should be %", it_index, text, it.text);
        }
    }

    check_fd_set :: () {
        set: fd_set;
        #if OS == .WINDOWS {
            a: Socket = 0x104;
            b: Socket = 0x208;
            c: Socket = 0x30c;
            FD_SET(a, *set);
            FD_SET(b, *set);
            FD_SET(b, *set);
            assert(set.fd_count == 2, "a socket is in the set once: %", set.fd_count);
            FD_SET(c, *set);
            assert(FD_ISSET(a, *set) != 0 && FD_ISSET(b, *set) != 0 && FD_ISSET(c, *set) != 0);
            FD_CLR(a, *set);
            assert(set.fd_count == 2 && FD_ISSET(a, *set) == 0 && FD_ISSET(b, *set) != 0 && FD_ISSET(c, *set) != 0);
            FD_CLR(a, *set);
            assert(set.fd_count == 2);
            // The set holds at most fd_array.count sockets; further adds are ignored.
            full: fd_set;
            for i: 0..set.fd_array.count + 3 FD_SET(cast(Socket) (i + 1) * 4, *full);
            assert(full.fd_count == full.fd_array.count, "a full set holds %", full.fd_count);
        } else {
            // Descriptors in every word of the set, including each word's top bit.
            bits :: size_of(fd_mask) * 8;
            limit :: size_of(fd_set) * 8;
            descriptors :: s32.[0, 1, 5, bits - 1, bits, bits + 1, 2 * bits - 1, 100, limit - 1];
            for descriptors FD_SET(it, *set);
            for fd: 0..limit - 1 {
                expected := false;
                for descriptors if it == fd expected = true;
                assert((FD_ISSET(cast(s32) fd, *set) != 0) == expected, "descriptor % in the set: %", fd, !expected);
            }
            for descriptors {
                FD_CLR(it, *set);
                assert(FD_ISSET(it, *set) == 0, "descriptor % after FD_CLR", it);
            }
            for set.fds_bits assert(it == 0, "word % left %", it_index, it);
            // Descriptors outside the set are ignored rather than written out of bounds.
            FD_SET(-1, *set);
            FD_SET(limit, *set);
            FD_SET(limit + 1000, *set);
            for set.fds_bits assert(it == 0, "an out-of-range descriptor touched word %", it_index);
            assert(FD_ISSET(-1, *set) == 0 && FD_ISSET(limit, *set) == 0);
            assert(FD_ISSET(3, null) == 0);
            FD_SET(3, null);
            FD_CLR(3, null);
        }
    }

    #if OS != .WINDOWS {
        check_control_message_macros :: () {
            header_size := size_of(cmsghdr);
            alignment := size_of(type_of(cmsghdr.cmsg_len));
            aligned_header := (header_size + alignment - 1) / alignment * alignment;
            assert(CMSG_ALIGN(0) == 0 && CMSG_ALIGN(1) == alignment && CMSG_ALIGN(alignment) == alignment && CMSG_ALIGN(alignment + 1) == 2 * alignment);
            assert(CMSG_LEN(4) == cast(u32) (aligned_header + 4), "CMSG_LEN(4) %", CMSG_LEN(4));
            assert(CMSG_SIZE(4) == cast(u32) (aligned_header + CMSG_ALIGN(4)), "CMSG_SIZE(4) %", CMSG_SIZE(4));
            assert(CMSG_SIZE(0) == cast(u32) aligned_header);
            #if OS == .LINUX assert(CMSG_LEN(4) == 20 && CMSG_SIZE(4) == 24, "glibc's 64-bit CMSG_LEN/CMSG_SPACE");
            #if OS == .MACOS assert(CMSG_LEN(4) == 16 && CMSG_SIZE(4) == 16, "Darwin's CMSG_LEN/CMSG_SPACE");

            message: msghdr;
            assert(CMSG_FIRSTHDR(null) == null);
            assert(CMSG_FIRSTHDR(*message) == null, "no control buffer");
            buffer: [64] u8;
            message.msg_control = buffer.data;
            message.msg_controllen = xx (header_size - 1);
            assert(CMSG_FIRSTHDR(*message) == null, "a control buffer smaller than a header");
            message.msg_controllen = xx buffer.count;
            first := CMSG_FIRSTHDR(*message);
            assert(cast(*u8) first == buffer.data);
            assert(CMSG_DATA(first) == buffer.data + aligned_header);
        }
    }

    make_tcp_socket :: () -> Socket {
        s := socket(AF_INET, .SOCK_STREAM, .TCP);
        assert(s != INVALID_SOCKET, "socket: error %", get_last_socket_error());
        #if OS == .MACOS {
            // Report a send to a closed peer as an error instead of raising SIGPIPE.
            on: s32 = 1;
            assert(setsockopt(s, SOL_SOCKET, SO_NOSIGPIPE, *on, size_of(s32)) == 0);
        }
        return s;
    }

    local_address :: (s: Socket) -> sockaddr_in {
        address: sockaddr_in;
        length: socklen_t = size_of(sockaddr_in);
        assert(getsockname(s, cast(*sockaddr) *address, *length) == 0, "getsockname: error %", get_last_socket_error());
        return address;
    }

    connect_to :: (s: Socket, address: sockaddr_in) -> s32 {
        target := address;
        #if OS == .MACOS target.sin_len = size_of(sockaddr_in);
        return connect(s, cast(*sockaddr) *target, size_of(sockaddr_in));
    }

    #if OS == .LINUX SEND_FLAGS :: MSG.NOSIGNAL;
    else SEND_FLAGS :: cast(MSG) 0;
    NO_FLAGS :: cast(MSG) 0;

    send_bytes :: (s: Socket, data: *u8, count: s64) -> s64 {
        #if OS == .WINDOWS return send(s, data, cast(s32) count, NO_FLAGS);
        else return send(s, data, cast(u64) count, SEND_FLAGS);
    }

    recv_bytes :: (s: Socket, data: *u8, count: s64, flags := NO_FLAGS) -> s64 {
        #if OS == .WINDOWS return recv(s, data, cast(s32) count, flags);
        else return recv(s, data, cast(u64) count, flags);
    }

    would_block :: () -> bool {
        error := get_last_socket_error();
        #if OS == .WINDOWS return error == SOCKET_WOULDBLOCK;
        else return error == EWOULDBLOCK || error == EAGAIN;
    }

    // Waits until `s` is readable; false on timeout.
    wait_readable :: (s: Socket, milliseconds: s32) -> bool {
        readable: fd_set;
        FD_SET(s, *readable);
        timeout: timeval;
        timeout.tv_sec = xx (milliseconds / 1000);
        timeout.tv_usec = xx ((milliseconds % 1000) * 1000);
        result := select(cast(s32) s + 1, *readable, null, null, *timeout);
        assert(result >= 0, "select: error %", get_last_socket_error());
        return result > 0 && FD_ISSET(s, *readable) != 0;
    }

    // A connected pair: client and the server's end of the connection.
    connected_pair :: () -> client: Socket, server: Socket, listener: Socket {
        listener := make_tcp_socket();
        yes: s32 = 1;
        assert(setsockopt(listener, SOL_SOCKET, SO_REUSEADDR, xx *yes, size_of(s32)) == 0);
        assert(bind(listener, "127.0.0.1", 0) == 0, "bind 127.0.0.1:0: error %", get_last_socket_error());
        assert(listen(listener, SOMAXCONN) == 0, "listen: error %", get_last_socket_error());
        listening := local_address(listener);
        assert(ntohs(listening.sin_port) != 0, "the OS assigned a port");
        text := to_string(listening);
        assert(begins_with(text, "127.0.0.1:") && text.count > "127.0.0.1:".count, "listening on %", text);

        client := make_tcp_socket();
        // A loopback connect completes into the listen backlog before accept runs.
        assert(connect_to(client, listening) == 0, "connect: error %", get_last_socket_error());
        server, peer := accept(listener);
        assert(server != INVALID_SOCKET, "accept: error %", get_last_socket_error());
        client_address := local_address(client);
        assert(to_string(peer) == to_string(client_address), "accept reports the client: % vs %", to_string(peer), to_string(client_address));
        assert(ntohs(peer.sin_port) != ntohs(listening.sin_port));
        return client, server, listener;
    }

    check_tcp :: () {
        client, server, listener := connected_pair();
        defer {
            close_and_reset(*client);
            close_and_reset(*server);
            close_and_reset(*listener);
        }

        assert(set_keepalive(client, true), "keepalive on");
        assert(set_keepalive(client, true, 30, 5, 4), "keepalive with timings");
        assert(set_keepalive(client, false), "keepalive off");
        assert(!set_keepalive(client, true, -1), "a negative idle time");
        enabled: s32;
        length: socklen_t = size_of(s32);
        assert(getsockopt(client, SOL_SOCKET, SO_KEEPALIVE, xx *enabled, *length) == 0 && enabled == 0, "keepalive is off: %", enabled);

        // A small message read in pieces, after peeking at it.
        message := "partial reads come back in order";
        assert(send_bytes(client, message.data, message.count) == message.count);
        assert(wait_readable(server, 5000), "the message arrives");
        peeked: [8] u8;
        assert(recv_bytes(server, peeked.data, peeked.count, MSG.PEEK) == peeked.count);
        assert(cast(string) peeked == "partial ", "peeked %", cast(string) peeked);
        received: [..] u8;
        defer array_free(received);
        piece: [5] u8;
        while received.count < message.count {
            assert(wait_readable(server, 5000), "the rest arrives");
            got := recv_bytes(server, piece.data, piece.count);
            assert(got > 0 && got <= piece.count, "recv gave %", got);
            array_add(*received, ..array_view(piece, 0, got));
        }
        assert(cast(string) received == message, "reassembled %", cast(string) received);

        // Non-blocking sockets report that nothing is waiting instead of blocking.
        assert(set_blocking(client, false) && set_blocking(server, false));
        assert(recv_bytes(server, piece.data, piece.count) < 0 && would_block(), "an empty non-blocking recv");

        // 300 KiB from client to server and back, pumped with select; neither side can send it all at once.
        payload := NewArray(300 * 1024, u8);
        defer free(payload.data);
        for *payload it.* = cast,trunc(u8) (it_index * 31 + (it_index >> 9));
        echo_through(client, server, payload);

        // Back to blocking: shutting down the client's sending side is the end of the server's stream.
        assert(set_blocking(client, true) && set_blocking(server, true));
        #if OS == .WINDOWS assert(shutdown(client, SD_SEND) == 0);
        else assert(shutdown(client, .SHUT_WR) == 0);
        assert(recv_bytes(server, piece.data, piece.count) == 0, "end of stream after shutdown");
        assert(send_bytes(server, "bye".data, 3) == 3, "the other direction still works");
        bye: [3] u8;
        assert(recv_bytes(client, bye.data, 3) == 3 && cast(string) bye == "bye");
        close_and_reset(*server);
        assert(server == INVALID_SOCKET, "close_and_reset resets the handle");
        close_and_reset(*server);
        assert(recv_bytes(client, piece.data, piece.count) == 0, "end of stream after the peer closed");
    }

    echo_through :: (client: Socket, server: Socket, payload: [] u8) {
        sent := 0;
        at_server: [..] u8;
        defer array_free(at_server);
        echoed := 0;
        back: [..] u8;
        defer array_free(back);
        chunk: [4093] u8;
        start := current_time_monotonic();
        rounds := 0;
        while back.count < payload.count {
            assert(to_milliseconds(current_time_monotonic() - start) < 20000, "echo stalled: sent %, server has %, echoed %, back %", sent, at_server.count, echoed, back.count);
            rounds += 1;
            readable, writable: fd_set;
            FD_SET(client, *readable);
            FD_SET(server, *readable);
            if sent < payload.count FD_SET(client, *writable);
            if echoed < at_server.count FD_SET(server, *writable);
            timeout: timeval;
            timeout.tv_sec = 1;
            highest := max(cast(s64) client, cast(s64) server);
            ready := select(cast(s32) highest + 1, *readable, *writable, null, *timeout);
            assert(ready >= 0, "select: error %", get_last_socket_error());
            if FD_ISSET(client, *writable) {
                got := send_bytes(client, payload.data + sent, min(payload.count - sent, 7001));
                if got > 0 sent += got;
                else assert(would_block(), "client send: error %", get_last_socket_error());
            }
            if FD_ISSET(server, *readable) {
                got := recv_bytes(server, chunk.data, chunk.count);
                if got > 0 array_add(*at_server, ..array_view(chunk, 0, got));
                else assert(got < 0 && would_block(), "server recv gave %, error %", got, get_last_socket_error());
            }
            if FD_ISSET(server, *writable) {
                got := send_bytes(server, at_server.data + echoed, at_server.count - echoed);
                if got > 0 echoed += got;
                else assert(would_block(), "server send: error %", get_last_socket_error());
            }
            if FD_ISSET(client, *readable) {
                got := recv_bytes(client, chunk.data, 3001);
                if got > 0 array_add(*back, ..array_view(chunk, 0, got));
                else assert(got < 0 && would_block(), "client recv gave %, error %", got, get_last_socket_error());
            }
        }
        assert(sent == payload.count && back.count == payload.count, "sent %, back %", sent, back.count);
        for back if it != payload[it_index] {
            assert(false, "byte % came back as %, sent %", it_index, it, payload[it_index]);
            break;
        }
        assert(rounds > 1, "a 300 KiB echo took more than one select round");
    }

    check_refused_connection :: () {
        // A port that was just bound and released, with nobody listening on it.
        probe := make_tcp_socket();
        assert(bind(probe, htonl(INADDR_LOOPBACK), 0) == 0, "bind with the u32 overload: error %", get_last_socket_error());
        address := local_address(probe);
        assert(to_string(address) != "127.0.0.1:0");
        close_and_reset(*probe);

        client := make_tcp_socket();
        defer close_and_reset(*client);
        assert(connect_to(client, address) != 0, "connecting to a closed port");
        error := get_last_socket_error();
        #if OS == .WINDOWS assert(error == WSAECONNREFUSED, "error %", error);
        else assert(error == ECONNREFUSED, "error %", error);
    }

    check_closed_peer :: () {
        client, server, listener := connected_pair();
        defer close_and_reset(*client);
        close_and_reset(*listener);
        close_and_reset(*server);
        // The first send may still be accepted; the reset that answers it makes a later one fail.
        data: [1024] u8;
        failed := false;
        for 1..200 {
            if send_bytes(client, data.data, data.count) < 0 {
                failed = true;
                break;
            }
            sleep_milliseconds(1);
        }
        assert(failed, "sending to a closed peer fails");
        error := get_last_socket_error();
        #if OS == .WINDOWS assert(error == WSAECONNRESET || error == WSAECONNABORTED, "error %", error);
        else assert(error == EPIPE || error == ECONNRESET, "error %", error);
    }

    check_udp :: () {
        a := socket(AF_INET, .SOCK_DGRAM, .UDP);
        b := socket(AF_INET, .SOCK_DGRAM, .UDP);
        assert(a != INVALID_SOCKET && b != INVALID_SOCKET, "UDP sockets: error %", get_last_socket_error());
        defer {
            close_and_reset(*a);
            close_and_reset(*b);
        }
        assert(bind(a, "127.0.0.1", 0) == 0 && bind(b, "127.0.0.1", 0) == 0);
        a_address := local_address(a);
        b_address := local_address(b);
        assert(a_address.sin_port != b_address.sin_port);

        // Datagrams of different sizes (one empty; macOS caps a datagram at 9216 bytes) keep their boundaries.
        sizes :: s64.[1, 1200, 0, 8000];
        out: [8000] u8;
        for *out it.* = cast,trunc(u8) (it_index * 7);
        for sizes {
            #if OS == .WINDOWS sent := sendto(a, out.data, cast(s32) it, NO_FLAGS, cast(*sockaddr) *b_address, size_of(sockaddr_in));
            else sent := sendto(a, out.data, cast(u64) it, NO_FLAGS, cast(*sockaddr) *b_address, size_of(sockaddr_in));
            assert(sent == it, "sendto % bytes gave %, error %", it, sent, get_last_socket_error());
        }
        incoming: [65536] u8;
        for sizes {
            assert(wait_readable(b, 5000), "datagram % arrives", it_index);
            from: sockaddr_in;
            from_length: socklen_t = size_of(sockaddr_in);
            #if OS == .WINDOWS got := recvfrom(b, incoming.data, incoming.count, NO_FLAGS, cast(*sockaddr) *from, *from_length);
            else got := recvfrom(b, incoming.data, incoming.count, NO_FLAGS, cast(*sockaddr) *from, *from_length);
            assert(got == it, "datagram % has % bytes, expected %", it_index, got, it);
            assert(memcmp(incoming.data, out.data, it) == 0, "datagram % contents", it_index);
            assert(to_string(from) == to_string(a_address), "datagram from % (sent from %)", to_string(from), to_string(a_address));
        }

        // A datagram read into a short buffer loses its tail; the next read starts the next datagram.
        #if OS == .WINDOWS {
            sendto(a, out.data, 100, NO_FLAGS, cast(*sockaddr) *b_address, size_of(sockaddr_in));
            sendto(a, out.data + 1, 3, NO_FLAGS, cast(*sockaddr) *b_address, size_of(sockaddr_in));
        } else {
            sendto(a, out.data, 100, NO_FLAGS, cast(*sockaddr) *b_address, size_of(sockaddr_in));
            sendto(a, out.data + 1, 3, NO_FLAGS, cast(*sockaddr) *b_address, size_of(sockaddr_in));
        }
        assert(wait_readable(b, 5000));
        short: [10] u8;
        got := recv_bytes(b, short.data, short.count);
        // Winsock reports the truncation as WSAEMSGSIZE after filling the buffer.
        #if OS == .WINDOWS assert(got == -1 && get_last_socket_error() == WSAEMSGSIZE, "short read gave %", got);
        else assert(got == 10, "short read gave %", got);
        assert(memcmp(short.data, out.data, 10) == 0);
        assert(wait_readable(b, 5000));
        got = recv_bytes(b, short.data, short.count);
        assert(got == 3 && memcmp(short.data, out.data + 1, 3) == 0, "the next datagram: %", got);

        // Nothing more is waiting.
        assert(set_blocking(b, false));
        assert(recv_bytes(b, short.data, short.count) < 0 && would_block());
        assert(!wait_readable(b, 10), "select times out on an idle socket");
    }

    check_ipv6_loopback :: () {
        listener := socket(AF_INET6, .SOCK_STREAM, .TCP);
        if listener == INVALID_SOCKET {
            print("socket-loopback: no IPv6 sockets here (error %), skipping the IPv6 checks\n", get_last_socket_error());
            return;
        }
        defer close_and_reset(*listener);
        if bind(listener, "::1", 0, AF_INET6) != 0 {
            print("socket-loopback: ::1 is not available here (error %), skipping the IPv6 checks\n", get_last_socket_error());
            return;
        }
        assert(listen(listener, 4) == 0);
        listening: sockaddr_in6;
        length: socklen_t = size_of(sockaddr_in6);
        assert(getsockname(listener, cast(*sockaddr) *listening, *length) == 0);
        port := ntohs(listening.sin6_port);
        assert(port != 0);
        assert(to_string(listening) == tprint("[::1]:%", port), "listening on %", to_string(listening));

        client := socket(AF_INET6, .SOCK_STREAM, .TCP);
        defer close_and_reset(*client);
        // Bind the client to ::1 with the [16] u8 overload so its address is known.
        loopback: [16] u8;
        loopback[15] = 1;
        assert(bind(client, loopback, 0) == 0, "bind [16] u8: error %", get_last_socket_error());
        target := listening;
        assert(connect(client, cast(*sockaddr) *target, size_of(sockaddr_in6)) == 0, "connect ::1: error %", get_last_socket_error());
        server, peer := accept_v6(listener);
        assert(server != INVALID_SOCKET);
        defer close_and_reset(*server);
        assert(begins_with(to_string(peer), "[::1]:"), "peer %", to_string(peer));
        assert(send_bytes(client, "six".data, 3) == 3);
        got: [3] u8;
        assert(recv_bytes(server, got.data, 3) == 3 && cast(string) got == "six");
    }

    check_bad_arguments :: () {
        s := make_tcp_socket();
        defer close_and_reset(*s);
        assert(bind(s, "not an address", 0) == -1, "an unparseable address");
        assert(bind(s, "127.0.0.1", 0, 77) == -1, "an unknown family");
        assert(bind(s, "::1", 0) == -1, "an IPv6 address for an IPv4 bind");
        // A port that is taken (the listener is not closed yet) cannot be bound again.
        listener := make_tcp_socket();
        defer close_and_reset(*listener);
        assert(bind(listener, "127.0.0.1", 0) == 0 && listen(listener, 1) == 0);
        taken := local_address(listener);
        assert(bind(s, "127.0.0.1", ntohs(taken.sin_port)) != 0, "binding a port that is in use");
        error := get_last_socket_error();
        #if OS == .WINDOWS assert(error == WSAEADDRINUSE || error == WSAEACCES, "error %", error);
        else assert(error == EADDRINUSE, "error %", error);
    }

    #if OS != .WINDOWS {
        // Hands one end of a socket pair across another with SCM_RIGHTS, built with the CMSG helpers.
        check_descriptor_passing :: () {
            channel: [2] s32;
            carried: [2] s32;
            assert(socketpair(AF_UNIX, .SOCK_STREAM, cast(IPPROTO) 0, *channel) == 0, "socketpair: error %", errno());
            assert(socketpair(AF_UNIX, .SOCK_STREAM, cast(IPPROTO) 0, *carried) == 0);
            defer for channel close(it);
            defer close(carried[1]);

            byte: u8 = #char "x";
            vector: iovec;
            vector.iov_base = *byte;
            vector.iov_len = 1;
            control: [64] u8;
            message: msghdr;
            message.msg_iov = *vector;
            message.msg_iovlen = 1;
            message.msg_control = control.data;
            message.msg_controllen = xx CMSG_SIZE(size_of(s32));
            header := CMSG_FIRSTHDR(*message);
            assert(header != null);
            header.cmsg_len = xx CMSG_LEN(size_of(s32));
            header.cmsg_level = SOL_SOCKET;
            header.cmsg_type = xx SCM.RIGHTS;
            memcpy(CMSG_DATA(header), *carried[0], size_of(s32));
            assert(sendmsg(channel[0], *message, NO_FLAGS) == 1, "sendmsg: error %", errno());
            close(carried[0]);

            received_byte: u8;
            vector.iov_base = *received_byte;
            incoming: [64] u8;
            reply: msghdr;
            reply.msg_iov = *vector;
            reply.msg_iovlen = 1;
            reply.msg_control = incoming.data;
            reply.msg_controllen = incoming.count;
            assert(recvmsg(channel[1], *reply, NO_FLAGS) == 1 && received_byte == #char "x", "recvmsg: error %", errno());
            got := CMSG_FIRSTHDR(*reply);
            assert(got != null && got.cmsg_level == SOL_SOCKET && got.cmsg_type == xx SCM.RIGHTS, "a rights message came back");
            assert(got.cmsg_len == xx CMSG_LEN(size_of(s32)), "cmsg_len %", got.cmsg_len);
            passed: s32;
            memcpy(*passed, CMSG_DATA(got), size_of(s32));
            defer close(passed);
            // The received descriptor is the carried socket: what goes in comes out of its partner.
            assert(write(passed, "fd".data, 2) == 2);
            check: [2] u8;
            assert(read(carried[1], check.data, 2) == 2 && cast(string) check == "fd");
        }
    }

    begins_with :: (s: string, prefix: string) -> bool {
        if s.count < prefix.count return false;
        return string.{ prefix.count, s.data } == prefix;
    }
}
