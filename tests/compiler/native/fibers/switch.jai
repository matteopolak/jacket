// The fiber intrinsics on their own: three fibers on stacks of their own pass control round
// robin, and each keeps integer, double and float locals that live across every switch (in
// callee-saved registers once the program is optimized). Then a fiber context is prepared again
// and a thread's own stack is switched back to from a fiber. The output is the same under
// `jaic -interp` and in a native build.
#import "Basic";

Fiber_Context :: struct {
    sp: *void;
    stack_low: *void;
    stack_high: *void;
    stack_dealloc: *void;
}

fiber_init :: (fiber: *Fiber_Context, stack: *void, size: s64, entry: (data: *void) #c_call, data: *void) #intrinsic;
fiber_switch :: (from: *Fiber_Context, to: *Fiber_Context) #intrinsic;
fiber_free :: (fiber: *Fiber_Context) #intrinsic;

STACK_BYTES :: 64 * 1024;
ROUNDS :: 50;

contexts: [4] Fiber_Context;
checksums: [4] s64;
mismatches: s64;

// Where each fiber goes next: 1 -> 2 -> 3 -> back to the main stack (0) on the last round.
destination :: (index: s64, round: s64) -> s64 {
    if index == 3 && round == ROUNDS return 0;
    if index == 3 return 1;

    return index + 1;
}

body :: (index: s64) {
    count: s64 = 0;
    double := cast(float64) index;
    single := 0.5 * cast(float32) index;
    integer := index * 1000;

    for round: 1..ROUNDS {
        double = double * 1.0001 + cast(float64) round;
        single += 0.25;
        integer += round * index;
        count += 1;

        expected_integer := index * 1000 + index * (round * (round + 1) / 2);
        expected_single := 0.5 * cast(float32) index + 0.25 * cast(float32) round;

        fiber_switch(*contexts[index], *contexts[destination(index, round)]);

        // Everything this fiber held across the switch must have come back unchanged.
        if integer != expected_integer || single != expected_single || count != round mismatches += 1;
    }

    checksums[index] = integer + cast(s64) double + count;
}

entry :: (data: *void) #c_call {
    new_context: #Context;
    push_context new_context {
        body(cast(s64) data);
    }

    // A fiber never returns; this one is finished.
    while true fiber_switch(*contexts[cast(s64) data], *contexts[0]);
}

hops: s64;

hop_entry :: (data: *void) #c_call {
    new_context: #Context;
    push_context new_context {
        // Switch back to the main stack three times, then once more for good.
        for 1..3 {
            hops += 1;
            fiber_switch(*contexts[1], *contexts[0]);
        }
    }

    while true fiber_switch(*contexts[1], *contexts[0]);
}

main :: () {
    stacks: [4] *void;

    for 1..3 {
        stacks[it] = alloc(STACK_BYTES);
        fiber_init(*contexts[it], stacks[it], STACK_BYTES, entry, cast(*void) it);
    }

    fiber_switch(*contexts[0], *contexts[1]);

    // The fibers are suspended after their last switch: let each one finish.
    for 1..3 fiber_switch(*contexts[0], *contexts[it]);

    print("mismatches %\n", mismatches);
    for 1..3 print("fiber %: %\n", it, checksums[it]);

    assert(mismatches == 0);

    // The same context and stack can be prepared again once the fiber is done.
    for 0..3 checksums[it] = 0;
    for 1..3 fiber_init(*contexts[it], stacks[it], STACK_BYTES, entry, cast(*void) it);

    fiber_switch(*contexts[0], *contexts[1]);
    for 1..3 fiber_switch(*contexts[0], *contexts[it]);

    print("again: mismatches %\n", mismatches);
    for 1..3 print("fiber %: %\n", it, checksums[it]);

    // A fiber that comes back to the main stack several times.
    fiber_init(*contexts[1], stacks[1], STACK_BYTES, hop_entry, null);
    for 1..4 fiber_switch(*contexts[0], *contexts[1]);

    print("hops %\n", hops);

    for 1..3 {
        fiber_free(*contexts[it]);
        free(stacks[it]);
    }

    print("ok\n");
}
