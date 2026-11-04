// Sanitized-only harness: establish visible logical bounds inside an owned allocation.
#import "Basic";
M :: #import "meow_hash";
P :: #import "Extensions/Meow_Portable";

libc :: #system_library "libc"; @native(external)
poison :: (address: *void, size: u64) #foreign libc "__asan_poison_memory_region";
unpoison :: (address: *void, size: u64) #foreign libc "__asan_unpoison_memory_region";

is_poisoned :: (address: *void) -> s32 #foreign libc "__asan_address_is_poisoned";

c_malloc :: (size: u64) -> *void #foreign libc "malloc";
c_free :: (memory: *void) #foreign libc "free";

main :: () {
    for count: 1..513 for offset : 0..15 {
        base := cast(*u8) c_malloc(cast(u64) (count + offset + 32));

        assert(base != null);
        source := base + offset + 16;

        for 0..count - 1 source[it] = cast,trunc(u8) (it * 73 + count);

        expected := P.hash_bytes(([] u8).{ count, source });

        poison(base, cast(u64) (offset + 16));
        poison(source + count, 16);
        assert(is_poisoned(base) != 0 && is_poisoned(source) == 0);
        assert(is_poisoned(source + count) != 0, "end visibility, count %, offset %", count, offset);
        #asm { pxor hash:, hash; }
        M.MeowHash(M.MeowDefaultSeed, cast(u64) count, source, hash);
        assert(M.MeowU64From(hash, 0) == expected[0] && M.MeowU64From(hash, 1) == expected[1]);
        unpoison(base, cast(u64) (count + offset + 32));
        c_free(base);
    }

    print("ok\n");
}
