// A 64-bit signed/unsigned cast evaluated at compile time reinterprets the bits, for `xx` and `cast`.
// The same casts in the program's own run trap with "cast of -1 to `u64` overflows"
// (crates/jaic-cli/tests/diagnostics/runtime_failures.rs), and narrowing casts stay checked.
#import "Basic";

all_ones :: #run -> u64 {
    x: s64 = -1;
    return xx x;
};

cast_all_ones :: #run -> u64 {
    x: s64 = -1;
    return cast(u64) x;
};

back_to_signed :: #run -> s64 {
    x: u64 = 0xffff_ffff_ffff_ffff;
    return cast(s64) x;
};

#assert 10 < (#run -> u64 {
    x: s64 = -1;
    return xx x;
}) "an autocast in a run reinterprets the bits";

main :: () {
    assert(all_ones == 0xffff_ffff_ffff_ffff);
    assert(cast_all_ones == 0xffff_ffff_ffff_ffff);
    assert(back_to_signed == -1);

    print("ok\n");
}
