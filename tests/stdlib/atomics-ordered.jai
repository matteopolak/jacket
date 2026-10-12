// The ordered Atomics operations (atomic_load, atomic_store, atomic_exchange,
// atomic_compare_exchange and the atomic_fetch_* family): the result and the value left behind
// for every width, each memory order, floats, bools, enums and pointers, then a release/acquire
// hand-off between two threads and a lock built from the orders.
#import "Basic";
#import "Thread";
#import "Atomics";

Color :: enum u8 {
    RED;
    GREEN;
    BLUE;
}

check_integer :: ($T: Type) {
    value: T = 5;

    assert(atomic_load(*value) == 5, "%: load", T);
    assert(atomic_load(*value, .ACQUIRE) == 5, "%: acquire load", T);
    assert(atomic_load(*value, .RELAXED) == 5, "%: relaxed load", T);
    atomic_store(*value, 9);
    assert(value == 9, "%: store left %", T, value);
    atomic_store(*value, 10, .RELEASE);
    assert(value == 10, "%: release store left %", T, value);
    atomic_store(*value, 11, .RELAXED);
    assert(value == 11, "%: relaxed store left %", T, value);

    old := atomic_exchange(*value, 3);

    assert(old == 11 && value == 3, "%: exchange gave % and left %", T, old, value);
    old = atomic_exchange(*value, 4, .ACQ_REL);
    assert(old == 3 && value == 4, "%: acq_rel exchange gave % and left %", T, old, value);

    old = atomic_fetch_add(*value, 4);
    assert(old == 4 && value == 8, "%: fetch_add gave % and left %", T, old, value);
    old = atomic_fetch_sub(*value, 3, .RELAXED);
    assert(old == 8 && value == 5, "%: fetch_sub gave % and left %", T, old, value);
    old = atomic_fetch_and(*value, 0b100, .ACQUIRE);
    assert(old == 5 && value == 0b100, "%: fetch_and gave % and left %", T, old, value);
    old = atomic_fetch_or(*value, 0b011, .RELEASE);
    assert(old == 0b100 && value == 0b111, "%: fetch_or gave % and left %", T, old, value);
    old = atomic_fetch_xor(*value, 0b101);
    assert(old == 0b111 && value == 0b010, "%: fetch_xor gave % and left %", T, old, value);

    // Adding wraps at the width of the type.
    top: T = 0;
    top = ~top;
    old = atomic_fetch_add(*top, 1);
    assert(old == ~cast(T) 0 && top == 0, "%: wrapping add gave % and left %", T, old, top);

    ok, found := atomic_compare_exchange(*value, 2, 20);

    assert(ok && found == 2 && value == 20, "%: exchange hit gave % % and left %", T, ok, found, value);
    ok, found = atomic_compare_exchange(*value, 2, 30, .ACQ_REL, .ACQUIRE);
    assert(!ok && found == 20 && value == 20, "%: exchange miss gave % % and left %", T, ok, found, value);
    ok, found = atomic_compare_exchange(*value, 20, 21, .RELEASE, .RELAXED);
    assert(ok && value == 21, "%: release exchange left %", T, value);
    ok, found = atomic_compare_exchange(*value, 21, 22, .RELAXED, .RELAXED);
    assert(ok && value == 22, "%: relaxed exchange left %", T, value);
}

