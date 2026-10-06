// A `name:` destination of a vector instruction whose result is a general-purpose value
// (sign masks, lane extracts, float-to-integer conversions) declares a general-purpose register.
// rules: asm.27
#import "Basic";

fail :: (code: int, what: string) {
    print("FAIL %: %\n", code, what);
    exit(xx code);
}

main :: () {
    // Find the first byte equal to a needle, as a substring search does.
    bytes: [16] u8;
    for i: 0..15 bytes[i] = cast(u8) (#char "a" + i);
    needle: [16] u8;
    memset(needle.data, #char "f", 16);
    pb, pn := bytes.data, needle.data;
    results: struct {
        index, found: u16;
    };
    pr := *results;
    #asm {
        movdqu.x  hay:, [pb];
        movdqu.x  nee:, [pn];
        pcmpeqb.x hay, nee;
        pmovmskb.x found_gpr:, hay;
        tzcnt.w   index_gpr:, found_gpr;
        shl.d     found_gpr, 16;
        or.w      found_gpr, index_gpr;
        mov.d     [pr], found_gpr;
    }
    if results.index != 5 fail(1, "pmovmskb into a declared register (index)");
    if results.found != 0x20 fail(2, "pmovmskb into a declared register (found)");

    signs := float32.[-1, 2, -3, -4];
    ps := signs.data;
    mask: u64;
    #asm {
        movups.x v:, [ps];
        movmskps m:, v;
        mov mask, m;
    }
    if mask != 0b1101 fail(3, "movmskps into a declared register");

    doubles := float64.[2.75, -1];
    pd := doubles.data;
    truncated: s64;
    second: u64;
    #asm {
        movupd.x w:, [pd];
        cvttsd2si t:, w;
        mov truncated, t;
        pextrq e:, w, 1;
        mov second, e;
    }
    if truncated != 2 fail(4, "cvttsd2si into a declared register");
    if second != 0xbff0_0000_0000_0000 fail(5, "pextrq into a declared register");

    print("ok\n");
}
