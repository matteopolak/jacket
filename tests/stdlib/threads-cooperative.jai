// Threads run under the interpreter's cooperative scheduler: start, join, a mutex and an atomic.
#import "Basic";
#import "Atomics";
#import "Thread";

counter: s64;
total: s64;
mutex: Mutex;

worker :: (thread: *Thread) -> s64 {
    for 0..99 {
        lock(*mutex);
        total += 1;
        unlock(*mutex);
        atomic_add(*counter, 1);
    }
    return thread.index;
}

main :: () {
    // As in `jai` (with Thread's DEBUG off), a mutex is just the OS lock and is recursive.
    #assert size_of(Mutex) == size_of(Per_OS_Critical_Section_Type);
    #assert type_info(Mutex).members.count == 1;
    init(*mutex);
    lock(*mutex); lock(*mutex); unlock(*mutex); unlock(*mutex);
    threads: [3] Thread;
    for *threads {
        thread_init(it, worker);
        thread_start(it);
    }
    for *threads thread_deinit(it);
    assert(total == 300, "total %", total);
    assert(counter == 300, "counter %", counter);
    print("ok\n");
}
