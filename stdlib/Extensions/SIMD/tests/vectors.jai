// Every operation of Extensions/SIMD against a scalar expectation.
//
//     jaic stdlib/Extensions/SIMD/tests/vectors.jai -interp

#import "Basic";
#import "Math";
SIMD :: #import "Extensions/SIMD";

using SIMD;

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

same :: (a: Vector($T, $N), expected: [N] T, what: string, loc := #caller_location) {
    for 0..N - 1 if a.lanes[it] != expected[it] {
        check(false, tprint("%: lane % is %, expected % (got %)", what, it, a.lanes[it], expected[it], a.lanes), loc);
        return;
    }
}

making :: () {
    same(splat(u8x16, 7), .[7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7], "splat");
    same(indices(s32x4), .[0, 1, 2, 3], "indices");

    bytes: [20] u8;
    for 0..19 bytes[it] = cast(u8) it;

    same(load(u8x16, bytes.data + 3), .[3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18], "load from a pointer");
    same(load(u8x16, bytes, 4), .[4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19], "load from an array");
    same(load(u8x16, "abcdefghijklmnopq", 1), .[#char "b", #char "c", #char "d", #char "e", #char "f", #char "g", #char "h", #char "i", #char "j", #char "k", #char "l", #char "m", #char "n", #char "o", #char "p", #char "q"], "load from a string");
    same(load_partial(u16x8, cast(*u16) bytes.data, 3, 9), .[0x0100, 0x0302, 0x0504, 9, 9, 9, 9, 9], "load_partial");

    out: [8] s32;

    store(indices(s32x4) * 3, out, 2);
    check(out[2] == 0 && out[3] == 3 && out[4] == 6 && out[5] == 9 && out[6] == 0, tprint("store: %", out));

    v := indices(f32x4);
    v[2] = 9.5;
    check(v[2] == 9.5 && v[3] == 3, "index and assign");
}

arithmetic :: () {
    a := splat(u8x16, 250);

    same(low_half(a + 10), .[4, 4, 4, 4, 4, 4, 4, 4], "u8 + wraps");
    same(low_half(3 - indices(u8x16)), .[3, 2, 1, 0, 255, 254, 253, 252], "scalar - vector wraps");
    same(indices(s16x8) * -2, .[0, -2, -4, -6, -8, -10, -12, -14], "s16 *");
    same(splat(s32x4, 7) / s32x4.{ .[1, 2, 3, -7] }, .[7, 3, 2, -1], "s32 /");
    same(-f64x2.{ .[1.5, -2] }, .[-1.5, 2], "float negate");
    same(-s8x16.{ .[-128, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }, .[-128, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "s8 negate wraps");

    same(min(s32x4.{ .[1, -5, 3, 4] }, s32x4.{ .[2, -6, 3, 0] }), .[1, -6, 3, 0], "min");
    same(max(f32x4.{ .[1, -5, 3, 4] }, f32x4.{ .[2, -6, 3, 0] }), .[2, -5, 3, 4], "max");
    same(abs(s16x8.{ .[-3, 3, 0, -32768, 1, -1, 7, -7] }), .[3, 3, 0, -32768, 1, 1, 7, 7], "abs");
    same(sqrt(f64x2.{ .[9, 2.25] }), .[3, 1.5], "sqrt");

    same(saturating_add(u8x16.{ .[250, 1, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255] }, splat(u8x16, 10)), .[255, 11, 138, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 255], "u8 saturating_add");
    same(low_half(saturating_sub(u8x16.{ .[5, 20, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }, splat(u8x16, 10))), .[0, 10, 0, 245, 0, 0, 0, 0], "u8 saturating_sub");
    same(saturating_add(s8x16.{ .[120, -120, 5, -5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }, s8x16.{ .[10, -10, -10, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }), .[127, -128, -5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "s8 saturating_add");
    same(saturating_sub(s16x8.{ .[-32000, 32000, 5, 0, 0, 0, 0, 0] }, s16x8.{ .[1000, -1000, 10, -32768, 0, 0, 0, 0] }), .[-32768, 32767, -5, 32767, 0, 0, 0, 0], "s16 saturating_sub");
}

bitwise :: () {
    a := u32x4.{ .[0xF0F0, 0xFF, 0, 0xFFFF_FFFF] };

    same(a & 0xFF, .[0xF0, 0xFF, 0, 0xFF], "&");
    same(0x100 | a, .[0xF1F0, 0x1FF, 0x100, 0xFFFF_FFFF], "|");
    same(a ^ a, .[0, 0, 0, 0], "^");
    same(~a, .[0xFFFF_0F0F, 0xFFFF_FF00, 0xFFFF_FFFF, 0], "~");
    same(a << 4, .[0xF_0F00, 0xFF0, 0, 0xFFFF_FFF0], "<<");
    same(a >> 4, .[0xF0F, 0xF, 0, 0x0FFF_FFFF], ">> on unsigned");
    same(s32x4.{ .[-16, 16, -1, 0] } >> 2, .[-4, 4, -1, 0], ">> keeps the sign");
}

masks :: () {
    text := "find the q in here, quick";
    chunk := load(u8x16, text, 0);
    q := eq(chunk, #char "q");

    check(any(q) && !all(q) && !none(q), "any/all/none");
    check(first_true(q) == 9 && last_true(q) == 9 && count_true(q) == 1, tprint("one q at 9: % % %", first_true(q), last_true(q), count_true(q)));
    check(to_bits(q) == 1 << 9, "to_bits");

    spaces := eq(chunk, #char " ");

    check(to_bits(spaces) == (1 << 4) | (1 << 8) | (1 << 10) | (1 << 13), tprint("spaces: %", formatInt(to_bits(spaces), base = 2)));
    check(to_bits(spaces | q) == to_bits(spaces) | to_bits(q), "mask |");
    check(to_bits(spaces & q) == 0 && to_bits(~q) == 0xFFFF & ~(1 << 9) && to_bits(q ^ q) == 0, "mask & ~ ^");
    check(spaces[4] && !spaces[5], "mask index");
    check(first_true(eq(chunk, #char "z")) == -1 && last_true(eq(chunk, #char "z")) == -1, "first_true of none");

    m := mask_from_bits(Mask(s32, 4), 0b1010);

    same(select(m, splat(s32x4, 1), splat(s32x4, 2)), .[2, 1, 2, 1], "select ints");
    same(select(lt(f32x4.{ .[1, 5, 2, 8] }, 4), splat(f32x4, 0), f32x4.{ .[1, 5, 2, 8] }), .[0, 5, 0, 8], "select floats");

    f := f32x4.{ .[1, 2, 3, 4] };

    check(to_bits(eq(f, 2)) == 0b10 && to_bits(ne(f, 2)) == 0b1101 && to_bits(le(f, 2)) == 0b11, "float compares");
    check(to_bits(gt(f, 2)) == 0b1100 && to_bits(ge(f, 2)) == 0b1110 && to_bits(lt(f, 2)) == 0b1, "more float compares");
    nan := f32x4.{ .[FLOAT32_NAN, 1, 1, 1] };

    check(!eq(nan, nan)[0] && ne(nan, nan)[0] && !lt(nan, nan)[0], "NaN compares unequal");
    check(to_bits(lt(s8x16.{ .[-1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }, 0)) == 1, "signed compare");
    check(to_bits(lt(u8x16.{ .[255, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }, 128)) == 0xFFFE, "unsigned compare");
    check(to_bits(eq(splat(u8x64, 1), 1)) == 0xFFFF_FFFF_FFFF_FFFF, "64 lanes to_bits");
}

reductions :: () {
    a := s32x4.{ .[3, -7, 10, 2] };

    check(reduce_add(a) == 8 && reduce_min(a) == -7 && reduce_max(a) == 10, "add/min/max");
    check(reduce_add(splat(u8x16, 20)) == 64, "reduce_add wraps");
    b := u16x8.{ .[0xF0, 0xF1, 0xF3, 0xF7, 0xFF, 0xF0, 0xF0, 0xF0] };

    check(reduce_and(b) == 0xF0 && reduce_or(b) == 0xFF && reduce_xor(b) == (0xF0 ^ 0xF1 ^ 0xF3 ^ 0xF7 ^ 0xFF ^ 0xF0 ^ 0xF0 ^ 0xF0), "and/or/xor");
}

rearranging :: () {
    a := indices(u8x16);
    b := indices(u8x16) + 16;

    same(shuffle(a, .[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]), .[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0], "shuffle reverses");
    same(shuffle(s32x4.{ .[10, 20, 30, 40] }, s32x4.{ .[50, 60, 70, 80] }, .[0, 4, 1, 7]), .[10, 50, 20, 80], "two-input shuffle");
    same(reverse(s32x4.{ .[1, 2, 3, 4] }), .[4, 3, 2, 1], "reverse");
    same(window(a, b, 0), a.lanes, "window 0");
    same(window(a, b, 16), b.lanes, "window N");
    same(window(a, b, 14), .[14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29], "window 14");

    table := u8x16.{ .[100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115] };

    same(lookup(table, u8x16.{ .[0, 15, 16, 255, 128, 127, 3, 3, 1, 2, 4, 5, 6, 7, 8, 9] }), .[100, 115, 0, 0, 0, 0, 103, 103, 101, 102, 104, 105, 106, 107, 108, 109], "lookup");
    wide := lookup(table, indices(u8x32));

    check(wide[15] == 115 && wide[16] == 0 && wide[31] == 0, "lookup with 32 lanes");
}

changing_shape :: () {
    same(convert(s32x4, f32x4.{ .[1.9, -1.9, 0, 100] }), .[1, -1, 0, 100], "float to int truncates");
    same(convert(Vector(u8, 4), s32x4.{ .[256, 257, -1, 7] }), .[0, 1, 255, 7], "narrowing keeps low bits");
    same(convert(f64x2, Vector(s16, 2).{ .[-3, 5] }), .[-3, 5], "int to float");
    same(bitcast(u32x4, splat(u8x16, 1)), .[0x0101_0101, 0x0101_0101, 0x0101_0101, 0x0101_0101], "bitcast");
    a := indices(u16x8);

    same(low_half(a), .[0, 1, 2, 3], "low_half");
    same(high_half(a), .[4, 5, 6, 7], "high_half");
    same(combine(high_half(a), low_half(a)), .[4, 5, 6, 7, 0, 1, 2, 3], "combine");
}

bits :: () {
    check(bit_count(0) == 0 && bit_count(0xFF00_0000_0000_0001) == 9, "bit_count");
    check(trailing_zeros(0) == 64 && trailing_zeros(8) == 3 && trailing_zeros(1 << 63) == 63, "trailing_zeros");
    check(leading_zeros(0) == 64 && leading_zeros(1) == 63 && leading_zeros(1 << 63) == 0, "leading_zeros");
}

main :: () {
    making();
    arithmetic();
    bitwise();
    masks();
    reductions();
    rearranging();
    changing_shape();
    bits();

    if failures {
        print("% failures\n", failures);
        exit(1);
    }
}
