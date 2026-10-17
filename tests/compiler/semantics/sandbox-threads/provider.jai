// Private SandboxHost token protocol; never linked to a native pthread library.
#assert OS == .WASM, "opaque scheduler tokens require the private WASM provider";

#import "Basic";

pthread_create :: (thread: *u64, attributes: *void, entry: *void, data: *void) -> s32 #foreign;
pthread_join :: (thread: u64, result: **void) -> s32 #foreign;
pthread_mutex_init :: (token: *void, attributes: *void) -> s32 #foreign;
pthread_mutex_lock :: (token: *void) -> s32 #foreign;
pthread_mutex_unlock :: (token: *void) -> s32 #foreign;
pthread_cond_init :: (token: *void, attributes: *void) -> s32 #foreign;
pthread_cond_wait :: (condition: *void, mutex: *void) -> s32 #foreign;
pthread_cond_timedwait :: (condition: *void, mutex: *void, deadline: *Provider_Time) -> s32 #foreign;
pthread_cond_signal :: (condition: *void) -> s32 #foreign;
clock_gettime :: (clock: s32, result: *Provider_Time) -> s32 #foreign;
usleep :: (microseconds: u32) -> s32 #foreign;

Provider_Time :: struct {
    seconds: s64;
    nanoseconds: s64;
}

mutex: u64;
condition: u64;
permits: s32;
flag: s32;
starting_context: #Context;

initialize :: () {
    starting_context = context;
    assert(pthread_mutex_init(*mutex, null) == 0);
    assert(pthread_cond_init(*condition, null) == 0);
}

wait_permit :: (milliseconds: s32 = -1) -> s32 {
    assert(pthread_mutex_lock(*mutex) == 0);
    defer assert(pthread_mutex_unlock(*mutex) == 0);
    deadline: Provider_Time;

    if milliseconds >= 0 {
        assert(clock_gettime(0, *deadline) == 0);
        deadline.nanoseconds += cast(s64) milliseconds * 1000000;
        deadline.seconds += deadline.nanoseconds / 1000000000;
        deadline.nanoseconds %= 1000000000;
    }

    while permits == 0 {
        result := ifx milliseconds < 0 then pthread_cond_wait(*condition, *mutex)
            else pthread_cond_timedwait(*condition, *mutex, *deadline);
        if result != 0 return result;
    }

    permits -= 1;

    return 0;
}

signal_permit :: () {
    assert(pthread_mutex_lock(*mutex) == 0);
    permits += 1;
    assert(pthread_cond_signal(*condition) == 0);
    assert(pthread_mutex_unlock(*mutex) == 0);
}
