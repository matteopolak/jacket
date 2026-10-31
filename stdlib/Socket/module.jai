// Network sockets for TCP, UDP and Unix-domain use: Berkeley sockets on Unix, Winsock on Windows.
//
// The native calls (`socket`, `bind`, `listen`, `connect`, `send`, `recv`, `select`, ...) keep their
// system signatures. Around them are portable helpers: `socket_init`, `set_blocking`, `set_keepalive`,
// `close_and_reset`, `bind` and `accept` overloads that take a text address, `FD_SET` and friends,
// and `to_string` for addresses. For HTTP, TLS or an event loop use `Extensions/Http`,
// `Extensions/Tls` and `Extensions/Io`.
//
// A call fails the way the system's does: it returns -1 (or `INVALID_SOCKET`) and
// `get_last_socket_error()` has the code. Close every socket you create or accept with
// `close_and_reset`. Sockets start blocking; after `set_blocking(s, false)`, calls return at once with
// `SOCKET_WOULDBLOCK` and reads and writes may move fewer bytes than asked. Call `socket_init` first on
// Windows. Ports in `sockaddr_in` are in network order (`htons`); the `bind` helpers take host order.
// `to_string` allocates with `context.allocator`.
//
// ```jai,skip_on(wasm)
// assert(socket_init());
//
// listener := socket(AF_INET, .SOCK_STREAM, .TCP);
// defer close_and_reset(*listener);
//
// // Port 0 lets the OS choose; getsockname tells which one it was.
// assert(bind(listener, "127.0.0.1", 0) == 0);
// assert(listen(listener, 4) == 0);
//
// address: sockaddr_in;
// length: socklen_t = size_of(sockaddr_in);
// assert(getsockname(listener, cast(*sockaddr) *address, *length) == 0);
//
// client := socket(AF_INET, .SOCK_STREAM, .TCP);
// defer close_and_reset(*client);
// assert(connect(client, cast(*sockaddr) *address, length) == 0);
//
// server, peer := accept(listener);
// defer close_and_reset(*server);
//
// assert(send(client, "ping".data, 4, cast(MSG) 0) == 4);
//
// buffer: [16] u8;
// got := recv(server, buffer.data, buffer.count, cast(MSG) 0);
// assert(got == 4 && string.{got, buffer.data} == "ping");
// ```

#assert OS_IS_UNIX || OS == .WINDOWS || OS == .WASM;

// The module is the files of internal/net loaded into one scope: the per-OS native declarations
// (socket-abi-<os>.jai) are the public names, beside the hand-written helpers around them.
#load "../internal/net/socket.jai";
