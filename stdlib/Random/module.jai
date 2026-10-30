// Random: a fast 128-bit multiplicative generator for non-cryptographic random numbers.
//
// Every procedure comes in two forms: one that takes a `*Random_State` of your own and one without a state
// that uses `context.mcg_state` (the module adds that field to the context, so each thread and each context
// copy has its own stream). The sequence is fully determined by the seed. A context that is never seeded
// starts from the same state every run, so seed from the clock when runs should differ. Nothing here is
// suitable for cryptography, and a state must not be shared between threads without a lock.

// The generator's 128-bit state, as two halves. Seed it with `random_seed` before drawing: an all-zero state (a
// plain `state: Random_State;`) is a fixed point of the generator and only ever yields zero.
//
// Seeding keeps the low half odd, which is what gives the generator its full period; the draw itself needs no other setup.
Random_State :: Impl.Random_State;
// What `random_seed` accepts for a seed from the clock: any struct with a `u64` named `low` and an `s64` named `high`, such as an `Apollo_Time`.
Some_Big_Time_Value :: Impl.Some_Big_Time_Value;
// The next 64 random bits, advancing the state. Without a state it draws from `context.mcg_state`.
//
// All 64 bits are usable; take `% n` of the result for a small bound, remembering the slight bias that causes when `n` is not a power of two.
//
// ```jai
// state: Random_State;
// random_seed(*state, cast(u64) 42);
// first := random_get(*state);
//
// random_seed(*state, cast(u64) 42);
// assert(random_get(*state) == first); // same seed, same stream
// assert(random_get(*state) != first);
// ```
random_get :: Impl.random_get;
// Starts the generator from `new_seed`. A `u64` seed fully determines the stream, as does a `Random_State`, which is copied with its low half forced odd, or a time value (`low` and `high` are copied the same way).
//
// Without a state the context's generator is seeded. Seeding twice with equal values gives equal streams.
//
// ```jai
// a, b: Random_State;
// random_seed(*a, cast(u64) 7);
// random_seed(*b, cast(u64) 7);
// assert(random_get(*a) == random_get(*b));
//
// Time :: struct { low: u64; high: s64; }
// random_seed(*a, Time.{ 4, 9 });
// assert(a.low == 5 && a.high == 9);
// ```
random_seed :: Impl.random_seed;
// A `float` in `[0, 1]`, both ends possible, from 24 random bits divided by `2^24 - 1`. The values are evenly spaced, 24 bits of resolution.
//
// Use `random_get_zero_to_one_open` when `1` must not occur, for example to index an array with `cast(int) (r * count)`.
//
// ```jai
// state: Random_State;
// random_seed(*state, cast(u64) 1);
// for 1..1000 {
//     r := random_get_zero_to_one(*state);
//     assert(r >= 0 && r <= 1);
// }
// ```
random_get_zero_to_one :: Impl.random_get_zero_to_one;
// A `float` in `[0, 1)`: 24 random bits divided by `2^24`, so `1` never occurs.
//
// ```jai
// state: Random_State;
// random_seed(*state, cast(u64) 1);
// for 1..1000 {
//     r := random_get_zero_to_one_open(*state);
//     assert(r >= 0 && r < 1);
//     assert(cast(int) (r * 10) < 10);
// }
// ```
random_get_zero_to_one_open :: Impl.random_get_zero_to_one_open;
// A `float64` in `[0, 1)` from all 64 random bits, rounding down to the largest value below 1 when the division would round up to 1. The values are not evenly spaced: the `float64` grid is finer near 0 than near 1, so prefer `random_get_zero_to_one_open_new` when an even grid matters.
random_get_zero_to_one_new :: Impl.random_get_zero_to_one_new;
// A `float64` in `[0, 1)` built from 53 random bits, so every multiple of `2^-53` is equally likely. This is the form to prefer for double precision.
//
// ```jai
// state: Random_State;
// random_seed(*state, cast(u64) 3);
// for 1..1000 {
//     r := random_get_zero_to_one_open_new(*state);
//     assert(r >= 0 && r < 1);
// }
// ```
random_get_zero_to_one_open_new :: Impl.random_get_zero_to_one_open_new;
// A `float` between `min` and `max`, both ends possible, computed as `min + (max - min) * r` and clamped back into the range if rounding strays.
//
// The bounds must be in order: with `min > max` the result is `min` (a draw that happens to be exactly zero gives `max`, an inconsistency of the clamping). Equal bounds give that value. The draw has 24 bits of resolution, so the result takes at most 2^24 distinct values across the range.
//
// ```jai
// state: Random_State;
// random_seed(*state, cast(u64) 5);
// for 1..1000 {
//     v := random_get_within_range(*state, -2.0, 3.0);
//     assert(v >= -2 && v <= 3);
// }
// assert(random_get_within_range(*state, 4.0, 4.0) == 4);
// ```
random_get_within_range :: Impl.random_get_within_range;

#scope_file
Impl :: #import "internal/random/mcg";
