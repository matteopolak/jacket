#import "Basic";

#if OS != .NONE {
    #import "Thread";
}

saved: #Context;
ticks: int;

make_counter :: () -> *int {
    push_allocator(temp);
    return New(int);
}

collect :: () -> [..] int {
    items: [..] int;

    push_context saved {
        array_add(*items, 1);
    }

    return items;
}

// Never reached from `main`, an exported procedure or an interrupt handler.
spare :: () -> *int {
    return New(int);
}

factorial :: (n: int) -> int {
    result := 1;
    for 2..n result *= it;
    return result;
}

buffer_work :: () {
    scratch: [64] u8;
    scratch[0] = 1;
}

#program_export "on_tick"
on_tick :: () #c_call #no_context {
    ticks += 1;
} @Interrupt

main :: () {
    make_counter();
    collect();
    factorial(5);
    buffer_work();
}
