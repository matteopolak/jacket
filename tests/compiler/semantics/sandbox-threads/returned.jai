#load "provider.jai";

quick :: (data: *void) -> *void #c_call {
    return data;
}

main :: () {
    initialize();
    thread: u64;

    assert(pthread_create(*thread, null, cast(*void) quick, null) == 0);
    assert(wait_permit() == 0);
    print("woken\n");
}
