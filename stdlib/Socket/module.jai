// Socket: the system's network sockets (Berkeley sockets on Unix, Winsock on Windows) for TCP, UDP and Unix-domain use.
//
// The module exposes the native calls (`socket`, `bind`, `listen`, `connect`, `send`, `recv`, `select`, ...) with their
// native signatures, and a small set of portable helpers around them: `socket_init`, `set_blocking`, `set_keepalive`,
// `close_and_reset`, the `accept` and `bind` overloads, `FD_SET` and its siblings, and `to_string` for addresses. For
// HTTP, TLS or an event loop use `Extensions/Http`, `Extensions/Tls` and `Extensions/Io` instead.
//
// Calls report failure the way the system does: -1 (or `INVALID_SOCKET`) and an error code from
// `get_last_socket_error()`. Nothing here allocates except the `to_string` calls, which use `context.allocator`.
// A socket is owned by whoever created or accepted it and is released with `close_and_reset`. Sockets start blocking;
// `set_blocking(s, false)` makes `recv`, `send`, `accept` and `connect` return at once with `SOCKET_WOULDBLOCK`, and
// reads and writes may then transfer fewer bytes than requested. Addresses go through `sockaddr_in` (IPv4) and
// `sockaddr_in6` (IPv6); ports in these records are in network order (`htons`) while the helper `bind` overloads take host order.
// Windows needs `socket_init` before the first call, and the descriptor-set and control-message helpers differ by
// platform (`CMSG_*` exist on Unix only).
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

#assert OS_IS_UNIX || OS == .WINDOWS;

// The module is the files of internal/net loaded into one scope: the per-OS native declarations
// (socket-abi-<os>.jai) are the public names, beside the hand-written helpers around them.
#load "../internal/net/socket.jai";
