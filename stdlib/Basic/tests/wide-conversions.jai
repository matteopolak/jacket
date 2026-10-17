#import "Basic";

check_unsigned :: (value: $T, expected: u64) {
    wide := to_u128(value);
    assert(wide.low == expected && wide.high == 0);
}

main :: () {
    check_unsigned(cast(u8) 255, 255);
    check_unsigned(cast(u16) 65535, 65535);
    check_unsigned(cast(u32) 0xffffffff, 0xffffffff);
    check_unsigned(cast(u64) 0xffffffffffffffff, 0xffffffffffffffff);
    minus_one := to_s128(cast(s8) -1);

    assert(minus_one.low == 0xffffffffffffffff && minus_one.high == -1);
    minimum := to_s128(cast(s64) (-9223372036854775807 - 1));

    assert(minimum.low == 0x8000000000000000 && minimum.high == -1);
    positive := to_s128(cast(u32) 0xffffffff);

    assert(positive.low == 0xffffffff && positive.high == 0);
    unsigned_high := to_s128(cast(u64) 0xffffffffffffffff);

    assert(unsigned_high.low == 0xffffffffffffffff && unsigned_high.high == -1);
    q, r, error := signed_divide_with_remainder(to_s128(-9), to_s128(2));

    assert(!error && q.low == 0xfffffffffffffffc && q.high == -1);
    assert(r.low == 0xffffffffffffffff && r.high == -1);
    q, r, error = signed_divide_with_remainder(to_s128(9), to_s128(-2));
    assert(!error && q.low == 0xfffffffffffffffc && q.high == -1 && r.low == 1 && r.high == 0);
    q, r, error = signed_divide_with_remainder(S128.{ 0, -9223372036854775807 - 1 }, S128.{ 0xffffffffffffffff, -1 });
    assert(error && q.low == S128_ERROR.low && q.high == S128_ERROR.high && r.low == S128_ERROR.low);
    print("ok\n");
}
