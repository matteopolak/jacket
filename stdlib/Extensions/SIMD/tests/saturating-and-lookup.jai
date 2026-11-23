// Saturating and absolute values, partial loads and stores with offsets, byte lookup, float square root and
// bit reinterpretation, each checked against values worked out by hand.
#import "Basic";
SIMD :: #import "Extensions/SIMD";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

main :: () {
    // Saturating arithmetic on bytes: 250 + 10 clamps to 255, 5 - 10 clamps to 0.
    bytes_a := SIMD.splat(SIMD.u8x16, 250);
    bytes_b := SIMD.splat(SIMD.u8x16, 10);

    expect(SIMD.reduce_min(SIMD.saturating_add(bytes_a, bytes_b)) == 255, "saturating_add clamps to 255");
    expect(SIMD.reduce_max(SIMD.saturating_sub(SIMD.splat(SIMD.u8x16, 5), bytes_b)) == 0, "saturating_sub clamps to 0");

    // Absolute value of signed lanes.
    signed_items := s32.[-3, 3, -7, 0];
    signed := SIMD.load(SIMD.Vector(s32, 4), signed_items);
    absolute := SIMD.abs(signed);

    expect(absolute[0] == 3 && absolute[1] == 3 && absolute[2] == 7 && absolute[3] == 0, "abs of mixed signs");

    // A partial load fills the missing lanes; a store at an offset leaves the rest alone.
    source := s32.[1, 2, 3];
    partial := SIMD.load_partial(SIMD.Vector(s32, 4), source.data, 3, 9);

    expect(partial[0] == 1 && partial[1] == 2 && partial[2] == 3 && partial[3] == 9, "load_partial fills with 9");

    buffer: [6] s32;

    SIMD.store(partial, buffer, 2);
    expect(buffer[0] == 0 && buffer[1] == 0 && buffer[2] == 1 && buffer[3] == 2 && buffer[4] == 3 && buffer[5] == 9, "store at offset 2");

    // Byte lookup: lane i of the result is table[indices[i]].
    table := SIMD.load(SIMD.u8x16, "0123456789abcdef");
    index_items := u8.[15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
    indices := SIMD.load(SIMD.u8x16, index_items);
    picked := SIMD.lookup(table, indices);

    expect(picked[0] == #char "f" && picked[1] == #char "0" && picked[2] == #char "1" && picked[15] == #char "e", "lookup rotates the table");

    // Square root of float lanes.
    float_items := float32.[4.0, 9.0, 16.0, 25.0];
    floats := SIMD.load(SIMD.Vector(float32, 4), float_items);
    roots := SIMD.sqrt(floats);

    expect(roots[0] == 2.0 && roots[1] == 3.0 && roots[2] == 4.0 && roots[3] == 5.0, "sqrt of perfect squares");

    // Converting integers to floats, and reinterpreting 1.0 as its bits.
    int_items := s32.[3, -1, 4, 1];
    ints := SIMD.load(SIMD.Vector(s32, 4), int_items);
    as_float := SIMD.convert(SIMD.Vector(float32, 4), ints);

    expect(as_float[0] == 3.0 && as_float[1] == -1.0 && as_float[2] == 4.0 && as_float[3] == 1.0, "convert s32 to float32");

    bits := SIMD.bitcast(SIMD.Vector(u32, 4), SIMD.splat(SIMD.Vector(float32, 4), 1.0));

    expect(bits[0] == 0x3F800000 && bits[3] == 0x3F800000, "bitcast of 1.0 is 0x3F800000");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
