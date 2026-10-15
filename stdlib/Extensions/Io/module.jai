// Io: fibers and a cooperative scheduler. A jaic extension, not official Jai: other Jai compilers
// have no such module, and it may change between jaic releases. See docs/stdlib/io-fibers.md.
//
//     Io :: #import "Extensions/Io";
//
//     main :: () {
//         Io.io_run(() {                          // the scheduler runs on this thread
//             results := Io.make_channel(int, 4);
//
//             for 1..4 Io.spawn((n: int) { Io.sleep(10 * n); Io.send(results, n); }, it);
//
//             for 1..4 {
//                 value, err := Io.recv(results);
//                 print("%\n", value);
//             }
//         });
//     }
//
// A fiber is a lightweight thread that the program switches itself: it has its own small stack,
// and runs until it blocks (`sleep`, `yield`, a channel, a `Wait_Group`, a `Fiber_Mutex`), when the
// scheduler runs another one. All fibers of an `io_run` share one OS thread, so they never run at
// the same time and need no locks to share data. A blocking call returns `.CANCELED` once
// `cancel(fiber)` has been called for its fiber.
//
// Fibers need the compiler's stack-switching intrinsics, which native builds and the interpreter
// (`jaic -interp`) have and WebAssembly does not: importing this module for wasm is a compile error.
// The stack of each fiber, in bytes, including its guard page. Address space only: the OS commits
// pages as the fiber touches them (Windows commits it all up front).
#module_parameters(STACK_BYTES := 256 * 1024, POOL_LIMIT := 1024, TEMPORARY_BYTES := 2048);

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
