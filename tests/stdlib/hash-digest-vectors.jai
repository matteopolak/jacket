// Hash, Crc, md5, xxHash and meow_hash at run time, against published test vectors
// (RFC 1321, the CRC-64 check values, the xxHash reference outputs) and values from an
// independent reference model: empty input, block and padding boundaries, long input, seeds
// and streaming in uneven chunks.
#import "Basic";
#import "Hash";
#import "Crc";
#import "md5";
#import "xxHash";
#import "meow_hash";

main :: () {
    // A 1000-byte pattern used for the longer inputs: byte i is (7i + 3) mod 256.
    pattern: [1000] u8;
    for *pattern it.* = cast,trunc(u8) (it_index * 7 + 3);
    long := string.{ pattern.count, pattern.data };

    hashes(long);
    crc(long);
    md5_digests(long);
    xxhash(long);

    assert(MEOW_HASH_VERSION == 5 && MEOW_HASH_VERSION_NAME == "0.5/calico" && MEOW_PAGESIZE == 4096);
    assert(MeowDefaultSeed.count == 128 && MeowDefaultSeed[0] == 0x32 && MeowDefaultSeed[1] == 0x43 && MeowDefaultSeed[127] == 0xe6, "meow seed is pi's hex digits");
    print("ok\n");
}

hashes :: (long: string) {
    assert(djb2_hash("") == 5381 && djb2_hash("abc") == 0x0b885c8b && djb2_hash(long) == 0xa4455971);
    abc := "abc";
    assert(sdbm_hash(abc.data, abc.count) == 0x27e7b71d && sdbm_hash(abc.data, abc.count, 0) == 0x3025f862 && sdbm_hash(long.data, long.count) == 0xe05847b1);
    assert(sdbm_hash(null, 0) == 5381, "no bytes leave the initial value");
    a := "a";
    foobar := "foobar";
    assert(fnv1a_hash(null, 0) == 0xcbf29ce484222325 && fnv1a_hash(a.data, a.count) == 0xaf63dc4c8601ec8c && fnv1a_hash(foobar.data, foobar.count) == 0x85944171f73967e8);
    assert(fnv1a_hash(cast(u64) 42) == 0xaf63a74c8601927d && fnv1a_hash(cast(u64) 97) == 0xaf63dc4c8601ec8c, "one-value step");
    assert(fnv1a_hash(foobar.data + 3, 3, fnv1a_hash(foobar.data, 3)) == fnv1a_hash(foobar.data, 6), "chained");
    assert(knuth_hash(12345) == 0xa12ce22b4ed990ad && knuth_hash(0) == 0);

    assert(get_hash(12345) == 0x199856df && get_hash(-1) == 0xbdb6ee35, "integer get_hash");
    assert(get_hash(cast(u8) 7) == get_hash(7) && get_hash(true) == get_hash(1), "small integers and bools hash by value");
    E :: enum {
        A;
        B :: 12345;
    }
    assert(get_hash(E.B) == get_hash(12345));
    assert(get_hash("abc") == 0xf173e0ab && get_hash("abc") != get_hash("abd") && get_hash("") == 5381);
    half: float32 = 1.5;
    wide: float64 = 1.5;
    assert(get_hash(half) == 0xb8291f84 && get_hash(wide) == 0x3d0f084c, "floats hash their bytes: % %", get_hash(half), get_hash(wide));
    numbers := s32.[1, 2, 3];
    assert(get_hash(numbers) == 0x07dd0e7f, "array get_hash: %", get_hash(numbers));
    empty: [] s32;
    assert(get_hash(empty) == 5381);
    p := *numbers[0];
    assert(get_hash(p) == get_hash(p) && get_hash(p) != get_hash(*numbers[1]), "pointers hash by address");

    // sdbm_float_hash treats -0 as +0.
    signed_zero := float.[0, -0.0, 1];
    plain_zero := float.[0, 0, 1];
    assert(sdbm_float_hash(signed_zero.data, 3) == 0x2989c5c4 && sdbm_float_hash(plain_zero.data, 3) == 0x2989c5c4);
}

crc :: (long: string) {
    assert(crc64("123456789") == 0x6c40df5f0b497347, "CRC-64/ECMA-182 check value: %", formatInt(crc64("123456789"), base = 16));
    assert(crc64_we("123456789") == 0x62ec59e3f1a4f00a, "CRC-64/WE check value: %", formatInt(crc64_we("123456789"), base = 16));
    assert(crc64("") == 0 && crc64_we("") == 0 && crc64("a") == 0x548f120162451c62);
    assert(crc64(long) == 0x98aba5ce5ce043a1 && crc64_we(long) == 0x6a0ff4f988a7174d);
}

