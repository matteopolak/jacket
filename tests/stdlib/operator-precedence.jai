// Binary operator precedence. Jai's table differs from C's: the bitwise and shift
// operators share one level that binds tighter than `*`, and `%` binds looser than `*`
// and `/` but tighter than `+` and `-`. Checks marked `C:` give that value with C's
// grouping. The first block repeats results recorded from the real compiler.
#import "Basic";

main :: () {
    // Recorded output (open-jai `utils/stress.jai`, section 1001 and 1104).
    assert(1 << 2 + 3 == 7);
    assert(8 >> 1 + 1 == 5);
    assert(1 << 2 * 3 == 12);
    assert(10 % 3 * 2 == 4);
    assert(1 | 2 & 4 == 0);
    assert(1 + 2 * 3 << 1 == 13);
    assert(1 << 1 << 2 == 8);
    assert(0x0F & 0x33 == 0x03);
    #assert 1 << 2 + 3 == 7;
    #assert 10 % 3 * 2 == 4;

    // The same at run time.
    one, two, three, four, eight, ten := 1, 2, 3, 4, 8, 10;
    assert(one << two + three == 7);
    assert(eight >> one + one == 5);
    assert(one << two * three == 12);
    assert(ten % three * two == 4);
    assert(one | two & four == 0);
    assert(one + two * three << one == 13);

    // Bitwise and shift operators: one level, left to right.
    a, b := 0x0F, 0x30;
    assert(a << 4 | b == 0xF0);
    assert(b | a << 4 == 0x3F0);       // C: 0xF0
    assert(0xF0 ^ a & 0x0C == 0x0C);   // C: 0xFC
    assert(a >> 2 & 0x2 == 0x2);
    x: u8 = 0x81;
    assert(x <<< 1 | 0x10 == 0x13);
    // They bind tighter than arithmetic on either side.
    assert(2 * a & 0x3 == 6);          // C: 2
    assert(0x10 & a + 1 == 1);         // C: 0x10
    assert(100 - 1 << 2 == 96);        // C: 396

    // `%` is below `*` and `/`, above `+` and `-`.
    tab, col := 4, 6;
    assert(tab - col % tab == 2);      // a tab stop: same as C
    assert(col % tab + 1 == 3);
    assert(17 % 4 * 3 == 5);           // C: 3
    assert(17 * 4 % 3 == 2);
    assert(100 % 30 / 3 == 0);         // C: 3
    assert(100 / 3 % 30 == 3);
    assert(col % 2 == 0);

    // `*` and `/` share a level, left to right.
    assert(7 / 2 * 2 == 6);
    assert((col + 7) / 8 * 8 == 8);

    // Prefix operators bind tighter than every binary operator.
    assert(-a & 0xFF == 0xF1);
    assert(~a & 0xFF == 0xF0);
    neg_shift := -3;
    assert(-neg_shift & 7 == 3);
    flag := false;
    assert(!flag && true);

    // Comparisons are below arithmetic, `&&` is above `||`.
    assert(1 + 2 == 3);
    assert(true || false && false);
    assert(!(false && false || true) == false);

    // A prefix cast takes the bitwise level into its value, nothing looser.
    byte: u8 = 0x66;
    assert(cast(u32) byte << 16 == 0);
    assert(cast(u32) byte << 4 | 1 == 0x61);
    assert(cast(s64) byte * 2 + 1 == 205);

    print("ok\n");
}
