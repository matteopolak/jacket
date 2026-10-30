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
    wide := "257";
    narrow, narrow_ok := String.parse_int(*wide, u8);

    assert(!narrow_ok && narrow == 0);
    wide = "255";
    narrow, narrow_ok = String.parse_int(*wide, u8);
    assert(narrow_ok && narrow == 255);
    wide = "-129";
    signed_byte, signed_ok := String.parse_int(*wide, s8);

    assert(!signed_ok && signed_byte == 0);
    wide = "-1";
    narrow, narrow_ok = String.parse_int(*wide, u8);
    assert(!narrow_ok);
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
