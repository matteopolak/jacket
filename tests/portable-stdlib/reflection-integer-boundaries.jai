// Reviewed Reflection scalar parsing and integer destination range checks.
// Returned bounds are exercised only where both fit the input type.
#import "Basic";
R :: #import "Reflection";

check_signed :: ($T: Type, low: s64, high: s64, low_text: string, high_text: string) {
    value: T;
    cursor := low_text;

    assert(R.set_value_from_string(value, *cursor));
    assert(value == low && cursor == "tail");
    cursor = high_text;
    assert(R.set_value_from_string(value, *cursor));
    assert(value == high && cursor == "tail");
    info := cast(*Type_Info_Integer) type_info(T);
    stored, minimum, maximum := R.range_check_and_store(low, info, *value);

    assert(stored && value == low && minimum == low && maximum == high);

    if size_of(T) < 8 {
        stored, minimum, maximum = R.range_check_and_store(low - 1, info, *value);
        assert(!stored && value == low && minimum == low && maximum == high);
        stored, minimum, maximum = R.range_check_and_store(high + 1, info, *value);
        assert(!stored && value == low && minimum == low && maximum == high);
    }
}

check_unsigned :: ($T: Type, high: u64, high_text: string) {
    value: T;
    cursor := high_text;

    assert(R.set_value_from_string(value, *cursor));
    assert(value == high && cursor == "tail");
    info := cast(*Type_Info_Integer) type_info(T);
    stored, minimum, maximum := R.range_check_and_store(high, info, *value);

    assert(stored && value == high && minimum == 0 && maximum == high);

    if size_of(T) < 8 {
        stored, minimum, maximum = R.range_check_and_store(high + 1, info, *value);
        assert(!stored && value == high && minimum == 0 && maximum == high);
    }
}

Signed_Kind :: enum s16 {
    NEGATIVE :: -23;
    POSITIVE :: 300;
}

main :: () {
    check_signed(s8, -128, 127, "-128tail", "127tail");
    check_signed(s16, -32768, 32767, "-32768tail", "32767tail");
    check_signed(s32, -2147483648, 2147483647, "-2147483648tail", "2147483647tail");
    check_signed(s64, -9223372036854775807 - 1, 9223372036854775807,
                 "-9223372036854775808tail", "9223372036854775807tail");
    check_unsigned(u8, 255, "255tail");
    check_unsigned(u16, 65535, "65535tail");
    check_unsigned(u32, 0xffffffff, "4294967295tail");
    check_unsigned(u64, 0xffffffffffffffff, "18446744073709551615tail");
    kind := Signed_Kind.POSITIVE;

    assert(R.set_value_from_string(kind, "NEGATIVE") && kind == .NEGATIVE);
    info := cast(*Type_Info_Enum) type_info(Signed_Kind);

    assert(R.get_enum_value(*kind, info) == -23);
    R.set_enum_value(*kind, info, 300);
    assert(kind == .POSITIVE);
    print("ok\n");
}
