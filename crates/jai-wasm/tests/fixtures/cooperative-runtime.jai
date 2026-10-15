// Private SandboxHost scheduler protocol, not the public Thread module or a native pthread binding.
pthread_create :: (thread: *u64, attributes: *void, entry: *void, data: *void) -> s32 #foreign;
pthread_join :: (thread: u64, result: **void) -> s32 #foreign;
pthread_mutex_init :: (token: *void, attributes: *void) -> s32 #foreign;
pthread_mutex_lock :: (token: *void) -> s32 #foreign;
pthread_mutex_unlock :: (token: *void) -> s32 #foreign;
pthread_mutex_destroy :: (token: *void) -> s32 #foreign;
sched_yield :: () -> s32 #foreign;
write :: (descriptor: s32, text: *u8, count: u64) -> s64 #foreign;

Work :: struct {
    completed: int;
    failure: bool;
}

mutex: u64;
started: int;
total: int;

worker :: (work: *Work) -> *void #c_call {
    if pthread_mutex_lock(*mutex) != 0 {
        work.failure = true;
        return work;
    }

    started += 1;

    if pthread_mutex_unlock(*mutex) != 0 {
        work.failure = true;
        return work;
    }

    while true {
        if pthread_mutex_lock(*mutex) != 0 {
            work.failure = true;
            return work;
        }

        ready := started == 3;

        if pthread_mutex_unlock(*mutex) != 0 {
            work.failure = true;
            return work;
        }

        if ready break;

        if sched_yield() != 0 {
            work.failure = true;
            return work;
        }
    }

    for 0..9 {
        if pthread_mutex_lock(*mutex) != 0 {
            work.failure = true;
            return work;
        }

        previous := total;

        // Another worker must block on this mutex while the holder yields.
        if sched_yield() != 0 {
            work.failure = true;
            return work;
        }

        total = previous + 1;
        work.completed += 1;

        if pthread_mutex_unlock(*mutex) != 0 {
            work.failure = true;
            return work;
        }

        if sched_yield() != 0 {
            work.failure = true;
            return work;
        }
    }

    return work;
}

main :: () -> s32 {
    if pthread_mutex_init(*mutex, null) != 0 return 1;

    handles: [3] u64;
    work: [3] Work;

    for 0..2 {
        if pthread_create(*handles[it], null, cast(*void) worker, *work[it]) != 0 return 2;
    }

    for 0..2 {
        result: *void;
        if pthread_join(handles[it], *result) != 0 return 3;
        if result != cast(*void) *work[it] || work[it].failure || work[it].completed != 10 return 4;
    }

    if total != 30 || started != 3 return 5;
    if pthread_mutex_destroy(*mutex) != 0 return 6;
    if write(1, "ok\n".data, 3) != 3 return 7;

    return 0;
}
