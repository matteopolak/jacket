#import "Basic";
S :: #import "Socket";

main :: () {
    #if OS == .WINDOWS {
        #assert type_of(S.WINSOCK_VERSION) == u16;
        #assert is_constant(S.WINSOCK_VERSION);
        #assert size_of(S.WSADATA) == 408;
        #assert size_of(S.fd_set) == 520;
        #assert size_of(S.SCOPE_ID) == 4;
        #assert type_of(S.FD_ISSET) == type_of(S.__WSAFDIsSet);
        #assert cast,no_check(u32) S.FIONBIO == 0x8004667e;
        #assert S.SOCKET_WOULDBLOCK_CONNECT == S.WSAEWOULDBLOCK;
        scope: S.SCOPE_ID;
        scope.Value = 0xa1234567;
        assert(S.get_anon_struct_Zone(scope) == 0x1234567 && S.get_anon_struct_Level(scope) == 10);
        S.set_anon_struct_Zone(*scope, 0xffffffff);
        assert(scope.Value == 0xafffffff);
        S.set_anon_struct_Level(*scope, 0x12345678);
        assert(scope.Value == 0x8fffffff);
        initialized := S.socket_init();

        assert(initialized);

        defer {
            status := S.WSACleanup();
            assert(status == 0);
        }

        server := S.socket(S.AF_INET, S.SOCK.STREAM, S.IPPROTO.TCP);

        assert(server != S.INVALID_SOCKET);
        defer S.close_and_reset(*server);
        assert(S.bind(server, "127.0.0.1", 0) == 0 && S.listen(server, 8) == 0);
        address: S.sockaddr_in;
        length: s32 = size_of(S.sockaddr_in);

        assert(S.getsockname(server, cast(*S.sockaddr) *address, *length) == 0);
        text := S.to_string(address);

        assert(text.count > 10);
        for i: 0..9 assert(text.data[i] == "127.0.0.1:"[i]);

        free(text);
        v6: S.sockaddr_in6;
        raw: [16] u8;
        raw[15] = 1;
        memcpy(*v6.sin6_addr, raw.data, 16);
        v6.sin6_scope_id = 44;
        text = S.to_string(v6);
        assert(text == "0:0:0:0:0:0:0:1:0");
        free(text);
        client := S.socket(S.AF_INET, S.SOCK.STREAM, S.IPPROTO.TCP);

        assert(client != S.INVALID_SOCKET);
        defer S.close_and_reset(*client);
        assert(S.set_blocking(client, false) && S.set_blocking(client, true));
        assert(S.set_keepalive(client, true, 1000, 1000, 99));
        assert(S.set_keepalive(client, false));
        assert(S.connect(client, cast(*S.sockaddr) *address, length) == 0);
        peer, remote := S.accept(server);

        assert(peer != S.INVALID_SOCKET && remote.sin_family == S.AF_INET);
        defer S.close_and_reset(*peer);
        assert(S.send(client, "copper".data, 6, cast(S.MSG) 0) == 6);
        bytes: [16] u8;
        received := S.recv(peer, bytes.data, 16, cast(S.MSG) 0);

        assert(received == 6 && to_string(bytes.data, received) == "copper");
        set: S.fd_set;

        S.FD_SET(client, *set);
        S.FD_SET(client, *set);
        S.FD_SET(peer, *set);
        assert(set.fd_count == 2 && S.FD_ISSET(client, *set) == 1);
        S.FD_CLR(client, *set);
        assert(set.fd_count == 1 && S.FD_ISSET(client, *set) == 0 && S.FD_ISSET(peer, *set) == 1);
        previous := context.logger;

        context.logger = capture_error;
        failed := S.set_blocking(S.INVALID_SOCKET, true);
        code := S.get_last_socket_error();

        context.logger = previous;
        assert(!failed && logged == 1 && cast(u32) code == S.WSAENOTSOCK);
        print("ok\n");
    } else {
        assert(false, "Windows socket execution requires an actual Windows host");
    }
}

#scope_file
logged: s64;

capture_error :: (message: string, _data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
    logged += 1;
}
