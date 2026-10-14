#import "Basic";

Signed :: enum s16 {
    NEGATIVE :: -23;
    ZERO :: 0;
    HIGH :: 300;
}

Wide :: enum u64 {
    HIGH :: 0xffffffffffffffff;
    LOW :: 0;
}

check_range :: ($T: Type, expected_low: T, expected_high: T) {
    low, high := get_integer_range(T);
    assert(low == expected_low && high == expected_high);
}

main :: () {
    check_range(s8, -128, 127);
    check_range(u8, 0, 255);
    check_range(s16, -32768, 32767);
    check_range(u16, 0, 65535);
    check_range(s32, -2147483648, 2147483647);
    check_range(u32, 0, 0xffffffff);
    check_range(s64, -9223372036854775807 - 1, 9223372036854775807);
    check_range(u64, 0, 0xffffffffffffffff);
    assert(clamp_to_another_integer_type(cast(s64) -900, u8) == 0);
    assert(clamp_to_another_integer_type(cast(u64) 0xffffffffffffffff, s64) == 9223372036854775807);
    assert(clamp_to_another_integer_type(cast(s64) -900, s8) == -128);
    assert(clamp_to_another_integer_type(cast(s64) 900, s8) == 127);
    assert(clamp_to_another_integer_type(cast(s64) -12, s16) == -12);
    assert(clamp_to_another_integer_type(cast(u64) 42, u8) == 42);
    assert(clamp_to_another_integer_type(cast(s8) -128, s64) == -128);

    names := enum_names(Signed);
    numbers := enum_values_as_s64(Signed);
    values := enum_values_as_enum(Signed);
    info := cast(*Type_Info_Enum) type_info(Signed);

    assert(names.count == 3 && names.data == info.names.data);
    assert(numbers.count == 3 && numbers.data == info.values.data);
    assert(values.count == 3);
    assert(names[0] == "NEGATIVE" && names[1] == "ZERO" && names[2] == "HIGH");
    assert(numbers[0] == -23 && numbers[1] == 0 && numbers[2] == 300);
    assert(values[0] == .NEGATIVE && values[1] == .ZERO && values[2] == .HIGH);
    wide := enum_values_as_enum(Wide);

    assert(wide[0] == .HIGH && wide[1] == .LOW);
    print("ok\n");
}
