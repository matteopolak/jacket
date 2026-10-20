// A signed constant written as a hex bit pattern that fills the type (`0x8000_0000` for an
// `s32`) is the negative number its sign bit makes, in comparisons and when widened.
#import "Basic";
#import "Math";

Narrow: s8 : 0x80;
Mid: s32 : 0x8000_0000;
Wide: s64 : 0x8000_0000_0000_0000;
AllOnes: s16 : 0xffff;
Largest: s32 : 0x7fff_ffff;

main :: () {
    assert(Narrow < 0 && Narrow == -128);
    assert(Mid < 0 && Mid == -2147483648);
    assert(Wide < 0 && Wide == -9223372036854775807 - 1);
    assert(AllOnes == -1);
    assert(Largest > 0 && Largest == 2147483647);

    assert(cast(s64) Narrow == -128);
    assert(cast(s64) Mid == -2147483648);
    assert(cast(s64) AllOnes == -1);
    assert(Mid + 1 == -2147483647);

    // The limits the Math module exports.
    assert(S8_MIN == -128 && S8_MAX == 127);
    assert(S16_MIN == -32768 && S16_MAX == 32767);
    assert(S32_MIN < 0 && S32_MIN == -2147483648 && S32_MAX == 2147483647);
    assert(S64_MIN < 0 && S64_MAX > 0);
    assert(cast(s64) S32_MIN == -2147483648);
    assert(U8_MAX == 255 && U16_MAX == 65535 && U32_MAX == 4294967295);
    assert(U64_MAX == 18446744073709551615);

    // Compile time agrees.
    #run assert(Mid < 0 && S32_MIN == -2147483648);

    print("ok\n");
}
