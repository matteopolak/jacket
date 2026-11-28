// Returns whether the variable `s` describes is a spilled member of a user-defined type: bit 0 of the packed field.
get_spilled_udt_member :: (s: *Codeview_Symbol_Defrange_Register_Rel) -> bool {
    return (s.spilled_udt_member_and_offset_parent & 1) != 0;
}

// Returns the offset of the member in its parent type: the packed field shifted right by 4 bits.
get_offset_parent :: (s: *Codeview_Symbol_Defrange_Register_Rel) -> u16 {
    return s.spilled_udt_member_and_offset_parent >> 4;
}
