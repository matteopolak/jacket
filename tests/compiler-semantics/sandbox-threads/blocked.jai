#load "provider.jai";

stuck :: (data: *void) -> *void #c_call {
    push_context, defer_pop starting_context;
    assert(wait_permit() == 0);
    return data;
}

main :: () {
    initialize();
    thread: u64;

    assert(pthread_create(*thread, null, cast(*void) stuck, null) == 0);
    assert(pthread_join(thread, null) == 0);
    print("joined\n");
}
