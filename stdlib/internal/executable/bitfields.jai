// Returns the symbol table index stored in the low 24 bits of `fields`.
get_isym :: (fields: dylib_reference) -> u32 {
    return ((fields.__bitfield >> 0) & 0xffffff);
}

// Stores the low 24 bits of `value` as the symbol table index, keeping the flags. Does nothing if `fields` is null.
set_isym :: (fields: *dylib_reference, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xff000000) | ((lane & 0xffffff) << 0);
}

// Returns the flags byte stored in the high 8 bits of `fields`.
get_flags :: (fields: dylib_reference) -> u32 {
    return ((fields.__bitfield >> 24) & 0xff);
}

// Stores the low 8 bits of `value` as the flags byte, keeping the symbol index. Does nothing if `fields` is null.
set_flags :: (fields: *dylib_reference, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xffffff) | ((lane & 0xff) << 24);
}

// Returns the sub-image index stored in the low 8 bits of `fields`.
get_isub_image :: (fields: twolevel_hint) -> u32 {
    return ((fields.__bitfield >> 0) & 0xff);
}

// Stores the low 8 bits of `value` as the sub-image index, keeping the table-of-contents index. Does nothing if
// `fields` is null.
set_isub_image :: (fields: *twolevel_hint, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xffffff00) | ((lane & 0xff) << 0);
}

// Returns the table-of-contents index stored in the high 24 bits of `fields`.
get_itoc :: (fields: twolevel_hint) -> u32 {
    return ((fields.__bitfield >> 8) & 0xffffff);
}

// Stores the low 24 bits of `value` as the table-of-contents index, keeping the sub-image index. Does nothing if
// `fields` is null.
set_itoc :: (fields: *twolevel_hint, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xff) | ((lane & 0xffffff) << 8);
}

// Returns the symbol or section number stored in the low 24 bits of `fields`.
get_r_symbolnum :: (fields: relocation_info) -> u32 {
    return ((fields.__bitfield >> 0) & 0xffffff);
}

// Stores the low 24 bits of `value` as the symbol or section number, keeping the other fields. Does nothing if
// `fields` is null.
set_r_symbolnum :: (fields: *relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xff000000) | ((lane & 0xffffff) << 0);
}

// Returns whether bit 24 of `fields`, the PC-relative flag, is set.
get_r_pcrel :: (fields: relocation_info) -> bool {
    return ((fields.__bitfield >> 24) & 0x1) != 0;
}

// Sets or clears bit 24 of `fields`, the PC-relative flag, keeping the other fields. Does nothing if `fields` is
// null.
set_r_pcrel :: (fields: *relocation_info, value: bool) {
    if !fields return;
    lane := cast(u32) value;
    fields.__bitfield = (fields.__bitfield & 0xfeffffff) | ((lane & 0x1) << 24);
}

// Returns the 2-bit length code in bits 25 and 26 of `fields`; the patched field is `1 << length` bytes wide.
get_r_length :: (fields: relocation_info) -> u32 {
    return ((fields.__bitfield >> 25) & 0x3);
}

// Stores the low 2 bits of `value` as the length code, keeping the other fields. Does nothing if `fields` is null.
set_r_length :: (fields: *relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xf9ffffff) | ((lane & 0x3) << 25);
}

// Returns whether bit 27 of `fields`, the external-symbol flag, is set.
get_r_extern :: (fields: relocation_info) -> bool {
    return ((fields.__bitfield >> 27) & 0x1) != 0;
}

// Sets or clears bit 27 of `fields`, the external-symbol flag, keeping the other fields. Does nothing if `fields`
// is null.
set_r_extern :: (fields: *relocation_info, value: bool) {
    if !fields return;
    lane := cast(u32) value;
    fields.__bitfield = (fields.__bitfield & 0xf7ffffff) | ((lane & 0x1) << 27);
}

// Returns the relocation type stored in the high 4 bits of `fields`.
get_r_type :: (fields: relocation_info) -> u32 {
    return ((fields.__bitfield >> 28) & 0xf);
}

// Stores the low 4 bits of `value` as the relocation type, keeping the other fields. Does nothing if `fields` is
// null.
set_r_type :: (fields: *relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xfffffff) | ((lane & 0xf) << 28);
}

// Returns the address stored in the low 24 bits of a scattered relocation.
get_r_address :: (fields: scattered_relocation_info) -> u32 {
    return ((fields.__bitfield >> 0) & 0xffffff);
}

// Stores the low 24 bits of `value` as the address, keeping the other fields. Does nothing if `fields` is null.
set_r_address :: (fields: *scattered_relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xff000000) | ((lane & 0xffffff) << 0);
}

// Returns the relocation type stored in bits 24 to 27 of a scattered relocation.
get_r_type :: (fields: scattered_relocation_info) -> u32 {
    return ((fields.__bitfield >> 24) & 0xf);
}

// Stores the low 4 bits of `value` as the relocation type, keeping the other fields. Does nothing if `fields` is
// null.
set_r_type :: (fields: *scattered_relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xf0ffffff) | ((lane & 0xf) << 24);
}

// Returns the 2-bit length code in bits 28 and 29 of a scattered relocation.
get_r_length :: (fields: scattered_relocation_info) -> u32 {
    return ((fields.__bitfield >> 28) & 0x3);
}

// Stores the low 2 bits of `value` as the length code, keeping the other fields. Does nothing if `fields` is null.
set_r_length :: (fields: *scattered_relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xcfffffff) | ((lane & 0x3) << 28);
}

// Returns whether bit 30 of a scattered relocation, the PC-relative flag, is set.
get_r_pcrel :: (fields: scattered_relocation_info) -> bool {
    return ((fields.__bitfield >> 30) & 0x1) != 0;
}

// Sets or clears bit 30, the PC-relative flag, keeping the other fields. Does nothing if `fields` is null.
set_r_pcrel :: (fields: *scattered_relocation_info, value: bool) {
    if !fields return;
    lane := cast(u32) value;
    fields.__bitfield = (fields.__bitfield & 0xbfffffff) | ((lane & 0x1) << 30);
}

// Returns whether bit 31, the flag that marks a relocation as scattered, is set.
get_r_scattered :: (fields: scattered_relocation_info) -> bool {
    return ((fields.__bitfield >> 31) & 0x1) != 0;
}

// Sets or clears bit 31, the scattered flag, keeping the other fields. Does nothing if `fields` is null.
set_r_scattered :: (fields: *scattered_relocation_info, value: bool) {
    if !fields return;
    lane := cast(u32) value;
    fields.__bitfield = (fields.__bitfield & 0x7fffffff) | ((lane & 0x1) << 31);
}
