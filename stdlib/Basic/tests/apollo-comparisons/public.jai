B :: #import "Basic";

using,only(.["<", "<=", ">", ">=", "==", "!", "+", "-"]) B;

Address :: #type,isa B.S128;

verify :: (a: $T, b: T, expected: s32) {
    B.assert((a < b) == (expected < 0));
    B.assert((a <= b) == (expected <= 0));
    B.assert((a > b) == (expected > 0));
    B.assert((a >= b) == (expected >= 0));
    B.assert((a == b) == (expected == 0));
    B.assert((a != b) == (expected != 0));
}

main :: () {
    negative := B.S128.{ 0xffffffffffffffff, -1 };
    zero := B.S128.{};
    positive := B.S128.{ 1, 0 };

    B.assert(!zero && !!negative && !!positive);
    verify(negative, zero, -1);
    verify(zero, positive, -1);
    verify(positive, negative, 1);
    verify(zero, zero, 0);
    verify(B.S128.{ 0, 1 }, B.S128.{ 0xffffffffffffffff, 0 }, 1);
    verify(B.S128.{ 0, -1 }, B.S128.{ 0xffffffffffffffff, -1 }, -1);
    verify(B.S128.{ 0, -9223372036854775808 }, B.S128.{ 0xffffffffffffffff, 9223372036854775807 }, -1);

    a := B.seconds_to_apollo(7);
    b := B.seconds_to_apollo(2);

    verify(a, b, 1);
    verify(b, a, -1);
    verify(a, a, 0);
    B.assert(!B.Apollo_Time.{} && !!a);
    verify(B.seconds_to_apollo(-1), B.seconds_to_apollo(0), -1);
    difference := a - b;

    B.assert(difference.low == 5000000000000000 && difference.high == 0);
    sum := a + b;

    B.assert(sum.low == 9000000000000000 && sum.high == 0);

    address := Address.{ 0xffffffffffffffff, 0 };
    step := Address.{ 1, 0 };
    next := address + step;

    B.assert(next.low == 0 && next.high == 1);
    verify(address, Address.{ 0, 1 }, -1);
    back := next - B.S128.{ 1, 0 };

    B.assert(back.low == address.low && back.high == address.high);

    values: [3] s64;
    pointer := *values[0];
    advanced := pointer + 2;

    B.assert(advanced == *values[2] && advanced > pointer && pointer < advanced);
    B.assert(advanced >= pointer && pointer <= advanced && advanced != pointer);
}
