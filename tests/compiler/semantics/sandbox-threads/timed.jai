#load "provider.jai";
#import "Atomics";

late :: (data: *void) -> *void #c_call {
    push_context, defer_pop starting_context;
    assert(usleep(50000) == 0);
    signal_permit();
    assert(usleep(50000) == 0);
    atomic_write(*flag, 1);

    return data;
}

main :: () {
    initialize();
    thread: u64;

    assert(pthread_create(*thread, null, cast(*void) late, null) == 0);
    early := wait_permit(10);
    later := wait_permit(1000);

    assert(early == 110, "provider ETIMEDOUT: %", early);
    assert(later == 0);
    while atomic_read(*flag) == 0 {}

    assert(pthread_join(thread, null) == 0);
    print("TIMEOUT SUCCESS\n");
}
