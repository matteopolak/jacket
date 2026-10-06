// #c_call procedures that a thread of C's own calls and that wait for Jai threads (a mutex, a
// condition variable, a join, a sleep), callbacks nested in C calls, and a Jai thread that keeps
// running while another one waits in C. The output is the same under `jaic run` and natively.
#import "Basic";
#import "Thread";

blocking :: #library "libblocking";

Callback :: #type (x: s32) -> s32 #c_call;

start_caller :: (cb: Callback, x: s32) #foreign blocking;
finish_caller :: () -> s32 #foreign blocking;
call_back :: (cb: Callback, x: s32) -> s32 #foreign blocking;
wait_for_flag :: (flag: *s32) -> s32 #foreign blocking;

mutex: Mutex;
condition: Condition_Variable;
ready: bool;
value: s32;

worker: Thread;
worker_result: s32;

flag: s32;

// Waits for `mutex`, which the main thread holds until it has set `value`.
read_locked :: (x: s32) -> s32 #c_call {
    push_context {
        lock(*mutex);
        defer unlock(*mutex);
        return value + x;
    }
}

// Waits until the main thread signals `condition`.
wait_until_ready :: (x: s32) -> s32 #c_call {
    push_context {
        lock(*mutex);
        defer unlock(*mutex);
        while !ready wait(*condition, *mutex);
        return value * x;
    }
}

// Joins a Jai thread the main thread started.
join_worker :: (x: s32) -> s32 #c_call {
    push_context {
        thread_deinit(*worker);
        return worker_result + x;
    }
}

nap :: (milliseconds: s32) -> s32 #c_call {
    push_context {
        sleep_milliseconds(milliseconds);
    }
    return milliseconds;
}

// C -> Jai -> C -> Jai ..., sleeping at the bottom.
nested :: (depth: s32) -> s32 #c_call {
    if depth == 0 return nap(2) - 1;
    return depth * 10 + call_back(nested, depth - 1);
}

slow_worker :: (thread: *Thread) -> s64 {
    sleep_milliseconds(20);
    worker_result = 40;
    return 0;
}

nesting_worker :: (thread: *Thread) -> s64 {
    worker_result = call_back(nested, 2);
    return 0;
}

// Stays in C until the main thread, running Jai code meanwhile, sets `flag`.
flag_waiter :: (thread: *Thread) -> s64 {
    worker_result = wait_for_flag(*flag);
    return 0;
}

main :: () {
    init(*mutex);
    init(*condition);

    lock(*mutex);
    start_caller(read_locked, 1);
    sleep_milliseconds(20);
    value = 41;
    unlock(*mutex);
    print("mutex: %\n", finish_caller());

    start_caller(wait_until_ready, 2);
    sleep_milliseconds(20);
    lock(*mutex);
    value = 21;
    ready = true;
    wake(*condition);
    unlock(*mutex);
    print("condition: %\n", finish_caller());

    thread_init(*worker, slow_worker);
    thread_start(*worker);
    start_caller(join_worker, 2);
    print("join: %\n", finish_caller());

    start_caller(nap, 10);
    print("sleep: %\n", finish_caller());

    print("nested: %\n", call_back(nested, 3));
    start_caller(nested, 3);
    print("nested on a thread of C: %\n", finish_caller());
    thread_init(*worker, nesting_worker);
    thread_start(*worker);
    thread_deinit(*worker);
    print("nested on a Jai thread: %\n", worker_result);

    thread_init(*worker, flag_waiter);
    thread_start(*worker);
    sleep_milliseconds(50);
    total := 0;
    for 1..100000 total += it;
    flag = 1;
    thread_deinit(*worker);
    print("progress while a thread waits in C: % %\n", total, worker_result);
}
