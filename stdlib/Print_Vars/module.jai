Print_Vars_Config :: struct {
    control_flags: enum_flags u32 {
        PRINT_NAME :: 0x1;
        PRINT_VALUE :: 0x2;
        PRINT_TYPE :: 0x4;
    } = xx 0x7;
    before_name := "";
    after_name := " = ";
    before_value := "";
    after_value := " (";
    before_type := "";
    after_type := ")";
    line_separator := "\n";
    Indentation :: enum u8 {
        NONE;
        LEFT_JUSTIFIED;
        RIGHT_JUSTIFIED;
    }
    name_indentation := Indentation.LEFT_JUSTIFIED;
    value_indentation := Indentation.LEFT_JUSTIFIED;
}

#load "../internal/developer-tools/print-vars.jai";
