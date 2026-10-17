// Direct language intrinsic semantics; no Math wrappers or stdlib special cases.
sqrt :: (x: float32) -> float32 #intrinsic;
sin :: (x: float32) -> float32 #intrinsic;
cos :: (x: float32) -> float32 #intrinsic;
floor :: (x: float32) -> float32 #intrinsic;
ceil :: (x: float32) -> float32 #intrinsic;

Bits :: union {
    f: float32;
    u: u32;
}

main :: () {
    actual: Bits;
    actual.f = sqrt(cast(float32) 9);
    print("direct sqrt32(9) bits: %; expected 1077936128\n", actual.u);
    assert(actual.f == 3, "direct binary32 sqrt must return 3 for 9");
    assert(sin(cast(float32) 0) == 0 && cos(cast(float32) 0) == 1);
    assert(floor(cast(float32) 1.75) == 1 && ceil(cast(float32) -1.75) == -1);
    negative_zero: Bits;
    negative_zero.u = 0x80000000;
    result: Bits;
    result.f = sqrt(negative_zero.f);
    assert(result.u == 0x80000000);
    print("direct binary32 intrinsics ok\n");
}

#scope_file
#import "Basic";
