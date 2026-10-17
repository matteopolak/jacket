// Extensions/Io over loopback TCP on the default backend: addresses, an echo server with many
// client fibers, a transfer far larger than the socket buffers, end of stream, resets, refused
// connections, closing under a waiting reader, and name lookups. The same checks run once per
// backend in backends.jai.
#load "support/suite.jai";

main :: () {
    run_group(run_tcp);
    print("ok (backend %)\n", last_backend);
}
