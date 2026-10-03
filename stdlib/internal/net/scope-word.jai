zone :: (word: u32) -> u32 {
    return word & 0x0fffffff;
}

level :: (word: u32) -> u32 {
    return word >> 28;
}

with_zone :: (word: u32, value: u32) -> u32 {
    return (word & 0xf0000000) | (value & 0x0fffffff);
}

with_level :: (word: u32, value: u32) -> u32 {
    return (word & 0x0fffffff) | ((value & 15) << 28);
}
