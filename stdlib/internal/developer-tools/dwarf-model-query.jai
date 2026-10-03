get_type :: (cu: Compilation_Unit, type: u32) -> *Debug_Type {
    if cu.types.count < 0 || (cu.types.count > 0 && !cu.types.data) {
        log_error("Debug_Info: invalid DWARF type table\n");
        return null;
    }

    found: *Debug_Type;

    for *entry: cu.types {
        if entry.offset != type continue;

        if found {
            log_error("Debug_Info: ambiguous DWARF type offset %\n", type);
            return null;
        }

        found = entry;
    }

    return found;
}

#scope_file
#import "Basic";
