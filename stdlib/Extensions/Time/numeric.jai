// Checked integer and 128-bit arithmetic shared by every area.

#scope_module

range_error :: () -> Time_Error {
    return .{ .OUT_OF_RANGE, "outside supported range", -1 };
}

cmp64 :: (a: s64, b: s64) -> int {
    if a < b return -1;
    if a > b return 1;
    return 0;
}

floor_div :: (a: s64, b: s64) -> s64 {
    q := a / b;
    if a % b < 0 q -= 1;
    return q;
}

modulo :: (a: s64, b: s64) -> s64 {
    r := a % b;
    if r < 0 r += b;
    return r;
}

wide_mul :: (a: s64, b: s64) -> S128 {
    negative := (a < 0) != (b < 0);
    ua := cast,no_check(u64) a;

    if a < 0 ua = 0 - ua;

    ub := cast,no_check(u64) b;
    if b < 0 ub = 0 - ub;

    h, l := multiply_64bits_with_overflow(ua, ub);
    v := S128.{ l, cast, no_check(s64) h };

    if negative v = S128.{} - v;

    return v;
}

narrow :: (v: S128) -> s64 {
    return cast,no_check(s64) v.low;
}

wide_div :: (v: S128, divisor: s64) -> S128, s64 {
    if v == to_s128(narrow(v)) return to_s128(narrow(v) / divisor), narrow(v) % divisor;
    q, r, _ := signed_divide_with_remainder(v, to_s128(divisor));
    return q, narrow(r);
}

wide_compare :: (a: S128, b: S128) -> int {
    if a < b return -1;
    if a > b return 1;
    return 0;
}

wide_abs :: (v: S128) -> S128 {
    if v.high < 0 return S128.{} - v;
    return v;
}

wide_float :: (v: S128) -> float64 {
    n := wide_abs(v);
    f := cast(float64) n.high * 18446744073709551616.0 + cast(float64) n.low;
    return ifx v.high < 0 then -f else f;
}
