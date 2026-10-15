// Deadlocks `jaic -interp` reports instead of hanging (a native build hangs). With no argument, two
// Jai threads each wait for the other; with `callback`, a thread of C's own waits in a #c_call
// procedure for a mutex the main thread holds while the main thread waits for that procedure.
#import "Basic";
#import "Thread";

blocking :: #library "libblocking";

Callback :: #type (x: s32) -> s32 #c_call;
start_caller :: (cb: Callback, x: s32) #foreign blocking;

// Native scheduler joins are tested through actual SDK signatures. Public
// thread_deinit requires completion; thread_is_done may poll and stay runnable.
#if OS == .WINDOWS {
    sdk :: #system_library "Kernel32";
    Native_Run :: #type (argument: *void) -> u32 #c_call;
    create_native :: (attributes: *void, stack_size: u64, run: Native_Run, argument: *void, flags: u32, id: *u32) -> *void #foreign sdk "CreateThread";
    join_native :: (handle: *void, milliseconds: u32) -> u32 #foreign sdk "WaitForSingleObject";
} else {
    sdk :: #system_library "libc";
    Native_Run :: #type (argument: *void) -> *void #c_call;
    create_native :: (handle: *u64, attributes: *void, run: Native_Run, argument: *void) -> s32 #foreign sdk "pthread_create";
    join_native :: (handle: u64, result: **void) -> s32 #foreign sdk "pthread_join";
}

mutex: Mutex;
done: Condition_Variable;
done_mutex: Mutex;
ready: bool;

lock_mutex :: (x: s32) -> s32 #c_call {
    push_context {
        lock(*mutex);
        lock(*done_mutex);
        ready = true;
        wake(*done);
        unlock(*done_mutex);
        unlock(*mutex);
    }

    return x;
}

#if OS == .WINDOWS Native_Result :: u32;
else Native_Result :: *void;

wait_for_main :: (argument: *void) -> Native_Result #c_call {
    push_context {
        lock(*mutex);
        unlock(*mutex);
    }

    #if OS == .WINDOWS return 0;
    else return null;
}

main :: () {
    // The test times the report from here, so how long the compile took does not count.
    write_string("started\n", true);
    init(*mutex);
    init(*done);
    init(*done_mutex);
    lock(*mutex);
    args := get_command_line_arguments();
    if args.count > 1 && args[1] == "callback" {
        start_caller(lock_mutex, 1);
        lock(*done_mutex);
        while !ready wait(*done, *done_mutex);
        unlock(*done_mutex);
    } else {
        #if OS == .WINDOWS {
            handle := create_native(null, 0, wait_for_main, null, 0, null);
            assert(handle != null);
            join_native(handle, 0xffff_ffff);
        } else {
            handle: u64;
            status := create_native(*handle, null, wait_for_main, null);
            assert(status == 0);
            join_native(handle, null);
        }
    }

    print("not reached\n");
}
