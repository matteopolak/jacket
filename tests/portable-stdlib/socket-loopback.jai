#import "Basic";
S :: #import "Socket";

main :: () {
    assert(S.socket_init());
    server := S.socket(S.AF_INET, S.SOCK.STREAM, S.IPPROTO.TCP);

    assert(server >= 0);
    defer S.close_and_reset(*server);
    assert(S.bind(server, "127.0.0.1", 0) == 0);
    assert(S.listen(server, 8) == 0);
    address: S.sockaddr_in;
    length: S.socklen_t = size_of(S.sockaddr_in);

    assert(S.getsockname(server, cast(*S.sockaddr) *address, *length) == 0);
    client := S.socket(S.AF_INET, S.SOCK.STREAM, S.IPPROTO.TCP);

    assert(client >= 0);
    defer S.close_and_reset(*client);
    assert(S.set_blocking(client, false));
    assert(S.set_blocking(client, true));
    assert(S.set_keepalive(client, true));
    assert(S.connect(client, cast(*S.sockaddr) *address, length) == 0);
    peer, remote := S.accept(server);

    assert(peer >= 0);
    assert(remote.sin_family == S.AF_INET);
    defer S.close_and_reset(*peer);
    assert(S.send(client, "copper".data, 6, cast(S.MSG) 0) == 6);
    buffer: [16] u8;
    received := S.recv(peer, buffer.data, buffer.count, cast(S.MSG) 0);

    assert(received == 6 && to_string(buffer.data, received) == "copper");
    set: S.fd_set;

    S.FD_SET(7, *set);
    S.FD_SET(34, *set);
    assert(S.FD_ISSET(7, *set) == 1 && S.FD_ISSET(34, *set) == 1);
    S.FD_CLR(7, *set);
    assert(S.FD_ISSET(7, *set) == 0 && S.FD_ISSET(34, *set) == 1);
    print("ok\n");
}
