get_line_number :: (using info: Codeview_Line_Info) -> u32 {
    return line_and_delta_and_flag & 0x00ffffff;
}

get_line_number_end_delta :: (using info: Codeview_Line_Info) -> u8 {
    return cast(u8) ((line_and_delta_and_flag >> 24) & 0x7f);
}

line_info_is_for_statement :: (using info: Codeview_Line_Info) -> bool {
    return (line_and_delta_and_flag & 0x80000000) != 0;
}
