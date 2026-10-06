// rules: ops.1 ops.2 ops.3 ops.4 ops.5 ops.6 ops.7 ops.8 ops.9 ops.10 ops.11 ops.12 ops.13 ops.14 ops.15
// Self-checking program for the precedence table in docs/language/operators.md. Operands are
// runtime variables so the parser's grouping, not constant folding, decides each result.
// jailint: allow-file(bitwise_precedence, identical_operands) -- this test pins down how Jai groups operators.
#import "Basic";

Point :: struct {
    x: int;
    inner: *Point;
}

main :: () {
    one, two, three, four, eight, ten := 1, 2, 3, 4, 8, 10;

    // ops.1: every binary level is left associative.
    assert(eight / two / two == 2, "8 / 2 / 2");
    assert(one << one << two == 8, "1 << 1 << 2");
    assert(ten - four - three == 3, "10 - 4 - 3");
    assert(ten % four % three == 2, "10 mod 4 mod 3");

    // ops.2: postfix binds before prefix.
    p := Point.{ x = 5 };
    assert(-p.x == -5, "-p.x is -(p.x)");
    q := Point.{ x = 7, inner = *p };
    ptr := *q.inner;
    #assert type_of(ptr) == **Point;
    arr := int.[3, 4];
    assert(-arr[1] == -4, "-a[i] is -(a[i])");

    // ops.3: prefix binds before the bitwise level.
    mask := 0b1100;
    assert(~mask & 0xF == 0b0011, "~a & b is (~a) & b");
    assert(-two & 0xFF == 0xFE, "-a & b is (-a) & b");

    // ops.4: & | ^ << >> share one level, left to right.
    assert(one | two & four == 0, "1 | 2 & 4 is (1 | 2) & 4");
    assert(four & two | one == 1, "4 & 2 | 1");
    assert(one ^ three & two == 2, "1 ^ 3 & 2 is (1 ^ 3) & 2");

    // ops.5: a prefix cast takes the bitwise level into its operand.
    b: u8 = 0x12;
    r := cast(u32) b << 4 | 1;
    assert(r == 0x21, "cast(u32) b << 4 | 1 is cast(u32) ((b << 4) | 1)");

    // ops.6: bitwise binds before multiplicative.
    assert(one << two * three == 12, "1 << 2 * 3 is (1 << 2) * 3");
    assert(ten * one >> one == 0, "10 * 1 >> 1 is 10 * (1 >> 1)");

    // ops.7: % is below * and /.
    assert(ten % three * two == 4, "10 mod 3 * 2 is 10 mod (3 * 2)");
    assert(ten * three % four == 2, "10 * 3 mod 4 is (10 * 3) mod 4");

    // ops.8: % is above + and -.
    tab, col := 4, 6;
    assert(tab - col % tab == 2, "tab - col mod tab");
    assert(col % tab + one == 3, "col mod tab + 1");

    // ops.9: additive binds before relational.
    assert(one + two < four, "1 + 2 < 4");
    assert(!(one + three < four), "1 + 3 < 4 is false");
    // Relational binds before equality: (1 < 2) == (3 < 4).
    assert(one < two == three < four, "a < b == c < d");

    // ops.10: bitwise binds before equality.
    x0f, x33 := 0x0F, 0x33;
    assert(x0f & x33 == 0x03, "0x0F & 0x33 == 0x03");

    // ops.11: equality binds before &&.
    t := true;
    assert(one == one && t, "a == b && c");
    assert(!(one == two && t), "(1 == 2) && true is false");

    // ops.12: && binds before ||.
    f := false;
    assert(t || f && f, "a || b && c is a || (b && c)");
    assert(!(f && t || f), "(f && t) || f is false");

    // ops.13: cast(float) (hex >> 16) & 0xFF converts the masked integer.
    hex: u32 = 0x7F3C00;
    red := cast(float) (hex >> 16) & 0xFF;
    #assert type_of(red) == float;
    assert(red == 127, "the mask applies before the cast");

    // ops.14: cast(s64) p - cast(s64) q subtracts two integers.
    words: [4] u32;
    p0 := *words[0];
    p2 := *words[2];
    diff := cast(s64) p2 - cast(s64) p0;
    #assert type_of(diff) == s64;
    assert(diff == 8, "byte distance between two u32 slots");

    // ops.15: b | a << 8 is (b | a) << 8.
    a8, b8 := 0x01, 0x02;
    assert(b8 | a8 << 8 == 0x300, "b | a << 8 is (b | a) << 8");
    assert(a8 << 8 | b8 == 0x102, "a << 8 | b is (a << 8) | b");

    print("ok\n");
}
