#import "Basic";
#import "Thread";
#import "File";
#load "helpers.jai";

saved: #Context;
guard: Mutex;

make_counter :: () -> *int {
    return New(int);
}

collect :: () -> [..] int {
    items: [..] int;
    array_add(*items, 1);
    return items;
}

describe :: (n: int) -> string {
    return tprint("n = %", n);
}

fetch :: () -> *int {
    return grow();
}

factorial :: (n: int) -> int {
    if n < 2 return 1;
    return n * factorial(n - 1);
}

ping :: (n: int) {
    if n > 0 pong(n - 1);
}

pong :: (n: int) {
    if n > 0 ping(n - 1);
}

buffer_work :: () {
    scratch: [4096] u8;
    scratch[0] = 1;
}

report :: () {
    print("tick\n");
}

service :: () {
    report();
    lock(*guard);
    unlock(*guard);
    sleep_milliseconds(1);
    cell := New(int);

    free(cell);
}

#program_export "on_tick"
on_tick :: () #c_call #no_context {
    push_context saved {
        service();
    }
} @Interrupt

main :: () {
    make_counter();
    collect();
    describe(1);
    fetch();
    factorial(5);
    ping(3);
    buffer_work();
}
