// test: skip_on(.WASM)
// CPU feature detection and choosing a version of a procedure by it.
//
//     jaic stdlib/Extensions/SIMD/tests/cpu-dispatch.jai -interp

#import "Basic";
SIMD :: #import "Extensions/SIMD";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

// The same sum in three versions; the compiler is asked for AVX2 and SSE4.2 code in two of them.
sum_base :: (values: [] s32) -> s64 {
    total: s64;
    for values total += it;
    return total;
}

sum_sse42 :: (values: [] s32) -> s64 #target_features "sse4.2" {
    total: s64;
    for values total += it;
    return total;
}

sum_avx2 :: (values: [] s32) -> s64 #target_features "avx2,bmi2" {
    total: s64;
    for values total += it;
    return total;
}

// Features this architecture lacks are ignored, not an error.
sum_other :: (values: [] s32) -> s64 #target_features "sve,neon,simd128,not-a-feature" {
    total: s64;
    for values total += it;
    return total;
}

tiers :: () {
    f := SIMD.cpu_features();

    // Every tier has the ones below it.
    if f.avx512 check(f.avx2, "AVX-512 without AVX2");
    if f.avx2 check(f.sse42, "AVX2 without SSE4.2");

    #if CPU == .X64 {
        check(!f.neon && !f.sve, "arm features on x64");
    } else #if CPU == .ARM64 {
        check(f.neon, "arm64 without NEON");
        check(!f.sse42 && !f.avx2 && !f.avx512, "x64 features on arm64");
    } else {
        check(!f.sse42 && !f.avx2 && !f.avx512 && !f.neon && !f.sve, "features on an architecture with none");
    }

    best := SIMD.best_vector_tier();
    if f.sve check(best == .SVE, "best tier with SVE");
    else if f.avx512 check(best == .AVX512, "best tier with AVX-512");
    else if f.avx2 check(best == .AVX2, "best tier with AVX2");
    else if f.sse42 check(best == .SSE42, "best tier with SSE4.2");
    else check(best == .BASELINE, "best tier with none");
}

limits :: () {
    real := SIMD.cpu_features();

    SIMD.limit_vector_tier(.BASELINE);
    none := SIMD.cpu_features();

    check(!none.sse42 && !none.avx2 && !none.avx512 && !none.sve, "limit to the baseline");
    check(none.neon == real.neon, "NEON is the arm64 baseline, not a tier");
    check(SIMD.best_vector_tier() == .BASELINE, "baseline tier");

    SIMD.limit_vector_tier(.SSE42);
    f := SIMD.cpu_features();

    check(f.sse42 == real.sse42 && !f.avx2 && !f.avx512, "limit to SSE4.2");

    SIMD.limit_vector_tier(.AVX2);
    f = SIMD.cpu_features();
    check(f.avx2 == real.avx2 && !f.avx512, "limit to AVX2");

    SIMD.limit_vector_tier(.SVE);
    f = SIMD.cpu_features();
    check(f.sse42 == real.sse42 && f.avx2 == real.avx2 && f.avx512 == real.avx512 && f.sve == real.sve, "no limit gives the real features");
}

picking :: () {
    pick :: () -> int {
        // The versions return their tier, so the choice can be read off.
        table := SIMD.pick_target(tier_base, sse42 = tier_sse42, avx2 = tier_avx2, avx512 = tier_avx512, sve = tier_sve);
        return table();
    }

    SIMD.limit_vector_tier(.BASELINE);
    check(pick() == 0, "baseline");
    SIMD.limit_vector_tier(.SVE);
    real := SIMD.best_vector_tier();

    check(pick() == cast(int) real, "best available");

    // A tier without a version falls to the next one down.
    SIMD.limit_vector_tier(.SVE);
    f := SIMD.cpu_features();
    only_base := SIMD.pick_target(tier_base);

    check(only_base() == 0, "only a baseline version");
    avx2_only := SIMD.pick_target(tier_base, avx2 = tier_avx2);

    check(avx2_only() == ifx f.avx2 then 2 else 0, "AVX2 version used only where AVX2 exists");

    SIMD.limit_vector_tier(.SVE);
}

tier_base :: () -> int {
    return 0;
}

tier_sse42 :: () -> int {
    return 1;
}

tier_avx2 :: () -> int {
    return 2;
}

tier_avx512 :: () -> int {
    return 3;
}

tier_sve :: () -> int {
    return 4;
}

// A version compiled for a feature runs where the processor has it, and gives the same answer.
versions :: () {
    values: [100] s32;
    for 0..99 values[it] = cast(s32) (it * 7 - 300);

    expected := sum_base(values);
    f := SIMD.cpu_features();

    check(sum_other(values) == expected, "target features of other architectures");
    if CPU != .X64 || f.sse42 check(sum_sse42(values) == expected, "SSE4.2 version");
    if CPU != .X64 || f.avx2 check(sum_avx2(values) == expected, "AVX2 version");
}

main :: () {
    tiers();
    limits();
    picking();
    versions();

    if failures {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
