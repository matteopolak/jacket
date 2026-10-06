// rules: cast.29 cast.30 ops.17
// A cast converts its operand after typing it on its own; it does not push the target type into
// the operand's literals.
#import "Basic";

Flags :: enum_flags u8 {
    A;
    B;
    C;
}
Dir :: enum {
    NORTH;
    EAST;
    SOUTH;
    WEST;
}

main :: () {
    w: u16 = 15;
    v: u16 = 0xfff0;
    // `0` is matched with `w`: u16 arithmetic, wrapping, then the conversion.
    f := cast(float32) (0 - w);
    assert(f == 65521);
    g := cast(float32) ((0 - w) & v);
    assert(g == 65520);
    g2 := cast(float64) (2 * w - 1);
    assert(g2 == 29);
    // An s8 sum wraps before it is widened.
    x: s8 = 100;
    assert(cast(s32) (x + 100) == -56);
    // Integer division of integers, even under a float cast.
    n: s64 = 7;
    assert(cast(float64) (n / 2) == 3);
    assert(cast(float32) (1 / 3) == 0); // jailint: allow(integer_division_in_float) tested
    // Bare and literal-only operands are constants folded at the target.
    assert(cast(float32) 3 == 3.0);
    assert(cast(u8) 255 == 255);
    assert(cast(float64) -2 == -2.0);
    assert(cast(float32) (2.5 * 2) == 5);
    // Operands with no type of their own take the cast's type.
    flags := cast(Flags) (.A | .C);
    assert(flags == Flags.A | .C);
    d := cast(Dir) .WEST;
    assert(d == .WEST);
    print("ok\n");
}
