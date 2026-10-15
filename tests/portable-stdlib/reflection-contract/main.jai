// API and behavior provenance: reviewed specs/stdlib/modules/reflection.md.
#import "Basic";
R :: #import "Reflection";

Signed_Enum :: enum s16 {
    NEGATIVE :: -3;
    POSITIVE :: 9;
}

Unsigned_Enum :: enum u32 {
    HIGH :: 0xf0000001;
    LOW :: 5;
}

Record :: struct {
    marker: u8;
    count: s64;
    label: string;
}

signed_width :: ($T: Type) {
    low, high := R.signed_integer_range_from_size(size_of(T));
    expected_high := cast(s64) (cast(u64) 0x7fffffffffffffff >> (64 - size_of(T) * 8));

    assert(low == -expected_high - 1 && high == expected_high);
    destination: T;
    success, returned_low, returned_high := R.range_check_and_store(low, type_info(T), *destination);

    assert(success && destination == low && returned_low == low && returned_high == high);
    success = R.range_check_and_store(high, type_info(T), *destination);
    assert(success && destination == high);
    success = R.range_check_and_store(cast(u64) 0, type_info(T), *destination);
    assert(success && destination == 0);

    #if size_of(T) < 8 {
        destination = 7;
        success = R.range_check_and_store(low - 1, type_info(T), *destination);
        assert(!success && destination == 7);
        success = R.range_check_and_store(high + 1, type_info(T), *destination);
        assert(!success && destination == 7);
        success = R.range_check_and_store(cast(u64) high + 1, type_info(T), *destination);
        assert(!success && destination == 7);
    }
}

unsigned_width :: ($T: Type) {
    low, high := R.unsigned_integer_range_from_size(size_of(T));

    assert(low == 0 && high == 0xffffffffffffffff >> (64 - size_of(T) * 8));
    destination: T;
    success, returned_low, returned_high := R.range_check_and_store(high, type_info(T), *destination);

    assert(success && destination == high && returned_low == low && returned_high == high);
    success = R.range_check_and_store(low, type_info(T), *destination);
    assert(success && destination == 0);
    destination = 7;
    success = R.range_check_and_store(cast(s64) -1, type_info(T), *destination);
    assert(!success && destination == 7);

    #if size_of(T) < 8 {
        success = R.range_check_and_store(high + 1, type_info(T), *destination);
        assert(!success && destination == 7);
    }
}

array_shapes :: () {
    fixed := u32.[2, 4, 6];
    count, data := R.get_array_count_and_data(*fixed, type_info(type_of(fixed)));

    assert(count == 3 && data == fixed.data);
    view: [] u32 = fixed;

    count, data = R.get_array_count_and_data(*view, type_info(type_of(view)));
    assert(count == 3 && data == fixed.data);
    dynamic: [..] u32;

    defer array_free(dynamic);
    count, data = R.get_array_count_and_data(*dynamic, type_info(type_of(dynamic)));
    assert(count == 0 && data == null);
    array_add(*dynamic, cast(u32) 27);
    count, data = R.get_array_count_and_data(*dynamic, type_info(type_of(dynamic)));
    assert(count == 1 && data == dynamic.data && (cast(*u32) data).* == 27);
}

enums_and_fields :: () {
    value, found := R.enum_name_to_value(Signed_Enum, "NEGATIVE");

    assert(found && value == .NEGATIVE && R.enum_value_to_name(value) == "NEGATIVE");
    _, found = R.enum_name_to_value(Signed_Enum, "negative");
    assert(!found && R.enum_value_to_name(cast(Signed_Enum) 4) == "");
    R.set_enum_value(*value, type_info(Signed_Enum), 9);
    assert(value == .POSITIVE && R.get_enum_value(*value, type_info(Signed_Enum)) == 9);
    R.set_enum_value(*value, type_info(Signed_Enum), -3);
    assert(value == .NEGATIVE && R.get_enum_value(*value, type_info(Signed_Enum)) == -3);
    unsigned := Unsigned_Enum.LOW;

    R.set_enum_value(*unsigned, type_info(Unsigned_Enum), 0xf0000001);
    assert(unsigned == .HIGH && R.get_enum_value(*unsigned, type_info(Unsigned_Enum)) == 0xf0000001);
    info := type_info(Record);
    field := R.get_struct_field_info(info, "label");
    record: Record;

    assert(field != null && field.type == type_info(string));
    assert(cast(*u8) *record + field.offset_in_bytes == cast(*u8) *record.label);
    assert(R.get_struct_field_info(info, "missing") == null);
}

parse_values :: () {
    small: s8 = 11;
    cursor := "-128rest";

    assert(R.set_value_from_string(small, *cursor) && small == -128 && cursor == "rest");
    cursor = "128rest";
    assert(!R.set_value_from_string(small, *cursor) && small == -128);
    cursor = "invalid";
    assert(!R.set_value_from_string(small, *cursor) && cursor == "invalid");
    largest: u64;

    assert(R.set_value_from_string(largest, "18446744073709551615") && largest == 0xffffffffffffffff);
    minimum: s64;

    assert(R.set_value_from_string(minimum, "-9223372036854775808") && minimum == -9223372036854775808);
    flag := true;

    cursor = "falsehood";
    assert(R.set_value_from_string(flag, *cursor) && !flag && cursor == "hood");
    cursor = "truest";
    assert(R.set_value_from_string(flag, *cursor) && flag && cursor == "st");
    cursor = "TRUE";
    assert(!R.set_value_from_string(flag, *cursor) && flag && cursor == "TRUE");
    single: float32;

    cursor = "1.25tail";
    assert(R.set_value_from_string(single, *cursor) && single == 1.25 && cursor == "tail");
    double: float64;

    cursor = "-2.5e2rest";
    assert(R.set_value_from_string(double, *cursor) && double == -250 && cursor == "rest");
    text: string;

    cursor = "  \"literal\\bytes\"tail";
    assert(R.set_value_from_string(text, *cursor) && text == "literal\\bytes" && cursor == "tail");
    free(text);
    cursor = "\"unterminated";
    assert(!R.set_value_from_string(text, *cursor) && cursor == "\"unterminated");
    choice := Signed_Enum.NEGATIVE;

    cursor = "POSITIVE rest";
    assert(R.set_value_from_string(choice, *cursor) && choice == .POSITIVE && cursor == " rest");
    cursor = "MISSING";
    assert(!R.set_value_from_string(choice, *cursor) && choice == .POSITIVE);
    unsupported: Record;

    cursor = "anything";
    assert(!R.set_value_from_string(unsupported, *cursor) && cursor == "anything");
}

main :: () {
    signed_width(s8);
    signed_width(s16);
    signed_width(s32);
    signed_width(s64);
    unsigned_width(u8);
    unsigned_width(u16);
    unsigned_width(u32);
    unsigned_width(u64);
    array_shapes();
    enums_and_fields();
    parse_values();
    print("ok\n");
}
