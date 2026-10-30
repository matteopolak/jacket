// PCG: a small 32-bit-output permuted congruential generator (XSH-RR) over a 64-bit state.
//
// Each procedure comes in two forms: one that takes the state as a `*u64` you own, and one without a state
// that uses a single module-global `u64`. The global is shared by every thread and is not locked, so use
// your own state when more than one thread draws. The global starts at a fixed value, so an unseeded program
// sees the same numbers every run. The generator is not suitable for cryptography.

// The next 32 random bits, advancing the state. Any `u64` is a valid state, including 0.
//
// The sequence is fixed by the state, so a seed reproduces it on every platform.
//
// ```jai
// state: u64;
// random_seed(*state, 42);
// assert(state == 0x91778aed87ee5eb1); // seeding sets the state from the seed
// assert(random_get(*state) == 0x75830bbd);
// assert(random_get(*state) == 0x0e6dfdb2);
// ```
random_get :: Impl.random_get;
// A value in `[0, bound)` without modulo bias, drawn by rejecting the few low outputs that would make some results likelier. `bound` must not be 0 (an assert fails); with `bound` 1 the result is always 0.
//
// More than one draw may be consumed, so the state afterwards depends on the bound.
//
// ```jai
// state: u64;
// random_seed(*state, 9);
// counts: [6] int;
// for 1..6000 counts[random_get_within_bound(*state, 6)] += 1;
// for counts assert(it > 800 && it < 1200);
// ```
random_get_within_bound :: Impl.random_get_within_bound;
// The same as `random_get_within_bound`, kept as a second name for programs that call it.
random_get_within_bound2 :: Impl.random_get_within_bound2;
// Starts the generator from `new_seed`: the state becomes `seed * 6364136223846793005 + 1442695040888963407` (wrapping), so the first draw comes from that value. Equal seeds give equal sequences.
//
// Without a state it seeds the shared global. Writing a different `u64` straight into your own state is also valid; seeding only spreads small seeds over the 64 bits.
//
// ```jai
// a, b: u64;
// random_seed(*a, 1234);
// random_seed(*b, 1234);
// for 1..10 assert(random_get(*a) == random_get(*b));
// ```
random_seed :: Impl.random_seed;
// A `float` in `[0, 1)` made by dividing a 32-bit draw by `2^32`.
//
// A float has only 24 bits of precision, so a draw within 128 of the top would round up to `1.0`; those draws give the largest float below 1 instead, so `1` never occurs.
//
// ```jai
// state: u64;
// random_seed(*state, 3);
// for 1..1000 {
//     r := random_get_zero_to_one(*state);
//     assert(r >= 0 && r < 1);
// }
// ```
random_get_zero_to_one :: Impl.random_get_zero_to_one;
// A `float` between `min` and `max`, interpolated from `random_get_zero_to_one`. The result is clamped into the range (so `max` itself can occur); equal bounds give that value.
//
// The bounds can be given in either order: with `min > max` the result lies between `max` and `min`.
//
// ```jai
// state: u64;
// random_seed(*state, 7);
// for 1..1000 {
//     v := random_get_within_range(*state, -1.0, 2.0);
//     assert(v >= -1 && v <= 2);
//     w := random_get_within_range(*state, 2.0, -1.0);
//     assert(w >= -1 && w <= 2);
// }
// assert(random_get_within_range(*state, 5.0, 5.0) == 5);
// ```
random_get_within_range :: Impl.random_get_within_range;

#scope_file
Impl :: #import "internal/random/pcg";
