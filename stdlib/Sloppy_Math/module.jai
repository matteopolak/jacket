// Approximate float comparison and the numeric limits of a type.

// The smallest value of the numeric type `T`: the lowest integer, so `-128` for `s8` and `0` for any unsigned type.
//
// For `float32` and `float64` this is the smallest positive normal number (`Math.FLOAT32_MIN`), not the most
// negative finite value; use `-max_of_type(float32)` for that. Any other type fails an assert.
//
// ```jai
// assert(min_of_type(s8) == -128);
// assert(min_of_type(s32) == -2147483648);
// assert(min_of_type(u16) == 0);
// assert(min_of_type(float32) > 0);
// ```
min_of_type :: Impl.min_of_type;
// The largest finite value of the numeric type `T`: `255` for `u8`, `Math.FLOAT32_MAX` for `float32`. Any non-numeric type fails an assert.
//
// ```jai
// #import "Math";
// assert(max_of_type(u8) == 255);
// assert(max_of_type(s64) == 9223372036854775807);
// assert(max_of_type(u64) == 0xFFFF_FFFF_FFFF_FFFF);
// assert(max_of_type(float32) == FLOAT32_MAX);
// ```
max_of_type :: Impl.max_of_type;
// True when `abs(f)` is at most `epsilon`, which is `0.0001` unless given. The bound is inclusive, and a NaN is never approximately zero.
//
// The default is an absolute tolerance chosen for values near 1; it is far too large for values measured in micro-units and too small for large ones.
//
// ```jai
// assert(is_approximately_zero(0.00005));
// assert(is_approximately_zero(-0.00005));
// assert(!is_approximately_zero(0.01));
// assert(is_approximately_zero(0.01, 0.1));
// ```
is_approximately_zero :: Impl.is_approximately_zero;
// True when `f0` and `f1` differ by at most `epsilon` (`0.0001` unless given). The test is absolute, not relative, and the bound is inclusive.
//
// The `Vector2` and `Vector3` overloads compare every component with the same `epsilon`. Any NaN component makes the answer false, and two infinities of the same sign are not close (their difference is NaN).
//
// ```jai
// #import "Math";
// assert(values_are_close(1.0, 1.00005));
// assert(!values_are_close(1.0, 1.01));
// assert(values_are_close(1.0, 1.01, 0.1));
// assert(values_are_close(xyz(1, 2, 3), xyz(1, 2, 3.00001)));
// assert(!values_are_close(xy(1, 2), xy(1, 3)));
// ```
values_are_close :: Impl.values_are_close;

#scope_file
Impl :: #import "internal/numerics/sloppy";
