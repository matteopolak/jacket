#import "Basic";

main :: () {
    integer, signed, bits := is_integer_type(s8);

    assert(integer && signed && bits == 8);
    integer, signed, bits = is_integer_type(u64);
    assert(integer && !signed && bits == 64);
    integer, _, _ = is_integer_type(float64);
    assert(!integer);
    assert(type_info(FormatFloat).name == "FormatFloat");
    assert(type_info(FormatStruct).name == "FormatStruct");
    assert(type_info(FormatArray).name == "FormatArray");
    assert(type_info(Print_Style).name == "Print_Style");
    value: float64 = 1.25;
    real := formatFloat(value);

    assert(real.value.type == type_info(float64));
    assert((cast(*float64) real.value.value_pointer).* == 1.25);
    assert(real.width == -1 && real.trailing_width == -1);
    assert(real.mode == .DECIMAL && real.zero_removal == .YES);
    assert(!real.thousand_separators && real.thousand_separator_character == 44);
    assert(real.decimal_separator_character == 46 && real.exponent_character == 101);
    assert(real.positive_number_prefix == .NONE);
    explicit := formatFloat(value, width = 12, trailing_width = 5, mode = .SCIENTIFIC, zero_removal = .NO);

    assert(explicit.width == 12 && explicit.trailing_width == 5 && explicit.mode == .SCIENTIFIC && explicit.zero_removal == .NO);
    item: Item = .{ count = 9 };
    record := formatStruct(item, use_long_form_if_more_than_this_many_members = 2, use_newlines_if_long_form = true);

    assert(record.value.type == type_info(Item));
    assert((cast(*Item) record.value.value_pointer).count == 9);
    assert(record.use_long_form_if_more_than_this_many_members == 2 && record.use_newlines_if_long_form);
    assert(record.begin_string == "{" && record.end_string == "}" && record.indentation_width == 4);
    array: FormatArray;

    assert(array.separator == ", " && array.begin_string == "[" && array.end_string == "]");
    assert(array.printing_stopped_early_string == "..." && array.stop_printing_after_this_many_elements == 100);
    assert(!array.draw_separator_after_last_element);
    print("ok\n");
}

Item :: struct {
    count: int;
}
