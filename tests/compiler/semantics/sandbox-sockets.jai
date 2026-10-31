// An echo over a socket the host pre-opened (`JAIC_TCPLISTEN`), as a WASI program uses one. Driven by
// `crates/jaic-cli/tests/sandbox_host.rs`, which connects to the listener.
#import "Basic";
S :: #import "Socket";

libc :: #system_library "libc";
getenv :: (name: *u8) -> *u8 #foreign libc;

main :: () {
    count := getenv("LISTEN_FDS");

    assert(count != null && to_string(count) == "1", "LISTEN_FDS names one listener");

    listener: S.Socket = 3;
    peer_address: S.sockaddr_in;
    length: S.socklen_t = size_of(S.sockaddr_in);

    // A listener is not a connection.
    buffer: [64] u8;

    assert(S.recv(listener, buffer.data, buffer.count, cast(S.MSG) 0) == -1);

    peer := S.accept(listener, cast(*S.sockaddr) *peer_address, *length);

    assert(peer != S.INVALID_SOCKET, "accept failed: %", cast(s32) S.get_last_socket_error());
    assert(length == 0, "the peer address is not reported");

    // A connection is not a listener.
    assert(S.accept(peer, cast(*S.sockaddr) *peer_address, *length) == -1);

    // Peek leaves the bytes where they are; the reads after it take them.
    peeked := S.recv(peer, buffer.data, 3, .PEEK);

    assert(peeked > 0 && peeked <= 3);

    total := 0;

    while true {
        received := S.recv(peer, buffer.data + total, cast(u64) (buffer.count - total), cast(S.MSG) 0);
        assert(received >= 0);
        if received == 0 break;
        total += received;
    }

    request := to_string(buffer.data, total);

    assert(request == "copper", "got %", request);

    reply := tprint("echo: %", request);

    assert(S.send(peer, reply.data, cast(u64) reply.count, cast(S.MSG) 0) == reply.count);
    assert(S.shutdown(peer, .WR) == 0);

    closed := peer;

    S.close_and_reset(*closed);
    assert(closed == S.INVALID_SOCKET);
    assert(S.recv(peer, buffer.data, buffer.count, cast(S.MSG) 0) == -1, "a closed descriptor is refused");

    print("ok\n");
}