check_other :: () {
    flag := false;

    atomic_store(*flag, true, .RELEASE);
    assert(atomic_load(*flag, .ACQUIRE), "bool store");
    assert(atomic_exchange(*flag, false), "bool exchange returns the old value");
    assert(!flag, "bool exchange stores");

    color := Color.RED;

    atomic_store(*color, .BLUE);
    assert(atomic_load(*color) == .BLUE, "enum load");
    ok, found := atomic_compare_exchange(*color, .BLUE, .GREEN);

    assert(ok && found == .BLUE && color == .GREEN, "enum compare exchange");

    ratio: float32 = 1.5;

    assert(atomic_load(*ratio) == 1.5, "float32 load");
    atomic_store(*ratio, 2.5);
    assert(atomic_exchange(*ratio, 3.5) == 2.5 && ratio == 3.5, "float32 exchange");
    ok_f, found_f := atomic_compare_exchange(*ratio, 3.5, 4.5);

    assert(ok_f && found_f == 3.5 && ratio == 4.5, "float32 compare exchange");

    wide: float64 = -0.25;

    assert(atomic_exchange(*wide, 8.0) == -0.25 && wide == 8.0, "float64 exchange");

    items: [4] s64;
    cursor: *s64 = *items[0];

    assert(atomic_load(*cursor) == *items[0], "pointer load");
    atomic_store(*cursor, *items[1], .RELEASE);
    assert(cursor == *items[1], "pointer store");
    before := atomic_exchange(*cursor, *items[2]);

    assert(before == *items[1] && cursor == *items[2], "pointer exchange");
    before = atomic_fetch_add(*cursor, 8);
    assert(before == *items[2] && cursor == *items[3], "pointer fetch_add moves by bytes");
    before = atomic_fetch_sub(*cursor, 16, .RELAXED);
    assert(before == *items[3] && cursor == *items[1], "pointer fetch_sub moves by bytes");
    ok_p, found_p := atomic_compare_exchange(*cursor, *items[1], *items[0]);

    assert(ok_p && found_p == *items[1] && cursor == *items[0], "pointer compare exchange");
}

// A producer fills a plain buffer, then publishes it with a release store; the consumer waits with
// acquire loads and must see the whole buffer.
Handoff :: struct {
    ready: u32;
    payload: [64] s64;
    sum: s64;
}

handoff: Handoff;

produce :: (thread: *Thread) -> s64 {
    for *handoff.payload it.* = it_index + 1;
    atomic_store(*handoff.ready, 1, .RELEASE);

    return 0;
}

consume :: (thread: *Thread) -> s64 {
    while atomic_load(*handoff.ready, .ACQUIRE) == 0 sleep_milliseconds(1);

    for handoff.payload handoff.sum += it;

    return 0;
}

// A spin lock from an acquire exchange and a release store guarding plain counters.
Lock_State :: struct {
    taken: u32;
    count: s64;
    pad: s64;
    done: s64;
}

lock_state: Lock_State;
LOCK_THREADS :: 4;
LOCK_ROUNDS :: 5000;

contend :: (thread: *Thread) -> s64 {
    for 1..LOCK_ROUNDS {
        while atomic_exchange(*lock_state.taken, 1, .ACQUIRE) != 0 {}

        seen := lock_state.count;

        lock_state.pad += 1;
        lock_state.count = seen + 1;
        atomic_store(*lock_state.taken, 0, .RELEASE);
    }

    atomic_fetch_add(*lock_state.done, 1, .RELEASE);

    return 0;
}

main :: () {
    check_integer(u8);
    check_integer(s8);
    check_integer(u16);
    check_integer(s16);
    check_integer(u32);
    check_integer(s32);
    check_integer(u64);
    check_integer(s64);
    check_other();

    consumer, producer: Thread;

    assert(thread_init(*consumer, consume));
    assert(thread_init(*producer, produce));
    assert(thread_start(*consumer));
    sleep_milliseconds(5);
    assert(thread_start(*producer));
    thread_deinit(*consumer);
    thread_deinit(*producer);
    assert(handoff.sum == 64 * 65 / 2, "the consumer summed % through the acquire load", handoff.sum);

    threads: [LOCK_THREADS] Thread;
    for *threads assert(thread_init(it, contend));
    for *threads assert(thread_start(it));
    while atomic_load(*lock_state.done, .ACQUIRE) != LOCK_THREADS sleep_milliseconds(1);
    for *threads thread_deinit(it);

    assert(lock_state.count == LOCK_THREADS * LOCK_ROUNDS, "the lock lost updates: %", lock_state.count);
}
