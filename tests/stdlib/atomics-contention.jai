// Atomics from several threads at once: every update lands exactly once. Each thread hammers the
// same words with add, increment/decrement, and/or/xor, bit_test_and_set and swap, and a spin
// lock built from atomic_swap guards a non-atomic counter.
#import "Basic";
#import "Thread";
#import "Atomics";

THREADS :: 4;
ROUNDS :: 5000;

Counters :: struct {
    added: s64;
    added_narrow: u16;
    up_down: s32;
    reached_zero: s64;
    toggles: u64;
    claimed_bits: u64;
    claims_won: s64;
    or_bits: u32;
    and_bits: u32;
    spin: bool;
    guarded: s64;
    racy_pad: s64;
    finished: s64;
}

counters: Counters;

hammer :: (thread: *Thread) -> s64 {
    me := cast(s64) thread.data;
    for round: 0..ROUNDS - 1 {
        atomic_add(*counters.added, 3);
        atomic_add(*counters.added_narrow, 1);
        atomic_increment(*counters.up_down);
        if atomic_decrement(*counters.up_down) atomic_increment(*counters.reached_zero);
        // Each thread flips its own byte back and forth: the end state shows no flip was lost.
        atomic_xor(*counters.toggles, (cast(u64) 0xff) << (8 * me));
        // Sixty-four bits claimed by whoever sets each one first: exactly one winner per bit.
        if !atomic_bit_test_and_set(*counters.claimed_bits, round % 64) atomic_increment(*counters.claims_won);
        // Each thread sets and clears its own bits in shared words.
        atomic_or(*counters.or_bits, (cast(u32) 1) << (me * 8 + round % 8));
        atomic_and(*counters.and_bits, ~((cast(u32) 1) << (me * 8 + round % 8)));
        // A spin lock from atomic_swap guarding a plain read-modify-write.
        while atomic_swap(*counters.spin, true) {}
        seen := counters.guarded;
        for 1..2 counters.racy_pad += 1;
        counters.guarded = seen + 1;
        atomic_write(*counters.spin, false);
    }
    atomic_increment(*counters.finished);
    return 0;
}

main :: () {
    counters.and_bits = 0xffff_ffff;
    threads: [THREADS] Thread;
    for *threads {
        it.data = cast(*void) it_index;
        assert(thread_init(it, hammer));
    }
    for *threads assert(thread_start(it));
    // Wait for the workers by spinning on an atomic before joining them.
    while atomic_read(*counters.finished) != THREADS sleep_milliseconds(1);
    for *threads thread_deinit(it);

    total :: THREADS * ROUNDS;
    assert(counters.added == 3 * total, "added %", counters.added);
    assert(counters.added_narrow == cast(u16) total, "added_narrow %", counters.added_narrow);
    assert(counters.up_down == 0, "up_down %", counters.up_down);
    assert(counters.reached_zero >= 1 && counters.reached_zero <= total, "reached_zero %", counters.reached_zero);
    // ROUNDS is even, so every byte was flipped back.
    assert(counters.toggles == 0, "toggles %", formatInt(counters.toggles, base = 16));
    assert(counters.claimed_bits == 0xffff_ffff_ffff_ffff, "claimed %", formatInt(counters.claimed_bits, base = 16));
    assert(counters.claims_won == 64, "claims won %", counters.claims_won);
    assert(counters.or_bits == 0xffff_ffff, "or_bits %", formatInt(counters.or_bits, base = 16));
    assert(counters.and_bits == 0, "and_bits %", formatInt(counters.and_bits, base = 16));
    assert(counters.guarded == total, "guarded %", counters.guarded);
    assert(counters.racy_pad == 2 * total, "pad %", counters.racy_pad);
    print("ok\n");
}
