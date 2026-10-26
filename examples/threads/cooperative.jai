// This example targets the browser/WASI runtime. These are its private scheduler
// hooks, not the public desktop/Android Thread module or a native pthread ABI.
#scope_file

#import "Basic";

#assert OS == .WASM;

demo_thread_create :: (handle: *u64, attributes: *void, entry: *void, data: *void) -> s32 #foreign "pthread_create";
demo_thread_join :: (handle: u64, result: **void) -> s32 #foreign "pthread_join";
demo_mutex_init :: (token: *void, attributes: *void) -> s32 #foreign "pthread_mutex_init";
demo_mutex_lock :: (token: *void) -> s32 #foreign "pthread_mutex_lock";
demo_mutex_unlock :: (token: *void) -> s32 #foreign "pthread_mutex_unlock";
demo_mutex_destroy :: (token: *void) -> s32 #foreign "pthread_mutex_destroy";
demo_condition_init :: (token: *void, attributes: *void) -> s32 #foreign "pthread_cond_init";
demo_condition_wait :: (token: *void, mutex: *void) -> s32 #foreign "pthread_cond_wait";
demo_condition_signal :: (token: *void) -> s32 #foreign "pthread_cond_signal";
demo_condition_broadcast :: (token: *void) -> s32 #foreign "pthread_cond_broadcast";
demo_condition_destroy :: (token: *void) -> s32 #foreign "pthread_cond_destroy";
demo_abort :: () #foreign "abort";

// SandboxHost keys these tokens by address. Wasi_Runtime uses two u64 words
// for mutex owner/depth. This storage describes only these runtime providers.
Demo_Token :: struct {
    words: [2] u64;
}

Demo_Slice :: struct {
    first, last, primes: int;
}

Demo_Pool :: struct {
    mutex: Demo_Token;
    ready: Demo_Token;
    started: int;
    slices: [] Demo_Slice;
}

Demo_Worker :: struct {
    pool: *Demo_Pool;
    index, completed: int;
}

Demo_Ring :: struct {
    mutex: Demo_Token;
    readable, writable: Demo_Token;
    slots: [4] int;
    head, tail, count, waits, produced: int;
}

require_runtime :: (ok: bool) #no_context {
    if !ok {
        demo_abort();
    }
}

check_runtime :: (status: s32) #no_context {
    require_runtime(status == 0);
}

count_primes :: (first: int, last: int) -> int #no_context {
    primes := 0;

    for n: first..last {
        if n < 2 continue;

        prime := true;
        divisor := 2;

        while divisor * divisor <= n {
            if n % divisor == 0 {
                prime = false;
                break;
            }

            divisor += 1;
        }

        if prime primes += 1;
    }

    return primes;
}

prime_worker :: (worker: *Demo_Worker) -> *void #c_call {
    pool := worker.pool;

    check_runtime(demo_mutex_lock(*pool.mutex));
    pool.started += 1;
    check_runtime(demo_condition_broadcast(*pool.ready));
    // Every real worker must reach this barrier. Serial callback execution
    // cannot complete it.
    while pool.started != 3 check_runtime(demo_condition_wait(*pool.ready, *pool.mutex));

    check_runtime(demo_mutex_unlock(*pool.mutex));

    index := worker.index;
    while index < pool.slices.count {
        slice := *pool.slices[index];
        slice.primes = count_primes(slice.first, slice.last);
        worker.completed += 1;
        index += 3;
    }

    return worker;
}

produce_squares :: (ring: *Demo_Ring) -> *void #c_call {
    for 1..10 {
        check_runtime(demo_mutex_lock(*ring.mutex));

        while ring.count == ring.slots.count {
            ring.waits += 1;
            check_runtime(demo_condition_wait(*ring.writable, *ring.mutex));
        }

        ring.slots[ring.tail] = it * it;
        ring.tail = (ring.tail + 1) % ring.slots.count;
        ring.count += 1;
        ring.produced += 1;
        check_runtime(demo_condition_signal(*ring.readable));
        check_runtime(demo_mutex_unlock(*ring.mutex));
    }

    return ring;
}

#scope_export

demo_threads :: () {
    slices: [8] Demo_Slice;
    for *slices {
        it.first = it_index * 1000;
        it.last = it.first + 999;
    }

    pool: Demo_Pool;
    pool.slices = slices;
    check_runtime(demo_mutex_init(*pool.mutex, null));
    check_runtime(demo_condition_init(*pool.ready, null));

    workers: [3] Demo_Worker;
    handles: [3] u64;

    for *workers {
        it.pool = *pool;
        it.index = it_index;
        check_runtime(demo_thread_create(*handles[it_index], null, cast(*void) prime_worker, it));
    }

    for workers {
        result: *void;
        check_runtime(demo_thread_join(handles[it_index], *result));
        expected := ifx it_index == 2 then 2 else 3;
        require_runtime(result == cast(*void) *workers[it_index] && it.completed == expected);
    }

    require_runtime(pool.started == 3);
    check_runtime(demo_condition_destroy(*pool.ready));
    check_runtime(demo_mutex_destroy(*pool.mutex));

    total := 0;
    for slices total += it.primes;

    print("3 workers counted the primes in 8 slices of 1000:");
    for slices print(" %", it.primes);

    print("\n  (% primes below 8000)\n", total);

    ring: Demo_Ring;

    check_runtime(demo_mutex_init(*ring.mutex, null));
    check_runtime(demo_condition_init(*ring.readable, null));
    check_runtime(demo_condition_init(*ring.writable, null));
    producer: u64;

    check_runtime(demo_thread_create(*producer, null, cast(*void) produce_squares, *ring));

    print("The consumer received: ");
    sum := 0;

    for 1..10 {
        check_runtime(demo_mutex_lock(*ring.mutex));
        while ring.count == 0 check_runtime(demo_condition_wait(*ring.readable, *ring.mutex));

        value := ring.slots[ring.head];

        ring.head = (ring.head + 1) % ring.slots.count;
        ring.count -= 1;
        check_runtime(demo_condition_signal(*ring.writable));
        check_runtime(demo_mutex_unlock(*ring.mutex));
        print("% ", value);
        sum += value;
    }

    print("(sum %)\n", sum);
    result: *void;

    check_runtime(demo_thread_join(producer, *result));
    require_runtime(result == cast(*void) *ring && ring.produced == 10 && ring.count == 0 && sum == 385 && ring.waits > 0);
    check_runtime(demo_condition_destroy(*ring.readable));
    check_runtime(demo_condition_destroy(*ring.writable));
    check_runtime(demo_mutex_destroy(*ring.mutex));
    print("The producer had to wait for a free slot % times.\n", ring.waits);
}
