// Reviewed Basic wide-integer helpers, through the public facade. Struct-wide
// arithmetic operators from an old private file are not a reviewed public API.
// rules: num.15
#import "Basic";

main :: () -> s64 {
    max_limb: u64 = 0xffff_ffff_ffff_ffff;

    if !same(to_s128(0), S128.{}) return 4;

    minimum := S128.{ 0, cast,no_check(s64) 0x8000_0000_0000_0000 };

    high, low := multiply_64bits_with_overflow(max_limb, max_limb);

    if high != max_limb - 1 || low != 1 return 6;

    quotient, remainder, error := unsigned_divide_with_remainder(U128.{ 0, 1 }, U128.{ 3, 0 });
    if error || !same(quotient, U128.{ 6148914691236517205, 0 }) || !same(remainder, U128.{ 1, 0 }) return 7;

    q_top, r_top, e_top := unsigned_divide_with_remainder(U128.{ max_limb, max_limb }, U128.{ 0, 0x8000_0000_0000_0000 });
    if e_top || !same(q_top, U128.{ 1, 0 }) || !same(r_top, U128.{ max_limb, 0x7fff_ffff_ffff_ffff }) return 8;

    q_zero, r_zero, e_zero := unsigned_divide_with_remainder(U128.{ 8, 0 }, U128.{});
    if !e_zero || !same(q_zero, U128_ERROR) || !same(r_zero, U128_ERROR) return 9;

    q_signed, r_signed, e_signed := signed_divide_with_remainder(to_s128(-7), to_s128(3));
    if e_signed || !same(q_signed, to_s128(-2)) || !same(r_signed, to_s128(-1)) return 10;

    q_negative, r_negative, e_negative := signed_divide_with_remainder(to_s128(7), to_s128(-3));
    if e_negative || !same(q_negative, to_s128(-2)) || !same(r_negative, to_s128(1)) return 11;

    q_min, r_min, e_min := signed_divide_with_remainder(minimum, to_s128(1));
    if e_min || !same(q_min, minimum) || !same(r_min, S128.{}) return 12;

    q_overflow, r_overflow, e_overflow := signed_divide_with_remainder(minimum, to_s128(-1));
    if !e_overflow || !same(q_overflow, S128_ERROR) || !same(r_overflow, S128_ERROR) return 13;

    minus_one: S128 = to_s128(-1);

    logical := logical_shift_right(minus_one, cast(u8) 127);

    if !same(logical, S128.{ 1, 0 }) return 17;
    // Signed inputs to to_u128 are rejected by the reviewed #modify guard;
    // the separate negative fixture retains that exact signed-byte boundary.
    if !same(to_u128(cast(u8) 255), U128.{ 255, 0 }) return 18;
    if !same(to_s128(max_limb), S128.{ max_limb, -1 }) return 19;

    return 0;
}

#scope_file
same :: (a: $T, b: T) -> bool {
    return a.low == b.low && a.high == b.high;
}
