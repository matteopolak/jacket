// Formatter record factories (Any payload) and the strings they print as.
// Reflection assertions prove the selected Any descriptor and member offset.
#import "Basic";

Wrapped_Value :: struct {
    marker: s64;
    using #as formatter: Formatter;
}

Sample_Record :: struct {
    count: s64;
    name: string;
}

Sample_Flags :: enum_flags u8 {
    A :: 1;
    B :: 2;
    C :: 4;
}

Procedure_Kind :: #type (s64) -> bool;

main :: () -> s64 {
    integer: s64 = -9223372036854775808;
    legacy := formatInt(integer, minimum_digits = 1);
    if legacy.value.type != type_info(s64) || legacy.base != 10 || legacy.padding != #char "0" return 1;
    if tprint("%", legacy) != "-9223372036854775808" return 2;
    if tprint("%", formatInt(12034, minimum_digits = 7, digits_per_comma = 3, comma_string = ",")) != "0,012,034" return 3;
    if tprint("%", formatInt(255, base = 16, minimum_digits = 4)) != "00ff" return 4;
    big: u64 = 0xfedc_ba98_7654_3210;
    if tprint("%", formatInt(big, base = 16)) != "fedcba9876543210" return 19;

    number: float64 = 1.25;
    legacy_float := formatFloat(number, trailing_width = 2, zero_removal = .NO);
    if legacy_float.value.type != type_info(float64) || legacy_float.mode != .DECIMAL return 5;
    if tprint("%", legacy_float) != "1.25" return 6;
    if tprint("%", formatFloat(number, width = 6, trailing_width = 2, zero_removal = .NO)) != "  1.25" return 7;
    negative_zero: float64 = -0.0;
    if tprint("%", formatFloat(negative_zero, trailing_width = 2, zero_removal = .NO)) != "-0.00" return 8;

    text := "view";
    wrapped := Wrapped_Value.{ marker = 99, formatter = .{ value = text } };
    member := isFormatter(cast(*Type_Info_Struct) type_info(Wrapped_Value));
    if !member || member.offset_in_bytes == 0 || member.type != type_info(Formatter) return 9;
    if tprint("%", wrapped) != "view" return 10;
    direct := Formatter.{ value = text };
    if tprint("%", direct) != "view" return 11;

    sample := Sample_Record.{ count = 3, name = "x" };
    legacy_struct := formatStruct(sample);
    if legacy_struct.value.type != type_info(Sample_Record) || legacy_struct.use_long_form_if_more_than_this_many_members != 5 return 12;
    expected := "{3, \"x\"}";
    if tprint("%", legacy_struct) != expected || tprint("%", formatStruct(sample)) != expected return 13;
    if tprint("%2:%1:\%", "first", "second") != "second:first:%" return 14;
    if tprint("%", cast(Sample_Flags) 9) != "A | 0x8" return 15;
    if tprint("%", cast(Sample_Flags) 0) != "0" return 16;
    if type_to_string(Procedure_Kind) != "procedure (s64) -> bool" return 17;
    if tprint("%", formatStruct(*sample)) != expected return 18;
    return 0;
}
