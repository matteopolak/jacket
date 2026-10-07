// Reflection at run time: parsing text into typed values through Any (every integer width with
// range checks, hex/binary/underscores, enums by name or value, bools, strings, floats) and the
// Type_Info helpers.
#import "Basic";
#import "Reflection";

Small :: enum s8 {
    LOW :: -100;
    MID :: 0;
    HIGH :: 100;
}

Wide :: enum u64 {
    NONE :: 0;
    TOP :: 0xffff_ffff_ffff_ffff;
}

Record :: struct {
    id: u16;
    name: string;
    weight: float64;
}

main :: () {
    integers();
    other_values();
    type_helpers();
    print("ok\n");
}

integers :: () {
    i8_value: s8;
    assert(set_value_from_string(i8_value, "-128") && i8_value == -128);
    assert(set_value_from_string(i8_value, "127") && i8_value == 127);
    assert(!set_value_from_string(i8_value, "128") && i8_value == 127, "s8 overflow leaves the value alone");
    assert(!set_value_from_string(i8_value, "-129"));
    u8_value: u8;
    assert(set_value_from_string(u8_value, "255") && u8_value == 255);
    assert(!set_value_from_string(u8_value, "256") && !set_value_from_string(u8_value, "-1"));
    assert(set_value_from_string(u8_value, "-0") && u8_value == 0, "minus zero is zero");
    u16_value: u16;
    assert(set_value_from_string(u16_value, "0xFFFF") && u16_value == 0xffff && !set_value_from_string(u16_value, "0x1_0000"));
    s32_value: s32;
    assert(set_value_from_string(s32_value, "-2147483648") && s32_value == -2147483648 && !set_value_from_string(s32_value, "2147483648"));
    assert(set_value_from_string(s32_value, "0b1010_1010") && s32_value == 170);
    assert(set_value_from_string(s32_value, "  +1_000_000") && s32_value == 1000000, "leading spaces, plus sign, underscores");
    s64_value: s64;
    assert(set_value_from_string(s64_value, "-9223372036854775808") && s64_value == -9223372036854775807 - 1);
    assert(set_value_from_string(s64_value, "9223372036854775807") && s64_value == 9223372036854775807);
    assert(!set_value_from_string(s64_value, "9223372036854775808") && !set_value_from_string(s64_value, "-9223372036854775809"));
    u64_value: u64;
    assert(set_value_from_string(u64_value, "18446744073709551615") && u64_value == 0xffff_ffff_ffff_ffff);
    assert(!set_value_from_string(u64_value, "18446744073709551616"), "u64 overflow");
    assert(!set_value_from_string(u64_value, "99999999999999999999999"));
    assert(!set_value_from_string(u64_value, "") && !set_value_from_string(u64_value, "x") && !set_value_from_string(u64_value, "-"));

    // The pointer form consumes exactly what it parsed, and nothing when it fails.
    text := "12 0x1f rest";
    assert(set_value_from_string(s32_value, *text) && s32_value == 12 && text == " 0x1f rest");
    assert(set_value_from_string(s32_value, *text) && s32_value == 31 && text == " rest");
    assert(!set_value_from_string(s32_value, *text) && text == " rest");
    digits_then_letters := "42abc";
    assert(set_value_from_string(u8_value, *digits_then_letters) && u8_value == 42 && digits_then_letters == "abc");

    // Null pieces are refused.
    nothing: Any;
    assert(!set_value_from_string(nothing, "1"));
    assert(!set_value_from_string(s32_value, cast(*string) null));
}

other_values :: () {
    small: Small;
    assert(set_value_from_string(small, "LOW") && small == .LOW);
    assert(set_value_from_string(small, "100") && small == .HIGH, "an enum also takes its numeric value");
    assert(!set_value_from_string(small, "LOWER") && small == .HIGH);
    assert(!set_value_from_string(small, "low"), "names are case-sensitive");
    words := "MID, HIGH";
    assert(set_value_from_string(small, *words) && small == .MID && words == ", HIGH");
    wide: Wide;
    assert(set_value_from_string(wide, "TOP") && wide == .TOP && cast(u64) wide == 0xffff_ffff_ffff_ffff);

    flag: bool;
    assert(set_value_from_string(flag, "true") && flag && set_value_from_string(flag, "false") && !flag);
    assert(!set_value_from_string(flag, "truest") && !set_value_from_string(flag, "TRUE") && !set_value_from_string(flag, "1"));
    rest := "true)";
    assert(set_value_from_string(flag, *rest) && flag && rest == ")");

    s: string;
    assert(set_value_from_string(s, "\"quoted text\"") && s == "quoted text");
    assert(set_value_from_string(s, "\"\"") && s == "");
    assert(!set_value_from_string(s, "\"unterminated") && !set_value_from_string(s, "bare"));
    quoted := "\"a\" \"b\"";
    assert(set_value_from_string(s, *quoted) && s == "a" && quoted == " \"b\"");

    f32: float32;
    f64: float64;
    assert(set_value_from_string(f32, "2.5") && f32 == 2.5 && set_value_from_string(f64, "-0.125e3") && f64 == -125);
    assert(set_value_from_string(f64, "0.1") && f64 == 0.1, "float64 parses to the nearest double");
    assert(!set_value_from_string(f64, "nope"));
    record: Record;
    assert(!set_value_from_string(record, "1"), "structs are not parsed");
}

type_helpers :: () {
    fixed: [5] s32;
    count, data := get_array_count_and_data(*fixed, cast(*Type_Info_Array) type_info([5] s32));
    assert(count == 5 && data == *fixed);
    dynamic: [..] int;
    array_add(*dynamic, 1, 2, 3);
    count, data = get_array_count_and_data(*dynamic, cast(*Type_Info_Array) type_info([..] int));
    assert(count == 3 && data == dynamic.data);
    view: [] int = dynamic;
    count, data = get_array_count_and_data(*view, cast(*Type_Info_Array) type_info([] int));
    assert(count == 3 && data == dynamic.data);
    array_free(dynamic);

    lo, hi := unsigned_integer_range_from_size(1);
    assert(lo == 0 && hi == 255);
    lo, hi = unsigned_integer_range_from_size(4);
    assert(hi == 0xffff_ffff);
    lo, hi = unsigned_integer_range_from_size(8);
    assert(hi == 0xffff_ffff_ffff_ffff);
    slo, shi := signed_integer_range_from_size(2);
    assert(slo == -32768 && shi == 32767);
    slo, shi = signed_integer_range_from_size(8);
    assert(slo == -9223372036854775807 - 1 && shi == 9223372036854775807);

    member := get_struct_field_info(type_info(Record), "weight");
    assert(member != null && member.name == "weight" && member.type == type_info(float64) && member.offset_in_bytes == 24, "weight member: %", member.offset_in_bytes);
    assert(get_struct_field_info(type_info(Record), "missing") == null);

    value, found := enum_name_to_value(Small, "HIGH");
    assert(found && value == .HIGH);
    value, found = enum_name_to_value(Small, "NOPE");
    assert(!found);
    assert(enum_value_to_name(Small.LOW) == "LOW" && enum_value_to_name(cast(Small) 5) == "" && enum_value_to_name(Wide.TOP) == "TOP");

    small := Small.LOW;
    assert(get_enum_value(*small, type_info(Small)) == -100, "signed 8-bit enum value");
    wide := Wide.TOP;
    assert(get_enum_value(*wide, type_info(Wide)) == -1, "u64 values come back as their bits");
    set_enum_value(*small, type_info(Small), 100);
    assert(small == .HIGH);
    set_enum_value(*wide, type_info(Wide), 0);
    assert(wide == .NONE);
}
