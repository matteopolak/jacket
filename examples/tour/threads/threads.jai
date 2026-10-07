// Threads: a Thread_Group sharing out work, and a producer and a consumer
// handing items over through semaphores. In the browser every thread takes
// turns on one host thread: a thread that waits is suspended and resumed later.

#import "Thread";

tour_threads :: () {
    // A Thread_Group runs one procedure over many work items on a few threads.
    // Here each item counts the primes in its own slice of numbers.
    slices: [8] Slice;
    for *slices {
        it.first = it_index * 1000;
        it.last = it.first + 999;
    }

    group: Thread_Group;
    init(*group, 3, count_primes_in_slice);
    group.logging = false;
    start(*group);
    for *slices add_work(*group, it);

    // Collect finished items until all eight are back.
    done := 0;
    while done < slices.count {
        done += get_completed_work(*group).count;
        if done < slices.count sleep_milliseconds(1);
    }
    shutdown(*group);

    total := 0;
    for slices total += it.primes;
    print("3 workers counted the primes in 8 slices of 1000:");
    for slices print(" %", it.primes);
    print("\n  (% primes below 8000)\n", total);

    // A producer and a consumer share a ring of four slots. One semaphore counts
    // the free slots, the other the filled ones, so neither side ever overruns.
    ring: Ring_Buffer;
    init(*ring.free_slots, ring.slots.count);
    init(*ring.filled_slots, 0);

    producer: Thread;
    thread_init(*producer, produce_squares);
    producer.data = *ring;
    thread_start(*producer);

    print("The consumer received: ");
    sum := 0;
    for 1..10 {
        wait_for(*ring.filled_slots);
        value := ring.slots[it % ring.slots.count];
        signal(*ring.free_slots);
        print("% ", value);
        sum += value;
    }
    print("(sum %)\n", sum);

    thread_is_done(*producer, -1);
    thread_deinit(*producer);
    print("The producer had to wait for a free slot % times.\n", ring.waits);
}

count_primes_in_slice :: (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status {
    slice := cast(*Slice) work;
    for n: slice.first..slice.last {
        if n < 2 continue;
        is_prime := true;
        d := 2;
        while d * d <= n {
            if n % d == 0 {
                is_prime = false;
                break;
            }
            d += 1;
        }
        if is_prime slice.primes += 1;
    }
    return .CONTINUE;
}

// One work item: a range of numbers and the count found in it.
Slice :: struct {
    first, last: int;
    primes: int;
}

Ring_Buffer :: struct {
    slots: [4] int;
    free_slots: Semaphore;
    filled_slots: Semaphore;
    waits: int;
}

produce_squares :: (thread: *Thread) -> s64 {
    ring := cast(*Ring_Buffer) thread.data;
    for 1..10 {
        // A zero timeout only checks: count the times the ring was full.
        if wait_for(*ring.free_slots, 0) == .TIMEOUT {
            ring.waits += 1;
            wait_for(*ring.free_slots);
        }
        ring.slots[it % ring.slots.count] = it * it;
        signal(*ring.filled_slots);
    }
    return 0;
}
