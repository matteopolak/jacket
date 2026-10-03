#assert OS == .WINDOWS;

get_anon_struct_Zone :: (fields: ABI.SCOPE_ID) -> u32 {
    return Word.zone(fields.Value);
}

set_anon_struct_Zone :: (fields: *ABI.SCOPE_ID, value: u32) {
    fields.Value = Word.with_zone(fields.Value, value);
}

get_anon_struct_Level :: (fields: ABI.SCOPE_ID) -> u32 {
    return Word.level(fields.Value);
}

set_anon_struct_Level :: (fields: *ABI.SCOPE_ID, value: u32) {
    fields.Value = Word.with_level(fields.Value, value);
}

#scope_file
ABI :: #import "internal/net/socket-abi";
Word :: #import "internal/net/scope-word";
