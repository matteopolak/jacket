#import "Basic";
M :: #import "meow_hash";
Again :: #import "meow_hash";

main :: () {
    assert(type_info(M.reg) == type_info(__reg));
    assert(*M.MeowMaskLen == *Again.MeowMaskLen);
    assert(*M.MeowShiftAdjust == *Again.MeowShiftAdjust);
    assert(*M.MeowDefaultSeed == *Again.MeowDefaultSeed);
    data := cast(*u8) alloc(64);

    defer free(data);
    memset(data, 0, 64);
    data[0] = 97;
    data[1] = 98;
    data[2] = 99;
    #asm { pxor hash:, hash; pxor streamed:, streamed; }
    M.MeowHash(M.MeowDefaultSeed, 3, data, hash);
    lo := M.MeowU64From(hash, 0);
    hi := M.MeowU64From(hash, 1);

    // Independent licensed C source build on this host, not a supplied binary.
    assert(lo == 0x19f773ec430acc3b && hi == 0x5f6d236b424c9ab4);
    assert(M.MeowU32From(hash, 0) == 0x430acc3b && M.MeowU32From(hash, 3) == 0x5f6d236b);
    M.MeowHash(M.MeowDefaultSeed, 0, null, hash);
    assert(M.MeowU64From(hash, 0) == 0x657ca02a5859c045 && M.MeowU64From(hash, 1) == 0x75a7b5550383265e);
    state: M.meow_state;

    assert((cast(*Type_Info_Struct) type_info(M.meow_state)).name == "meow_state");
    M.MeowBegin(*state, M.MeowDefaultSeed);
    M.MeowAbsorb(*state, 1, data);
    M.MeowAbsorb(*state, 2, data + 1);
    assert(state.TotalLengthInBytes == 3 && state.BufferLen == 3);
    output: [128] u8;

    M.MeowEnd(*state, streamed, output.data);
    assert(M.MeowU64From(streamed, 0) == lo && M.MeowU64From(streamed, 1) == hi);
    M.MeowDefaultSeed[0] ^= 1;
    M.MeowHash(M.MeowDefaultSeed, 3, data, hash);
    assert(M.MeowU64From(hash, 0) != lo || M.MeowU64From(hash, 1) != hi);
    M.MeowDefaultSeed[0] ^= 1;
    original := M.MeowMaskLen[13];

    M.MeowMaskLen[13] = 0;
    M.MeowHash(M.MeowDefaultSeed, 3, data, hash);
    assert(M.MeowU64From(hash, 0) != lo || M.MeowU64From(hash, 1) != hi);
    M.MeowMaskLen[13] = original;
    M.MeowShiftAdjust[0] = 1;
    M.MeowHash(M.MeowDefaultSeed, 3, data, hash);
    assert(M.MeowU64From(hash, 0) != lo || M.MeowU64From(hash, 1) != hi);
    M.MeowShiftAdjust[0] = 0;
    M.MeowHash(M.MeowDefaultSeed, 3, data, hash);
    assert(M.MeowU64From(hash, 0) == lo && M.MeowU64From(hash, 1) == hi);
    print("ok\n");
}
