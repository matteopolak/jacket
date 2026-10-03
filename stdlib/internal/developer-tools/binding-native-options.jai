set_ExcludeDeclarationsFromPCH :: (fields: *CXIndexOptions, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~cast(u16) 1) | cast(u16) value;
}

get_ExcludeDeclarationsFromPCH :: (fields: CXIndexOptions) -> bool {
    return (fields.__bitfield & 1) != 0;
}

set_DisplayDiagnostics :: (fields: *CXIndexOptions, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~cast(u16) 2) | ((cast(u16) value) << 1);
}

get_DisplayDiagnostics :: (fields: CXIndexOptions) -> bool {
    return (fields.__bitfield & 2) != 0;
}

set_StorePreamblesInMemory :: (fields: *CXIndexOptions, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~cast(u16) 4) | ((cast(u16) value) << 2);
}

get_StorePreamblesInMemory :: (fields: CXIndexOptions) -> bool {
    return (fields.__bitfield & 4) != 0;
}
