// test: only(.WASM, mode = .NATIVE)
// A descriptor the host did not pre-open as a socket is refused by the four calls WASI has for sockets, with -1 and an error.
#import "Basic";
S :: #import "Socket";

main :: () {
    buffer: [16] u8;
    length: S.socklen_t;
    peer: S.sockaddr_in;

    assert(S.recv(99, buffer.data, buffer.count, cast(S.MSG) 0) == -1);
    assert(S.get_last_socket_error() != 0);

    assert(S.send(99, buffer.data, buffer.count, cast(S.MSG) 0) == -1);
    assert(S.get_last_socket_error() != 0);

    assert(S.accept(99, cast(*S.sockaddr) *peer, *length) == -1);
    assert(S.get_last_socket_error() != 0);

    assert(S.shutdown(99, .RDWR) == -1);
    assert(S.get_last_socket_error() != 0);

    // Flags WASI cannot express are refused before the call.
    assert(S.recv(99, buffer.data, buffer.count, cast(S.MSG) 1) == -1);
    assert(cast(s32) S.get_last_socket_error() == 22);

    print("ok\n");
}
