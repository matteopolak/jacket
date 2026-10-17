H :: #import "xxHash";

#assert H.XXH_VERSION_NUMBER == 700;
probe :: #library "abi_probe";
check_size :: (which: s32) -> u64 #foreign probe "binding_size";
check_offset :: (which: s32) -> u64 #foreign probe "binding_offset";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    check(type_info(H.XXH32_state_t).name == "XXH32_state_t");
    check(type_info(H.XXH64_state_t).name == "XXH64_state_t");
    check(H.XXH_versionNumber() == 700);
    check(size_of(H.XXH32_state_t) == check_size(5));
    check(size_of(H.XXH64_state_t) == check_size(6));
    s32state: H.XXH32_state_t;
    s64state: H.XXH64_state_t;

    check(cast(u64) *s32state.mem32 - cast(u64) *s32state == check_offset(3));
    check(cast(u64) *s64state.mem64 - cast(u64) *s64state == check_offset(4));
    check(H.native_XXH32(null, 0, 0) == 0x02cc5d05);
    check(H.native_XXH64(null, 0, 0) == 0xef46db3751d8e999);
    input := "the streaming result must equal the one shot hash across block boundaries";

    H.XXH32_reset(*s32state, 123);
    H.XXH64_reset(*s64state, 123);

    for i: 0..input.count - 1 {
        H.XXH32_update(*s32state, input.data + i, 1);
        H.XXH64_update(*s64state, input.data + i, 1);
    }

    check(H.XXH32_digest(*s32state) == H.native_XXH32(input.data, cast(u32) input.count, 123));
    check(H.XXH64_digest(*s64state) == H.native_XXH64(input.data, cast(u32) input.count, 123));
    H.XXH64_reset(*s64state, 0x123456789abcdef0);
    H.XXH64_update(*s64state, null, 0);
    check(H.XXH64_digest(*s64state) != H.native_XXH64(null, 0, 0x9abcdef0));
}
