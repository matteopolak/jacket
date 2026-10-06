// Machine_X64 instruction hooks run through jaic's portable #asm lowering.
#import "Basic";
#import "Machine_X64";

main :: () -> s64 {
    a := rdtsc();
    b := rdtsc();
    if b < a return 1;
    stamp := rdtscp();
    if stamp < a return 2;
    pause();
    lfence();
    sfence();
    mfence();
    x: int = 5;
    prefetch(*x, .T0);
    prefetch(*x, .NTA);
    rdrand();
    rdseed();
    cpu := get_cpu_info();
    if cpu.vendor != .UNKNOWN && cpu.vendor != .INTEL && cpu.vendor != .AMD return 3;
    enable_feature(*cpu.feature_leaves, .AVX2);
    if !check_feature(cpu.feature_leaves, .AVX2) return 4;
    disable_feature(*cpu.feature_leaves, .AVX2);
    if check_feature(cpu.feature_leaves, .AVX2) return 5;
    print("ok\n");
    return 0;
}
