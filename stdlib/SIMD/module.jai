// SIMD: fixed-width vectors of lanes with lane-wise arithmetic, comparison, min/max and memory access.
//
// A `SIMD_Array(N, T)` holds `N` lanes of `T` and is a plain value: it is copied by assignment and no
// operation allocates. Every operation is an ordinary loop over the lanes. There are no hardware vector
// instructions behind them and no platform-specific path, so results are identical on every target; the
// compiler may still vectorise the loops. Integer arithmetic wraps on overflow. Comparisons return a
// vector of 1 and 0 lanes, and `get_mask` packs those into a bit mask.
//
// ```jai
// a := set(4, cast(s32) 3);
// b := a + set(4, cast(s32) 4);
// assert(horizontal_sum(b) == 28);
// assert(get_mask(b > a) == 0b1111);
// ```

#load "../internal/simd/types.jai";
#load "../internal/simd/lanes.jai";
