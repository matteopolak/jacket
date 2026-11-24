// min_of_type and max_of_type return the integer limits, and the float minimum is the smallest positive normal value.
#import "Basic";
M :: #import "Math";
S :: #import "Sloppy_Math";

main :: () {
    assert(S.min_of_type(s8) == -128);
    assert(S.max_of_type(s8) == 127);
    assert(S.min_of_type(u8) == 0);
    assert(S.max_of_type(u8) == 255);
    assert(S.min_of_type(s16) == -32768);
    assert(S.max_of_type(s16) == 32767);
    assert(S.min_of_type(u16) == 0);
    assert(S.max_of_type(u16) == 65535);
    assert(S.min_of_type(s32) == -2147483648);
    assert(S.max_of_type(s32) == 2147483647);
    assert(S.min_of_type(u32) == 0);
    assert(S.max_of_type(u32) == 4294967295);
    assert(S.min_of_type(s64) == -9223372036854775807 - 1);
    assert(S.max_of_type(s64) == 9223372036854775807);
    assert(S.min_of_type(u64) == 0);
    assert(S.max_of_type(u64) == 0xFFFF_FFFF_FFFF_FFFF);

    assert(S.min_of_type(float32) == M.FLOAT32_MIN);
    assert(S.max_of_type(float32) == M.FLOAT32_MAX);
    assert(S.min_of_type(float32) > 0);
    assert(-S.max_of_type(float32) < 0);
    assert(S.min_of_type(float64) == M.FLOAT64_MIN);
    assert(S.max_of_type(float64) == M.FLOAT64_MAX);
    assert(S.min_of_type(float64) > 0);

    print("ok\n");
}
