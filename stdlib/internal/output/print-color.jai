Console_Style :: enum {
    NORMAL :: 0;
    BOLD :: 1;
    UNDERLINE :: 4;
}

Console_Color :: enum_flags u16 {
    BLACK :: 30;
    RED :: 31;
    GREEN :: 32;
    BLUE :: 34;
    CYAN :: 36;
    MAGENTA :: 35;
    YELLOW :: 33;
    WHITE :: 37;
    HI_BLACK :: 90;
    HI_RED :: 91;
    HI_GREEN :: 92;
    HI_BLUE :: 94;
    HI_CYAN :: 96;
    HI_MAGENTA :: 95;
    HI_YELLOW :: 93;
    HI_WHITE :: 97;
    BG_BLACK :: 40;
    BG_RED :: 41;
    BG_GREEN :: 42;
    BG_BLUE :: 44;
    BG_CYAN :: 46;
    BG_MAGENTA :: 45;
    BG_YELLOW :: 43;
    BG_WHITE :: 47;
    BG_HI_BLACK :: 100;
    BG_HI_RED :: 101;
    BG_HI_GREEN :: 102;
    BG_HI_BLUE :: 104;
    BG_HI_CYAN :: 106;
    BG_HI_MAGENTA :: 105;
    BG_HI_YELLOW :: 103;
    BG_HI_WHITE :: 107;
}

set_console_color :: (color: Console_Color, style: Console_Style = .NORMAL, to_standard_error := false) {
    write_string("\e[", to_standard_error);
    write_number(cast(s64) style, to_standard_error = to_standard_error);
    write_string(";", to_standard_error);
    write_number(cast(s64) color, to_standard_error = to_standard_error);
    write_string("m", to_standard_error);
}

reset_console_color :: (to_standard_error := false) {
    write_string("\e[0m", to_standard_error);
}

with_console_color :: (color: Console_Color, code: Code, style: Console_Style = .NORMAL, to_standard_error := false) #expand {
    set_console_color(color, style, to_standard_error);
    defer reset_console_color(to_standard_error);
    #insert code;
}

print_color :: (format_string: string, args: ..Any, color: Console_Color, style: Console_Style = .NORMAL, to_standard_error := false) {
    set_console_color(color, style, to_standard_error);
    defer reset_console_color(to_standard_error);
    B.print(format_string, ..args, to_standard_error = to_standard_error);
}

#scope_file
B :: #import "Basic";
