ignored_asserts: Tables.Table(Source_Code_Location, bool, hash_scl, compare_scl);

#scope_file
Tables :: #import "Hash_Table";
Hash :: #import "Hash";

hash_scl :: (loc: Source_Code_Location) -> u32 {
    return Hash.get_hash(loc.fully_pathed_filename) ^ cast(u32) loc.line_number ^ cast(u32) loc.character_number;
}

compare_scl :: (a: Source_Code_Location, b: Source_Code_Location) -> bool {
    return a.line_number == b.line_number && a.character_number == b.character_number && a.fully_pathed_filename == b.fully_pathed_filename;
}
