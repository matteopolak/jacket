// EVEX decorations in #asm: `[mem]!` broadcast, `!z`/`!u` rounding, and `&*` masking.
#import "Basic";

fail :: (what: string) {
    print("FAIL: %\n", what);
    exit(1);
}

main :: () {
    array: [16] s32;
    ap := array.data;
    value: float = 127.53;
    vp := *value;

    #asm AVX512F {
        cvtps2dq v1:, [vp]!;
        movdqu [ap], v1;
    }
    for array if it != 128 fail("broadcast + round to nearest");

    #asm AVX512F {
        broadcastss v2:, [vp];
        cvtps2dq v3:, v2 !z;
        movdqu [ap], v3;
    }
    for array if it != 127 fail("truncate");

    #asm AVX512F {
        broadcastss v2:, [vp];
        cvtps2dq v3:, v2 !u;
        movdqu [ap], v3;
    }
    for array if it != 128 fail("round up");

    #asm AVX512F {
        mov init:, 0b0101_0101_0101_0101;
        kmovw mask:, init;
        broadcastss v4:, [vp];
        cvtps2dq v5: &* mask, v4 !z;
        movdqu [ap], v5;
    }
    for array {
        if it_index % 2 == 0 {
            if it != 127 fail("masked lane");
        } else {
            if it != 0 fail("zeroed lane");
        }
    }

    print("ok\n");
}
