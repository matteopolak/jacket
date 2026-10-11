// meow_hash: MeowHash fills the caller's `#asm` vector register, MeowU64From reads a lane of it.
// The expected digests were produced by the published Meow hash 0.5 (calico) x64 AES-NI
// implementation, compiled separately and run with the default seed over the inputs built by
// `fill_input` below, so the test pins the module to the real algorithm: the block loop, the
// zero-extended tail with its two-piece case, the length folding and the final shuffle. On CPUs
// other than x64 the module has no hash, so the program only prints ok.
#import "Basic";
#import "meow_hash";

#if CPU == .X64 {
    Vector :: struct {
        count: s64;
        low: u64;
        high: u64;
    }

    vectors :: Vector.[
        .{ 0, 0x657ca02a5859c045, 0x75a7b5550383265e },
        .{ 1, 0x41f51837a24b495f, 0x8d11a6cc856c9416 },
        .{ 15, 0xca1396366eeef68a, 0x528ea0dec2d5d3b4 },
        .{ 16, 0xcbb20ec6e7d406b9, 0x721c42957b9da197 },
        .{ 31, 0x9b65ccd37c16af69, 0x472a18ed912ae531 },
        .{ 32, 0xb76d5035bef80f3c, 0xbff7384c6065627a },
        .{ 33, 0x036a020205f49d7c, 0x7df941f7eda6387d },
        .{ 100, 0xfcd9721b516c0f95, 0x26ac6e7df82adc35 },
        .{ 255, 0x4d9675cd07f7dea9, 0x26dde6a2dd2f6899 },
        .{ 256, 0xc4fbf61dcd5be3b9, 0x3a86be2e8b652253 },
        .{ 257, 0xc56917d86aa20147, 0xe709902246103e79 },
        .{ 300, 0xdab3c5ee43c2a845, 0x9b4b29b787d2f8da },
        .{ 511, 0xc7e20a795289ca3b, 0x058dc2428c58b4e3 },
        .{ 512, 0xeacc2088f485074a, 0xb1aeda9c2c3e312b },
        .{ 700, 0x21c4e2885bea99f9, 0x8491470e4b8ee2c0 },
        .{ 1000, 0x663cbacc129b1b41, 0x16bbd6bce07befab },
    ];

    failures := 0;

    fill_input :: (data: *u8, count: s64) {
        for 0..count - 1 data[it] = cast,trunc(u8) (it * 31 + count);
    }

    check_vector :: (v: Vector) {
        data := NewArray(v.count + 1, u8);
        defer free(data.data);
        fill_input(data.data, v.count);

        #asm { hash: vec; }
        MeowHash(MeowDefaultSeed, xx v.count, data.data, hash);
        got0 := MeowU64From(hash, 0);
        got1 := MeowU64From(hash, 1);
        if got0 != v.low || got1 != v.high {
            print("FAIL length %: got %/%, want %/%\n", v.count, got0, got1, v.low, v.high);
            failures += 1;
        }
    }
}

main :: () {
    #if CPU == .X64 {
        for vectors check_vector(it);

        // Equal input hashes equal; one differing byte or length changes the result.
        a := u8.[1, 2, 3, 4, 5];
        b := u8.[1, 2, 3, 4, 6];
        c := u8.[1, 2, 3, 4, 5, 0];
        #asm { ha: vec; }
        MeowHash(MeowDefaultSeed, 5, a.data, ha);
        #asm { hb: vec; }
        MeowHash(MeowDefaultSeed, 5, b.data, hb);
        #asm { hc: vec; }
        MeowHash(MeowDefaultSeed, 6, c.data, hc);
        #asm { hd: vec; }
        MeowHash(MeowDefaultSeed, 5, a.data, hd);
        if MeowU64From(ha, 0) != MeowU64From(hd, 0) || MeowU64From(ha, 1) != MeowU64From(hd, 1) {
            print("FAIL not deterministic\n");
            failures += 1;
        }
        if MeowU64From(ha, 0) == MeowU64From(hb, 0) {
            print("FAIL byte change\n");
            failures += 1;
        }
        if MeowU64From(ha, 0) == MeowU64From(hc, 0) {
            print("FAIL length change\n");
            failures += 1;
        }
        if failures != 0 exit(1);
    }
    print("ok\n");
}
