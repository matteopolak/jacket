// Source acceptance for independently authored portable modules.
Crc :: #import "Crc";
Hash :: #import "Hash";
Memory :: #import "Memory";
XXH :: #import "xxHash";

check_crc :: () -> bool {
    return Crc.crc64("") == 0 && Crc.crc64("123456789") == 0x6c40_df5f_0b49_7347 && Crc.crc64_we("123456789") == 0x62ec_59e3_f1a4_f00a;
}

check_copy :: () -> bool {
    bytes: [6] u8 = .[0,1,2,3,4,5];
    Memory.copy(bytes.data + 1, bytes.data, 5);
    if bytes[0] != 0 || bytes[1] != 0 || bytes[5] != 4 return false;
    Memory.copy(bytes.data, bytes.data + 1, 5);
    return bytes[0] == 0 && bytes[1] == 1 && bytes[4] == 4;
}

check_hashes :: () -> bool {
    input := "abc";
    if Hash.djb2_hash(input) != 193485963 return false;
    if Hash.fnv1a_hash(input.data, input.count) != 0xe71fa2190541574b return false;
    scalar: s64 = 7;
    if Hash.get_hash(*scalar) != cast,trunc(u32) (Hash.knuth_hash(cast(u64) *scalar ^ 5381) >> 32) return false;
    if Hash.get_hash(cast(u64) 7) != cast,trunc(u32) (Hash.knuth_hash(cast(u64) (7 ^ 5381)) >> 32) return false;
    if XXH.native_XXH32(null, 0, 0) != 0x02cc5d05 return false;
    if XXH.native_XXH64(null, 0, 0) != 0xef46db3751d8e999 return false;
    input = "0123456789abcdefghijklmnopqrstuvABCDEFGHIJKLMNOPQRSTUVWXYZ";
    state32: XXH.XXH32_state_t;
    state64: XXH.XXH64_state_t;
    XXH.XXH32_reset(*state32, 17);
    XXH.XXH64_reset(*state64, 17);
    index: s64 = 0;
    while index < input.count {
        XXH.XXH32_update(*state32, input.data + index, 1);
        XXH.XXH64_update(*state64, input.data + index, 1);
        index += 1;
    }
    return XXH.XXH32_digest(*state32) == XXH.native_XXH32(input.data, cast(u32) input.count, 17) && XXH.XXH64_digest(*state64) == XXH.native_XXH64(input.data, cast(u32) input.count, 17);
}

#assert #run check_crc();
#assert #run check_copy();
#assert #run check_hashes();

main :: () -> int {
    if !check_crc() return 1;
    if !check_copy() return 3;
    if !check_hashes() return 4;
    return 0;
}
