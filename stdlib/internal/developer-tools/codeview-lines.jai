// Returns the line number of `info`: the low 24 bits of `line_and_delta_and_flag`.
get_line_number :: (using info: Codeview_Line_Info) -> u32 {
    return line_and_delta_and_flag & 0x00ffffff;
}

// Returns the line-end delta of `info`: bits 24 to 30 of `line_and_delta_and_flag`.
get_line_number_end_delta :: (using info: Codeview_Line_Info) -> u8 {
    return cast(u8) ((line_and_delta_and_flag >> 24) & 0x7f);
}

// Returns whether the row marks a statement: bit 31 of `line_and_delta_and_flag`.
line_info_is_for_statement :: (using info: Codeview_Line_Info) -> bool {
    return (line_and_delta_and_flag & 0x80000000) != 0;
}
