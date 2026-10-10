// Code the unoptimized backend writes in a shorter form must still compute the same thing:
// constant divisors and shifts, compound assignment to a variable, struct copies of every size,
// returning a call's result into its own argument.
#import "Basic";

Odd3 :: struct {
    a: u8;
    b: u8;
    c: u8;
}

Odd13 :: struct {
    a: s64;
    b: s32;
    c: u8;
}

Big :: struct {
    v: [10] s64;
}

Pair :: struct {
    x: s64;
    y: s64;
}

swap :: (p: Pair) -> Pair {
    r: Pair;
    r.x = p.y;
    r.y = p.x;
    return r;
}

grow :: (b: Big) -> Big {
    r := b;
    for 0..9 r.v[it] += 1;
    return r;
}

bump :: (p: *s64) -> s64 {
    p.* += 1;
    return p.*;
}

main :: () {
    minimum: s64 = -9223372036854775807 - 1;
    minus_one: s64 = -1;
    assert(minimum / -1 == minimum);
    assert(minimum % -1 == 0);
    assert(minimum / minus_one == minimum);
    assert(minimum % minus_one == 0);
    assert(-7 / 2 == -3 && -7 % 2 == -1 && 7 / -2 == -3 && 7 % -2 == 1);
    n: s64 = -7;
    assert(n / 2 == -3 && n % 2 == -1 && n / -2 == 3 && n % -2 == -1);
    u: u64 = 18446744073709551615;
    assert(u / 10 == 1844674407370955161 && u % 10 == 5);
    small: u8 = 200;
    assert(small / 7 == 28 && small % 7 == 4);

    one: u32 = 1;
    assert(one << 31 == 2147483648);
    assert(one << 32 == 0);
    assert((one << 5) >> 5 == 1);
    neg: s32 = -64;
    assert(neg >> 3 == -8);
    assert(neg >> 40 == -1);
    amount := 3;
    assert(neg >> amount == -8);

    total: s64 = 5;
    total += 3;
    total *= 2;
    total -= 1;
    total <<= 1;
    assert(total == 30);
    pair := Pair.{ 1, 2 };
    pair.x += 10;
    pair.y *= pair.x;
    assert(pair.x == 11 && pair.y == 22);
    pointer := *pair;
    pointer.x += 1;
    assert(pair.x == 12);
    counter: s64 = 0;
    counter += bump(*counter);
    assert(counter == 2);
    values: [3] s64;
    index := 0;
    values[index] += 5;
    index += 1;
    values[index] += 7;
    assert(values[0] == 5 && values[1] == 7 && index == 1);

    pair = swap(pair);
    assert(pair.x == 22 && pair.y == 12);
    pair = swap(pair);
    pair = swap(pair);
    assert(pair.x == 22 && pair.y == 12);

    a3: Odd3 = .{ 1, 2, 3 };
    b3 := a3;
    a3.a = 9;
    assert(b3.a == 1 && b3.b == 2 && b3.c == 3);
    a13: Odd13 = .{ 123456789012, 77, 5 };
    b13 := a13;
    assert(b13.a == 123456789012 && b13.b == 77 && b13.c == 5);
    zeroed: Odd13;
    assert(zeroed.a == 0 && zeroed.b == 0 && zeroed.c == 0);

    big: Big;
    for 0..9 big.v[it] = it;
    big = grow(big);
    big = grow(big);
    for 0..9 assert(big.v[it] == it + 2);
    copy := big;
    copy.v[0] = 100;
    assert(big.v[0] == 2);

    print("ok\n");
}
