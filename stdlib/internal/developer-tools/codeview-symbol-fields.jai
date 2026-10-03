get_spilled_udt_member :: (s: *Codeview_Symbol_Defrange_Register_Rel) -> bool {
    return (s.spilled_udt_member_and_offset_parent & 1) != 0;
}

get_offset_parent :: (s: *Codeview_Symbol_Defrange_Register_Rel) -> u16 {
    return s.spilled_udt_member_and_offset_parent >> 4;
}
