#import "Basic";

MASK :: 0xffff_ffff;

main :: () {
    h: s32 = 7;
    u: u8 = 3;
    // Fits the type.
    a := (0x7fff_fff0 - 40) / h;
    // Bit patterns: bitwise operators and equality.
    low := h & 0xffff_ffff;
    all_ones := h == 0xffff_ffff;
    // `~` is negative until it takes the type.
    cleared := u & ~0x7;
    // Too big for the type's bits: the compiler widens the expression instead.
    wide := u + 300;
    big := h * (1 << 40);
    under := u < 300;
    // The same bits whether the constant or the sum wraps.
    minus_one := h + 0xffff_ffff;
    // Spelled out.
    narrow := cast(u8) 300;
    as_s32 := h + cast,no_check(s32) 0xffff_ffff;
    // Named constants and constant-only expressions.
    named := h + MASK;
    folded := 0xffff_ffff - 40;
    // The other operand is wide enough.
    w: s64 = 1;
    fine := (0xffff_ffff - 40) / w;
    print("% % % % % % % % % % % % %\n", under, minus_one, a, low, all_ones, cleared, wide, big, narrow, as_s32, named, folded, fine);
}
