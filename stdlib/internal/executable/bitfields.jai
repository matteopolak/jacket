// Storage masks are part of the public contract.
get_isym :: (fields: dylib_reference) -> u32 {
    return ((fields.__bitfield >> 0) & 0xffffff);
}

set_isym :: (fields: *dylib_reference, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xff000000) | ((lane & 0xffffff) << 0);
}

get_flags :: (fields: dylib_reference) -> u32 {
    return ((fields.__bitfield >> 24) & 0xff);
}

set_flags :: (fields: *dylib_reference, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xffffff) | ((lane & 0xff) << 24);
}

get_isub_image :: (fields: twolevel_hint) -> u32 {
    return ((fields.__bitfield >> 0) & 0xff);
}

set_isub_image :: (fields: *twolevel_hint, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xffffff00) | ((lane & 0xff) << 0);
}

get_itoc :: (fields: twolevel_hint) -> u32 {
    return ((fields.__bitfield >> 8) & 0xffffff);
}

set_itoc :: (fields: *twolevel_hint, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xff) | ((lane & 0xffffff) << 8);
}

get_r_symbolnum :: (fields: relocation_info) -> u32 {
    return ((fields.__bitfield >> 0) & 0xffffff);
}

set_r_symbolnum :: (fields: *relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xff000000) | ((lane & 0xffffff) << 0);
}

get_r_pcrel :: (fields: relocation_info) -> bool {
    return ((fields.__bitfield >> 24) & 0x1) != 0;
}

set_r_pcrel :: (fields: *relocation_info, value: bool) {
    if !fields return;
    lane := cast(u32) value;
    fields.__bitfield = (fields.__bitfield & 0xfeffffff) | ((lane & 0x1) << 24);
}

get_r_length :: (fields: relocation_info) -> u32 {
    return ((fields.__bitfield >> 25) & 0x3);
}

set_r_length :: (fields: *relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xf9ffffff) | ((lane & 0x3) << 25);
}

get_r_extern :: (fields: relocation_info) -> bool {
    return ((fields.__bitfield >> 27) & 0x1) != 0;
}

set_r_extern :: (fields: *relocation_info, value: bool) {
    if !fields return;
    lane := cast(u32) value;
    fields.__bitfield = (fields.__bitfield & 0xf7ffffff) | ((lane & 0x1) << 27);
}

get_r_type :: (fields: relocation_info) -> u32 {
    return ((fields.__bitfield >> 28) & 0xf);
}

set_r_type :: (fields: *relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xfffffff) | ((lane & 0xf) << 28);
}

get_r_address :: (fields: scattered_relocation_info) -> u32 {
    return ((fields.__bitfield >> 0) & 0xffffff);
}

set_r_address :: (fields: *scattered_relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xff000000) | ((lane & 0xffffff) << 0);
}

get_r_type :: (fields: scattered_relocation_info) -> u32 {
    return ((fields.__bitfield >> 24) & 0xf);
}

set_r_type :: (fields: *scattered_relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xf0ffffff) | ((lane & 0xf) << 24);
}

get_r_length :: (fields: scattered_relocation_info) -> u32 {
    return ((fields.__bitfield >> 28) & 0x3);
}

set_r_length :: (fields: *scattered_relocation_info, value: u32) {
    if !fields return;
    lane := value;
    fields.__bitfield = (fields.__bitfield & 0xcfffffff) | ((lane & 0x3) << 28);
}

get_r_pcrel :: (fields: scattered_relocation_info) -> bool {
    return ((fields.__bitfield >> 30) & 0x1) != 0;
}

set_r_pcrel :: (fields: *scattered_relocation_info, value: bool) {
    if !fields return;
    lane := cast(u32) value;
    fields.__bitfield = (fields.__bitfield & 0xbfffffff) | ((lane & 0x1) << 30);
}

get_r_scattered :: (fields: scattered_relocation_info) -> bool {
    return ((fields.__bitfield >> 31) & 0x1) != 0;
}

set_r_scattered :: (fields: *scattered_relocation_info, value: bool) {
    if !fields return;
    lane := cast(u32) value;
    fields.__bitfield = (fields.__bitfield & 0x7fffffff) | ((lane & 0x1) << 31);
}