md5_digests :: (long: string) {
    check :: (input: string, expected: string, loc := #caller_location) {
        digest := md5(input);
        assert(digest == expected, "md5 of % bytes: %", input.count, digest, loc = loc);
        free(digest);
    }
    check("", "d41d8cd98f00b204e9800998ecf8427e");
    check("a", "0cc175b9c0f1b6a831c399e269772661");
    check("abc", "900150983cd24fb0d6963f7d28e17f72");
    check("message digest", "f96b697d7cb7938d525a2f31aaf161d0");
    check("abcdefghijklmnopqrstuvwxyz", "c3fcd3d76192e4007dfb496cca67e13b");
    check("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789", "d174ab98d277d9f5a5611c2c9f419d9f");
    check("12345678901234567890123456789012345678901234567890123456789012345678901234567890", "57edf4a22be3c955ac49da2e2107b67a");
    check("The quick brown fox jumps over the lazy dog", "9e107d9d372bb6826bd81d3542a419d6");
    check("héllo wörld", "ed0c22cc110ede12327851863c078138");
    // Lengths around the 56-byte padding limit and the 64-byte block.
    a_run: [65] u8;
    for *a_run it.* = #char "A";
    runs := string.["e38a93ffe074a99b3fed47dfbe37db21", "a2f3e2024931bd470555002aa5ccc010", "5f1c4bb2970471a5c75b7ba1dc9ee3ed", "d289a97565bc2d27ac8b8545a5ddba45", "162b6d6eb17cd9da55f95f8c73a32dda"];
    lengths := int.[55, 56, 63, 64, 65];
    for lengths check(string.{ it, a_run.data }, runs[it_index]);
    check(long, "10046f077f2082ac19676b8079f1cb1a");
}

xxhash :: (long: string) {
    assert(XXH_versionNumber() == 700 && XXH_VERSION_NUMBER == XXH_VERSION_MAJOR * 10000 + XXH_VERSION_MINOR * 100 + XXH_VERSION_RELEASE);
    empty := "";
    assert(native_XXH32(empty.data, 0, 0) == 0x02cc5d05 && native_XXH64(empty.data, 0, 0) == 0xef46db3751d8e999);
    abc := "abc";
    assert(native_XXH32(abc.data, 3, 0) == 0x32d153ff && native_XXH64(abc.data, 3, 0) == 0x44bc2cf5ad770999);
    assert(native_XXH32(abc.data, 3, 1) == 0xaa3da8ff && native_XXH64(abc.data, 3, 1) == 0xbea9ca8199328908);
    spam := "Nobody inspects the spammish repetition";
    assert(native_XXH32(spam.data, xx spam.count, 0) == 0xe2293b2f && native_XXH64(spam.data, xx spam.count, 0) == 0xfbcea83c8a378bf1);

    // Every tail shape: lengths on both sides of the 16- and 32-byte stripes.
    lengths := u32.[1, 3, 4, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 100, 1000];
    expected32 := u32.[0x21ae663a, 0xcd3d2242, 0x725479f2, 0xf3a90eb0, 0x29b5025a, 0x82d3c84d, 0x8845547d, 0x450e5f84, 0x1f73d3c7, 0xe988ccb7, 0x94f536d5, 0x0bedc2f7, 0xecd2ac44, 0x73091a4d, 0xc85a5152];
    expected64 := u64.[0x1f25c8d0bc1f4bb6, 0x31d2363f52e564c9, 0x9bb64b7d66ee9fda, 0x9a7b149959ce60d8, 0xdab99d95c6f90092, 0x1b47cb8243cc8e32, 0x434850232b787be2, 0x1efa7025f1b97a7a, 0xa2aa5f33cc4a6119, 0x23c3c17ef790fd97, 0x50a7cfc7ba588784, 0x5e3e54b431c7493c, 0x0eb64b3ef6eeb01f, 0xa61f8d4c170fe531, 0x5f235fa033f1a3fb];
    for lengths {
        h32 := native_XXH32(long.data, it, 0);
        h64 := native_XXH64(long.data, it, 0);
        assert(h32 == expected32[it_index], "XXH32 of % bytes: %", it, formatInt(h32, base = 16));
        assert(h64 == expected64[it_index], "XXH64 of % bytes: %", it, formatInt(h64, base = 16));
    }
    assert(native_XXH32(long.data, 1000, 0x9747b28c) == 0xd869231f && native_XXH64(long.data, 1000, 0x9747b28c) == 0x9796f63c03442fab, "seeded");
    assert(native_XXH32(long.data, 1000, 1) == 0xce37dff7 && native_XXH64(long.data, 1000, 1) == 0xe67a374d77eccc3f);

    // Streaming in uneven chunks gives the one-shot digest.
    s32: XXH32_state_t;
    s64: XXH64_state_t;
    XXH32_reset(*s32, 0x9747b28c);
    XXH64_reset(*s64, 0x9747b28c);
    offset := 0;
    chunk := 1;
    while offset < long.count {
        size := min(chunk, long.count - offset);
        XXH32_update(*s32, long.data + offset, xx size);
        XXH64_update(*s64, long.data + offset, xx size);
        offset += size;
        chunk = chunk * 3 + 1;
    }
    assert(XXH32_digest(*s32) == 0xd869231f && XXH64_digest(*s64) == 0x9796f63c03442fab, "streamed digests");
    assert(XXH32_digest(*s32) == 0xd869231f, "digest does not disturb the state");
    XXH32_update(*s32, null, 0);
    assert(XXH32_digest(*s32) == 0xd869231f, "an empty update changes nothing");

    copy: [8] u8;
    memcpy_bytes(copy.data, abc.data, 3);
    memcpy_bytes(copy.data + 3, abc.data, 0);
    assert(copy[0] == #char "a" && copy[2] == #char "c" && copy[3] == 0);
}
