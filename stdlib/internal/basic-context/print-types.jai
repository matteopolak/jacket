Formatter :: struct {
    value: Any;
}

FormatFloat :: struct {
    #as using formatter: Formatter;
    width := -1;
    trailing_width := -1;
    Zero_Removal :: enum {
        YES;
        NO;
        ONE_ZERO_AFTER_DECIMAL;
    }
    zero_removal := Zero_Removal.YES;
    Mode :: enum {
        DECIMAL;
        SCIENTIFIC;
        SHORTEST;
    }
    mode := Mode.DECIMAL;
    thousand_separators := false;
    thousand_separator_character: u8 = #char ",";
    decimal_separator_character: u8 = #char ".";
    exponent_character: u8 = #char "e";
    Positive_Number_Prefix :: enum {
        NONE;
        PLUS;
        SPACE;
    }
    positive_number_prefix := Positive_Number_Prefix.NONE;
}

FormatStruct :: struct {
    #as using formatter: Formatter;
    draw_type_name := false;
    use_long_form_if_more_than_this_many_members := 5;
    separator_between_name_and_value := " = ";
    short_form_separator_between_fields := ", ";
    long_form_separator_between_fields := "; ";
    begin_string := "{";
    end_string := "}";
    indentation_width: s32 = 4;
    use_newlines_if_long_form := false;
}

FormatArray :: struct {
    #as using formatter: Formatter;
    separator := ", ";
    begin_string := "[";
    end_string := "]";
    printing_stopped_early_string := "...";
    draw_separator_after_last_element := false;
    stop_printing_after_this_many_elements := 100;
}

Print_Style :: struct {
    default_format_int: FormatInt;
    default_format_float: FormatFloat;
    default_format_struct: FormatStruct;
    default_format_array: FormatArray;
    default_format_absolute_pointer: FormatInt = .{ base = 16, digits_per_comma = 4, comma_string = "_" };
    struct_printer: (builder: *String_Builder, any: Any, struct_printer_data: *void) -> bool;
    struct_printer_data: *void;
    indentation_depth: s32;
    state_flags: enum u32 {
        INSIDE_DATA_STRUCTURE_PRINTING :: 0x1;
    }
    log_runtime_errors := true;
}

FormatInt :: struct {
    using #as formatter: Formatter;
    base := 10;
    minimum_digits := 1;
    padding: u8 = #char "0";
    digits_per_comma: u16 = 0;
    comma_string := "";
}
