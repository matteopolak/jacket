// Prints the source text, value and type of expressions: `print_vars(x, y)` writes one line per
// variable, like `x = 7 (s32)`. `print_vars_to` writes into a `String_Builder`, `log_vars` logs.
//
// ```jai
// count := 3;
// print_vars(count);
// ```

// How `print_vars` lays out a line: which of the name, value and type appear, the text around each,
// and how the columns are aligned.
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
