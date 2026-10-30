#assert OS == .WINDOWS || OS == .MACOS || OS == .LINUX || OS == .ANDROID;

#load "../internal/thread/module.jai";

thread_init_handler: #type (thread: *Thread);

// Prepares `thread` to run `proc`. Start it with `thread_start`. Returns `false` when `proc` is null, the storage size is negative or an allocation fails.
//
// - thread: set up by this call. It has to stay where it is until `thread_deinit`.
// - proc: runs on the new thread and is given `thread`.
// - temporary_storage_size: bytes of temporary storage for the thread. 0 gives it none.
// - starting_storage: storage to use instead of allocating `temporary_storage_size` bytes.
// - name: for debuggers and tools that list threads.
thread_init :: (thread: *Thread, proc: Thread_Proc, temporary_storage_size: s32 = 16384, starting_storage: *Temporary_Storage = null, name := "") -> bool {
    return prepare_thread(thread, proc, temporary_storage_size, starting_storage, name, *thread_init_handler);
}
