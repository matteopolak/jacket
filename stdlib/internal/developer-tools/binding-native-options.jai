// Sets the `ExcludeDeclarationsFromPCH` bit of `fields` to `value`.
set_ExcludeDeclarationsFromPCH :: (fields: *CXIndexOptions, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~cast(u16) 1) | cast(u16) value;
}

// Returns the `ExcludeDeclarationsFromPCH` bit of `fields`.
get_ExcludeDeclarationsFromPCH :: (fields: CXIndexOptions) -> bool {
    return (fields.__bitfield & 1) != 0;
}

// Sets the `DisplayDiagnostics` bit of `fields` to `value`.
set_DisplayDiagnostics :: (fields: *CXIndexOptions, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~cast(u16) 2) | ((cast(u16) value) << 1);
}

// Returns the `DisplayDiagnostics` bit of `fields`.
get_DisplayDiagnostics :: (fields: CXIndexOptions) -> bool {
    return (fields.__bitfield & 2) != 0;
}

// Sets the `StorePreamblesInMemory` bit of `fields` to `value`.
set_StorePreamblesInMemory :: (fields: *CXIndexOptions, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~cast(u16) 4) | ((cast(u16) value) << 2);
}

// Returns the `StorePreamblesInMemory` bit of `fields`.
get_StorePreamblesInMemory :: (fields: CXIndexOptions) -> bool {
    return (fields.__bitfield & 4) != 0;
}
