#assert OS == .WINDOWS || OS == .MACOS || OS == .LINUX || OS == .ANDROID;

#load "../internal/thread/module.jai";

thread_init_handler: #type (thread: *Thread);

thread_init :: (thread: *Thread, proc: Thread_Proc, temporary_storage_size: s32 = 16384, starting_storage: *Temporary_Storage = null, name := "") -> bool {
    return prepare_thread(thread, proc, temporary_storage_size, starting_storage, name, *thread_init_handler);
}
