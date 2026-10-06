// Full Basic dependency path and independently specified parse expectations.
#import "Basic";

main :: () -> s64 {
    minimum, min_ok, min_rest := string_to_int(" -9223372036854775808 tail");
    if !min_ok || minimum != -9223372036854775808 || min_rest != " tail" return 1;
    maximum, max_ok, max_rest := string_to_int("18446744073709551615!", T = u64);
    if !max_ok || maximum != 0xffff_ffff_ffff_ffff || max_rest != "!" return 2;
    overflow_text := "18446744073709551616!";
    _, overflow_ok, overflow_rest := string_to_int(overflow_text, T = u64);
    if overflow_ok || overflow_rest != overflow_text return 3;
    underflow_text := "-9223372036854775809";
    _, underflow_ok, underflow_rest := string_to_int(underflow_text);
    if underflow_ok || underflow_rest != underflow_text return 4;
    zero, zero_ok, zero_rest := string_to_int("-0suffix");
    if !zero_ok || zero != 0 || zero_rest != "suffix" return 5;
    narrow, narrow_ok, narrow_rest := string_to_int("-128x", T = s8);
    if !narrow_ok || narrow != -128 || narrow_rest != "x" return 6;
    _, narrow_overflow, narrow_overflow_rest := string_to_int("256", T = u8);
    if narrow_overflow || narrow_overflow_rest != "256" return 7;
    hex, hex_ok, hex_rest := string_to_int("fFz", base = 16, T = u16);
    if !hex_ok || hex != 255 || hex_rest != "z" return 8;
    _, no_ok, no_rest := string_to_int("  -word");
    if no_ok || no_rest != "  -word" return 9;
    _, negative_ok, negative_rest := string_to_int("-1", T = u64);
    if negative_ok || negative_rest != "-1" return 10;

    decimal, decimal_ok, decimal_rest := string_to_float64(" -.125e2rest");
    if !decimal_ok || decimal != -12.5 || decimal_rest != "rest" return 11;
    incomplete, incomplete_ok, incomplete_rest := string_to_float64("1.25e+");
    if !incomplete_ok || incomplete != 1.25 || incomplete_rest != "e+" return 12;
    if to_upper(#char "z") != #char "Z" || to_lower(#char "Q") != #char "q" return 13;
    if !is_alnum(#char "_") || is_alpha(#char "0") || !is_space(13) || is_print(127) return 14;
    return 0;
}
