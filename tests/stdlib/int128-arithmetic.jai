// Concrete arithmetic boundary inputs. Integration runs this authored body;
// parser acceptance alone does not establish any assertion below.
#load "../../stdlib/Basic/Int128.jai";

main :: () -> s64 {
    max_limb: u64 = 0xffff_ffff_ffff_ffff;
    carried := U128.{max_limb, 0} + U128.{1, 0};
    if carried != U128.{0, 1} return 1;
    wrapped := U128.{max_limb, max_limb} + U128.{1, 0};
    if wrapped != U128.{} return 2;
    borrowed := U128.{} - U128.{1, 0};
    if borrowed != U128.{max_limb, max_limb} return 3;
    if -to_s128(0) != S128.{} return 4;
    minimum := S128.{0, cast,no_check(s64) 0x8000_0000_0000_0000};
    if -minimum != minimum return 5;

    high, low := multiply_64bits_with_overflow(max_limb, max_limb);
    if high != max_limb-1 || low != 1 return 6;
    quotient, remainder, error := unsigned_divide_with_remainder(U128.{0, 1}, U128.{3, 0});
    if error || quotient != U128.{6148914691236517205, 0} || remainder != U128.{1, 0} return 7;
    q_top, r_top, e_top := unsigned_divide_with_remainder(U128.{max_limb, max_limb}, U128.{0, 0x8000_0000_0000_0000});
    if e_top || q_top != U128.{1, 0} || r_top != U128.{max_limb, 0x7fff_ffff_ffff_ffff} return 8;
    q_zero, r_zero, e_zero := unsigned_divide_with_remainder(U128.{8, 0}, U128.{});
    if !e_zero || q_zero != U128_ERROR || r_zero != U128_ERROR return 9;

    q_signed, r_signed, e_signed := signed_divide_with_remainder(to_s128(-7), to_s128(3));
    if e_signed || q_signed != to_s128(-2) || r_signed != to_s128(-1) return 10;
    q_negative, r_negative, e_negative := signed_divide_with_remainder(to_s128(7), to_s128(-3));
    if e_negative || q_negative != to_s128(-2) || r_negative != to_s128(1) return 11;
    q_min, r_min, e_min := signed_divide_with_remainder(minimum, to_s128(1));
    if e_min || q_min != minimum || r_min != S128.{} return 12;
    q_overflow, r_overflow, e_overflow := signed_divide_with_remainder(minimum, to_s128(-1));
    if !e_overflow || q_overflow != S128_ERROR || r_overflow != S128_ERROR return 13;

    if (U128.{1, 0} << cast(u8) 127) != U128.{0, 0x8000_0000_0000_0000} return 14;
    if (U128.{1, 0} << cast(u8) 128) != U128.{} return 15;
    minus_one: S128 = to_s128(-1);
    if (minus_one >> cast(u8) 127) != minus_one return 16;
    logical := logical_shift_right(minus_one, cast(u8) 127);
    if logical != S128.{1, 0} return 17;
    if to_u128(cast(s8) -1) != U128.{max_limb, max_limb} return 18;
    if to_s128(max_limb) != S128.{max_limb, 0} return 19;
    if U128.{3, 0} * cast(s64) -1 != U128.{max_limb-2, max_limb} return 20;
    return 0;
}
