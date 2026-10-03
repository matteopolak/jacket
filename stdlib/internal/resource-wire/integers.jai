store_u16 :: (data: *u8, offset: s64, value: u16) {
    data[offset] = cast,truncate(u8) value;
    data[offset + 1] = cast,truncate(u8) (value >> 8);
}

store_u32 :: (data: *u8, offset: s64, value: u32) {
    for byte: 0..3 data[offset + byte] = cast,truncate(u8) (value >> (byte * 8));
}

store_u64 :: (data: *u8, offset: s64, value: u64) {
    for byte: 0..7 data[offset + byte] = cast,truncate(u8) (value >> (byte * 8));
}

contains_extent :: (data: string, offset: s64, count: s64) -> bool {
    return data.count >= 0 && offset >= 0 && count >= 0 && offset <= data.count && count <= data.count - offset && (count == 0 || data.data != null);
}

load_u16 :: (data: string, offset: s64) -> u16 {
    return (cast(u16) data[offset]) | ((cast(u16) data[offset + 1]) << 8);
}

load_u32 :: (data: string, offset: s64) -> u32 {
    value: u32;
    for byte: 0..3 value |= (cast(u32) data[offset + byte]) << (byte * 8);
    return value;
}

load_u64 :: (data: string, offset: s64) -> u64 {
    value: u64;
    for byte: 0..7 value |= (cast(u64) data[offset + byte]) << (byte * 8);
    return value;
}
