// The portable hash agrees with the native MeowHash for a non-default seed, at lengths that cross the
// 16, 32, 256 and 512 byte block boundaries. Runs the native comparison on x86-64 only.
#import "Basic";
#import "Extensions/Meow_Portable";
#import "meow_hash";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

main :: () {
    seed: [128] u8;
    for 0..127 seed[it] = cast,trunc(u8) (255 - it * 3);

    data: [1100] u8;
    for 0..1099 data[it] = cast,trunc(u8) (it * 13 + 5);

    lengths := int.[0, 1, 15, 16, 17, 31, 32, 33, 255, 256, 257, 511, 512, 513, 1000, 1100];

    for count: lengths {
        portable := hash_bytes(([] u8).{ count, data.data }, seed);

        #if CPU == .X64 {
            #asm { pxor hash:, hash; }
            MeowHash(seed, cast(u64) count, data.data, hash);
            native_low := MeowU64From(hash, 0);
            native_high := MeowU64From(hash, 1);

            expect(portable[0] == native_low && portable[1] == native_high, tprint("count % with a custom seed", count));
        }
    }

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
