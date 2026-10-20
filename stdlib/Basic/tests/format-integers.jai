#import "Basic";

Distinct_Int :: #type,distinct s32;

main :: () {
    assert(sprint("%", 0) == "0");
    assert(sprint("%", cast(s8) -128) == "-128");
    assert(sprint("%", cast(s16) -32768) == "-32768");
    assert(sprint("%", cast(s32) -2147483648) == "-2147483648");
    assert(sprint("%", 0x8000_0000_0000_0000 + cast(s64) 0) == "-9223372036854775808");
    assert(sprint("%", cast(u64) 0xffff_ffff_ffff_ffff) == "18446744073709551615");
    assert(sprint("%", cast(u8) 255) == "255");
    assert(sprint("%", cast(Distinct_Int) -7) == "-7");

    // Other bases print the bits of the value at its own size.
    assert(sprint("%", formatInt(cast(s8) -1, base = 16)) == "ff");
    assert(sprint("%", formatInt(5, base = 2)) == "101");
    assert(sprint("%", formatInt(35, base = 36)) == "z");
    assert(sprint("%", formatInt(0xdead_beef, base = 16)) == "deadbeef");

    assert(sprint("%", formatInt(42, minimum_digits = 5)) == "00042");
    assert(sprint("%", formatInt(-42, minimum_digits = 5, padding = #char " ")) == "-   42");
    assert(sprint("%", formatInt(1234567, digits_per_comma = 3, comma_string = ",")) == "1,234,567");
    assert(sprint("%", formatInt(12, minimum_digits = 5, digits_per_comma = 2, comma_string = "_")) == "0_00_12");

    assert(sprint("a % b %\x1f c", 1, 22) == "a 1 b 22% c");
    assert(sprint("plain text") == "plain text");
    print("ok\n");
}
