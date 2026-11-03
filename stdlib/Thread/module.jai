// Threads and what they need to cooperate: `Thread` to run a procedure on its own thread, `Mutex`, `Condition_Variable`
// and `Semaphore` to coordinate, and `Thread_Group` for a pool of workers.
//
// Not available on wasm: it has no operating-system threads.
//
// ```jai
// Shared :: struct {
//     mutex: Mutex;
//     total: int;
// }
//
// add_many :: (thread: *Thread) -> s64 {
//     shared := cast(*Shared) thread.data;
//
//     for 1..1000 {
//         lock(*shared.mutex);
//         shared.total += 1;
//         unlock(*shared.mutex);
//     }
//
//     return 0;
// }
//
// shared: Shared;
//
// init(*shared.mutex);
// threads: [4] Thread;
// for *thread: threads {
//     assert(thread_init(thread, add_many));
//     thread.data = *shared;
//     thread_start(thread);
// }
//
// for *thread: threads {
//     assert(thread_is_done(thread, -1));
//     thread_deinit(thread);
// }
//
// assert(shared.total == 4000);
// destroy(*shared.mutex);
// ```

// test: skip_on(.WASM, reason = "Thread has no wasm target (no OS threads)")

#assert OS == .WINDOWS || OS == .MACOS || OS == .LINUX || OS == .ANDROID;

#load "../internal/thread/module.jai";

// Set this to a procedure that runs on every new thread, with the thread as its argument, before the thread's own procedure.
thread_init_handler: #type (thread: *Thread);

// Prepares `thread` to run `proc`. Start it with `thread_start`. Returns `false` when `proc` is null, the storage size is
// negative or an allocation fails.
//
// ```jai
// hello :: (thread: *Thread) -> s64 {
//     return 0;
// }
//
// thread: Thread;
//
// assert(thread_init(*thread, hello, name = "hello"));
// thread_start(*thread);
// assert(thread_is_done(*thread, -1));
// thread_deinit(*thread);
// ```
//
// - thread: set up by this call. It has to stay where it is until `thread_deinit`.
// - proc: runs on the new thread and is given `thread`.
// - temporary_storage_size: bytes of temporary storage for the thread. 0 gives it none.
// - starting_storage: storage to use instead of allocating `temporary_storage_size` bytes.
// - name: for debuggers and tools that list threads.
thread_init :: (thread: *Thread, proc: Thread_Proc, temporary_storage_size: s32 = 16384, starting_storage: *Temporary_Storage = null, name := "") -> bool {
    return prepare_thread(thread, proc, temporary_storage_size, starting_storage, name, *thread_init_handler);
} @RegistersBorrower(0,1,0)

#load "idle.jai";
