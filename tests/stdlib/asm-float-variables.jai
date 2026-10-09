// A floating-point variable is a vector register in #asm: vector instructions read and write it
// (`addss x, y`) and movd/movq move its bits to and from an integer variable.
// rules: asm.1 asm.28
#import "Basic";

main :: () {
    x: float32 = 1.5;
    y: float32 = 2.25;
    #asm {
        addss x, y;
        mulss x, x;
    }
    assert(x == 14.0625);

    bits: u32;
    #asm {
        movd bits, x;
    }
    assert(bits == 0x41610000);

    z: float32;
    #asm {
        movd z, bits;
        addss z, y;
    }
    assert(z == 16.3125);

    d: float64 = -0.5;
    wide: u64;
    #asm {
        movq wide, d;
    }
    assert(wide == 0xbfe0000000000000);
    e: float64;
    #asm {
        movq e, wide;
        sqrtsd e, e;
    }
    assert(e != e);

    // A float variable in a vector instruction and an integer variable in a conversion.
    n: s32 = 7;
    f: float32;
    #asm {
        cvtsi2ss f, n;
        addss f, f;
    }
    assert(f == 14);
    m: s32;
    #asm {
        cvttss2si m, f;
    }
    assert(m == 14);
    print("ok\n");
}
