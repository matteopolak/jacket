// Prepared actual Basic fixture. It must execute through the real module.
// Edge forms of print: non-decimal negatives, rounding, zero removal, enums out of range,
// arrays stopped early, Any without a value and quoted strings.
#import "Basic";

Letter :: enum {
    A;
    B;
    C :: 10;
    D;
}

Bits :: enum_flags u8 {
    X;
    Y;
    Z;
}

Small :: enum u8 {
    LO :: 1;
    HI :: 200;
}

Quoted :: struct {
    s: string;
}

c_side :: () #c_call {}

main :: () -> s64 {
    // A negative integer in another base is its two's complement at its own width.
    if tprint("%", formatInt(-255, base = 16)) != "ffffffffffffff01" return 1;
    if tprint("%", formatInt(cast(s8) -1, base = 16)) != "ff" return 2;
    if tprint("%", formatInt(cast(s16) -2, base = 2)) != "1111111111111110" return 3;
    if tprint("%", formatInt(-42, minimum_digits = 5)) != "-00042" return 4;

    // Fixed precision rounds ties away from zero.
    if tprint("%", formatFloat(-2.5, trailing_width = 0)) != "-3" return 5;
    if tprint("%", formatFloat(0.375, trailing_width = 2)) != "0.38" return 6;

    // Removed trailing zeros keep their width as spaces.
    if tprint("%", formatFloat(2.0, trailing_width = 2)) != "2   " return 7;
    if tprint("%", formatFloat(1.25, trailing_width = 5, zero_removal = .ONE_ZERO_AFTER_DECIMAL)) != "1.25   " return 8;
    if tprint("%", formatFloat(3.14159, width = 10)) != "  3.14159" return 9;

    // Infinity and NaN.
    zero := 0.0;
    if tprint("% % %", 1.0 / zero, -1.0 / zero, zero / zero) != "Inf -Inf NaN" return 10;

    // Enums: values without a name.
    if tprint("%", cast(Letter) 5) != "(enum out of range: 5)" return 11;
    if tprint("%", cast(Small) 7) != "(enum out of range: 7)" return 12;
    if tprint("%", Letter.D) != "D" return 13;
    if tprint("%", cast(Bits) 0) != "0" return 14;
    if tprint("%", cast(Bits) 0xff) != "X | Y | Z | 0xf8" return 15;
    if tprint("%", cast(Bits) 0x10) != "10" return 16;
    if tprint("%", Bits.X | .Z) != "X | Z" return 17;

    // Arrays stopped early: the marker follows the last element.
    numbers := int.[1, 2, 3];
    if tprint("%", FormatArray.{ value = numbers, printing_stopped_early_string = "...", stop_printing_after_this_many_elements = 2 }) != "[1, 2...]" return 18;

    // An Any without a type.
    nothing: Any;
    if tprint("%", nothing) != "Any.{}" return 19;

    // Strings inside structures escape newlines only.
    if tprint("%", Quoted.{ "a\nb\"c\td" }) != "{\"a\\nb\"c\td\"}" return 20;

    // A formatter prints the value it wraps.
    if tprint("%", FormatInt.{ value = 5 }) != "5" return 21;

    // Type names.
    anonymous: enum u8 {
        P;
        Q;
    } = .Q;
    if tprint("% %", anonymous, type_of(anonymous)) != "Q (anonymous enum)" return 22;
    if tprint("%", type_of(c_side)) != "procedure () #c_call" return 23;
    return 0;
}
