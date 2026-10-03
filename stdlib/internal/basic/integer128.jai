S128 :: struct {
    low: u64;
    high: s64;
}

U128 :: struct {
    low: u64;
    high: u64;
}

U128_ERROR :: U128.{ 0xffffffffffffffff, 0xffffffffffffffff };
S128_ERROR :: S128.{ 0xffffffffffffffff, -1 };

operator == :: inline (a: S128, b: S128) -> bool {
    return a.low == b.low && a.high == b.high;
}

operator ! :: inline (a: S128) -> bool {
    return a.low == 0 && a.high == 0;
}

operator < :: (a: S128, b: S128) -> bool {
    if a.high != b.high return a.high < b.high;
    return a.low < b.low;
}

operator <= :: (a: S128, b: S128) -> bool {
    if a.high != b.high return a.high < b.high;
    return a.low <= b.low;
}

operator > :: (a: S128, b: S128) -> bool {
    if a.high != b.high return a.high > b.high;
    return a.low > b.low;
}

operator >= :: (a: S128, b: S128) -> bool {
    if a.high != b.high return a.high > b.high;
    return a.low >= b.low;
}

operator + :: (a: S128, b: S128) -> S128 #no_aoc {
    low := a.low + b.low;
    carry: s64 = ifx low < a.low then 1 else 0;
    return .{ low, a.high + b.high + carry };
}

operator - :: (a: S128, b: S128) -> S128 #no_aoc {
    borrow: s64 = ifx a.low < b.low then 1 else 0;
    return .{ a.low - b.low, a.high - b.high - borrow };
}

operator - :: (a: U128, b: U128) -> U128 #no_aoc {
    borrow: u64 = ifx a.low < b.low then 1 else 0;
    return .{ a.low - b.low, a.high - b.high - borrow };
}

multiply_64bits_with_overflow :: (a: u64, b: u64) -> (high: u64, low: u64) #no_aoc {
    a0 := a & 0xffffffff;
    a1 := a >> 32;
    b0 := b & 0xffffffff;
    b1 := b >> 32;
    w0 := a0 * b0;
    t := a1 * b0 + (w0 >> 32);
    w1 := (t & 0xffffffff) + a0 * b1;

    return a1 * b1 + (t >> 32) + (w1 >> 32), (w1 << 32) | (w0 & 0xffffffff);
}

to_u128 :: (a: $T) -> U128 #modify {
    return T == u8 || T == u16 || T == u32 || T == u64, "to_u128 requires a built-in unsigned integer type";
} {
    return .{ cast(u64) a, 0 };
}

to_s128 :: (a: $T) -> S128 #modify {
    return (cast(*Type_Info) T).type == .INTEGER, "to_s128 requires a reflected integer type";
} {
    return .{ cast,no_check(u64) a, (cast(s64) a) >> 63 };
}

logical_shift_right :: (a: S128, $$x: u8) -> S128 {
    if x == 0 return a;
    if x >= 128 return .{};

    high := cast(u64) a.high;
    if x >= 64 return .{ high >> (x - 64), 0 };

    return .{ (a.low >> x) | (high << (64 - x)), cast(s64) (high >> x) };
}

unsigned_divide_with_remainder :: (u: U128, v: U128) -> quotient: U128, remainder: U128, overflow_or_divide_by_zero: bool #no_aoc {
    if v.low == 0 && v.high == 0 return U128_ERROR, U128_ERROR, true;

    q: U128;
    r: U128;
    bit: s64 = 127;

    while bit >= 0 {
        carry := r.high >> 63;

        r.high = (r.high << 1) | (r.low >> 63);
        r.low <<= 1;
        if bit >= 64 r.low |= (u.high >> (bit - 64)) & 1;
        else r.low |= (u.low >> bit) & 1;

        if carry != 0 || r.high > v.high || (r.high == v.high && r.low >= v.low) {
            borrow: u64;
            if r.low < v.low borrow = 1;

            r.low -= v.low;
            r.high -= v.high + borrow;
            if bit >= 64 q.high |= (cast(u64) 1) << (bit - 64);
            else q.low |= (cast(u64) 1) << bit;
        }

        bit -= 1;
    }

    return q, r, false;
}

signed_divide_with_remainder :: inline (dividend: S128, divisor: S128) -> quotient: S128, remainder: S128, overflow_or_divide_by_zero: bool #no_aoc {
    if dividend.low == 0 && dividend.high == -9223372036854775808 && divisor.low == 0xffffffffffffffff && divisor.high == -1 return S128_ERROR, S128_ERROR, true;

    a := U128.{ dividend.low, cast,no_check(u64) dividend.high };
    b := U128.{ divisor.low, cast,no_check(u64) divisor.high };

    if dividend.high < 0 a = negate_wide(a);
    if divisor.high < 0 b = negate_wide(b);

    q, r, error := unsigned_divide_with_remainder(a, b);
    if error return S128_ERROR, S128_ERROR, true;
    if (dividend.high < 0) != (divisor.high < 0) q = negate_wide(q);
    if dividend.high < 0 r = negate_wide(r);

    return S128.{ q.low, cast,no_check(s64) q.high }, S128.{ r.low, cast,no_check(s64) r.high }, false;
}

#if CPU == .X64 {
    test_unsigned_division :: (random_iterations := 0, random_get: () -> u64 = null) {
        exercise_unsigned_division(0, 0, 1);
        exercise_unsigned_division(0xffffffffffffffff, 0, 1);
        exercise_unsigned_division(0, 1, 3);
        exercise_unsigned_division(0xffffffffffffffff, 1, 2);
        exercise_unsigned_division(0xffffffffffffffff, 0xfffffffffffffffe, 0xffffffffffffffff);

        if random_iterations > 0 assert(random_get != null);

        iteration: int;
        while iteration < random_iterations {
            divisor := random_get();

            assert(divisor != 0);
            high := random_get() % divisor;
            low := random_get();

            exercise_unsigned_division(low, high, divisor);
            iteration += 1;
        }
    }
}

#scope_file
exercise_unsigned_division :: (low: u64, high: u64, divisor: u64) #no_aoc {
    quotient, remainder, error := unsigned_divide_with_remainder(.{ low, high }, .{ divisor, 0 });

    print("%:% / % = % remainder %\n", high, low, divisor, quotient.low, remainder.low);
    assert(!error && quotient.high == 0 && remainder.high == 0 && remainder.low < divisor);
    product_high, product_low := multiply_64bits_with_overflow(quotient.low, divisor);
    reconstructed_low := product_low + remainder.low;
    carry: u64;

    if reconstructed_low < product_low carry = 1;

    assert(reconstructed_low == low && product_high + carry == high);
}

negate_wide :: (v: U128) -> U128 #no_aoc {
    low := 0 - v.low;
    high := ~v.high;
    if low == 0 high += 1;
    return .{ low, high };
}
