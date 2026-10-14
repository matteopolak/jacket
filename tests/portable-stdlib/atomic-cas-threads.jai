#import "Basic";
A :: #import "Atomics";
T :: #import "Thread";

Shared :: struct {
    sequence: s64;
    narrow: u16;
    bits: u64;
    toggles: u64;
    set: u32;
    clear: u32 = 0xffffffff;
    decrement: s32 = 4000;
    zeros: s64;
}

Worker :: struct {
    shared: *Shared;
    index: s64;
    sum: s64;
    claims: s64;
}

worker :: (thread: *T.Thread) -> s64 {
    state := cast(*Worker) thread.data;
    shared := state.shared;

    for round: 0..999 {
        state.sum += A.atomic_add(*shared.sequence, 1);
        A.atomic_increment(*shared.narrow);
        if !A.atomic_bit_test_and_set(*shared.bits, round % 64) state.claims += 1;

        A.atomic_xor(*shared.toggles, (cast(u64) 0xff) << (state.index * 8));
        A.atomic_or(*shared.set, (cast(u32) 1) << (state.index * 8 + round % 8));
        A.atomic_and(*shared.clear, ~((cast(u32) 1) << (state.index * 8 + round % 8)));
        if A.atomic_decrement(*shared.decrement) A.atomic_increment(*shared.zeros);
    }

    return 0;
}

main :: () {
    shared: Shared;
    states: [4] Worker;
    threads: [4] T.Thread;

    for index: 0..3 {
        states[index].shared = *shared;
        states[index].index = index;
        threads[index].data = *states[index];
        assert(T.thread_init(*threads[index], worker));
        T.thread_start(*threads[index]);
    }

    sum: s64;
    claims: s64;

    for index: 0..3 {
        assert(T.thread_is_done(*threads[index], 10000));
        T.thread_deinit(*threads[index]);
        sum += states[index].sum;
        claims += states[index].claims;
    }

    assert(shared.sequence == 4000 && sum == 4000 * 3999 / 2);
    assert(shared.narrow == 4000 && shared.bits == 0xffffffffffffffff && claims == 64);
    assert(shared.toggles == 0 && shared.set == 0xffffffff && shared.clear == 0);
    assert(shared.decrement == 0 && shared.zeros == 1);
    print("ok\n");
}
