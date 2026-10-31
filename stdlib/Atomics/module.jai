// Lock-free reads and updates of a shared integer, pointer or bool.
//
// - [atomic_add], [atomic_increment] and [atomic_decrement] for counters.
// - [atomic_and], [atomic_or], [atomic_xor] and [atomic_bit_test_and_set] for flag words.
// - [atomic_read], [atomic_write] and [atomic_swap] for plain loads, stores and exchanges.
//
// Targets must be 1, 2, 4 or 8 bytes and naturally aligned. `compare_and_swap` is built into the language.
//
// ```jai
// counter: s64;
// before := atomic_add(*counter, 5);
// assert(before == 0 && counter == 5);
// ```

// Stores `value` in `dest` and returns what was there before.
//
// ```jai
// x := 1;
// old := atomic_swap(*x, 2);
// assert(old == 1 && x == 2);
// ```
atomic_swap :: atomic_exchange;

// Returns the value at `pointer`.
//
// ```jai
// x := 7;
// assert(atomic_read(*x) == 7);
// ```
atomic_read :: atomic_load;

// Stores `value` at `pointer`.
//
// ```jai
// x := 0;
// atomic_write(*x, 9);
// assert(x == 9);
// ```
atomic_write :: atomic_store;

#load "../internal/concurrency/atomics.jai";
