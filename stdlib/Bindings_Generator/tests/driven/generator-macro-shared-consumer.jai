// These exact fragments are compared with real public API output by the state
// fixture. This separate consumer checks Jai syntax and runtime meaning.
#import "Basic";
#import "Math";

RenamedArithmetic :: 7 + 35;

RenamedEnum :: enum {
    Member :: 11;
}

main :: () {
    wide: u64 = 0xffffffffffffffff;

    assert(wide == U64_MAX);
    floating: float32 = 1.5;

    assert(cast(float64) floating == 1.5);
    assert("héllo".count == 6);
    converted := cast,truncate(s32) -3;

    assert(converted == -3);
    assert(RenamedArithmetic == 42);
    assert(cast(s64) RenamedEnum.Member == 11);
    assert((7 + 35) == 42 && (7 + 35) == 42 && (7 & 3) == 3);
    print("consumer ok\n");
}
