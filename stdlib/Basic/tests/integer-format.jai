#import "Basic";

main :: () {
    assert(type_info(FormatInt).name == "FormatInt");
    assert(sprint("%", formatInt(-9223372036854775807 - 1)) == "-9223372036854775808");
    assert(sprint("%", formatInt(cast(u64) 0xffffffffffffffff, base = 16)) == "ffffffffffffffff");
    assert(sprint("%", formatInt(5, base = 2, minimum_digits = 8)) == "00000101");
    assert(sprint("%", formatInt(1234567, digits_per_comma = 3, comma_string = ",")) == "1,234,567");
    assert(sprint("%", formatInt(-12, minimum_digits = 4, padding = cast(u8) 32)) == "-  12");
    assert(sprint("%", formatInt(0)) == "0");
    assert(sprint("%", formatInt(cast(s8) -1, base = 16)) == "ff");
    assert(sprint("%", formatInt(63, base = 64)) == ".");
    assert(sprint("%", formatInt(62, base = 63)) == "_");
    assert(sprint("%", formatInt(64, base = 64)) == "10");
    padded := sprint("%", formatInt(1, minimum_digits = 100));

    defer free(padded);
    assert(padded.count == 64 && padded[63] == 49);
    print("ok\n");
}
