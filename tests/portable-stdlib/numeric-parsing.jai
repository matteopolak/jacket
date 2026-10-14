#import "Basic";
String :: #import "String";
Reflection :: #import "Reflection";

main :: () {
    n, ok, rest := string_to_int("-9223372036854775808tail");

    assert(ok && n == -9223372036854775808 && rest == "tail");
    n, ok, rest = string_to_int("9223372036854775808");
    assert(!ok);
    u, good, tail := string_to_int("ffffffffffffffff!", 16, u64);

    assert(good && u == 0xffffffffffffffff && tail == "!");
    line := "  -19;tail";
    parsed, valid := String.parse_int(*line);

    assert(valid && parsed == -19 && line == ";tail");
    b := false;
    input := "truest";

    assert(Reflection.set_value_from_string(b, *input));
    assert(b && input == "st");
    small: s8 = 7;

    input = "128";
    assert(!Reflection.set_value_from_string(small, *input));
    assert(small == 7);
    print("ok\n");
}
