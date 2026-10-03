// Decode a two's-complement native field before performing checked narrowing.
// Masking discards unrelated modifier/pointer bits; subtraction stays in s32.
signed_word :: (packed: u64) -> s32 {
    value := cast(s32) (packed & 0xffff);
    if value >= 0x8000 value -= 0x10000;
    return value;
}

high_signed_word :: (packed: u64) -> s32 {
    return signed_word(packed >> 16);
}
