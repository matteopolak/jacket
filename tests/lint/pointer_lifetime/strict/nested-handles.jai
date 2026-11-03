#import "Basic";

lib :: #library "c";
close_handle :: (handle: s64) #foreign lib "close_handle"; @Closes
access_handle :: (handle: s64) #foreign lib "access_handle"; @NoLifetimeEffects

Inner :: struct {
    handle: s64;
    value: int;
}

Outer :: struct {
    link: *Inner;
}

close_child :: (outer: *Outer) {
    close_handle(outer.link.handle);
}

safe_header :: () {
    link := New(Inner);
    outer := Outer.{ link = link };

    close_child(*outer);
    link.value = 42;
    assert(link.value == 42);
    free(link);
}

// Opaque scalar handles keep their direct close/use contract.
bad_direct :: (handle: s64) {
    close_handle(handle);
    access_handle(handle);
}
