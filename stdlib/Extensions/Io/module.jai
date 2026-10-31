// Fibers and non-blocking IO. [io_run] starts a scheduler on the calling thread; inside it [spawn]
// starts fibers, which wait for sleeps, channels, sockets and files without blocking the thread.
// [make_channel], [Wait_Group] and [Fiber_Mutex] coordinate fibers, [tcp_listen], [accept] and
// [connect] do TCP, and [Reader] and [Writer] give files, sockets and pipes one shape.
// @experimental
//
// ```jai,skip_on(wasm)
// Io :: #import "Extensions/Io";
//
// results: *Io.Channel(int); // a fiber's procedure cannot capture locals: share through a global or pass data
//
// main :: () {
//     Io.io_run(() { // the scheduler runs on this thread
//         results = Io.make_channel(int, 4);
//         defer Io.free_channel(results);
//
//         for 1..4 Io.spawn((n: int) { Io.sleep(10 * n); Io.send(results, n); }, it);
//
//         total := 0;
//
//         for 1..4 {
//             value, err := Io.recv(results);
//             assert(err == .NONE);
//             total += value;
//         }
//
//         assert(total == 10);
//     });
// }
// ```
//
// A fiber is a lightweight thread that the program switches itself. It has its own small stack and
// runs until it blocks (`sleep`, `yield`, a channel, a `Wait_Group`, a `Fiber_Mutex`), when the
// scheduler runs another one. All fibers of an `io_run` share one OS thread, so they never run at
// the same time and need no locks to share data. A blocking call returns `.CANCELED` once
// `cancel` has been called for its fiber.
//
// WebAssembly cannot switch stacks, so importing this module for wasm is a compile error.

#module_parameters(
    // The stack of each fiber, in bytes, including its guard page. Address space only: the OS commits
    // pages as the fiber touches them (Windows commits it all up front).
    STACK_BYTES := 256 * 1024,
    // How many finished fibers keep their stack for reuse.
    POOL_LIMIT := 1024,
    // The temporary storage each fiber gets, in bytes; `0` gives none.
    TEMPORARY_BYTES := 2048
);

#assert OS != .WASM "Extensions/Io needs fibers, and WebAssembly code cannot switch stacks";

#load "fiber_context.jai";
#load "scheduler.jai";
#load "sync.jai";
#load "net.jai";
#load "pipe.jai";
#load "files.jai";
#load "stream.jai";
#load "json.jai";
#load "runtime.jai";
#load "interrupt.jai";

#scope_module
#import "Basic";
#load "ownership.jai";
Json :: #import "Extensions/Json";
#load "stacks.jai";
#load "overflow.jai";
#load "platform.jai";
#load "engine.jai";
#load "engine_poll.jai";
#load "engine_uring.jai";
#load "engine_iocp.jai";
#load "engine_ioring.jai";
#load "files_pool.jai";
#load "win32.jai";
#load "resolver.jai";
