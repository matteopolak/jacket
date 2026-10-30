#assert OS == .WINDOWS;

// Reads the zone part of an IPv6 scope id from `fields`.
get_anon_struct_Zone :: (fields: SCOPE_ID) -> u32 {
    return Word.zone(fields.Value);
}

// Sets the zone part of an IPv6 scope id in `fields`, keeping the other bits.
set_anon_struct_Zone :: (fields: *SCOPE_ID, value: u32) {
    fields.Value = Word.with_zone(fields.Value, value);
}

// Reads the level part of an IPv6 scope id from `fields`.
get_anon_struct_Level :: (fields: SCOPE_ID) -> u32 {
    return Word.level(fields.Value);
}

// Sets the level part of an IPv6 scope id in `fields`, keeping the other bits.
set_anon_struct_Level :: (fields: *SCOPE_ID, value: u32) {
    fields.Value = Word.with_level(fields.Value, value);
}

#scope_file
Word :: #import "internal/net/scope-word";
