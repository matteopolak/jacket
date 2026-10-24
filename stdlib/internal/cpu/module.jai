// The vector features of the processor the program runs on, and a way to pick between versions of a
// procedure compiled for different ones. Extensions/SIMD re-exports it; stdlib code imports it from
// here. Detection runs once, on the first use.

Cpu_Features :: struct {
    // x64: SSE4.2 (and everything below it).
    sse42: bool;
    // x64: AVX2, and the operating system saves the vector registers it uses.
    avx2: bool;
    // x64: AVX-512 F, DQ, BW and VL, and the operating system saves the registers they use.
    avx512: bool;
    // arm64: NEON, which every arm64 processor has.
    neon: bool;
    // arm64: SVE, reported only where it is cheap to ask (Linux).
    sve: bool;
}

// The best vector instruction set a processor has, in the order code is picked: each tier has
// everything below it. On arm64 the baseline is NEON.
Vector_Tier :: enum u8 {
    BASELINE;
    SSE42;
    AVX2;
    AVX512;
    SVE;
}

// What the processor has, less whatever limit_vector_tier took away. Detected on the first call.
cpu_features :: () -> Cpu_Features {
    if !detected detect();
    return features;
}

// The best tier the processor offers.
best_vector_tier :: () -> Vector_Tier {
    f := cpu_features();
    if f.sve return .SVE;
    if f.avx512 return .AVX512;
    if f.avx2 return .AVX2;
    if f.sse42 return .SSE42;

    return .BASELINE;
}

// Pretend the processor has nothing above `tier`: for tests and for comparing versions on one
// machine. It applies to procedures picked afterwards, so call it before the first call of
// anything dispatched.
limit_vector_tier :: (tier: Vector_Tier) {
    limit = tier;
    detected = false;
}

// The best version the processor can run: the highest tier with a version given, else `baseline`.
// A version that needs a feature is compiled with `#target_features` and listed under its tier.
//
//     table = pick_target(count_base, sse42 = count_sse42, avx2 = count_avx2);
pick_target :: (baseline: $F, sse42: F = null, avx2: F = null, avx512: F = null, sve: F = null) -> F {
    f := cpu_features();
    if sve != null && f.sve return sve;
    if avx512 != null && f.avx512 return avx512;
    if avx2 != null && f.avx2 return avx2;
    if sse42 != null && f.sse42 return sse42;

    return baseline;
}

#scope_file

detected := false;
features: Cpu_Features;
limit := Vector_Tier.SVE;

detect :: () {
    found: Cpu_Features;

    #if CPU == .X64 detect_x64(*found);
    #if CPU == .ARM64 detect_arm64(*found);

    if limit < .SVE found.sve = false;
    if limit < .AVX512 found.avx512 = false;
    if limit < .AVX2 found.avx2 = false;
    if limit < .SSE42 found.sse42 = false;

    features = found;
    detected = true;
}

#if CPU == .X64 {
    cpuid :: (leaf: u32, subleaf: u32) -> (a: u32, b: u32, c: u32, d: u32) {
        a = leaf;
        c = subleaf;
        #asm { cpuid a, b, c, d; }
        return a, b, c, d;
    }

    // Control register 0 of the extended state: which register files the operating system saves
    // when it switches threads. Only valid when cpuid reports OSXSAVE.
    enabled_state :: () -> u64 {
        hi: u32;
        lo: u32;
        index: u32 = 0;

        #asm { xgetbv hi, lo, index; }

        return ((cast(u64) hi) << 32) | cast(u64) lo;
    }

    detect_x64 :: (found: *Cpu_Features) {
        highest, _b, _c, _d := cpuid(0, 0);
        if highest < 1 return;

        _a, _b2, ecx, _d2 := cpuid(1, 0);

        found.sse42 = (ecx & (1 << 20)) != 0;

        os_saves_state := (ecx & (1 << 27)) != 0;
        has_avx := (ecx & (1 << 28)) != 0;

        if !os_saves_state || !has_avx || highest < 7 return;

        state := enabled_state();
        // XMM and YMM state (bits 1 and 2) for AVX; opmask, ZMM0-15 upper halves and ZMM16-31
        // (bits 5 to 7) as well for AVX-512.
        ymm_saved := (state & 0x6) == 0x6;
        zmm_saved := (state & 0xE6) == 0xE6;

        if !ymm_saved return;

        _a3, ebx, _c3, _d3 := cpuid(7, 0);

        found.avx2 = (ebx & (1 << 5)) != 0;

        // F (16), DQ (17), BW (30) and VL (31).
        wanted: u32 = (1 << 16) | (1 << 17) | (1 << 30) | (1 << 31);

        found.avx512 = zmm_saved && (ebx & wanted) == wanted;
        if found.avx512 found.avx2 = true;
        if found.avx2 found.sse42 = true;
    }
}

#if CPU == .ARM64 {
    detect_arm64 :: (found: *Cpu_Features) {
        found.neon = true;

        #if OS == .LINUX || OS == .ANDROID {
            // AT_HWCAP is 16; HWCAP_SVE is bit 22.
            found.sve = (c_getauxval(16) & (1 << 22)) != 0;
        }
    }

    #if OS == .LINUX || OS == .ANDROID {
        libc :: #system_library "libc";
        c_getauxval :: (kind: u64) -> u64 #foreign libc "getauxval";
    }
}
