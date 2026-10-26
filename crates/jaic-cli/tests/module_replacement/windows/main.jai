// The library's native Windows code with a project-owned Windows module that has almost nothing in it.
#import "Basic";
#import "File";
#import "File_Utilities";
#import "File_Watcher";
#import "Process";
#import "System";
#import "Thread";
#import "Socket";
Native :: #import "Windows";

worker :: (thread: *Thread) -> s64 {
    return 0;
}

main :: () {
    id: Native.DWORD = Native.GetCurrentProcessId();
    thread: Thread;

    thread_init(*thread, worker);
    thread_start(*thread);
    thread_deinit(*thread);
    print("% %\n", id, get_error_value());
}
